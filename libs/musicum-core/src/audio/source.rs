use std::{
    collections::VecDeque,
    path::Path,
};

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
    fn bit_depth(&self) -> u8;
    fn is_exhausted(&self) -> bool;
    fn seek(&mut self, position: f64);
    fn duration(&self) -> f64;
}

pub trait AudioNode: AudioSource {
    fn connect(&mut self, source: Box<dyn AudioSource>);
}

#[allow(dead_code)]
pub struct PluginNode {}

// ── SymphoniaSource ──────────────────────────────────────────────────────────

const RUBATO_CHUNK: usize = 1024;

pub struct SymphoniaSource {
    format:     Box<dyn symphonia::core::formats::FormatReader>,
    decoder:    Box<dyn symphonia::core::codecs::Decoder>,
    track_id:   u32,
    src_rate:   u32,
    out_rate:   u32,
    channels:   u8,
    duration:   f64,
    resampler:  Option<FftFixedIn<f32>>,
    decode_buf: Vec<Vec<f32>>,
    pending:    VecDeque<f32>,
    exhausted:  bool,
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

        let track_id = track.id;
        let src_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count() as u8).unwrap_or(2);
        let n_frames = track.codec_params.n_frames.unwrap_or(0);
        let duration = if src_rate > 0 { n_frames as f64 / src_rate as f64 } else { 0.0 };

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

        let decode_buf = vec![vec![0.0f32; RUBATO_CHUNK]; channels as usize];

        Ok(Self {
            format,
            decoder,
            track_id,
            src_rate,
            out_rate: target_sample_rate,
            channels: target_channels,
            duration,
            resampler,
            decode_buf,
            pending: VecDeque::new(),
            exhausted: false,
        })
    }

    fn decode_next(&mut self) -> bool {
        let ch = self.channels as usize;

        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(_) => return false,
            };
            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Convert to owned interleaved f32 before releasing the decoder borrow
            let frames = decoded.frames();
            let owned: Vec<f32> = (0..frames)
                .flat_map(|f| (0..ch).map(move |c| (c, f)))
                .map(|(c, f)| sample_to_f32(&decoded, c, f))
                .collect();
            drop(decoded);

            self.push_owned(owned, frames, ch);
            return true;
        }
    }

    fn push_owned(&mut self, samples: Vec<f32>, frames: usize, ch: usize) {
        if self.resampler.is_none() {
            self.pending.extend(samples);
            return;
        }

        // Fill planar decode_buf for rubato
        let chunk_frames = frames.min(RUBATO_CHUNK);
        for c in 0..ch {
            self.decode_buf[c].resize(chunk_frames, 0.0);
            for f in 0..chunk_frames {
                self.decode_buf[c][f] = samples[f * ch + c];
            }
            self.decode_buf[c].resize(RUBATO_CHUNK, 0.0);
        }

        if let Some(resampler) = &mut self.resampler {
            if let Ok(out) = resampler.process(&self.decode_buf, None) {
                let out_frames = out[0].len();
                for f in 0..out_frames {
                    self.pending.extend((0..ch).map(|c| out[c][f]));
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

impl AudioSource for SymphoniaSource {
    fn sample_rate(&self) -> u32 { self.out_rate }
    fn channels(&self) -> u8 { self.channels }
    fn bit_depth(&self) -> u8 { 32 }
    fn is_exhausted(&self) -> bool { self.exhausted }
    fn duration(&self) -> f64 { self.duration }

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

    fn seek(&mut self, position: f64) {
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
        self.exhausted = false;
    }
}
