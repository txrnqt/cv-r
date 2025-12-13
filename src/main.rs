mod camera;
mod detections;

use camera::Camera;
use detections::Detector;

fn main() -> anyhow::Result<()> {
    let mut cam = Camera::new("/dev/video0")?;
    let mut detector: Detector = Detector::new("src/content/jetson_orinnano.onnx")?;
    let mut results: Result<Vec<(f32, f32, f32, f32, f32)>, ort::Error>;

    cam.configure(640, 480)?;
    cam.start_stream()?;

    for i in 0..10 {
        let frame = cam.capture_frame()?;
        println!("Frame {}: {} bytes", i, frame.len());
        results = detector.run_inference(&frame);
        println!("results {:?}", results);
    }

    Ok(())
}
