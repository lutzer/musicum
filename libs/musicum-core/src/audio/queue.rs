use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use crate::audio::player::PlaybackEngine;
use crate::audio::registry::EditRegistry;
use crate::edit::ProcessorEdit;

pub struct QueueItem {
    pub title: String,
    pub path:  String,
    pub edits: Vec<ProcessorEdit>,
}

pub struct PlaybackQueue {
    items:         Vec<QueueItem>,
    current_index: usize,
    engine:        PlaybackEngine,
    registry:      Arc<EditRegistry>,
    device_name:   Option<String>,
}

impl PlaybackQueue {
    pub fn new(items: Vec<QueueItem>, registry: Arc<EditRegistry>, device_name: Option<String>) -> Result<Self> {
        if items.is_empty() {
            return Err(anyhow!("PlaybackQueue requires at least one item"));
        }
        let engine = PlaybackEngine::new(
            Path::new(&items[0].path),
            &items[0].edits,
            &registry,
            device_name.as_deref(),
        )?;
        engine.play();
        Ok(Self { items, current_index: 0, engine, registry, device_name })
    }

    pub fn engine(&self)         -> &PlaybackEngine     { &self.engine }
    pub fn engine_mut(&mut self) -> &mut PlaybackEngine { &mut self.engine }
    pub fn current_index(&self)  -> usize               { self.current_index }
    pub fn total(&self)          -> usize                { self.items.len() }
    pub fn current_title(&self)  -> &str                { &self.items[self.current_index].title }
    pub fn titles(&self)         -> Vec<&str>           { self.items.iter().map(|i| i.title.as_str()).collect() }
    pub fn current_edits(&self)  -> &[ProcessorEdit]    { &self.items[self.current_index].edits }
    pub fn has_any_edits(&self)  -> bool                { self.items.iter().any(|item| item.edits.iter().any(|e| e.enabled)) }

    pub fn next(&mut self) -> bool {
        if self.current_index + 1 >= self.items.len() {
            return false;
        }
        self.current_index += 1;
        self.replace_engine();
        true
    }

    /// If current position > 3 s: seek to 0.
    /// Otherwise go to previous clip if any; if already at 0 with low position: no-op (false).
    pub fn prev(&mut self) -> bool {
        if self.engine.position_secs() > 3.0 {
            self.engine.seek(0.0);
            return true;
        }
        if self.current_index == 0 {
            return false;
        }
        self.current_index -= 1;
        self.replace_engine();
        true
    }

    /// Call once per TUI tick. Returns `true` if the engine was advanced to the next clip.
    /// Returns `false` when the last clip has finished (queue exhausted).
    pub fn advance_if_finished(&mut self) -> bool {
        if !self.engine.is_finished() {
            return false;
        }
        if self.current_index + 1 >= self.items.len() {
            return false;
        }
        self.current_index += 1;
        self.replace_engine();
        true
    }

    fn replace_engine(&mut self) {
        let item = &self.items[self.current_index];
        if let Ok(eng) = PlaybackEngine::new(
            Path::new(&item.path),
            &item.edits,
            &self.registry,
            self.device_name.as_deref(),
        ) {
            eng.play();
            self.engine = eng;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::registry::EditRegistry;
    use crate::edit::{EditKind, ProcessorEdit};
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    fn enabled_structural_edit() -> ProcessorEdit {
        ProcessorEdit {
            uuid: Uuid::new_v4(),
            enabled: true,
            kind: EditKind::Structural {
                processor_id: "trim".to_string(),
                params: [("start".to_string(), 1.0_f64)].into(),
            },
        }
    }

    fn disabled_plugin_edit() -> ProcessorEdit {
        ProcessorEdit {
            uuid: Uuid::new_v4(),
            enabled: false,
            kind: EditKind::Plugin {
                plugin_id: "gain".to_string(),
                params: [("g".to_string(), 0.5_f32)].into(),
            },
        }
    }

    #[test]
    fn current_edits_returns_edits_for_current_item() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let edit = enabled_structural_edit();
        let items = vec![QueueItem {
            title: "a".to_string(),
            path: tmp.path().to_str().unwrap().to_string(),
            edits: vec![edit.clone()],
        }];
        let queue = PlaybackQueue::new(items, registry, None).unwrap();
        assert_eq!(queue.current_edits(), &[edit]);
    }

    #[test]
    fn has_any_edits_false_when_all_disabled() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem {
            title: "a".to_string(),
            path: tmp.path().to_str().unwrap().to_string(),
            edits: vec![disabled_plugin_edit()],
        }];
        let queue = PlaybackQueue::new(items, registry, None).unwrap();
        assert!(!queue.has_any_edits());
    }

    #[test]
    fn has_any_edits_true_when_one_enabled() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem {
            title: "a".to_string(),
            path: tmp.path().to_str().unwrap().to_string(),
            edits: vec![disabled_plugin_edit(), enabled_structural_edit()],
        }];
        let queue = PlaybackQueue::new(items, registry, None).unwrap();
        assert!(queue.has_any_edits());
    }

    fn temp_wav(frames: usize, sample_rate: u32) -> NamedTempFile {
        let tmp = NamedTempFile::new().unwrap();
        let spec = WavSpec { channels: 1, sample_rate, bits_per_sample: 32,
                             sample_format: SampleFormat::Float };
        let mut w = WavWriter::create(tmp.path(), spec).unwrap();
        for i in 0..frames { w.write_sample(i as f32 / frames as f32).unwrap(); }
        w.finalize().unwrap();
        tmp
    }

    #[test]
    fn new_single_item_sets_index_zero() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem {
            title: "track".to_string(),
            path: tmp.path().to_str().unwrap().to_string(),
            edits: vec![],
        }];
        let queue = PlaybackQueue::new(items, registry, None).unwrap();
        assert_eq!(queue.current_index(), 0);
        assert_eq!(queue.total(), 1);
        assert_eq!(queue.current_title(), "track");
    }

    #[test]
    fn new_empty_items_returns_error() {
        let registry = Arc::new(EditRegistry::default());
        let result = PlaybackQueue::new(vec![], registry, None);
        assert!(result.is_err());
    }

    #[test]
    fn next_advances_index() {
        let tmp1 = temp_wav(4410, 44_100);
        let tmp2 = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![
            QueueItem { title: "a".to_string(), path: tmp1.path().to_str().unwrap().to_string(), edits: vec![] },
            QueueItem { title: "b".to_string(), path: tmp2.path().to_str().unwrap().to_string(), edits: vec![] },
        ];
        let mut queue = PlaybackQueue::new(items, registry, None).unwrap();
        let moved = queue.next();
        assert!(moved);
        assert_eq!(queue.current_index(), 1);
        assert_eq!(queue.current_title(), "b");
    }

    #[test]
    fn next_at_last_returns_false() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem { title: "only".to_string(),
                                     path: tmp.path().to_str().unwrap().to_string(),
                                     edits: vec![] }];
        let mut queue = PlaybackQueue::new(items, registry, None).unwrap();
        assert!(!queue.next());
        assert_eq!(queue.current_index(), 0);
    }

    #[test]
    fn prev_at_start_with_low_position_returns_false() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem { title: "only".to_string(),
                                     path: tmp.path().to_str().unwrap().to_string(),
                                     edits: vec![] }];
        let mut queue = PlaybackQueue::new(items, registry, None).unwrap();
        // position is 0, index is 0: no-op
        assert!(!queue.prev());
    }

    #[test]
    fn prev_at_index_1_moves_back() {
        let tmp1 = temp_wav(4410, 44_100);
        let tmp2 = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![
            QueueItem { title: "a".to_string(), path: tmp1.path().to_str().unwrap().to_string(), edits: vec![] },
            QueueItem { title: "b".to_string(), path: tmp2.path().to_str().unwrap().to_string(), edits: vec![] },
        ];
        let mut queue = PlaybackQueue::new(items, registry, None).unwrap();
        queue.next();
        let moved = queue.prev();
        assert!(moved);
        assert_eq!(queue.current_index(), 0);
    }

    #[test]
    fn new_with_none_device_works() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem {
            title: "t".to_string(),
            path: tmp.path().to_str().unwrap().to_string(),
            edits: vec![],
        }];
        let result = PlaybackQueue::new(items, registry, None);
        assert!(result.is_ok());
    }

    #[test]
    fn new_with_bad_device_name_returns_error() {
        let tmp = temp_wav(4410, 44_100);
        let registry = Arc::new(EditRegistry::default());
        let items = vec![QueueItem {
            title: "t".to_string(),
            path: tmp.path().to_str().unwrap().to_string(),
            edits: vec![],
        }];
        let result = PlaybackQueue::new(items, registry, Some("__no_such_device__".to_string()));
        assert!(result.is_err());
    }
}
