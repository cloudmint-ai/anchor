use crate::*;
use chrono::{DateTime, FixedOffset, Timelike};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[cfg(not(feature = "wasm"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// 毫秒时间点，起点根据业务定义, 计算逻辑同Timestamp
pub type TimeElapsed = Timestamp;

// 毫秒时间戳
#[derive(
    Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Default, Eq, Ord, Hash,
)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const ZERO: Timestamp = Timestamp::new(0);

    pub const fn new(value: i64) -> Self {
        Self(value)
    }
    fn seconds(self) -> i64 {
        self.0 / 1000
    }
    fn seconds_and_remaining(self) -> (i64, i64) {
        (self.0 / 1000, self.0 % 1000)
    }
}

impl From<i64> for Timestamp {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Timestamp> for i64 {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl Add<TimeDelta> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        Timestamp::from(i64::from(self) + i64::from(rhs))
    }
}

impl AddAssign<TimeDelta> for Timestamp {
    fn add_assign(&mut self, rhs: TimeDelta) {
        self.0 += i64::from(rhs)
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = TimeDelta;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        TimeDelta(i64::from(self) - i64::from(rhs))
    }
}

impl Sub<TimeDelta> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        Timestamp::from(i64::from(self) - i64::from(rhs))
    }
}

impl SubAssign<TimeDelta> for Timestamp {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        self.0 -= i64::from(rhs)
    }
}

// 毫秒时间戳之间的毫秒时间间隔
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize)]
pub struct TimeDelta(i64);

impl TimeDelta {
    pub const ZERO: TimeDelta = TimeDelta::new(0);

    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn from_days(days: i64) -> Self {
        Self(days * 1000 * 60 * 60 * 24)
    }

    pub const fn from_hours(hours: i64) -> Self {
        Self(hours * 1000 * 60 * 60)
    }

    pub const fn from_minuts(minuts: i64) -> Self {
        Self(minuts * 1000 * 60)
    }

    pub const fn from_seconds(seconds: i64) -> Self {
        Self(seconds * 1000)
    }

    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }
}

impl From<i64> for TimeDelta {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TimeDelta> for i64 {
    fn from(value: TimeDelta) -> Self {
        value.0
    }
}

impl TryFrom<TimeDelta> for usize {
    type Error = Error;
    fn try_from(value: TimeDelta) -> Result<usize> {
        Ok(usize::try_from(value.0)?)
    }
}

impl TryFrom<TimeDelta> for Duration {
    type Error = Error;
    fn try_from(value: TimeDelta) -> Result<Duration> {
        Ok(Duration::from_millis(u64::try_from(value.0)?))
    }
}

impl TryFrom<Duration> for TimeDelta {
    type Error = Error;
    fn try_from(value: Duration) -> Result<TimeDelta> {
        let mut ms = value.as_millis();
        if value.as_micros() % 1000 >= 500 {
            ms += 1
        }
        Ok(TimeDelta(i64::try_from(ms)?))
    }
}

impl std::fmt::Display for TimeDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ms", self.0)
    }
}

impl Add<TimeDelta> for TimeDelta {
    type Output = TimeDelta;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        TimeDelta(self.0 + rhs.0)
    }
}

impl Sub<TimeDelta> for TimeDelta {
    type Output = TimeDelta;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        TimeDelta(self.0 - rhs.0)
    }
}

impl Mul<i64> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: i64) -> Self::Output {
        TimeDelta(self.0 * rhs)
    }
}

impl Mul<TimeDelta> for i64 {
    type Output = TimeDelta;

    fn mul(self, rhs: TimeDelta) -> Self::Output {
        TimeDelta(self * rhs.0)
    }
}

impl Div<TimeDelta> for TimeDelta {
    type Output = i64;

    fn div(self, rhs: TimeDelta) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Div<i64> for TimeDelta {
    type Output = TimeDelta;

    fn div(self, rhs: i64) -> Self::Output {
        TimeDelta(self.0 / rhs)
    }
}

// 由于 now 调用往往上下文不具备 Result 支持，异常概率也极低，故使用unwrap 和 must_into
// 注意 测试场景里应尽量使用指定Timestamp 而不是 now
// TODO feature 为 test 的时候禁用now
#[cfg(not(feature = "wasm"))]
pub fn now() -> Timestamp {
    Timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    )
}

pub fn utc() -> String {
    let now = now();
    let (now_secs, now_remaining) = now.seconds_and_remaining();
    let now_nanos = now_remaining * 1_000_000;
    utc_format(now_secs, now_nanos, "%Y-%m-%d %H:%M:%S%.3fZ")
}

pub fn utc_date() -> String {
    utc_format(now().seconds(), 0, "%Y-%m-%d")
}

pub fn utc_offset_date(offset_hours: i32) -> String {
    utc_offset_format(offset_hours, now().seconds(), 0, "%Y-%m-%d")
}

pub fn seconds_elapsed_today_with_offset(offset_hours: i32) -> u32 {
    let seconds = now().seconds();
    let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap();
    let time = DateTime::from_timestamp(seconds, 0)
        .unwrap()
        .with_timezone(&offset);
    time.hour() * 3600 + time.minute() * 60 + time.second()
}

// 由于 utc 调用往往上下文不具备 Result 支持，异常概率也极低，故使用unwrap 和 as
fn utc_format(seconds: i64, nanoseconds: i64, format: &'static str) -> String {
    DateTime::from_timestamp(seconds, nanoseconds as u32)
        .unwrap()
        .format(format)
        .to_string()
}

// 由于 utc 调用往往上下文不具备 Result 支持，异常概率也极低，故使用unwrap 和 as
fn utc_offset_format(
    offset_hours: i32, // 东八区 +8
    seconds: i64,
    nanoseconds: i64,
    format: &'static str,
) -> String {
    let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap();
    DateTime::from_timestamp(seconds, nanoseconds as u32)
        .unwrap()
        .with_timezone(&offset)
        .format(format)
        .to_string()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date,js_name=now)]
    fn js_now() -> f64; // js 使用 f64 存储一切数字
}

// now in ms
#[cfg(feature = "wasm")]
pub fn now() -> Timestamp {
    Timestamp::from(js_now() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test::case]
    fn test_ms_serde() {
        let now = Timestamp::from(1738825016812);
        let serialized = json::to_string(&now)?;
        assert_eq!(serialized, "1738825016812");

        let deserialized: Timestamp = json::from_str("1738825016812")?;
        assert_eq!(deserialized, now);
        let deserialized: i64 = deserialized.into();
        assert_eq!(deserialized, 1738825016812);

        let mut default: Timestamp = default!();
        assert_eq!(default, Timestamp::ZERO);

        default += TimeDelta::from_seconds(1);
        assert_eq!(default, Timestamp::from(1000));
    }

    #[test::case]
    fn test_format() {
        let now = 1738825016812;
        let now_secs = now / 1000;
        let now_nanos = (now % 1000) * 1_000_000;
        assert_eq!(
            utc_format(now_secs, now_nanos, "%Y-%m-%d %H:%M:%S%.3fZ"),
            "2025-02-06 06:56:56.812Z"
        );
        assert_eq!(utc_format(now_secs, now_nanos, "%Y-%m-%d"), "2025-02-06");

        assert_eq!(
            utc_offset_format(8, now_secs, now_nanos, "%Y-%m-%d %H:%M:%S%.3f+8"),
            "2025-02-06 14:56:56.812+8"
        );
        assert_eq!(
            utc_offset_format(8, now_secs, now_nanos, "%Y-%m-%d"),
            "2025-02-06"
        );
        assert_eq!(
            utc_offset_format(-3, now_secs, now_nanos, "%Y-%m-%d"),
            "2025-02-06"
        );
        assert_eq!(
            utc_offset_format(-7, now_secs, now_nanos, "%Y-%m-%d"),
            "2025-02-05"
        );
    }

    #[test::case]
    fn test_timedelta_from_ns() {
        let duration = Duration::new(5, 730023852);
        assert_eq!(duration.as_millis(), 5730);
        assert_eq!(TimeDelta::try_from(duration)?, TimeDelta::from_millis(5730));

        let duration = Duration::new(5, 730523852);
        assert_eq!(duration.as_millis(), 5730);
        assert_eq!(TimeDelta::try_from(duration)?, TimeDelta::from_millis(5731));

        let duration = Duration::new(5, 730500000);
        assert_eq!(duration.as_millis(), 5730);
        assert_eq!(TimeDelta::try_from(duration)?, TimeDelta::from_millis(5731));

        let duration = Duration::new(5, 730000000);
        assert_eq!(duration.as_millis(), 5730);
        assert_eq!(TimeDelta::try_from(duration)?, TimeDelta::from_millis(5730));

        let duration = Duration::new(5, 730499999);
        assert_eq!(duration.as_millis(), 5730);
        assert_eq!(TimeDelta::try_from(duration)?, TimeDelta::from_millis(5730));
    }
}
