// Audio module — pipeline overview:
//
//   SymphoniaSource  — decodes + resamples from file on the AudioProducer thread
//        ↓  (rtrb ring buffer — lock-free)
//   BufferedSource   — feeds cpal callback; advances playhead atomic
//        ↓
//   CpalOutput       — wraps cpal stream; calls fill_buffer() on every callback
//
// CpalEngine wires these together. SourceHandle gives the main thread
// lock-free reads of position, duration, seek state, and exhaustion.
pub mod buffer;
pub mod devices;
pub mod queue;
pub mod output;
pub mod player;
pub mod source;
pub mod engine;
pub mod producer;

#[cfg(test)]
mod tests;
