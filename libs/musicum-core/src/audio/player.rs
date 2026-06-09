use std::path::Path;

use crate::audio::engine::AudioEngine;
use crate::PlaybackQueue;

pub struct AudioPlayer {
    queue:   PlaybackQueue,
    engine:  Box<dyn AudioEngine>,
    looping: bool,
}

impl AudioPlayer {
    pub fn new(queue: PlaybackQueue, engine: Box<dyn AudioEngine>) -> Self {
        Self { queue, engine, looping: false }
    }

    pub fn prepare(&mut self) -> anyhow::Result<()> {
        self.load_current()
    }

    fn load_current(&mut self) -> anyhow::Result<()> {
        let path = Path::new(&self.queue.current_item().path).to_path_buf();
        self.engine.load(&path)
    }

    pub fn play(&mut self)  { 
        if !self.engine.is_exhausted() {
            let _ = self.engine.play(); 
        }
    }
    pub fn pause(&mut self) { let _ = self.engine.pause(); }

    pub fn seek(&mut self, time: f64) {
        self.engine.seek(time);
    }

    pub fn set_volume(&mut self, _volume: f32) {}
    pub fn set_looping(&mut self, looping: bool) { self.looping = looping; }

    pub fn next(&mut self) -> bool {
        if self.queue.next().is_some() {
            let _ = self.load_current();
            return true
        }
        false
    }

    pub fn previous(&mut self) -> bool {
        if self.queue.previous().is_some() {
            let _ = self.load_current();
            return true
        }
        false
    }

    pub fn queue(&self)          -> &PlaybackQueue { &self.queue }
    pub fn position_secs(&self)  -> f64         { self.engine.position_secs() }
    pub fn seekhead_secs(&self)  -> Option<f64> { self.engine.seekhead_secs() }
    pub fn duration_secs(&self)  -> f64         { self.engine.duration_secs() }
    pub fn is_paused(&self)     -> bool { !self.engine.is_playing() }
    pub fn is_looping(&self)    -> bool { self.looping }
    pub fn file_ended(&self)    -> bool { self.engine.is_exhausted() }
}
