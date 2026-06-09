use std::{collections::VecDeque, path::Path, sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}}};

use anyhow::{anyhow, Context};
use rubato::{FftFixedIn, Resampler};
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

pub trait AudioSource: Send {
    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u8;
    fn is_exhausted(&self) -> bool;
    fn duration_secs(&self) -> f64;
}


// All cross-thread coordination state for one loaded source.
// Shared between AudioProducer (background thread), BufferedSource (audio
// callback thread), and SourceHandle (main thread) via Arc::clone.
pub struct SharedPipelineState {
    pub seek_pending:  AtomicBool, // seek was requested; cleared by producer after flush
    pub seek_frame:    AtomicU64,  // target frame for the pending seek
    pub producer_done: AtomicBool, // producer has exhausted the decoder
    pub shutdown:      AtomicBool, // signal producer thread to exit
    pub playhead:      AtomicU64,  // total samples consumed (frames × channels)
    pub exhausted:     AtomicBool, // BufferedSource has played past end of file
}

impl SharedPipelineState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            seek_pending:  AtomicBool::new(false),
            seek_frame:    AtomicU64::new(0),
            producer_done: AtomicBool::new(false),
            shutdown:      AtomicBool::new(false),
            playhead:      AtomicU64::new(0),
            exhausted:     AtomicBool::new(false),
        })
    }
}

// Main-thread handle into a loaded source pipeline.
// All reads are lock-free atomic loads; no mutex needed.
#[derive(Clone)]
pub struct SourceHandle {
    state:       Arc<SharedPipelineState>,
    sample_rate: u32,
    channels:    u8,
    duration:    f64,
}

impl SourceHandle {
    pub fn new(
        state:       Arc<SharedPipelineState>,
        sample_rate: u32,
        channels:    u8,
        duration:    f64,
    ) -> Self {
        Self { state, sample_rate, channels, duration }
    }

    pub fn seek(&self, position_secs: f64) {
        let frame = (position_secs * self.sample_rate as f64) as u64;
        self.state.seek_frame.store(frame, Ordering::Release);
        self.state.exhausted.store(false, Ordering::Release);
        self.state.playhead.store(frame * self.channels as u64, Ordering::Release);
        self.state.seek_pending.store(true, Ordering::Release);
    }

    pub fn position_secs(&self) -> f64 {
        let samples = self.state.playhead.load(Ordering::Acquire);
        (samples / self.channels as u64) as f64 / self.sample_rate as f64
    }

    pub fn seekhead_secs(&self) -> Option<f64> {
        if self.state.seek_pending.load(Ordering::Acquire) {
            let frame = self.state.seek_frame.load(Ordering::Acquire);
            Some(frame as f64 / self.sample_rate as f64)
        } else {
            None
        }
    }

    pub fn duration_secs(&self) -> f64 { self.duration }

    pub fn is_exhausted(&self) -> bool { self.state.exhausted.load(Ordering::Acquire) }

    pub fn shutdown(&self) { self.state.shutdown.store(true, Ordering::Release); }
}

// ── SymphoniaSource ──────────────────────────────────────────────────────────

const RUBATO_CHUNK: usize = 1024;

// Decodes an audio file with Symphonia and resamples to the output rate with Rubato.
// Interleaved f32 samples are accumulated in `pending`; callers drain via fill_buffer().
pub struct SymphoniaSource {
    // format / decoder
    format:    Box<dyn symphonia::core::formats::FormatReader>,
    decoder:   Box<dyn symphonia::core::codecs::Decoder>,
    track_id:  u32,

    // rate / channel config
    src_rate:  u32,
    out_rate:  u32,
    channels:  u8,
    duration:  f64,

    // resampling (None when src_rate == out_rate)
    resampler:        Option<FftFixedIn<f32>>,
    resample_staging: Vec<Vec<f32>>,
    decode_buf:       Vec<Vec<f32>>,

    // output
    pending:   VecDeque<f32>,
    exhausted: bool,
}

impl SymphoniaSource {
    pub fn new(path: &Path, target_sample_rate: u32, target_channels: u8) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("cannot open {}", path.display()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .context("unsupported audio format")?;

        let format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("no supported audio track"))?;

        let track_id   = track.id;
        let src_rate   = track.codec_params.sample_rate.unwrap_or(44100);
        let channels   = track.codec_params.channels.map(|c| c.count() as u8).unwrap_or(2);
        let n_frames   = track.codec_params.n_frames.unwrap_or(0);
        let duration   = if src_rate > 0 { n_frames as f64 / src_rate as f64 } else { 0.0 };

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .context("no decoder for codec")?;

        let resampler = if src_rate != target_sample_rate {
            Some(FftFixedIn::<f32>::new(
                src_rate as usize,
                target_sample_rate as usize,
                RUBATO_CHUNK,
                2,
                channels as usize,
            )?)
        } else {
            None
        };

        let ch = channels as usize;
        Ok(Self {
            format,
            decoder,
            track_id,
            src_rate,
            out_rate: target_sample_rate,
            channels: target_channels,
            duration,
            resampler,
            resample_staging: vec![Vec::new(); ch],
            decode_buf:       vec![vec![0.0f32; RUBATO_CHUNK]; ch],
            pending:   VecDeque::new(),
            exhausted: false,
        })
    }

    // Decodes one packet and pushes the resulting samples through push_decoded().
    // Returns false when the format reader signals end-of-stream.
    fn decode_next(&mut self) -> bool {
        let ch = self.channels as usize;

        loop {
            let packet = match self.format.next_packet() {
                Ok(p)  => p,
                Err(_) => return false,
            };
            if packet.track_id() != self.track_id { continue; }

            let decoded = match self.decoder.decode(&packet) {
                Ok(d)  => d,
                Err(_) => continue,
            };

            let frames = decoded.frames();
            let owned: Vec<f32> = (0..frames)
                .flat_map(|f| (0..ch).map(move |c| (c, f)))
                .map(|(c, f)| Self::sample_to_f32(&decoded, c, f))
                .collect();
            drop(decoded);

            self.push_decoded(owned, frames);
            return true;
        }
    }

    // Appends decoded interleaved samples to `pending`, resampling if needed.
    // FftFixedIn requires exactly RUBATO_CHUNK input frames per call, so frames
    // are staged in `resample_staging` until a full block is ready; the remainder carries over.
    fn push_decoded(&mut self, samples: Vec<f32>, frames: usize) {
        let ch = self.channels as usize;

        if self.resampler.is_none() {
            self.pending.extend(samples);
            return;
        }

        for f in 0..frames {
            for c in 0..ch {
                self.resample_staging[c].push(samples[f * ch + c]);
            }
        }

        while self.resample_staging[0].len() >= RUBATO_CHUNK {
            for c in 0..ch {
                self.decode_buf[c].clear();
                self.decode_buf[c].extend_from_slice(&self.resample_staging[c][..RUBATO_CHUNK]);
                self.resample_staging[c].drain(..RUBATO_CHUNK);
            }
            if let Some(resampler) = &mut self.resampler {
                if let Ok(out) = resampler.process(&self.decode_buf, None) {
                    for f in 0..out[0].len() {
                        for c in 0..ch {
                            self.pending.push_back(out[c][f]);
                        }
                    }
                }
            }
        }
    }

    fn sample_to_f32(buf: &AudioBufferRef<'_>, channel: usize, frame: usize) -> f32 {
        use symphonia::core::audio::AudioBufferRef::*;
        match buf {
            F32(b) => b.chan(channel)[frame],
            F64(b) => b.chan(channel)[frame] as f32,
            S32(b) => b.chan(channel)[frame] as f32 / i32::MAX as f32,
            S24(b) => b.chan(channel)[frame].inner() as f32 / 8_388_607.0,
            S16(b) => b.chan(channel)[frame] as f32 / i16::MAX as f32,
            S8(b)  => b.chan(channel)[frame] as f32 / i8::MAX as f32,
            U32(b) => (b.chan(channel)[frame] as f32 / u32::MAX as f32) * 2.0 - 1.0,
            U24(b) => (b.chan(channel)[frame].inner() as f32 / 16_777_215.0) * 2.0 - 1.0,
            U16(b) => (b.chan(channel)[frame] as f32 / u16::MAX as f32) * 2.0 - 1.0,
            U8(b)  => (b.chan(channel)[frame] as f32 / u8::MAX as f32) * 2.0 - 1.0,
        }
    }
}

impl AudioSource for SymphoniaSource {
    fn sample_rate(&self) -> u32   { self.out_rate }
    fn channels(&self)    -> u8    { self.channels }
    fn is_exhausted(&self) -> bool { self.exhausted }
    fn duration_secs(&self) -> f64 { self.duration }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        if self.exhausted { return 0; }

        while self.pending.len() < buffer.len() {
            if !self.decode_next() {
                self.exhausted = true;
                break;
            }
        }

        let n = buffer.len().min(self.pending.len());
        for slot in buffer.iter_mut().take(n) {
            *slot = self.pending.pop_front().unwrap_or(0.0);
        }
        n
    }
}

impl SymphoniaSource {
    pub fn seek(&mut self, position: f64) {
        use symphonia::core::formats::{SeekMode, SeekTo};
        use symphonia::core::units::Time;

        let _ = self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time { time: Time::new(position as u64, position.fract()), track_id: None },
        );
        self.decoder.reset();
        if let Some(r) = &mut self.resampler {
            if let Ok(fresh) = FftFixedIn::<f32>::new(
                self.src_rate as usize,
                self.out_rate as usize,
                RUBATO_CHUNK,
                2,
                self.channels as usize,
            ) {
                *r = fresh;
            }
        }
        self.pending.clear();
        for v in &mut self.resample_staging { v.clear(); }
        self.exhausted = false;
    }
}
