use image::RgbImage;
use std::io;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

pub struct Camera {
    device: Option<Device>,
    stream: Option<Box<dyn std::any::Any>>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            device: None,
            stream: None,
        }
    }

    pub fn activate_camera<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.device = Some(Device::with_path(path)?);
        Ok(())
    }

    pub fn configure(&mut self, width: u32, height: u32) -> io::Result<()> {
        let device = self
            .device
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Device not initialized"))?;

        let mut fmt = device.format()?;
        fmt.width = width;
        fmt.height = height;
        fmt.fourcc = FourCC::new(b"YUYV");
        device.set_format(&fmt)?;

        println!("Configured: {}x{}", fmt.width, fmt.height);
        Ok(())
    }

    pub fn start_stream(&mut self) -> io::Result<()> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "Device not initialized"))?;

        let stream = Stream::with_buffers(device, Type::VideoCapture, 4)?;
        self.stream = Some(Box::new(stream));
        Ok(())
    }

    pub fn capture_frame(&mut self, width: u32, height: u32) -> io::Result<RgbImage> {
        if let Some(stream) = &mut self.stream {
            let stream_ref = stream
                .downcast_mut::<Stream>()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Stream type mismatch"))?;

            let (buf, _meta) = stream_ref.next()?;
            let img = Self::yuyv_to_rgb(&buf, width, height);
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
