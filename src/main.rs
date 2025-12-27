mod app_state;
mod camera;
mod detections;
use anyhow;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("starting");
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content/jetson_orinnano.onnx");
    println!("model path {:?}", model_path);
    let mut app = app_state::AppState::new(model_path).await?;
    println!("model loaded");
    app.activate_camera("/dev/video0").await?;

    loop {
        app.capture_loop().await?;
        println!("cap {:?}", app.latest_frame());
        println!("latest results {:?}", app.latest_results());
        println!("latest processed frame {:?}", app.latest_processed_frame());
        println!("latest yaw {:?}", app.latest_yaw());
    }

    Ok(())
}
