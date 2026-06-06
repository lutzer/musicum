use std::path::Path;

use crate::audio::source::{AudioSource, SymphoniaSource};
use crate::{PlaybackQueue, PlaybackQueueItem};
use crate::audio::output::AudioOutput;

pub struct AudioPlayer<'a> {
    queue:    PlaybackQueue,
    source: Option<&'a mut dyn AudioSource>,
    output:   &'a mut dyn AudioOutput,
    looping:  bool,
    volume:   f32,
    position: f64,
}

impl<'a> AudioPlayer<'a> {
    pub fn from_queue(queue: PlaybackQueue, output: &'a mut dyn AudioOutput) -> AudioPlayer<'a> {
        AudioPlayer { queue, source: None, output, looping: false, volume: 1.0, position: 0.0 }
    }

    pub fn from_item(item: PlaybackQueueItem, output: &'a mut dyn AudioOutput) -> AudioPlayer<'a> {
        AudioPlayer { queue: PlaybackQueue::new(vec![item]), source: None, output, looping: false, volume: 1.0, position: 0.0 }
    }

    pub fn prepare(&mut self) -> anyhow::Result<()> {
        let item = self.queue.current_item();
        let symphonia_source = SymphoniaSource::new(
            Path::new(&item.path), 
            self.output.sample_rate(), 
            self.output.channels())?;

        self.output.set_source(Box::new(symphonia_source))?;
        Ok(())
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

    pub fn queue(&self) -> &PlaybackQueue { &self.queue }

    pub fn position_secs(&self) -> f64 { self.position }
    pub fn duration_secs(&self) -> f64 { self.source.as_ref().map_or(0.0, |s| s.duration()) }

    pub fn is_paused(&self) -> bool { !self.output.is_playing() }

    pub fn is_looping(&self) -> bool { self.looping }
}
