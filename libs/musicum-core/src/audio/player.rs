use crate::{PlaybackQueue, PlaybackQueueItem};
use crate::audio::output::AudioOutput;

struct Player<'a> {
    queue:    PlaybackQueue,
    output:   &'a mut dyn AudioOutput,
    looping:  bool,
    volume:   f32,
    position: f64,
}

impl<'a> Player<'a> {
    pub fn from_queue(queue: PlaybackQueue, output: &'a mut dyn AudioOutput) -> Player<'a> {
        Player { queue, output, looping: false, volume: 1.0, position: 0.0 }
    }

    pub fn from_item(item: PlaybackQueueItem, output: &'a mut dyn AudioOutput) -> Player<'a> {
        Player { queue: PlaybackQueue::new(vec![item]), output, looping: false, volume: 1.0, position: 0.0 }
    }

    pub fn play(&mut self) { let _ = self.output.play(); }
    pub fn pause(&mut self) { let _ = self.output.pause(); }
    pub fn seek(&mut self, time: f64) { self.position = time; }
    pub fn set_volume(&mut self, _volume: f32) {}
    pub fn set_looping(&mut self, looping: bool) { self.looping = looping; }

    pub fn next(&mut self) {
        let _item = self.queue.next();
    }

    pub fn previous(&mut self) {
        let _item = self.queue.previous();
    }
}
