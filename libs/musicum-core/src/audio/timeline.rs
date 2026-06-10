use musicum_processor_sdk::processor::Segment;

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegment {
    pub out_start_frame: u64, // processed timeline
    pub src_start_frame: u64, // source timeline
    pub frames: u64,          // length in output frames
    pub rate: f64,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    segments: Vec<TimelineSegment>,
    output_frames: u64,
    sample_rate: u32,
}

impl Timeline {
    pub fn identity(source_frames: u64, sample_rate: u32) -> Self {
        Self {
            segments: vec![TimelineSegment {
                out_start_frame: 0,
                src_start_frame: 0,
                frames: source_frames,
                rate: 1.0,
            }],
            output_frames: source_frames,
            sample_rate,
        }
    }

    /// Apply one structural edit. `edit_segments` are in seconds over the
    /// *current output* of this timeline (= that processor's input domain).
    pub fn apply_edit(&mut self, edit_segments: &[Segment]) {
        let sr = self.sample_rate as f64;
        let mut new_segments = Vec::new();
        let mut out_cursor: u64 = 0;

        for edit in edit_segments {
            // Quantize + clamp to the current output domain.
            let start = ((edit.src_start.max(0.0) * sr).round() as u64).min(self.output_frames);
            let end = ((edit.src_end.max(0.0) * sr).round() as u64).min(self.output_frames);
            if end <= start {
                continue;
            }

            // Compose through the existing segments (split at boundaries).
            for existing in &self.segments {
                let ex_start = existing.out_start_frame;
                let ex_end = ex_start + existing.frames;
                let ov_start = start.max(ex_start);
                let ov_end = end.min(ex_end);
                if ov_end <= ov_start {
                    continue;
                }
                let offset = ov_start - ex_start;
                new_segments.push(TimelineSegment {
                    out_start_frame: out_cursor,
                    src_start_frame: existing.src_start_frame + offset,
                    frames: ov_end - ov_start,
                    rate: existing.rate * edit.rate,
                });
                out_cursor += ov_end - ov_start;
            }
        }
        self.segments = new_segments;
        self.output_frames = out_cursor;
    }

    pub fn output_frames(&self) -> u64 {
        self.output_frames
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn output_duration(&self) -> f64 {
        self.output_frames as f64 / self.sample_rate as f64
    }

    pub fn segment_at(&self, processed_frame: u64) -> Option<&TimelineSegment> {
        self.segments.iter().find(|s| {
            processed_frame >= s.out_start_frame
                && processed_frame < s.out_start_frame + s.frames
        })
    }

    /// processed → source, clamped to the source end of the last segment.
    pub fn source_time(&self, processed_secs: f64) -> f64 {
        let frame = (processed_secs * self.sample_rate as f64).round() as u64;
        let src_frame = match self.segment_at(frame) {
            Some(s) => s.src_start_frame + (frame - s.out_start_frame),
            None => self.segments.last().map_or(0, |s| s.src_start_frame + s.frames),
        };
        src_frame as f64 / self.sample_rate as f64
    }

    /// source → processed; None inside a removed region.
    pub fn processed_time(&self, source_secs: f64) -> Option<f64> {
        let frame = (source_secs * self.sample_rate as f64).round() as u64;
        self.segments
            .iter()
            .find(|s| frame >= s.src_start_frame && frame < s.src_start_frame + s.frames)
            .map(|s| {
                let out = s.out_start_frame + (frame - s.src_start_frame);
                out as f64 / self.sample_rate as f64
            })
    }

    /// source → processed; snaps to the start of the next surviving segment
    /// (output end if none follows). Used for the pause→resume playhead.
    pub fn processed_time_or_next(&self, source_secs: f64) -> f64 {
        if let Some(t) = self.processed_time(source_secs) {
            return t;
        }
        let frame = (source_secs * self.sample_rate as f64).round() as u64;
        self.segments
            .iter()
            .filter(|s| s.src_start_frame >= frame)
            .min_by_key(|s| s.src_start_frame)
            .map_or(self.output_duration(), |s| {
                s.out_start_frame as f64 / self.sample_rate as f64
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use musicum_processor_sdk::processor::Segment;

    fn seg(start: f64, end: f64) -> Segment {
        Segment { src_start: start, src_end: end, rate: 1.0 }
    }

    #[test]
    fn identity_maps_one_to_one() {
        let tl = Timeline::identity(1000, 100);
        assert_eq!(tl.output_frames(), 1000);
        assert!((tl.output_duration() - 10.0).abs() < 1e-9);
        assert!((tl.source_time(3.0) - 3.0).abs() < 1e-9);
        assert_eq!(tl.processed_time(3.0), Some(3.0));
    }

    #[test]
    fn single_trim_offsets_and_shortens() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(1.0, 8.0)]); // trim start=1.0, end=2.0 on 10s
        assert_eq!(tl.output_frames(), 700);
        assert!((tl.source_time(0.0) - 1.0).abs() < 1e-9);
        assert!((tl.source_time(3.0) - 4.0).abs() < 1e-9);
        assert_eq!(tl.processed_time(4.0), Some(3.0));
        assert_eq!(tl.processed_time(0.5), None); // before trim start
        assert_eq!(tl.processed_time(9.0), None); // after trim end
    }

    #[test]
    fn stacked_trims_compose() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(1.0, 9.0)]); // 10s → 8s, source [1,9]
        tl.apply_edit(&[seg(2.0, 7.0)]); // 8s → 5s, source [3,8]
        assert_eq!(tl.output_frames(), 500);
        assert!((tl.source_time(0.0) - 3.0).abs() < 1e-9);
        assert_eq!(tl.processed_time(3.0), Some(0.0));
        assert_eq!(tl.processed_time(2.0), None);
    }

    #[test]
    fn mid_cut_splits_into_two_segments() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(0.0, 4.0), seg(6.0, 10.0)]); // cut [4,6)
        assert_eq!(tl.output_frames(), 800);
        assert_eq!(tl.segment_at(0).unwrap().src_start_frame, 0);
        let second = tl.segment_at(400).unwrap();
        assert_eq!(second.out_start_frame, 400);
        assert_eq!(second.src_start_frame, 600);
        assert_eq!(tl.processed_time(5.0), None); // inside the cut
        assert_eq!(tl.processed_time(7.0), Some(5.0)); // after the cut
        assert!((tl.source_time(5.0) - 7.0).abs() < 1e-9);
    }

    #[test]
    fn mid_cut_after_trim_splits_existing_segment() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(1.0, 9.0)]); // source [1,9], 8s out
        tl.apply_edit(&[seg(0.0, 3.0), seg(5.0, 8.0)]); // cut processed [3,5)
        assert_eq!(tl.output_frames(), 600);
        // processed 3.0 now maps to source 1.0 + 5.0 = 6.0
        assert!((tl.source_time(3.0) - 6.0).abs() < 1e-9);
        assert_eq!(tl.processed_time(5.0), None); // source 5.0 is in the cut
    }

    #[test]
    fn empty_edit_yields_empty_timeline() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[]);
        assert_eq!(tl.output_frames(), 0);
        assert!(tl.segment_at(0).is_none());
        assert_eq!(tl.processed_time(5.0), None);
    }

    #[test]
    fn out_of_range_segments_are_clamped_and_empty_dropped() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(-5.0, 4.0), seg(8.0, 99.0), seg(20.0, 30.0)]);
        // → [0,4) + [8,10), third dropped entirely
        assert_eq!(tl.output_frames(), 600);
        assert!((tl.source_time(4.5) - 8.5).abs() < 1e-9);
    }

    #[test]
    fn boundaries_are_quantized_to_frames() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(0.004, 9.996)]); // rounds to frames 0..1000
        assert_eq!(tl.output_frames(), 1000);
    }

    #[test]
    fn source_time_clamps_past_output_end() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(1.0, 8.0)]);
        assert!((tl.source_time(99.0) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn processed_time_or_next_snaps_forward() {
        let mut tl = Timeline::identity(1000, 100);
        tl.apply_edit(&[seg(0.0, 4.0), seg(6.0, 10.0)]);
        assert!((tl.processed_time_or_next(5.0) - 4.0).abs() < 1e-9); // snaps to seg 2 start
        assert!((tl.processed_time_or_next(7.0) - 5.0).abs() < 1e-9); // normal mapping
        let mut empty = Timeline::identity(1000, 100);
        empty.apply_edit(&[]);
        assert!((empty.processed_time_or_next(5.0) - 0.0).abs() < 1e-9); // output end = 0
    }
}
