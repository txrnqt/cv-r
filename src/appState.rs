use crate::camera::Camera;
use crate::detections::Detector;
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};

struct AppState {
    detector: Detector,
    latest_frame: Option<RgbImage>,
    latest_processed_frame: Option<RgbImage>,
    latest_bbox: Option<(f32, f32, f32, f32, f32)>,
    latest_yaw: Option<f32>,
    has_json: bool,
    camera: Option<Camera>,
    calibration_data: Option<CalibrationData>,
} 

impl AppState {
    fn new() -> Self {
        Self {
            detector: Detector::new(),
            latest_frame: None,
            latest_processed_frame: None,
            latest_bbox: None,
            latest_yaw: None,
            has_json: false,
            camera: None,
            calibration_data: None,
        }
    }

    async fn load_calibration<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.calibration_data = Some(serde_json::from_str(
            &tokio::fs::read_to_string(path).await?,
        )?);
        Ok(())
    }

    fn get_f_x(&self) -> anyhow::Result<f64> {
        if let Some(ref calib) = self.calibration_data {
            Ok(calib.camera_matrix[0][0])
        } else {
            Err(anyhow::anyhow!("No calibration data loaded"))
        }
    }

    fn get_yaw(&mut self) -> anyhow::Result<()> {
        if let Some(bbox) = self.latest_bbox {
            let x = bbox.0;
            let f_x = self.get_f_x()?; // Removed await

            let image_center = 640.0 / 2.0;
            let yaw = ((x - image_center) / f_x as f32).atan();

            self.latest_yaw = Some(yaw);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn capture_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, std::io::Error> {
        if let Some(ref mut camera) = self.camera {
            let cap = camera.capture_frame();
            Ok(cap)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Camera not initialized",
            ))
        }
    }

    fn create_detector<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        Detector::set_session(path);
        Ok(())
    }

    fn run_inference(&mut self, image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<Vec, ort::Error> {
        self.detector.run_inference(image)
    }
}

#[derive(Serialize, Deserialize)]
pub struct CalibrationData {
    pub camera: String,
    pub camera_matrix: [[f64; 3]; 3],
    pub dist_coeffs: [f64; 5],
    pub resolution: (u32, u32),
}
