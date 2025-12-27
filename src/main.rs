mod app_state;
mod camera;
mod detections;

use anyhow;
use axiom_rs::Client;
use bytes::Bytes;
use image::{ImageBuffer, Rgb};
use serde_json::json;
use std::path::PathBuf;
use tokio::sync::mpsc;

struct SendData {
    raw_frame: Bytes,
    processed_frame: Bytes,
    yaw: f32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("starting");
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content/jetson_orinnano.onnx");
    println!("model path {:?}", model_path);
    let mut app = app_state::AppState::new(model_path).await?;
    println!("model loaded");
    app.activate_camera("/dev/video0").await?;

    let (tx, rx) = mpsc::channel::<SendData>(10);

    tokio::spawn(server(rx));

    loop {
        if let (Some(raw), Some(processed), Some(yaw)) = (
            app.latest_frame(),
            app.latest_processed_frame(),
            app.latest_yaw(),
        ) {
            tx.send(SendData {
                raw_frame: to_bytes(raw),
                processed_frame: to_bytes(processed),
                yaw,
            })
            .await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
    }
}

fn to_bytes(frame: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Bytes {
    Bytes::from(frame.into_raw())
}

async fn server(mut rx: mpsc::Receiver<SendData>) {
    let client = Client::new().expect("Failed to create Axiom client");
    let dataset = "frames";

    while let Some(data) = rx.recv().await {
        let entry = json!({
            "yaw": data.yaw,
            "raw_frame_size": data.raw_frame.len(),
            "processed_frame_size": data.processed_frame.len(),
        });

        if let Err(e) = client.ingest(dataset, vec![entry]).await {
            eprintln!("Axiom error: {e}");
        }
    }
}
