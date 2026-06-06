use crate::{PlaybackQueue, PlaybackQueueItem};
use crate::audio::engine::AudioEngine;

struct Player<'a> {
    queue: PlaybackQueue,
    engine: &'a dyn AudioEngine,
    looping: bool,
    volume: f32,
    position: f64,
}

impl<'a> Player<'a> {
    pub fn from_queue(queue: PlaybackQueue, engine: &'a dyn AudioEngine) -> Player<'a> {
        return Player { queue, engine, looping: false, volume: 1.0, position: 0.0 }
    }

    pub fn from_item(item: PlaybackQueueItem, engine: &'a dyn AudioEngine) -> Player<'a> {
        return Player { queue: PlaybackQueue::new(vec![item]), engine, looping: false, volume: 1.0, position: 0.0 }
    }

    pub fn prepare(&mut self) {
        
    }

    pub fn play(&mut self) {
        
    }

    pub fn pause(&mut self) {

    }

    pub fn seek(&mut self, time: f64) {
        self.position = time;
    }

    pub fn set_volume(&mut self, volume: f32) {

    }

    pub fn set_looping(&mut self, looping: bool) {

    }

    pub fn next(&mut self) {
        let item = self.queue.next();

    }

    pub fn previous(&mut self) {
        let item = self.queue.previous();
    }

}