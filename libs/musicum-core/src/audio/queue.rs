use crate::edit::{ProcessorEdit};

pub struct PlaybackQueueItem {
    pub title: String,
    pub path:  String,
    pub edits: Vec<ProcessorEdit>,
}

pub struct PlaybackQueue {
    items:         Vec<PlaybackQueueItem>,
    current_index: usize,
}

impl PlaybackQueue {

    pub fn new(items: Vec<PlaybackQueueItem>) -> PlaybackQueue {
        return PlaybackQueue {
            items, current_index: 0
        }
    }

    pub fn next(&mut self) -> Option<&PlaybackQueueItem> {
        if self.current_index < self.length() - 1 {
            self.current_index += 1;
            return Some(&self.items[self.current_index]);
        } else {
            return None
        }
    }

    pub fn previous(&mut self) -> Option<&PlaybackQueueItem> {
         if self.current_index > 0 {
            self.current_index -= 1;
            return Some(&self.items[self.current_index]);
        } else {
            return None
        }
    }

    pub fn length(&self) -> usize { self.items.len() }

    pub fn current_index(&self) -> usize { self.current_index }

    pub fn current_item(&self) -> &PlaybackQueueItem { &self.items[self.current_index] }

    pub fn items(&self) -> &Vec<PlaybackQueueItem> { &self.items }
}