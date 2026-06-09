use std::sync::{Arc};
use std::sync::atomic::{Ordering};
use crate::audio::source::{AudioSource, SharedPipelineState};


// AudioSource implementation for the audio callback thread.
// Drains the rtrb ring on every fill_buffer() call and advances the playhead
// atomic so the main thread can read the current position without locking.
// Outputs silence (and drains the ring) while a seek is in flight.
pub struct BufferedSource {
    ring_rx:     rtrb::Consumer<f32>,
    sample_rate: u32,
    channels:    u8,
    duration:    f64,
    state:       Arc<SharedPipelineState>,
}

impl BufferedSource {
    pub fn new(
        ring_rx:     rtrb::Consumer<f32>,
        sample_rate: u32,
        channels:    u8,
        duration:    f64,
        state:       Arc<SharedPipelineState>,
    ) -> Self {
        Self { ring_rx, sample_rate, channels, duration, state }
    }
}

impl AudioSource for BufferedSource {
    fn sample_rate(&self) -> u32   { self.sample_rate }
    fn channels(&self)    -> u8    { self.channels }
    fn duration_secs(&self) -> f64 { self.duration }
    fn is_exhausted(&self) -> bool { self.state.exhausted.load(Ordering::Acquire) }
    fn position_secs(&self) -> f64 {
        let samples = self.state.playhead.load(Ordering::Acquire);
        (samples / self.channels as u64) as f64 / self.sample_rate as f64
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        // Seek in flight: drain stale ring data, reset playhead to seek target,
        // and output silence for this callback. Producer clears seek_pending once
        // the ring is fully drained on its side.
        if self.state.seek_pending.load(Ordering::Acquire) {
            while self.ring_rx.pop().is_ok() {}
            let frame = self.state.seek_frame.load(Ordering::Acquire);
            self.state.playhead.store(frame * self.channels as u64, Ordering::Release);
            self.state.exhausted.store(false, Ordering::Release);
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
        self.state.playhead.fetch_add(filled as u64, Ordering::Release);

        if filled < buffer.len() && self.state.producer_done.load(Ordering::Acquire) {
            self.state.exhausted.store(true, Ordering::Release);
        }
        buffer.len()
    }
}
