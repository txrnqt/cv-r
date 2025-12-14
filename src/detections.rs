use ndarray::Array4;
use ort::{execution_providers::TensorRTExecutionProvider, session::Session, value::Value};

pub struct Detector {
    session: Option<Session>,
}

impl Detector {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn set_session<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.session::builder()?
            .with_execution_providers([TensorRTExecutionProvider::default().build()])?
            .commit_from_file(path)?;
        Ok(())
    }

    pub fn run_inference(
        &mut self,
        img: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    ) -> ort::Result<Vec<(f32, f32, f32, f32, f32)>> {
        let mut array = Array4::<f32>::zeros((1, 3, 640, 640));
        for (i, pixel) in img.pixels().enumerate() {
            let h = (i / 640) as usize;
            let w = (i % 640) as usize;
            array[[0, 0, h, w]] = pixel[0] as f32 / 255.0;
            array[[0, 1, h, w]] = pixel[1] as f32 / 255.0;
            array[[0, 2, h, w]] = pixel[2] as f32 / 255.0;
        }

        let shape = array.shape().to_vec();
        let (data, _offset) = array.into_raw_vec_and_offset();
        let input_tensor = Value::from_array((shape, data))?;
        let outputs = self.session.run(ort::inputs![input_tensor])?;

        let bboxes = {
            let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;
            let num_detections = shape[2] as usize;

            let mut bboxes = Vec::new();
            for i in 0..num_detections {
                let conf = data[4 * num_detections + i];
                if conf > 0.5 {
                    bboxes.push((
                        data[i],
                        data[num_detections + i],
                        data[2 * num_detections + i],
                        data[3 * num_detections + i],
                        conf,
                    ));
                }
            }
            bboxes
        };
        drop(outputs);
        Ok(self.run_nms(bboxes, 0.45))
    }

    fn run_nms(
        &self,
        bboxes: Vec<(f32, f32, f32, f32, f32)>,
        iou_threshold: f32,
    ) -> Vec<(f32, f32, f32, f32, f32)> {
        let mut sorted = bboxes;
        sorted.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap());

        let mut keep = Vec::new();
        while !sorted.is_empty() {
            let current = sorted.remove(0);
            keep.push(current);
            sorted.retain(|other| self.calculate_iou(&current, other) < iou_threshold);
        }
        keep
    }

    fn calculate_iou(
        &self,
        bbox1: &(f32, f32, f32, f32, f32),
        bbox2: &(f32, f32, f32, f32, f32),
    ) -> f32 {
        let inter_w = (bbox1.0 + bbox1.2 / 2.0).min(bbox2.0 + bbox2.2 / 2.0)
            - (bbox1.0 - bbox1.2 / 2.0).max(bbox2.0 - bbox2.2 / 2.0);
        let inter_h = (bbox1.1 + bbox1.3 / 2.0).min(bbox2.1 + bbox2.3 / 2.0)
            - (bbox1.1 - bbox1.3 / 2.0).max(bbox2.1 - bbox2.3 / 2.0);
        let inter = inter_w.max(0.0) * inter_h.max(0.0);
        let union = bbox1.2 * bbox1.3 + bbox2.2 * bbox2.3 - inter;
        inter / union
    }
}
