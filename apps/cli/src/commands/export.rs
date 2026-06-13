use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Args;
use musicum_core::{
    edit::ProcessorEdit,
    services::{clip_service, file_service},
};
use sea_orm::DatabaseConnection;

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

pub async fn run(_db: &DatabaseConnection, _args: ExportArgs) -> Result<()> {
    todo!("to be implemented")
}

#[allow(dead_code)]
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
