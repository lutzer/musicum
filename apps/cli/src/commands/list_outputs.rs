use anyhow::Result;
use musicum_core::list_output_devices;

pub fn run() -> Result<()> {
    let devices = list_output_devices();
    if devices.is_empty() {
        println!("No audio output devices found.");
    } else {
        for name in &devices {
            println!("{name}");
        }
    }
    Ok(())
}
