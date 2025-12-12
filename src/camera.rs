use std::io;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

pub struct Camera {
    device: Device,
    stream: Option<Stream<'static>>,
}

impl Camera {
    pub fn new(device_path: &str) -> io::Result<Self> {
        let device = Device::with_path(device_path)?;
        Ok(Camera {
            device,
            stream: None,
        })
    }

    pub fn configure(&mut self, width: u32, height: u32) -> io::Result<()> {
        let mut fmt = self.device.format()?;
        fmt.width = width;
        fmt.height = height;
        fmt.fourcc = FourCC::new(b"MJPG");
        self.device.set_format(&fmt)?;

        println!("Configured: {}x{}", fmt.width, fmt.height);
        Ok(())
    }

    pub fn start_stream(&mut self) -> io::Result<()> {
        let stream = Stream::with_buffers(&self.device, Type::VideoCapture, 4)?;
        self.stream = Some(unsafe { std::mem::transmute(stream) });
        Ok(())
    }

    pub fn capture_frame(&mut self) -> io::Result<Vec<u8>> {
        if let Some(stream) = &mut self.stream {
            let (buf, _meta) = stream.next()?;
            Ok(buf.to_vec())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Stream not started",
            ))
        }
    }
}
