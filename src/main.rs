mod camera;
mod detections;

use camera::Camera;
use detections::dector;
use std::{f32, io};

static mut RAW_FRAME: Vec<u8> = Vec::new();
static mut PROCESSED_FRAME: Vec<u8> = Vec::new();
static mut YAW: f32 = 0.0;
static mut CAMERAS: Vec<Camera>;
static mut DECECTOR: dector = dector::new("path");

fn captureLoop() {}

fn main() -> io::Result<()> {
    let mut cam = Camera::new("/dev/video0")?;

    cam.configure(640, 480)?;
    cam.start_stream()?;

    for i in 0..10 {
        let frame = cam.capture_frame()?;
        println!("Frame {}: {} bytes", i, frame.len());
    }

    Ok(())
}
