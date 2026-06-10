// Audio module — pipeline overview:
//
//   SymphoniaSource   — decodes + resamples from file (producer thread)
//        ↓
//   StructuralSource  — walks Timeline segments, seeks the decoder at cut
//        ↓               boundaries; everything below is in *processed* time
//   AudioProducer     — chunks into the rtrb ring + AudioStore
//        ↓  (rtrb ring buffer — lock-free)
//   BufferedSource    — feeds cpal callback; advances playhead atomic
//        ↓
//   StreamProcessorNode chain (gain, reverb, …)
//        ↓
//   CpalOutput        — wraps cpal stream; calls fill_buffer() on every callback
//
// CpalEngine wires these together and owns the Arc<RwLock<Timeline>>; the
// timeline is rebuilt only on the main thread while playback is paused.
// SourceHandle gives the main thread lock-free reads of position, duration,
// seek state, and exhaustion.
pub mod buffer;
pub mod devices;
pub mod queue;
pub mod output;
pub mod player;
pub mod source;
pub mod engine;
pub mod producer;
pub mod node;
pub mod chain;
pub mod timeline;
pub mod structural;

#[cfg(test)]
mod tests;
