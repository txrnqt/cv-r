use crate::camera::Camera;
use crate::detections::Detector;
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AppState {
    detector: Arc<Detector>,
    latest_frame: Option<RgbImage>,
    latest_processed_frame: Option<RgbImage>,
    latest_results: Option<Vec<(f32, f32, f32, f32, f32)>>,
    latest_yaw: Option<f32>,
    has_json: bool,
    camera: Option<Camera>,
    calibration_data: Option<CalibrationData>,
}

impl AppState {
    pub async fn load<P: AsRef<std::path::Path>>(model_path: P) -> anyhow::Result<Self> {
        let mut detector = Detector::new();
        detector.set_session(model_path)?;

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

    pub async fn load_calibration<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> anyhow::Result<()> {
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

    fn calculate_yaw(&mut self) -> anyhow::Result<()> {
        if let Some(ref bboxes) = self.latest_results {
            if let Some(bbox) = bboxes.first() {
                let x = bbox.0;
                let f_x = self.get_f_x()?;
                let image_center = 640.0 / 2.0;
                self.latest_yaw = Some(((x - image_center) / f_x as f32).atan());
                Ok(())
            } else {
                Err(anyhow::anyhow!("No bounding boxes in results"))
            }
        } else {
            Err(anyhow::anyhow!("No bounding box available"))
        }
    }

    async fn capture_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, std::io::Error> {
        if let Some(ref mut camera) = self.camera {
            camera.capture_frame(640, 480)
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

    pub async fn activate_camera<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> anyhow::Result<()> {
        let mut camera = Camera::new();
        camera.activate_camera(path)?;
        self.camera = Some(camera);
        Ok(())
    }

    fn plot_bbox(
        &mut self,
        img: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    ) -> anyhow::Result<()> {
        if let Some(ref bboxes) = self.latest_results {
            if let Some(bbox) = bboxes.first() {
                let x = bbox.0 as u32;
                let y = bbox.1 as u32;
                let w = bbox.2 as u32;
                let h = bbox.3 as u32;
                let color = Rgb([255, 0, 0]);

                let mut result = img.clone();

                let x2 = (x + w).min(result.width());
                let y2 = (y + h).min(result.height());

                for px in x..=x2 {
                    if y < result.height() {
                        result.put_pixel(px, y, color);
                    }
                    if y2 < result.height() {
                        result.put_pixel(px, y2, color);
                    }
                }

                for py in y..=y2 {
                    if x < result.width() {
                        result.put_pixel(x, py, color);
                    }
                    if x2 < result.width() {
                        result.put_pixel(x2, py, color);
                    }
                }

                self.latest_processed_frame = Some(result);
                Ok(())
            } else {
                Err(anyhow::anyhow!("No bounding boxes in results"))
            }
        } else {
            Err(anyhow::anyhow!("No bounding box available"))
        }
    }

    pub async fn capture_loop(&mut self) -> anyhow::Result<()> {
        match self.capture_frame().await {
            Ok(frame) => {
                self.latest_frame = Some(frame.clone());

                match self.run_inference(&frame).await {
                    Ok(results) => {
                        self.latest_results = Some(results);
                        let _ = self.calculate_yaw();
                        let _ = self.plot_bbox(&frame);
                    }
                    Err(e) => eprintln!("Inference error: {}", e),
                }
            }
            Err(e) => eprintln!("Capture error: {}", e),
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct CalibrationData {
    pub camera: String,
    pub camera_matrix: [[f64; 3]; 3],
    pub dist_coeffs: [f64; 5],
    pub resolution: (u32, u32),
}
