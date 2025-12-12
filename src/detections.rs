use image::{ImageBuffer, Rgb, imageops::FilterType};
use ndarray::Array4;
use ort::{execution_providers::TensorRTExecutionProvider, session::Session};

const TARGET_SIZE: u32 = 640;

pub struct dector {
    session: Session,
}

pub impl dector {
    pub fn new(path: String) -> ort::Result<()> {
        session = Session::builder()?
            .with_execution_providers([TensorRTExecutionProvider::default().build()])?
            .commit_from_file("yolov8n.onnx")?;
    }

    pub fn detect(frame: &[u8], width: u32, height: u32) -> ort::Result<Vec<Detection>> {
        let input = preprocess(frame, width, height);
        let outputs = session.run(ort::inputs!["images" => input.view()]?)?;
        let output = outputs["output0"].try_extract_tensor::<f32>()?.into_owned();
        let detections = postprocess(output);

        Ok(detections)
    }

    fn preprocess(frame: &[u8], width: u32, height: u32) -> Array4<f32> {
        let img: ImageBuffer<Rgb<u8>, _> = ImageBuffer::from_raw(width, height, frame.to_vec())
            .expect("Failed to create image buffer");

        let resized = image::imageops::resize(&img, TARGET_SIZE, TARGET_SIZE, FilterType::Bilinear);

        let mut array = Array4::<f32>::zeros((1, 3, TARGET_SIZE as usize, TARGET_SIZE as usize));

        for y in 0..TARGET_SIZE as usize {
            for x in 0..TARGET_SIZE as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                array[[0, 0, y, x]] = pixel[0] as f32 / 255.0;
                array[[0, 1, y, x]] = pixel[1] as f32 / 255.0;
                array[[0, 2, y, x]] = pixel[2] as f32 / 255.0;
            }
        }

        array
    }
}
