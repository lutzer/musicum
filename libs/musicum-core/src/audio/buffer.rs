use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::audio::source::{AudioSource, SymphoniaSource};

pub struct DecodedChunk {
    pub start_frame: usize,
    pub samples: Arc<[f32]>,
}

pub struct AudioStore {
    chunks:          Vec<DecodedChunk>,
    cached_frames:   usize,
    capacity_frames: usize,
    channels:        u8,
}

impl AudioStore {
    pub fn new(capacity_frames: usize, channels: u8) -> Self {
        Self { chunks: Vec::new(), cached_frames: 0, capacity_frames, channels }
    }

    pub fn insert(&mut self, chunk: DecodedChunk) {
        let frames = chunk.samples.len() / self.channels as usize;
        self.cached_frames += frames;
        self.chunks.push(chunk);
        while self.cached_frames > self.capacity_frames && !self.chunks.is_empty() {
            let evicted = self.chunks.remove(0);
            self.cached_frames -= evicted.samples.len() / self.channels as usize;
        }
    }

    pub fn get_chunk(&self, frame: usize) -> Option<&DecodedChunk> {
        let ch = self.channels as usize;
        self.chunks.iter().find(|c| {
            let frame_count = c.samples.len() / ch;
            frame >= c.start_frame && frame < c.start_frame + frame_count
        })
    }
}

#[derive(Clone)]
pub struct SourceHandle {
    seek_pending:  Arc<AtomicBool>,
    seek_frame:    Arc<AtomicU64>,
    playhead:      Arc<AtomicU64>,
    exhausted:     Arc<AtomicBool>,
    shutdown:      Arc<AtomicBool>,
    sample_rate:   u32,
    channels:      u8,
    duration:      f64,
}

impl SourceHandle {
    pub fn new(
        seek_pending:  Arc<AtomicBool>,
        seek_frame:    Arc<AtomicU64>,
        playhead:      Arc<AtomicU64>,
        exhausted:     Arc<AtomicBool>,
        shutdown:      Arc<AtomicBool>,
        sample_rate:   u32,
        channels:      u8,
        duration:      f64,
    ) -> Self {
        Self { seek_pending, seek_frame, playhead, exhausted, shutdown, sample_rate, channels, duration }
    }

    pub fn seek(&self, position_secs: f64) {
        let frame = (position_secs * self.sample_rate as f64) as u64;
        self.seek_frame.store(frame, Ordering::Release);
        self.exhausted.store(false, Ordering::Release);
        self.playhead.store(frame * self.channels as u64, Ordering::Release);
        self.seek_pending.store(true, Ordering::Release);
    }

    pub fn position_secs(&self) -> f64 {
        let samples = self.playhead.load(Ordering::Acquire);
        (samples / self.channels as u64) as f64 / self.sample_rate as f64
    }

    pub fn seekhead_secs(&self) -> Option<f64> {
        if self.seek_pending.load(Ordering::Acquire) {
            let frame = self.seek_frame.load(Ordering::Acquire);
            Some(frame as f64 / self.sample_rate as f64)
        } else {
            None
        }
    }

    pub fn duration_secs(&self) -> f64 { self.duration }

    pub fn is_exhausted(&self) -> bool { self.exhausted.load(Ordering::Acquire) }

    pub fn shutdown(&self) { self.shutdown.store(true, Ordering::Release); }
}

pub const CHUNK_FRAMES: usize = 4096;

pub struct AudioProducer {
    decoder:       SymphoniaSource,
    store:         Arc<Mutex<AudioStore>>,
    ring_tx:       rtrb::Producer<f32>,
    ring_capacity: usize,
    seek_pending:  Arc<AtomicBool>,
    seek_frame:    Arc<AtomicU64>,
    producer_done: Arc<AtomicBool>,
    shutdown:      Arc<AtomicBool>,
}

impl AudioProducer {
    pub fn new(
        decoder:       SymphoniaSource,
        store:         Arc<Mutex<AudioStore>>,
        ring_tx:       rtrb::Producer<f32>,
        ring_capacity: usize,
        seek_pending:  Arc<AtomicBool>,
        seek_frame:    Arc<AtomicU64>,
        producer_done: Arc<AtomicBool>,
        shutdown:      Arc<AtomicBool>,
    ) -> Self {
        Self { decoder, store, ring_tx, ring_capacity, seek_pending, seek_frame, producer_done, shutdown }
    }

    pub fn run(mut self) {
        let channels    = self.decoder.channels() as usize;
        let sample_rate = self.decoder.sample_rate();
        let mut start_frame: usize = 0;

        loop {
            if self.seek_pending.load(Ordering::Acquire) {
                let seek_f = self.seek_frame.load(Ordering::Acquire);
                self.decoder.seek(seek_f as f64 / sample_rate as f64);
                start_frame = seek_f as usize;
                loop {
                    if self.ring_tx.slots() == self.ring_capacity { break; }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                self.seek_pending.store(false, Ordering::Release);
            }

            let mut chunk_buf = vec![0.0f32; CHUNK_FRAMES * channels];
            let written = self.decoder.fill_buffer(&mut chunk_buf);
            if written == 0 {
                self.producer_done.store(true, Ordering::Release);
                loop {
                    if self.shutdown.load(Ordering::Acquire) { return; }
                    if self.seek_pending.load(Ordering::Acquire) {
                        self.producer_done.store(false, Ordering::Release);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                continue;
            }

            let samples: Arc<[f32]> = Arc::from(&chunk_buf[..written]);
            let chunk = DecodedChunk { start_frame, samples: samples.clone() };
            if let Ok(mut store) = self.store.lock() {
                store.insert(chunk);
            }
            start_frame += written / channels;

            let mut pushed = 0;
            while pushed < written {
                match self.ring_tx.push(samples[pushed]) {
                    Ok(()) => pushed += 1,
                    Err(_) => std::thread::sleep(std::time::Duration::from_millis(1)),
                }
            }
        }
    }
}

pub struct BufferedSource {
    ring_rx:       rtrb::Consumer<f32>,
    sample_rate:   u32,
    channels:      u8,
    duration:      f64,
    seek_pending:  Arc<AtomicBool>,
    seek_frame:    Arc<AtomicU64>,
    producer_done: Arc<AtomicBool>,
    playhead:      Arc<AtomicU64>,
    exhausted:     Arc<AtomicBool>,
}

impl BufferedSource {
    pub fn new(
        ring_rx:       rtrb::Consumer<f32>,
        sample_rate:   u32,
        channels:      u8,
        duration:      f64,
        seek_pending:  Arc<AtomicBool>,
        seek_frame:    Arc<AtomicU64>,
        producer_done: Arc<AtomicBool>,
        playhead:      Arc<AtomicU64>,
        exhausted:     Arc<AtomicBool>,
    ) -> Self {
        Self {
            ring_rx, sample_rate, channels, duration,
            seek_pending, seek_frame, producer_done,
            playhead, exhausted,
        }
    }
}

impl AudioSource for BufferedSource {
    fn sample_rate(&self) -> u32   { self.sample_rate }
    fn channels(&self)    -> u8    { self.channels }
    fn duration_secs(&self) -> f64 { self.duration }
    fn is_exhausted(&self) -> bool { self.exhausted.load(Ordering::Acquire) }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        if self.seek_pending.load(Ordering::Acquire) {
            while self.ring_rx.pop().is_ok() {}
            let frame = self.seek_frame.load(Ordering::Acquire);
            self.playhead.store(frame * self.channels as u64, Ordering::Release);
            self.exhausted.store(false, Ordering::Release);
            buffer.fill(0.0);
            return buffer.len();
        }

        let mut filled = 0;
        for slot in buffer.iter_mut() {
            match self.ring_rx.pop() {
                Ok(s) => { *slot = s; filled += 1; }
                Err(_) => { *slot = 0.0; }
            }
        }
        self.playhead.fetch_add(filled as u64, Ordering::Release);

        if filled < buffer.len() && self.producer_done.load(Ordering::Acquire) {
            self.exhausted.store(true, Ordering::Release);
        }
        buffer.len()
    }
}
