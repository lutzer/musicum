use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use musicum_core::{
    edit::ProcessorEdit,
    EditRegistry, ProcessorRegistry,
    services::{
        clip_service, file_service,
        export_service::{export_audio, ExportOptions},
    },
};
use sea_orm::DatabaseConnection;

use crate::output::{DetailItem, print_result};

#[derive(Args)]
pub struct ExportArgs {
    /// File or clip slug to export (auto-detects file first, then clip)
    pub slug: String,

    /// Destination file path; format inferred from extension (.wav .mp3 .flac .aiff .aif)
    pub output: PathBuf,

    /// Resolve slug as a file (no processors applied)
    #[arg(long, conflicts_with = "clip")]
    pub file: bool,

    /// Resolve slug as a clip (processors applied)
    #[arg(long, conflicts_with = "file")]
    pub clip: bool,

    /// Resample output to this sample rate (e.g. 44100)
    #[arg(long)]
    pub samplerate: Option<u32>,

    /// Remix to this channel count (1=mono, 2=stereo)
    #[arg(long)]
    pub channels: Option<u16>,

    /// Target bitrate in kbps for lossy formats (e.g. 192); ignored for lossless
    #[arg(long)]
    pub bitrate: Option<u32>,

    /// Overwrite output file if it already exists
    #[arg(long)]
    pub overwrite: bool,
}

pub async fn run(db: &DatabaseConnection, args: ExportArgs) -> Result<()> {
    let (file_path, edits) = resolve_target(db, &args.slug, args.file, args.clip).await?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Exporting {} → {}", args.slug, args.output.display()));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    let pb2 = pb.clone();
    let options = ExportOptions {
        sample_rate:  args.samplerate,
        channels:     args.channels,
        bitrate_kbps: args.bitrate,
        overwrite:    args.overwrite,
    };
    let mut proc_reg = ProcessorRegistry::new();
    proc_reg.load_dir(&musicum_core::config::Config::get().processors.processor_dir).ok();
    let registry = EditRegistry::new(std::sync::Arc::new(proc_reg));

    let result = export_audio(
        &file_path,
        &edits,
        &args.output,
        options,
        registry.registry(),
        move |cursor_secs, total_secs| {
            if total_secs > 0.0 && pb2.length().is_none() {
                pb2.set_length((total_secs * 1000.0) as u64);
                pb2.set_style(
                    ProgressStyle::with_template(
                        "{spinner:.green} {msg} [{bar:40}] {percent}% ({elapsed}/{eta})"
                    )
                    .unwrap()
                    .progress_chars("█░"),
                );
            }
            if total_secs > 0.0 {
                pb2.set_position((cursor_secs * 1000.0) as u64);
            }
        },
    ).await?;

    pb.finish_and_clear();

    let mut items = vec![
        DetailItem::Field("slug",     args.slug.clone()),
        DetailItem::Field("output",   result.output_path.display().to_string()),
        DetailItem::Field("format",   result.format.clone()),
        DetailItem::Field("duration", format!("{:.3}s", result.duration)),
        DetailItem::Field("rate",     format!("{}Hz", result.sample_rate)),
        DetailItem::Field("channels", result.channels.to_string()),
    ];
    if let Some(kbps) = result.bitrate_kbps {
        items.push(DetailItem::Field("bitrate", format!("{kbps}kbps")));
    }

    print_result("Exported", &items);
    Ok(())
}

async fn resolve_target(
    db: &DatabaseConnection,
    target: &str,
    force_file: bool,
    force_clip: bool,
) -> Result<(PathBuf, Vec<ProcessorEdit>)> {
    if force_file {
        let file = file_service::get_file_by_slug(db, target)
            .await
            .map_err(|_| anyhow!("no file with slug '{target}'"))?;
        return Ok((PathBuf::from(file.path), vec![]));
    }

    if force_clip {
        let clip = clip_service::get_clip_by_slug(db, target)
            .await
            .map_err(|_| anyhow!("no clip with slug '{target}'"))?;
        let file = file_service::get_file_by_id(db, &clip.file_id)
            .await
            .map_err(|_| anyhow!("parent file for clip '{target}' not found"))?;
        let edits = clip.processors.0;
        return Ok((PathBuf::from(file.path), edits));
    }

    if let Ok(file) = file_service::get_file_by_slug(db, target).await {
        return Ok((PathBuf::from(file.path), vec![]));
    }
    if let Ok(clip) = clip_service::get_clip_by_slug(db, target).await {
        if let Ok(file) = file_service::get_file_by_id(db, &clip.file_id).await {
            let edits = clip.processors.0;
            return Ok((PathBuf::from(file.path), edits));
        }
    }

    Err(anyhow!("'{target}' is not a known file or clip slug"))
}
