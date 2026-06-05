use crate::PlaybackQueue;
use crate::audio::engine::AudioEngine;

struct Player<'a> {
    queue: &'a PlaybackQueue,
    engine: &'a dyn AudioEngine,
    looping: bool,
    volume: f32,
    position: f64,
}

impl<'a> Player<'a> {
    fn new(queue: &'a PlaybackQueue, engine: &'a dyn AudioEngine) -> Player<'a> {
        return Player { queue, engine, looping: false, volume: 1.0, position: 0.0 }
    }

    fn play(&mut self) {
        
    }

    fn pause(&mut self) {

    }

    fn seek(&mut self, time: f64) {
        self.position = time;
    }

    fn set_volume(&mut self, volume: f32) {

    }

    fn set_looping(&mut self, looping: bool) {

    }
}