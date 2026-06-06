#[cfg(test)]
mod source_tests {
    use crate::audio::source::{AudioSource, SymphoniaSource};

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
                let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
                let s = (sample * i16::MAX as f32) as i16;
                writer.write_sample(s).unwrap();
                writer.write_sample(s).unwrap();
            }
            writer.finalize().unwrap();
        }
        path
    }

    #[test]
    fn constructs_from_wav() {
        let path = test_wav_path();
        let src = SymphoniaSource::new(&path, 48000, 2).unwrap();
        assert_eq!(src.sample_rate(), 48000);
        assert_eq!(src.channels(), 2);
        assert!(!src.is_exhausted());
        assert!(src.duration() > 0.9 && src.duration() < 1.1);
    }

    #[test]
    fn fill_buffer_returns_samples() {
        let path = test_wav_path();
        let mut src = SymphoniaSource::new(&path, 48000, 2).unwrap();
        let mut buf = vec![0.0f32; 1024];
        let written = src.fill_buffer(&mut buf);
        assert!(written > 0);
        assert!(buf[..written].iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn exhausts_after_full_drain() {
        let path = test_wav_path();
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
        let path = test_wav_path();
        let mut src = SymphoniaSource::new(&path, 44100, 2).unwrap();
        let mut buf = vec![0.0f32; 512];
        let written = src.fill_buffer(&mut buf);
        assert!(written > 0);
    }

    #[test]
    #[ignore]
    fn plays_one_second_of_audio() {
        use crate::audio::output::{AudioOutput, CpalOutput};
        let path = test_wav_path();
        let mut output = CpalOutput::new().unwrap();
        let src = SymphoniaSource::new(&path, output.sample_rate(), output.channels()).unwrap();
        output.set_source(Box::new(src)).unwrap();
        output.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        output.pause().unwrap();
    }
}
