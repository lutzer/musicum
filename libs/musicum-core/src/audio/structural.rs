use std::sync::{Arc, RwLock};

use crate::audio::source::{AudioSource, SeekableSource};
use crate::audio::timeline::Timeline;

// Presents the *processed* timeline by walking Timeline segments and seeking
// the wrapped decoder at segment boundaries. Runs on the producer thread;
// the RwLock is only ever read here (rebuilds happen on the main thread
// while playback is paused).
pub struct StructuralSource {
    decoder:      Box<dyn SeekableSource>,
    timeline:     Arc<RwLock<Timeline>>,
    sample_rate:  u32,
    channels:     u8,
    out_frame:    u64,         // processed playhead
    expected_src: Option<u64>, // decoder position we believe; None forces a seek
    exhausted:    bool,
}

impl StructuralSource {
    pub fn new(decoder: Box<dyn SeekableSource>, timeline: Arc<RwLock<Timeline>>) -> Self {
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        Self {
            decoder, timeline, sample_rate, channels,
            out_frame: 0, expected_src: None, exhausted: false,
        }
    }
}

impl AudioSource for StructuralSource {
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn channels(&self) -> u8 { self.channels }
    fn is_exhausted(&self) -> bool { self.exhausted }

    fn duration_secs(&self) -> f64 {
        self.timeline.read().unwrap().output_duration()
    }

    fn position_secs(&self) -> f64 {
        self.out_frame as f64 / self.sample_rate as f64
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        if self.exhausted { return 0; }
        let ch = self.channels as usize;
        let tl = self.timeline.read().unwrap();
        let mut filled = 0;

        while filled < buffer.len() {
            let Some(seg) = tl.segment_at(self.out_frame) else {
                self.exhausted = true;
                break;
            };
            debug_assert!(
                (seg.rate - 1.0).abs() < f64::EPSILON,
                "rate != 1.0 not supported in this iteration"
            );

            let src_target = seg.src_start_frame + (self.out_frame - seg.out_start_frame);
            if self.expected_src != Some(src_target) {
                self.decoder.seek(src_target as f64 / self.sample_rate as f64);
            }

            let frames_left = (seg.out_start_frame + seg.frames - self.out_frame) as usize;
            let want = ((buffer.len() - filled) / ch).min(frames_left) * ch;
            if want == 0 { break; }

            let n = self.decoder.fill_buffer(&mut buffer[filled..filled + want]);
            if n == 0 {
                // Decoder ran dry before the timeline's end (metadata mismatch).
                self.exhausted = true;
                break;
            }
            filled += n;
            let frames_read = (n / ch) as u64;
            self.out_frame += frames_read;
            self.expected_src = Some(src_target + frames_read);
        }
        filled
    }
}

impl SeekableSource for StructuralSource {
    fn seek(&mut self, position_secs: f64) {
        let tl = self.timeline.read().unwrap();
        let frame = (position_secs * self.sample_rate as f64).round() as u64;
        self.out_frame = frame.min(tl.output_frames());
        self.expected_src = None;
        self.exhausted = self.out_frame >= tl.output_frames();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::Segment;

    const SR: u32 = 100;
    const CH: u8 = 2;

    // Sample value == source frame index, on both channels.
    struct RampSource { frames: u64, pos: u64 }

    impl AudioSource for RampSource {
        fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
            let mut written = 0;
            while written + CH as usize <= buffer.len() && self.pos < self.frames {
                for c in 0..CH as usize {
                    buffer[written + c] = self.pos as f32;
                }
                written += CH as usize;
                self.pos += 1;
            }
            written
        }
        fn sample_rate(&self) -> u32 { SR }
        fn channels(&self) -> u8 { CH }
        fn is_exhausted(&self) -> bool { self.pos >= self.frames }
        fn duration_secs(&self) -> f64 { self.frames as f64 / SR as f64 }
        fn position_secs(&self) -> f64 { self.pos as f64 / SR as f64 }
    }

    impl SeekableSource for RampSource {
        fn seek(&mut self, position_secs: f64) {
            self.pos = (position_secs * SR as f64).round() as u64;
        }
    }

    fn make_tl(edit: &[Segment]) -> Arc<RwLock<Timeline>> {
        let mut tl = Timeline::identity(1000, SR);
        tl.apply_edit(edit);
        Arc::new(RwLock::new(tl))
    }

    fn seg(start: f64, end: f64) -> Segment {
        Segment { src_start: start, src_end: end, rate: 1.0 }
    }

    #[test]
    fn identity_passes_through_unchanged() {
        let tl = Arc::new(RwLock::new(Timeline::identity(1000, SR)));
        let mut s = StructuralSource::new(Box::new(RampSource { frames: 1000, pos: 0 }), tl);
        let mut buf = vec![0.0f32; 20];
        assert_eq!(s.fill_buffer(&mut buf), 20);
        assert_eq!(buf[0], 0.0);
        assert_eq!(buf[18], 9.0); // frame 9, left channel
    }

    #[test]
    fn trim_starts_at_offset() {
        let mut s = StructuralSource::new(
            Box::new(RampSource { frames: 1000, pos: 0 }),
            make_tl(&[seg(1.0, 8.0)]),
        );
        let mut buf = vec![0.0f32; 8];
        s.fill_buffer(&mut buf);
        assert_eq!(buf[0], 100.0); // first sample is source frame 100
        assert!((s.duration_secs() - 7.0).abs() < 1e-9);
    }

    #[test]
    fn cut_boundary_is_sample_exact_within_one_fill() {
        // segments [0,0.5) + [7.0,7.5): boundary at processed frame 50
        let mut s = StructuralSource::new(
            Box::new(RampSource { frames: 1000, pos: 0 }),
            make_tl(&[seg(0.0, 0.5), seg(7.0, 7.5)]),
        );
        let mut buf = vec![0.0f32; 200]; // 100 frames, spans the boundary
        assert_eq!(s.fill_buffer(&mut buf), 200);
        assert_eq!(buf[49 * CH as usize], 49.0);  // last frame of segment 1
        assert_eq!(buf[50 * CH as usize], 700.0); // first frame of segment 2
        assert_eq!(buf[99 * CH as usize], 749.0);
    }

    #[test]
    fn exhausts_after_last_segment() {
        let mut s = StructuralSource::new(
            Box::new(RampSource { frames: 1000, pos: 0 }),
            make_tl(&[seg(0.0, 0.2)]), // 20 output frames
        );
        let mut buf = vec![0.0f32; 100];
        assert_eq!(s.fill_buffer(&mut buf), 40); // 20 frames × 2 ch
        assert!(s.is_exhausted());
        assert_eq!(s.fill_buffer(&mut buf), 0);
    }

    #[test]
    fn empty_timeline_is_exhausted_immediately() {
        let mut s = StructuralSource::new(
            Box::new(RampSource { frames: 1000, pos: 0 }),
            make_tl(&[]),
        );
        let mut buf = vec![0.0f32; 10];
        assert_eq!(s.fill_buffer(&mut buf), 0);
        assert!(s.is_exhausted());
    }

    #[test]
    fn seek_operates_in_processed_domain() {
        // cut [0.25,5.0): processed 0.3 → source 5.0 + (0.3-0.25) = 5.05
        let mut s = StructuralSource::new(
            Box::new(RampSource { frames: 1000, pos: 0 }),
            make_tl(&[seg(0.0, 0.25), seg(5.0, 10.0)]),
        );
        s.seek(0.3);
        assert!((s.position_secs() - 0.3).abs() < 1e-9);
        let mut buf = vec![0.0f32; 2];
        s.fill_buffer(&mut buf);
        assert_eq!(buf[0], 505.0);
    }

    #[test]
    fn seek_past_end_marks_exhausted() {
        let mut s = StructuralSource::new(
            Box::new(RampSource { frames: 1000, pos: 0 }),
            make_tl(&[seg(0.0, 1.0)]),
        );
        s.seek(50.0);
        assert!(s.is_exhausted());
        let mut buf = vec![0.0f32; 10];
        assert_eq!(s.fill_buffer(&mut buf), 0);
    }
}
