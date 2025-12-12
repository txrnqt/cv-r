mod camera;

use camera::Camera;
use std::io;

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
