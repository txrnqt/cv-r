//! Minimal NetworkTables implementation tailored for this application

use std::{mem::ManuallyDrop, thread::JoinHandle, time::Duration};

/// Implementation of NetworkTables with predefined publishers and subscribers.
pub struct NetworkTablesInstance {
    shutdown: ManuallyDrop<tokio::sync::oneshot::Sender<()>>,
    thread: ManuallyDrop<JoinHandle<()>>,
    signals: MainThreadSignals,
}

impl NetworkTablesInstance {
    /// 
    pub fn new<S1: ToString, S2: ToString>(name: S1, ip: S2) -> Self {
        let name = name.to_string();
        let ip = ip.to_string();

        let (shutdown, rx) = tokio::sync::oneshot::channel();
        let (signals, nt_thread_signals) = MainThreadSignals::new();
        let thread = std::thread::spawn(move || {
            let shutdown = rx;
            tokio::runtime::Builder::new_current_thread().enable_all().build().expect("Unable to create tokio runtime").block_on(async move {
                tokio::select! {
                    _ = shutdown => {},
                    _ = nt_thread(name, ip, nt_thread_signals) => {},
                }
            })
        });

        let shutdown = ManuallyDrop::new(shutdown);
        let thread = ManuallyDrop::new(thread);

        Self {
            shutdown,
            thread,
            signals,
        }
    }
}

impl Drop for NetworkTablesInstance {
    fn drop(&mut self) {
        // SAFETY: we've pulled the Sender out of `self.shutdown`, so nothing may access `self.shutdown` any longer. Since this is 
        // the drop method, nothing else can access any member of this CameraThread after this function ends. The rest of this function
        // only deals with `self.thread`, so `self.shutdown` is not accessed after take.
        if let Err(_) = unsafe { ManuallyDrop::take(&mut self.shutdown) }.send(()) {
            eprintln!("Unable to send shutdown signal. Camera thread may have stopped early?");
        }

        // SAFETY: we've pulled the JoinHandle out of `self.thread`, so nothing may access `self.thread` any longer. Since this is 
        // the drop method, nothing else can access any member of this CameraThread after this function ends. This is the last access of
        // this function, so `self.thread` is not accessed after take.
        if let Err(e) = unsafe { ManuallyDrop::take(&mut self.thread) }.join() {
            eprintln!("Error stopping camera thread: {:?}", e);
        }
    }
}

struct MainThreadSignals {

}

struct NtThreadSignals {

}

impl MainThreadSignals {
    fn new() -> (MainThreadSignals, NtThreadSignals) {

        (
            MainThreadSignals {

            },
            NtThreadSignals {

            },
        )
    }
}

async fn nt_thread(name: String, ip: String, mut signals: NtThreadSignals) {
    loop {
        if let Err(e) = nt_connection(&name, &ip, &mut signals).await {
            eprintln!("{}", e);
            for cause in e.chain() {
                eprintln!("{}", cause);
            }
            continue;
        }
    }
}

async fn nt_connection(name: &str, ip: &str, signals: &mut NtThreadSignals) -> anyhow::Result<()> {
    tokio::time::sleep(Duration::from_secs(1)).await;

    

    Ok(())
}
