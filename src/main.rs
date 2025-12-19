mod app_state;
mod camera;
mod detections;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = app_state::AppState::new("model_path").await?;
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
