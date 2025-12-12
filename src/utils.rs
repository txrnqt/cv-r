use ort;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct CalibrationData {
    pub camera: String,
    pub camera_matrix: [[f64; 3]; 3],
    pub dist_coeffs: [f64; 5],
    pub resolution: (u32, u32),
}

pub impl CalibrationData {
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let data: CalibrationData = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(data)
    }

    pub fn get_camera_name() -> String {
        Ok(CalibrationData.camera)
    }

    pub fn get_f_x() -> f32 {
        Ok(CalibrationData.camera_matrix[0][0])
    }
}

pub fn getYaw(f_x: f32, bbox: [&f32]) -> f32 {
    let center_x = (bbox[0] + bbox[2]) / 2.0;
    let x = x / 640;
    let x = x / f_x;
    let yaw = x.atan();
    yaw
}

pub fn get_best_confidence(detections: Vec<ort::Session>) -> Option<usize> {
    let mut best_index: usize = 0;
    let mut best_confidence: f32 = 0.0;

    for (idx, detection) in detections.iter().enumerate() {
        if detection.confidence > best_confidence {
            best_confidence = detection.confidence;
            best_index = idx;
        }
    }

    if best_confidence > 0.0 {
        Some(best_index)
    } else {
        None
    }
}
