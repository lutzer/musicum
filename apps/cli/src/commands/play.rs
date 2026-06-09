use std::{ffi::OsStr, io, path::PathBuf, time::Duration};

use anyhow::{Result, anyhow, bail};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use musicum_core::{
    AudioPlayer, CpalEngine, PlaybackQueue, PlaybackQueueItem,
    ProcessorEdit,
    services::{clip_service, collection_service, file_service},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame, Terminal, TerminalOptions, Viewport,
};
use sea_orm::DatabaseConnection;

const MAX_QUEUE_VISIBLE: usize = 6;

pub async fn run(
    db: &DatabaseConnection,
    target: Option<String>,
    force_file: bool,
    force_clip: bool,
    force_collection: bool,
    loop_mode: bool,
    device: Option<String>,
) -> Result<()> {

    let target = target.ok_or_else(|| anyhow!("provide a target or --collection <slug>"))?;

    if let Ok(queue) = resolve_target(db, &target, force_file, force_clip, force_collection).await {
        // return play_single_clip(path, edits, loop_mode);
        return run_player(queue);
    }

    Err(anyhow!(
        "'{target}' is not a known file, clip, or collection slug, or an existing file path"
    ))
}


/// Finds out which file, clip or collection could be targeted by the target string
async fn resolve_target(
    db: &DatabaseConnection,
    target: &str,
    force_file: bool,
    force_clip: bool,
    force_collection: bool,
) -> Result<PlaybackQueue> {
    
    if force_clip {
        return Ok(get_clip_queue(&db, target).await?);
    }

    if force_file {
        return Ok(get_file_queue(&db, target).await?);
    }

    if force_collection {
        return Ok(get_collection_queue(&db, target).await?);
    }

    if let Ok(queue) = get_clip_queue(&db, target).await {
        return Ok(queue);
    }

    if let Ok(queue) = get_collection_queue(&db, target).await {
        return Ok(queue);
    }

    if let Ok(queue) = get_file_queue(&db, target).await {
        return Ok(queue);
    }

    let path = PathBuf::from(target);
    if path.exists() {
        return Ok(PlaybackQueue::new(vec![
            PlaybackQueueItem {
                title: path.file_name().unwrap_or(OsStr::new("unknown")).to_string_lossy().into_owned(),
                path: path.to_string_lossy().into_owned(),
                edits: vec![]
            }
        ]));
    }
    Err(anyhow!("'{target}' is not a known slug or an existing file path"))
}

async fn get_clip_queue(db: &DatabaseConnection, target: &str) -> Result<PlaybackQueue> {
    let clip = clip_service::get_clip_by_slug(db, target)
        .await
        .map_err(|_| anyhow!("no clip with slug '{target}'"))?;
    let file = file_service::get_file_by_id(db, &clip.file_id)
        .await
        .map_err(|_| anyhow!("parent file for clip '{target}' not found"))?;
    let edits = clip.processors.0;
    return Ok(PlaybackQueue::new(vec![
        PlaybackQueueItem {
            title: clip.title,
            path: file.path,
            edits: edits
        }
    ]));
}

async fn get_file_queue(db: &DatabaseConnection, target: &str) -> Result<PlaybackQueue> {
    let file = file_service::get_file_by_slug(db, target)
        .await
        .map_err(|_| anyhow!("no file with slug '{target}'"))?;
    return Ok(PlaybackQueue::new(vec![
        PlaybackQueueItem {
            title: file.name,
            path: file.path,
            edits: vec![]
        }
    ]));
}

async fn get_collection_queue(db: &DatabaseConnection, target: &str) -> Result<PlaybackQueue> {
    let (collection,clips) = collection_service::get_collection_with_clips(db, target)
        .await
        .map_err(|_| anyhow!("no collection with slug '{target}'"))?;
    let mut items = Vec::new();
    for c in &clips {
        let file = file_service::get_file_by_id(&db, &c.file_id).await
            .map_err(|_| anyhow!("parent file for clip '{}' not found", c.title))?;
        items.push(PlaybackQueueItem {
            title: c.title.clone(),
            path: file.path,
            edits: c.processors.0.clone(),
        });
    }
    if items.len() == 0 { bail!("collection does not contain any clips") }
    return Ok(PlaybackQueue::new(items));
}

fn run_player(
    queue: PlaybackQueue
) -> Result<()> {
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());

    let show_edits_row = true;

    let queue_rows = queue.length().min(MAX_QUEUE_VISIBLE) as u16;
    let base_height: u16 = 5 + 2  // status, bar, time, hints, separator + top/bottom border
        + u16::from(show_edits_row)
        + queue_rows;

    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions { viewport: Viewport::Inline(base_height) },
    )?;

    let mut queue_scroll: usize = 0;

    let engine = CpalEngine::new()?;
    let mut player = AudioPlayer::new(queue, Box::new(engine));
    player.prepare()?;
    player.play();

    loop {
        let ci = player.queue().current_index();
        if ci < queue_scroll {
            queue_scroll = ci;
        } else if ci >= queue_scroll + MAX_QUEUE_VISIBLE {
            queue_scroll = ci + 1 - MAX_QUEUE_VISIBLE;
        }

        terminal.draw(|f| draw(f, &player, show_edits_row, false, queue_scroll))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    (KeyCode::Char('p'), _) => {
                        if player.file_ended() {
                            player.seek(0.0);
                            player.play();
                        } else if player.is_paused() {
                            player.play();
                        } else {
                            player.pause();
                        }
                    }
                    (KeyCode::Char('l'), _) => player.set_looping(!player.is_looping()),
                    (KeyCode::Char('s'), _) => player.seek(0.0),
                    (KeyCode::Up, KeyModifiers::NONE) => {
                        player.previous();
                    }
                    (KeyCode::Down, KeyModifiers::NONE) => {
                        player.next();
                    }
                    (KeyCode::Right, KeyModifiers::NONE) => {
                        // let pos = queue.engine().position_secs();
                        player.seek((player.seekhead_secs().unwrap_or(player.position_secs()) + 3.0).min(player.duration_secs()));
                    }
                    (KeyCode::Left, KeyModifiers::NONE) => {
                        player.seek((player.seekhead_secs().unwrap_or(player.position_secs()) - 3.0).max(0.0));
                    }
                    (KeyCode::Right, KeyModifiers::SHIFT) => {
                        player.seek((player.seekhead_secs().unwrap_or(player.position_secs()) + 15.0).min(player.duration_secs()));
                    }
                    (KeyCode::Left, KeyModifiers::SHIFT) => {
                        player.seek((player.seekhead_secs().unwrap_or(player.position_secs()) - 15.0).max(0.0));
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

fn draw(
    f: &mut Frame,
    player: &AudioPlayer,
    show_edits_row: bool,
    queue_exhausted: bool,
    queue_scroll: usize,
) {
    let queue = player.queue();
    let pos = player.position_secs();
    let seek_pos = player.seekhead_secs();
    let dur = player.duration_secs();
    let queue_rows = queue.length().min(MAX_QUEUE_VISIBLE);

    let mut constraints = vec![
        Constraint::Length(1), // status + title
        Constraint::Length(1), // progress bar
        Constraint::Length(1), // time row
        Constraint::Length(1), // hints
    ];
    if show_edits_row {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Length(1)); // separator
    for _ in 0..queue_rows {
        constraints.push(Constraint::Length(1));
    }

    let border = Block::default().borders(Borders::ALL).padding(Padding::horizontal(1));
    f.render_widget(border.clone(), f.area());
    let inner = border.inner(f.area());
    let areas = Layout::vertical(constraints).split(inner);
    let mut area_idx = 0usize;

    // status + title
    let (status_icon, status_color) = if queue_exhausted {
        ("■", Color::DarkGray)
    } else if player.is_paused() {
        ("⏸", Color::Yellow)
    } else {
        ("▶", Color::Green)
    };
    let title_line = Line::from(vec![
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::raw("  "),
        Span::styled(queue.current_item().title.clone(), Style::default().add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(title_line), areas[area_idx]);
    area_idx += 1;

    // progress bar
    let ratio     = if dur > 0.0 { (pos / dur).clamp(0.0, 1.0) } else { 0.0 };
    let bar_width = areas[area_idx].width as usize;
    let filled    = (ratio * bar_width as f64).floor() as usize;
    let bar_str = if let Some(sh) = seek_pos {
        let seek_col = if dur > 0.0 {
            ((sh / dur).clamp(0.0, 1.0) * bar_width as f64).floor() as usize
        } else { 0 };
        let mut chars: Vec<char> = std::iter::repeat('█').take(filled)
            .chain(std::iter::repeat('░').take(bar_width.saturating_sub(filled)))
            .collect();
        if seek_col < chars.len() { chars[seek_col] = '┃'; }
        chars.into_iter().collect()
    } else {
        format!("{}{}", "█".repeat(filled), "░".repeat(bar_width.saturating_sub(filled)))
    };
    f.render_widget(
        Paragraph::new(bar_str).style(Style::default().fg(Color::White)),
        areas[area_idx],
    );
    area_idx += 1;

    // time row
    let elapsed = fmt_duration(pos);
    let total = fmt_duration(dur);
    let time_width = areas[area_idx].width as usize;
    let gap = time_width.saturating_sub(elapsed.len() + total.len());
    let time_str = format!("{}{}{}", elapsed, " ".repeat(gap), total);
    f.render_widget(
        Paragraph::new(time_str).style(Style::default().fg(Color::DarkGray)),
        areas[area_idx],
    );
    area_idx += 1;

    // hints
    let loop_color = if player.is_looping() { Color::Cyan } else { Color::DarkGray };
    let hints = Line::from(vec![
        Span::styled("[p] pause  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[l] loop  ", Style::default().fg(loop_color)),
        Span::styled("[↑↓] skip  [s] |←  [←/→] 3s  [S←/S→] 15s  [q] quit", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(hints), areas[area_idx]);
    area_idx += 1;

    // edits row
    if show_edits_row {
        let edit_str = fmt_edit_row(&queue.current_item().edits);
        f.render_widget(
            Paragraph::new(edit_str).style(Style::default().fg(Color::DarkGray)),
            areas[area_idx],
        );
        area_idx += 1;
    }

    // separator
    let sep = "─".repeat(areas[area_idx].width as usize);
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::DarkGray)),
        areas[area_idx],
    );
    area_idx += 1;

    // queue list
    let items = queue.items();
    let visible = &items[queue_scroll..(queue_scroll + queue_rows).min(items.len())];
    let ci = queue.current_index();
    for (i, item) in visible.iter().enumerate() {
        let abs_idx = queue_scroll + i;
        let title = item.title.clone();
        let line = if abs_idx == ci {
            Line::from(vec![
                Span::raw("▶ "),
                Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(vec![Span::raw(format!("  {title}"))])
        };
        f.render_widget(Paragraph::new(line), areas[area_idx]);
        area_idx += 1;
    }
}

fn fmt_duration(secs: f64) -> String {
    let s = secs as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

fn fmt_edit_row(edits: &Vec<ProcessorEdit>) -> String {
    edits
        .iter()
        .filter(|e| e.enabled)
        .map(|e| {
            let processor_id = &e.processor_id;
            let mut parts: Vec<String> = e.params.iter().map(|(k, v)| format!("{k}={v:.2}s")).collect();
            parts.sort();
            if parts.is_empty() {
                processor_id.clone()
            } else {
                format!("{processor_id} {}", parts.join(" "))
            }
        })
        .collect::<Vec<_>>()
        .join("  ")
}