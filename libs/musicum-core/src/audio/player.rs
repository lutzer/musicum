use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64};

use crate::audio::buffer::{AudioProducer, AudioStore, BufferedSource, SeekHandle};
use crate::audio::source::{AudioSource, SymphoniaSource};
use crate::{PlaybackQueue, PlaybackQueueItem};
use crate::audio::output::AudioOutput;

pub struct AudioPlayer<'a> {
    queue:       PlaybackQueue,
    output:      &'a mut dyn AudioOutput,
    looping:     bool,
    position:    f64,
    seek_handle: Option<SeekHandle>,
}

impl<'a> AudioPlayer<'a> {
    pub fn from_queue(queue: PlaybackQueue, output: &'a mut dyn AudioOutput) -> Self {
        Self { queue, output, looping: false, position: 0.0, seek_handle: None }
    }

    pub fn from_item(item: PlaybackQueueItem, output: &'a mut dyn AudioOutput) -> Self {
        Self {
            queue: PlaybackQueue::new(vec![item]),
            output, looping: false, position: 0.0, seek_handle: None,
        }
    }

    pub fn prepare(&mut self) -> anyhow::Result<()> {
        let item = self.queue.current_item();
        let decoder = SymphoniaSource::new(
            Path::new(&item.path),
            self.output.sample_rate(),
            self.output.channels(),
        )?;

        let sample_rate = decoder.sample_rate();
        let channels    = decoder.channels();
        let duration    = decoder.duration();
        let ring_cap    = 2 * sample_rate as usize * channels as usize;
        let store_cap   = 30 * sample_rate as usize * channels as usize;

        let (ring_tx, ring_rx) = rtrb::RingBuffer::new(ring_cap);
        let store         = Arc::new(Mutex::new(AudioStore::new(store_cap, channels)));
        let seek_pending  = Arc::new(AtomicBool::new(false));
        let seek_frame    = Arc::new(AtomicU64::new(0));
        let producer_done = Arc::new(AtomicBool::new(false));

        let producer = AudioProducer::new(
            decoder, store, ring_tx, ring_cap,
            seek_pending.clone(), seek_frame.clone(), producer_done.clone(),
        );
        let source = BufferedSource::new(
            ring_rx, sample_rate, channels, duration,
            seek_pending.clone(), producer_done,
        );

        std::thread::spawn(|| producer.run());
        self.output.set_source(Box::new(source))?;
        self.seek_handle = Some(SeekHandle::new(seek_pending, seek_frame, sample_rate));
        Ok(())
    }

    pub fn play(&mut self)  { let _ = self.output.play(); }
    pub fn pause(&mut self) { let _ = self.output.pause(); }

    pub fn seek(&mut self, time: f64) {
        self.position = time;
        if let Some(h) = &self.seek_handle {
            h.seek(time);
        }
    }

    pub fn set_volume(&mut self, _volume: f32) {}
    pub fn set_looping(&mut self, looping: bool) { self.looping = looping; }

    pub fn next(&mut self)     { let _ = self.queue.next(); }
    pub fn previous(&mut self) { let _ = self.queue.previous(); }

    pub fn queue(&self) -> &PlaybackQueue { &self.queue }
    pub fn position_secs(&self) -> f64 { self.position }
    pub fn duration_secs(&self) -> f64 { 0.0 }
    pub fn is_paused(&self)  -> bool { !self.output.is_playing() }
    pub fn is_looping(&self) -> bool { self.looping }
}
