use wasm_bindgen::prelude::*;

/// Initialize the audio processor module
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Example: Apply gain to audio samples
#[wasm_bindgen]
pub fn apply_gain(samples: &mut [f32], gain: f32) {
    for sample in samples.iter_mut() {
        *sample *= gain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_gain() {
        let mut samples = vec![0.5, -0.5, 1.0, -1.0];
        apply_gain(&mut samples, 0.5);
        assert_eq!(samples, vec![0.25, -0.25, 0.5, -0.5]);
    }
}
