#[cfg(test)]
fn test_wav_path() -> std::path::PathBuf {
    let path = std::env::temp_dir().join("musicum_test_440hz.wav");
    if !path.exists() {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for i in 0..44100 {
            let t = i as f32 / 44100.0;
            let s = ((2.0 * std::f32::consts::PI * 440.0 * t).sin() * i16::MAX as f32) as i16;
            writer.write_sample(s).unwrap();
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
    }
    path
}

#[cfg(test)]
mod source_tests {
    use crate::audio::source::{AudioSource, SymphoniaSource};

    #[test]
    fn constructs_from_wav() {
        let path = super::test_wav_path();
        let src = SymphoniaSource::new(&path, 48000, 2).unwrap();
        assert_eq!(src.sample_rate(), 48000);
        assert_eq!(src.channels(), 2);
        assert!(!src.is_exhausted());
        assert!(src.duration_secs() > 0.9 && src.duration_secs() < 1.1);
    }

    #[test]
    fn fill_buffer_returns_samples() {
        let path = super::test_wav_path();
        let mut src = SymphoniaSource::new(&path, 48000, 2).unwrap();
        let mut buf = vec![0.0f32; 1024];
        let written = src.fill_buffer(&mut buf);
        assert!(written > 0);
        assert!(buf[..written].iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn exhausts_after_full_drain() {
        let path = super::test_wav_path();
        let mut src = SymphoniaSource::new(&path, 48000, 2).unwrap();
        let mut buf = vec![0.0f32; 4096];
        loop {
            src.fill_buffer(&mut buf);
            if src.is_exhausted() { break; }
        }
        assert!(src.is_exhausted());
    }

    #[test]
    fn no_resample_when_rates_match() {
        let path = super::test_wav_path();
        let mut src = SymphoniaSource::new(&path, 44100, 2).unwrap();
        let mut buf = vec![0.0f32; 512];
        let written = src.fill_buffer(&mut buf);
        assert!(written > 0);
    }

    #[test]
    #[ignore]
    fn plays_one_second_of_audio() {
        use crate::audio::output::{AudioOutput, CpalOutput};
        let path = super::test_wav_path();
        let mut output = CpalOutput::new().unwrap();
        let src = SymphoniaSource::new(&path, output.sample_rate(), output.channels()).unwrap();
        output.set_source(Box::new(src)).unwrap();
        output.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        output.pause().unwrap();
    }
}

#[cfg(test)]
mod buffer_tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use crate::audio::producer::{AudioStore, AudioProducer, DecodedChunk,};
    use crate::audio::buffer::{BufferedSource};
    use crate::audio::source::{AudioSource, SymphoniaSource, SharedPipelineState, SourceHandle};

    fn make_chunk(start_frame: usize, frame_count: usize, channels: u8) -> DecodedChunk {
        DecodedChunk {
            start_frame,
            samples: Arc::from(vec![1.0f32; frame_count * channels as usize]),
        }
    }

    #[test]
    fn store_starts_empty() {
        let store = AudioStore::new(100, 2);
        assert!(store.get_chunk(0).is_none());
    }

    #[test]
    fn store_insert_and_retrieve() {
        let mut store = AudioStore::new(100, 2);
        store.insert(make_chunk(0, 10, 2));
        assert!(store.get_chunk(0).is_some());
        assert!(store.get_chunk(9).is_some());
        assert!(store.get_chunk(10).is_none());
    }

    #[test]
    fn store_get_chunk_by_mid_frame() {
        let mut store = AudioStore::new(100, 2);
        store.insert(make_chunk(0, 10, 2));
        store.insert(make_chunk(10, 10, 2));
        assert_eq!(store.get_chunk(5).unwrap().start_frame, 0);
        assert_eq!(store.get_chunk(10).unwrap().start_frame, 10);
        assert_eq!(store.get_chunk(15).unwrap().start_frame, 10);
    }

    #[test]
    fn store_evicts_oldest_when_over_capacity() {
        // capacity = 10 frames; inserting two 8-frame chunks forces eviction
        let mut store = AudioStore::new(10, 2);
        store.insert(make_chunk(0, 8, 2));   // cached = 8
        store.insert(make_chunk(8, 8, 2));   // cached = 16 > 10 → chunk0 evicted
        assert!(store.get_chunk(0).is_none());
        assert!(store.get_chunk(8).is_some());
    }

    #[test]
    fn seek_handle_sets_frame_and_pending() {
        let state = SharedPipelineState::new();
        let handle = SourceHandle::new(state.clone(), 48000, 2, 1.0);
        handle.seek(2.0);
        assert!(state.seek_pending.load(Ordering::Acquire));
        assert_eq!(state.seek_frame.load(Ordering::Acquire), 96000);
    }

    #[test]
    fn producer_fills_ring_with_samples() {
        use std::sync::Mutex;
        let path = super::test_wav_path();
        let decoder = SymphoniaSource::new(&path, 48000, 2).unwrap();
        let sample_rate = decoder.sample_rate() as usize;
        let channels    = decoder.channels() as usize;
        let ring_cap    = 2 * sample_rate * channels;
        let store_cap   = 30 * sample_rate * channels;

        let (ring_tx, ring_rx) = rtrb::RingBuffer::new(ring_cap);
        let store = Arc::new(Mutex::new(AudioStore::new(store_cap, channels as u8)));
        let state = SharedPipelineState::new();

        let producer = AudioProducer::new(decoder, store, ring_tx, state.clone());
        std::thread::spawn(|| producer.run());

        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(ring_rx.slots() > 0 || state.producer_done.load(Ordering::Acquire));
    }

    #[test]
    #[ignore]
    fn buffered_pipeline_plays_audio() {
        use std::sync::Mutex;
        use crate::audio::output::{AudioOutput, CpalOutput};

        let path = super::test_wav_path();
        let mut output  = CpalOutput::new().unwrap();
        let decoder     = SymphoniaSource::new(&path, output.sample_rate(), output.channels()).unwrap();
        let sample_rate = decoder.sample_rate();
        let channels    = decoder.channels();
        let duration    = decoder.duration_secs();
        let ring_cap    = 2 * sample_rate as usize * channels as usize;
        let store_cap   = 30 * sample_rate as usize * channels as usize;

        let (ring_tx, ring_rx) = rtrb::RingBuffer::new(ring_cap);
        let store = Arc::new(Mutex::new(AudioStore::new(store_cap, channels)));
        let state = SharedPipelineState::new();

        let producer = AudioProducer::new(decoder, store, ring_tx, state.clone());
        let source   = BufferedSource::new(ring_rx, sample_rate, channels, duration, state.clone());

        std::thread::spawn(|| producer.run());
        output.set_source(Box::new(source)).unwrap();
        output.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        output.pause().unwrap();
    }

    fn prefilled_ring(n: usize) -> (rtrb::Producer<f32>, rtrb::Consumer<f32>) {
        let (mut tx, rx) = rtrb::RingBuffer::new(n * 2);
        for i in 0..n {
            tx.push(i as f32 / 100.0).ok();
        }
        (tx, rx)
    }

    fn make_source(
        rx: rtrb::Consumer<f32>,
        state: Arc<SharedPipelineState>,
    ) -> (BufferedSource, Arc<SharedPipelineState>) {
        let src = BufferedSource::new(rx, 48000, 2, 1.0, state.clone());
        (src, state)
    }

    #[test]
    fn buffered_source_reads_samples_from_ring() {
        let (_tx, rx) = prefilled_ring(256);
        let state = SharedPipelineState::new();
        let (mut src, _) = make_source(rx, state);

        let mut buf = vec![0.0f32; 128];
        let n = src.fill_buffer(&mut buf);
        assert_eq!(n, 128);
        assert!(buf.iter().any(|&s| s > 0.0));
    }

    #[test]
    fn buffered_source_outputs_silence_when_seek_pending() {
        let (_tx, rx) = prefilled_ring(256);
        let state = SharedPipelineState::new();
        state.seek_pending.store(true, Ordering::Release);
        let (mut src, _) = make_source(rx, state);

        let mut buf = vec![1.0f32; 128];
        src.fill_buffer(&mut buf);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn buffered_source_exhausted_when_ring_empty_and_producer_done() {
        let (_tx, rx) = rtrb::RingBuffer::<f32>::new(64); // empty ring
        let state = SharedPipelineState::new();
        state.producer_done.store(true, Ordering::Release);
        let (mut src, _) = make_source(rx, state);

        let mut buf = vec![0.0f32; 64];
        src.fill_buffer(&mut buf);
        assert!(src.is_exhausted());
    }

    #[test]
    fn source_handle_position_from_playhead() {
        let state = SharedPipelineState::new();
        state.playhead.store(96000, Ordering::Release); // 1s at 48000 Hz stereo
        let handle = SourceHandle::new(state, 48000, 2, 5.0);
        // 96000 samples / 2 channels / 48000 Hz = 1.0 second
        assert!((handle.position_secs() - 1.0).abs() < 0.001);
    }

    #[test]
    fn source_handle_seekhead_none_when_not_pending() {
        let state = SharedPipelineState::new();
        state.seek_frame.store(96000, Ordering::Release);
        let handle = SourceHandle::new(state, 48000, 2, 5.0);
        assert!(handle.seekhead_secs().is_none());
    }

    #[test]
    fn source_handle_seekhead_some_when_pending() {
        let state = SharedPipelineState::new();
        state.seek_pending.store(true, Ordering::Release);
        state.seek_frame.store(48000, Ordering::Release); // 1s at 48000 Hz
        let handle = SourceHandle::new(state, 48000, 2, 5.0);
        let sh = handle.seekhead_secs();
        assert!(sh.is_some());
        assert!((sh.unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn source_handle_is_exhausted_reflects_atomic() {
        let state = SharedPipelineState::new();
        let handle = SourceHandle::new(state.clone(), 48000, 2, 5.0);
        assert!(!handle.is_exhausted());
        state.exhausted.store(true, Ordering::Release);
        assert!(handle.is_exhausted());
    }

    #[test]
    fn buffered_source_updates_playhead_atomic() {
        let (_tx, rx) = prefilled_ring(256);
        let state = SharedPipelineState::new();
        let (mut src, state) = make_source(rx, state);

        let mut buf = vec![0.0f32; 128];
        src.fill_buffer(&mut buf);
        assert_eq!(state.playhead.load(Ordering::Acquire), 128);
    }

    #[test]
    fn buffered_source_seek_resets_playhead() {
        let (_tx, rx) = prefilled_ring(256);
        let state = SharedPipelineState::new();
        state.seek_pending.store(true, Ordering::Release);
        state.seek_frame.store(48000, Ordering::Release); // seek to frame 48000
        state.playhead.store(9999, Ordering::Release);
        let mut src = BufferedSource::new(rx, 48000, 2, 5.0, state.clone());

        let mut buf = vec![0.0f32; 128];
        src.fill_buffer(&mut buf);
        // playhead = seek_frame * channels = 48000 * 2 = 96000
        assert_eq!(state.playhead.load(Ordering::Acquire), 96000);
    }
}
