use crate::edit::{ProcessorEdit};

pub struct QueueItem {
    pub title: String,
    pub path:  String,
    pub edits: Vec<ProcessorEdit>,
}

pub struct PlaybackQueue {
    items:         Vec<QueueItem>,
    current_index: usize,
}

impl PlaybackQueue {
    fn next(&mut self) -> Option<&QueueItem> {
        if self.current_index < self.length() - 2 {
            self.current_index += 1;
            return Some(&self.items[self.current_index]);
        } else {
            return None
        }
    }

    fn previous(&mut self) -> Option<&QueueItem> {
         if self.current_index > 0 {
            self.current_index -= 1;
            return Some(&self.items[self.current_index]);
        } else {
            return None
        }
    }

    fn length(&self) -> usize { self.items.len() }
}