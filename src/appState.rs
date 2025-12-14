use crate::camera::Camera;
use crate::detections::Detector;
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

struct AppState {
    detector: Arc<Detector>,
    latest_frame: Option<RgbImage>,
    latest_processed_frame: Option<RgbImage>,
    latest_results: Option<(f32, f32, f32, f32, f32)>,
    latest_yaw: Option<f32>,
    has_json: bool,
    camera: Option<Camera>,
    calibration_data: Option<CalibrationData>,
}

impl AppState {
    async fn load<P: AsRef<std::path::Path>>(model_path: P) -> anyhow::Result<Self> {
        let detector = Detector::load_from_file(model_path)?;

        Ok(Self {
            detector: Arc::new(detector),
            latest_frame: None,
            latest_processed_frame: None,
            latest_results: None,
            latest_yaw: None,
            has_json: false,
            camera: None,
            calibration_data: None,
        })
    }

    async fn load_calibration<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let json_str = tokio::fs::read_to_string(path).await?;
        self.calibration_data = Some(serde_json::from_str(&json_str)?);
        self.has_json = true;
        Ok(())
    }

    fn get_f_x(&self) -> anyhow::Result<f64> {
        if let Some(ref calib) = self.calibration_data {
            Ok(calib.camera_matrix[0][0])
        } else {
            Err(anyhow::anyhow!("No calibration data loaded"))
        }
    }

    fn calculate_yaw(&self) -> anyhow::Result<()> {
        if let Some(bbox) = self.latest_results {
            let x = bbox.0;
            let f_x = self.get_f_x()?;
            let image_center = 640.0 / 2.0;
            self.latest_yaw = ((x - image_center) / f_x as f32).atan();
            Ok(())
        } else {
            Err(anyhow::anyhow!("No bounding box available"))
        }
    }

    async fn capture_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, std::io::Error> {
        if let Some(ref mut camera) = self.camera {
            camera.capture_frame().await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Camera not initialized",
            ))
        }
    }

    async fn run_inference(
        &self,
        image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> Result<Vec<(f32, f32, f32, f32, f32)>, ort::Error> {
        self.detector.run_inference(image).await
    }

    async fn activate_camera<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.camera = Some(Camera.activate_camera(path).await?);
        Ok(())
    }

    fn plot_bbox(self) -> anyhow::Result<()> {
        if let Some(bbox) = self.latest_results {
            let x = bbox.0;
            let y = bbox.1;
            let w = bbox.2;
            let h = bbox.3;
            let image_center = 640.0 / 2.0;
            let color = Rgb([255, 0, 0]);
        } else {
            Err(anyhow::anyhow!("No bounding box available"))
        }
        let x2 = (x + w).min(img.width());
        let y2 = (y + w).min(img.height());

        for px in x..x2 {
            if y < img.height() {
                img.put_pixel(px, y, color);
            }
            if y2 < img.height() {
                img.put_pixel(px, y2, color);
            }
        }

        for py in y..y2 {
            if x < img.width() {
                img.put_pixel(x, py, color);
            }
            if x2 < img.width() {
                img.put_pixel(x2, py, color);
            }
        }
        self.latest_processed_frame = img;
        Ok(())
    }

    pub async fn capture_loop(&mut self) {
        self.latest_frame = self.capture_frame().await?;
        self.results = self.run_inference(self.latest_frame).await?;
        self.calculate_yaw();
        self.plot_bbox();
    }
}

#[derive(Serialize, Deserialize)]
pub struct CalibrationData {
    pub camera: String,
    pub camera_matrix: [[f64; 3]; 3],
    pub dist_coeffs: [f64; 5],
    pub resolution: (u32, u32),
}
