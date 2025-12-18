//! A common source of time between NT, timesync and cameras

use std::{cell::Cell, time::{Duration, SystemTime}};

struct StartTimeHolder {
    value: Cell<*const SystemTime>
}

/// SAFETY: contents of value are written only in the unsafe [`init_time`] function whose
/// safety preconditions guard against multiple mutable references to `StartTimeHolder.value`
/// being acquired at the same time.
unsafe impl Sync for StartTimeHolder {}

static START_TIME: StartTimeHolder = StartTimeHolder {
    value: Cell::new(std::ptr::null())
};

/// Initialize time. Sets the program's reference time to the current system time.
/// 
/// SAFETY: It is UB to call from multiple threads.
pub unsafe fn init_time() {
    if START_TIME.value.get().is_null() {
        let start = SystemTime::now();
        let start = Box::leak(Box::new(start));
        START_TIME.value.set(start as *mut _ as *const _);
    }
}

/// Get the current time relative to the program reference time in microseconds
pub fn now() -> u64 {
    let now = SystemTime::now();
    let duration = now.duration_since(unsafe { *START_TIME.value.get() }).expect("Time is in the past!");
    duration.as_micros() as u64
}

/// Convert a [`SystemTime`] into the microseconds with the same reference as [`now`]. If 
/// `time` precedes the reference time, returns [`None`].
pub fn convert(time: impl ConvertableToTime) -> Option<u64> {
    let duration = time.to_system_time().duration_since(unsafe { *START_TIME.value.get() }).ok()?;
    Some(duration.as_micros() as u64)
}

mod private {
    pub trait SealedForConvertableToTime {}
}

impl private::SealedForConvertableToTime for SystemTime {}
impl private::SealedForConvertableToTime for v4l::timestamp::Timestamp {}

/// Can convert into a [`SystemTime`]
pub trait ConvertableToTime : private::SealedForConvertableToTime {
    /// Convert to [`SystemTime`]
    fn to_system_time(self) -> SystemTime;
}

impl ConvertableToTime for SystemTime {
    fn to_system_time(self) -> SystemTime {
        self
    }
}

impl ConvertableToTime for v4l::timestamp::Timestamp {
    fn to_system_time(self) -> SystemTime {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_micros(self.usec as u64 + self.sec as u64 * 1000000)).expect("Unable to create SystemTime from Timestamp")
    }
}
