use std::time::SystemTime;

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};

use crate::windows::GlkWindow;

use super::Glk;

#[derive(Debug, Default)]
pub struct GlkTimeval {
    pub sec: i64,
    pub microsec: u32,
}

#[derive(Debug, Default)]
pub struct GlkDate {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub weekday: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub microsec: u32,
}

impl<T: GlkWindow + Default> Glk<T> {
    /*
     * Glk Section 10 - The System Clock
     */
    /// Gets the current system time in seconds and microseconds since 1970
    pub fn current_time(&self) -> GlkTimeval {
        let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
            return GlkTimeval::default();
        };
        GlkTimeval {
            sec: time.as_secs() as i64,
            microsec: time.subsec_micros(),
        }
    }

    /// Gets the current system time scaled down by a factor
    pub fn current_simple_time(&self, factor: u32) -> i32 {
        let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) else {
            return 0;
        };
        (time.as_secs() / factor as u64) as i32
    }

    /*
     * Glk Section 10.1 - Time and Date Conversions
     */
    /// Convert a given timestamp to a UTC GlkDate
    pub fn time_to_date_utc(&self, time: &GlkTimeval) -> GlkDate {
        let Some(naive) = NaiveDateTime::from_timestamp_opt(time.sec, time.microsec * 1000) else {
            return GlkDate::default();
        };
        let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
        build_glk_date(datetime, time.microsec)
    }

    /// Convert a given timestamp to a Local GlkDate
    pub fn time_to_date_local(&self, time: &GlkTimeval) -> GlkDate {
        let Some(naive) = NaiveDateTime::from_timestamp_opt(time.sec, time.microsec * 1000) else {
            return GlkDate::default();
        };
        let local = *Local::now().offset();
        let datetime: DateTime<Local> = DateTime::from_naive_utc_and_offset(naive, local);
        build_glk_date(datetime, time.microsec)
    }

    /// Convert a simple time multiplied by a factor to a UTC GlkDate
    pub fn simple_time_to_date_utc(&self, time: i32, factor: u32) -> GlkDate {
        let Some(naive) = NaiveDateTime::from_timestamp_opt((time * factor as i32) as i64, 0)
        else {
            return GlkDate::default();
        };
        let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
        build_glk_date(datetime, 0)
    }

    /// Convert a simple time multiplied by a factor to a Local GlkDate
    pub fn simple_time_to_date_local(&self, time: i32, factor: u32) -> GlkDate {
        let Some(naive) = NaiveDateTime::from_timestamp_opt((time * factor as i32) as i64, 0)
        else {
            return GlkDate::default();
        };
        let local = *Local::now().offset();
        let datetime: DateTime<Local> = DateTime::from_naive_utc_and_offset(naive, local);
        build_glk_date(datetime, 0)
    }
}

fn build_glk_date<T: TimeZone>(datetime: DateTime<T>, microsec: u32) -> GlkDate {
    GlkDate {
        year: datetime.year(),
        month: datetime.month() as i32,
        day: datetime.day() as i32,
        weekday: datetime.weekday().num_days_from_sunday() as i32,
        hour: datetime.hour() as i32,
        minute: datetime.minute() as i32,
        second: datetime.second() as i32,
        microsec,
    }
}
