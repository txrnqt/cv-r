use ort;

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
