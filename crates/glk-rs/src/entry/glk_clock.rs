use std::time::SystemTime;

use crate::windows::GlkWindow;

use super::Glk;

#[derive(Debug)]
pub struct GlkTimeval {
    pub sec: i64,
    pub microsec: i32,
}

impl<T: GlkWindow + Default> Glk<T> {
    /*
     * Glk Section 10 - The System Clock
     */
    /// Gets the current system time in seconds since 1970
    pub fn current_time(&self) -> GlkTimeval {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        match now {
            Ok(time) => GlkTimeval {
                sec: time.as_secs() as i64,
                microsec: time.subsec_micros() as i32,
            },
            Err(_) => GlkTimeval {
                sec: 0,
                microsec: 0,
            },
        }
    }
}
