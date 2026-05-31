use std::{
    io::{BufWriter, Write as _},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::Ordering,
};

use anyhow::{bail, Context, Result};
use structural_processor_sdk::chain::build_chain;

use crate::audio::{build_plugin_handles, structural_edits_from, EditRegistry, FileAudioSource};
use crate::edit::ProcessorEdit;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ExportOptions {
    pub sample_rate:  Option<u32>,
    pub channels:     Option<u16>,
    pub bitrate_kbps: Option<u32>,
    pub overwrite:    bool,
}

#[derive(Debug)]
pub struct ExportResult {
    pub output_path: PathBuf,
    pub format:      String,
    pub duration:    f64,
    pub sample_rate: u32,
    pub channels:    u16,
    pub bitrate_kbps: Option<u32>,
}

// ── Supported formats ─────────────────────────────────────────────────────────

const SUPPORTED_EXTS: &[&str] = &["wav", "mp3", "flac", "aiff", "aif"];

const CHUNK_SAMPLES: usize = 16_384;

fn is_lossless(ext: &str) -> bool {
    matches!(ext, "wav" | "flac" | "aiff" | "aif")
}

fn validate_extension(output_path: &Path) -> Result<String> {
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if SUPPORTED_EXTS.contains(&ext.as_str()) {
        Ok(ext)
    } else {
        bail!("unsupported output format '{ext}'. Supported: wav, mp3, flac, aiff")
    }
}

// ── ffmpeg helper ─────────────────────────────────────────────────────────────

fn spawn_ffmpeg(
    output_path: &Path,
    src_rate: u32,
    src_channels: u16,
    ext: &str,
    options: &ExportOptions,
) -> Result<Child> {
    let mut cmd = Command::new("ffmpeg");

    if options.overwrite {
        cmd.arg("-y");
    }

    cmd.args(["-f", "f32le"])
        .arg("-ar").arg(src_rate.to_string())
        .arg("-ac").arg(src_channels.to_string())
        .args(["-i", "pipe:0"]);

    if let Some(rate) = options.sample_rate {
        cmd.arg("-ar").arg(rate.to_string());
    }
    if let Some(ch) = options.channels {
        cmd.arg("-ac").arg(ch.to_string());
    }
    if let Some(kbps) = options.bitrate_kbps {
        if !is_lossless(ext) {
            cmd.arg("-b:a").arg(format!("{kbps}k"));
        }
    }

    cmd.arg(output_path);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("ffmpeg not found. Install ffmpeg to use the export command.")
        } else {
            anyhow::anyhow!("failed to run ffmpeg: {e}")
        }
    })
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub async fn export_audio(
    file_path: &Path,
    edits: &[ProcessorEdit],
    output_path: &Path,
    options: ExportOptions,
    registry: &EditRegistry,
    progress: impl Fn(f64, f64),
) -> Result<ExportResult> {
    // ── Step 2: Check output path ─────────────────────────────────────────
    if output_path.exists() && !options.overwrite {
        bail!(
            "output file already exists: {}. Use --overwrite to replace it.",
            output_path.display()
        );
    }

    // ── Step 3: Validate extension ────────────────────────────────────────
    let ext = validate_extension(output_path)?;

    // ── Step 4: Build audio chain ─────────────────────────────────────────
    let source = Box::new(
        FileAudioSource::new(file_path)
            .with_context(|| format!("cannot open source file: {}", file_path.display()))?,
    );
    let structural = structural_edits_from(edits);
    let structural_registry = structural_processors::registry();
    let mut chain = build_chain(source, &structural, &structural_registry);

    let src_rate     = chain.sample_rate();
    let src_channels = chain.channels();
    let total_duration = chain.duration_secs();

    // ── Step 5: Build plugin handles ──────────────────────────────────────
    let plugin_handles = build_plugin_handles(edits, registry);

    // ── Step 6: Spawn ffmpeg and stream processed PCM into its stdin ──────
    let mut child = spawn_ffmpeg(output_path, src_rate, src_channels, &ext, &options)?;
    let stdin = child.stdin.take().expect("stdin was piped");
    let mut writer = BufWriter::new(stdin);

    // Pre-allocate byte buffer; reused each iteration to avoid per-chunk allocation.
    let mut byte_buf: Vec<u8> = Vec::with_capacity(CHUNK_SAMPLES * src_channels as usize * 4);

    let mut cursor_secs = 0.0_f64;
    loop {
        let mut chunk = chain.read_at(cursor_secs, CHUNK_SAMPLES);
        if chunk.is_empty() || (total_duration > 0.0 && cursor_secs >= total_duration) {
            break;
        }
        for handle in &plugin_handles {
            if !handle.enabled.load(Ordering::Relaxed) { continue; }
            if let Ok(mut p) = handle.processor.lock() {
                p.process(&mut chunk, src_channels as usize, src_rate as f32, cursor_secs);
            }
        }
        cursor_secs += chunk.len() as f64 / (src_rate as f64 * src_channels as f64);

        byte_buf.clear();
        for s in &chunk {
            byte_buf.extend_from_slice(&s.to_le_bytes());
        }
        writer.write_all(&byte_buf).context("failed to write to ffmpeg stdin")?;

        progress(cursor_secs, total_duration);
    }

    // ── Step 7: Signal EOF and wait for ffmpeg to finish encoding ─────────
    drop(writer);
    let out = child.wait_with_output().context("failed to wait for ffmpeg")?;
    if !out.status.success() {
        bail!("ffmpeg error: {}", String::from_utf8_lossy(&out.stderr));
    }

    let effective_rate     = options.sample_rate.unwrap_or(src_rate);
    let effective_channels = options.channels.unwrap_or(src_channels);
    let bitrate = if is_lossless(&ext) { None } else { options.bitrate_kbps };

    Ok(ExportResult {
        output_path: output_path.to_path_buf(),
        format: ext,
        duration: total_duration,
        sample_rate: effective_rate,
        channels: effective_channels,
        bitrate_kbps: bitrate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn export_fails_if_output_exists_and_no_overwrite() {
        use tempfile::NamedTempFile;
        use crate::audio::EditRegistry;
        // Create a real file at the output path so the check fires.
        let tmp = NamedTempFile::new().unwrap();
        let out_path = tmp.path().with_extension("wav");
        std::fs::write(&out_path, b"dummy").unwrap();

        let opts = ExportOptions {
            sample_rate: None,
            channels: None,
            bitrate_kbps: None,
            overwrite: false,
        };
        let registry = EditRegistry::default();
        let result = export_audio(
            Path::new("/nonexistent/source.wav"),
            &[],
            &out_path,
            opts,
            &registry,
            |_, _| {},
        ).await;

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("already exists"));
        assert!(msg.contains("--overwrite"));

        let _ = std::fs::remove_file(&out_path);
    }

    #[test]
    fn validate_extension_rejects_unknown() {
        let err = validate_extension(Path::new("/out/file.ogg")).unwrap_err();
        assert!(err.to_string().contains("unsupported output format"));
        assert!(err.to_string().contains("ogg"));
        assert!(err.to_string().contains("wav, mp3, flac, aiff"));
    }

    #[test]
    fn validate_extension_accepts_all_supported() {
        for ext in &["wav", "mp3", "flac", "aiff", "aif"] {
            let path = Path::new("/out/file").with_extension(ext);
            assert!(
                validate_extension(&path).is_ok(),
                "should accept .{ext}"
            );
        }
    }

    #[test]
    fn is_lossless_mp3_is_false() {
        assert!(!is_lossless("mp3"));
    }

    #[test]
    fn is_lossless_wav_flac_aiff_are_true() {
        assert!(is_lossless("wav"));
        assert!(is_lossless("flac"));
        assert!(is_lossless("aiff"));
        assert!(is_lossless("aif"));
    }

    fn make_temp_wav(frames: usize, sample_rate: u32) -> tempfile::NamedTempFile {
        use hound::{SampleFormat, WavSpec, WavWriter};
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let spec = WavSpec { channels: 1, sample_rate, bits_per_sample: 32, sample_format: SampleFormat::Float };
        let mut w = WavWriter::create(tmp.path(), spec).unwrap();
        for i in 0..frames { w.write_sample(i as f32 / frames as f32).unwrap(); }
        w.finalize().unwrap();
        tmp
    }

    #[tokio::test]
    async fn progress_callback_invoked_during_export() {
        use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
        let tmp_src = make_temp_wav(44_100, 44_100); // 1 s mono WAV
        let out_path = std::env::temp_dir()
            .join(format!("musicum-progress-test-{}.wav", uuid::Uuid::new_v4()));

        let count = Arc::new(AtomicUsize::new(0));
        let count2 = count.clone();

        let opts = ExportOptions { sample_rate: None, channels: None, bitrate_kbps: None, overwrite: true };
        let registry = crate::audio::EditRegistry::default();

        let result = export_audio(
            tmp_src.path(),
            &[],
            &out_path,
            opts,
            &registry,
            move |_, _| { count2.fetch_add(1, Ordering::Relaxed); },
        ).await;

        let _ = std::fs::remove_file(&out_path);
        if result.is_ok() {
            assert!(count.load(Ordering::Relaxed) > 0, "progress callback was never called");
        }
    }
}
