use std::sync::{Arc, Mutex, atomic::Ordering};

use crate::audio::source::{AudioSource, SeekableSource, SharedPipelineState};
use crate::audio::structural::StructuralSource;


pub const CHUNK_FRAMES: usize = 4096;

// Background thread: pulls decoded chunks from StructuralSource and feeds
// the rtrb ring buffer. Also writes each chunk to AudioStore for seek caching;
// chunks are keyed in *processed* frames.
// Runs until shutdown is signalled or the source is exhausted (then idles,
// waiting for a seek or shutdown).
pub struct AudioProducer {
    decoder:  StructuralSource,
    store:    Arc<Mutex<AudioStore>>,
    ring_tx:  rtrb::Producer<f32>,
    state:    Arc<SharedPipelineState>,
}

impl AudioProducer {
    pub fn new(
        decoder:  StructuralSource,
        store:    Arc<Mutex<AudioStore>>,
        ring_tx:  rtrb::Producer<f32>,
        state:    Arc<SharedPipelineState>,
    ) -> Self {
        Self { decoder, store, ring_tx, state }
    }

    pub fn run(mut self) {
        let channels    = self.decoder.channels() as usize;
        let sample_rate = self.decoder.sample_rate();
        let mut start_frame: usize = 0;

        loop {
            // Handle a pending seek: re-position the decoder, then wait for
            // BufferedSource to drain the ring before clearing seek_pending.
            if self.state.seek_pending.load(Ordering::Acquire) {
                let seek_f = self.state.seek_frame.load(Ordering::Acquire);
                self.decoder.seek(seek_f as f64 / sample_rate as f64);
                start_frame = seek_f as usize;
                // Wait until BufferedSource has drained the old data.
                loop {
                    if self.ring_tx.slots() == self.ring_tx.buffer().capacity() { break; }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                self.state.seek_pending.store(false, Ordering::Release);
            }

            let mut chunk_buf = vec![0.0f32; CHUNK_FRAMES * channels];
            let written = self.decoder.fill_buffer(&mut chunk_buf);
            if written == 0 {
                // Decoder exhausted — idle until a seek wakes us or shutdown arrives.
                self.state.producer_done.store(true, Ordering::Release);
                loop {
                    if self.state.shutdown.load(Ordering::Acquire) { return; }
                    if self.state.seek_pending.load(Ordering::Acquire) {
                        self.state.producer_done.store(false, Ordering::Release);
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

            // Push the decoded chunk into the ring. Check free slots once per
            // iteration to avoid per-sample error handling.
            let mut pushed = 0;
            while pushed < written {
                let free = self.ring_tx.slots();
                if free == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    continue;
                }
                let batch = (written - pushed).min(free);
                for i in 0..batch {
                    self.ring_tx.push(samples[pushed + i]).ok();
                }
                pushed += batch;
            }
        }
    }
}



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

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.cached_frames = 0;
    }

    pub fn get_chunk(&self, frame: usize) -> Option<&DecodedChunk> {
        let ch = self.channels as usize;
        self.chunks.iter().find(|c| {
            let frame_count = c.samples.len() / ch;
            frame >= c.start_frame && frame < c.start_frame + frame_count
        })
    }
}