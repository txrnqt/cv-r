use anyhow::Ok;
use image::RgbImage;
use std::io;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

pub struct Camera {
    device: Device,
    stream: Option<Stream>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            device: Device,
            stream: None,
        }
    }
}

    pub fn activate_camera<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.device = Device::with_path(path)?;
        Ok(())
    }

    //TODO: set fps
    pub fn configure(&mut self, width: u32, height: u32) -> io::Result<()> {
        let mut fmt = self.device.format()?;
        fmt.width = width;
        fmt.height = height;

        fmt.fourcc = FourCC::new(b"YUYV");
        self.device.set_format(&fmt)?;

        println!("Configured: {}x{}", fmt.width, fmt.height);
        Ok(())
    }

    pub fn start_stream(&mut self) -> io::Result<()> {
        let stream = Stream::with_buffers(&self.device, Type::VideoCapture, 4)?;
        self.stream = Some(unsafe { std::mem::transmute(stream) });
        Ok(())
    }

    pub fn capture_frame(&mut self) -> io::Result<RgbImage> {
        if let Some(stream) = &mut self.stream {
            let (buf, _meta) = stream.next()?;

            let img = Self::yuyv_to_rgb(&buf, 640, 480);
            Ok(img)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Stream not started",
            ))
        }
    }

    fn yuyv_to_rgb(yuyv: &[u8], width: u32, height: u32) -> RgbImage {
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);

        for chunk in yuyv.chunks_exact(4) {
            let y0 = chunk[0] as f32;
            let u = chunk[1] as f32 - 128.0;
            let y1 = chunk[2] as f32;
            let v = chunk[3] as f32 - 128.0;

            for &y in &[y0, y1] {
                let r = (y + 1.402 * v).clamp(0.0, 255.0) as u8;
                let g = (y - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
                let b = (y + 1.772 * u).clamp(0.0, 255.0) as u8;
                rgb.extend_from_slice(&[r, g, b]);
            }
        }

        RgbImage::from_raw(width, height, rgb).unwrap()
    }
}
