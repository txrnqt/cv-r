#![forbid(clippy::unwrap_used)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod nt;
pub mod time;

#[allow(missing_docs)]
fn main() -> std::io::Result<()> {
    // SAFETY: only called here. No other threads call this method.
    unsafe { time::init_time() };

    Ok(())
}
