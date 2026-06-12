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
        PlaybackQueue {
            items, current_index: 0
        }
    }

    pub fn next_item(&mut self) -> Option<&PlaybackQueueItem> {
        if self.current_index < self.length() - 1 {
            self.current_index += 1;
            Some(&self.items[self.current_index])
        } else {
            None
        }
    }

    pub fn previous_item(&mut self) -> Option<&PlaybackQueueItem> {
         if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.items[self.current_index])
        } else {
            None
        }
    }

    pub fn length(&self) -> usize { self.items.len() }

    pub fn current_index(&self) -> usize { self.current_index }

    pub fn current_item(&self) -> &PlaybackQueueItem { &self.items[self.current_index] }

    pub fn items(&self) -> &Vec<PlaybackQueueItem> { &self.items }
}