//! 【小白】公用小零件：日期、钟点、编号标签。
//!
//! 没有「预约」「用户资料」。四本账可以共用一种「编号长什么样」，
//! 但不能把别人的整本账搬过来用。

use std::fmt;

/// 输入格式不对（例如日期不是 `YYYY-MM-DD`）。这不是业务拒绝，只是 CLI 解析失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 【小白】编号标签。能区分「这个」和「那个」就行，里面没有姓名、没有信誉。
pub trait OpaqueId: Clone + Eq + std::hash::Hash + Ord {
    fn as_str(&self) -> &str;
    fn from_string(s: String) -> Self;
}

/// 生成 `pub struct Foo(pub String)` 并实现 [`OpaqueId`]。
/// 各概念在自己的 crate 里定义自己的 ID，避免共享一个「上帝 ID 类型」。
#[macro_export]
macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub String);

        impl $crate::OpaqueId for $name {
            fn as_str(&self) -> &str {
                &self.0
            }
            fn from_string(s: String) -> Self {
                Self(s)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// 日历日，存 ISO `YYYY-MM-DD`。字典序比较与时间序一致。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date(pub String);

/// 当天时刻，存 `HH:MM`（24 小时）。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeOfDay(pub String);

/// CLI 传入的「现在」，格式 `YYYY-MM-DDTHH:MM`。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DateTime {
    pub date: Date,
    pub time: TimeOfDay,
}

pub type PartySize = u32;

impl Date {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        if raw.len() == 10 && raw.as_bytes()[4] == b'-' && raw.as_bytes()[7] == b'-' {
            Ok(Date(raw.to_string()))
        } else {
            Err(ParseError(format!("date 须为 YYYY-MM-DD，收到 {raw}")))
        }
    }
}

impl TimeOfDay {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        if raw.len() == 5 && raw.as_bytes()[2] == b':' {
            Ok(TimeOfDay(raw.to_string()))
        } else {
            Err(ParseError(format!("time 须为 HH:MM，收到 {raw}")))
        }
    }

    pub fn is_after(&self, other: &TimeOfDay) -> bool {
        self.0 > other.0
    }
}

impl DateTime {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        let (d, t) = raw
            .split_once('T')
            .ok_or_else(|| ParseError(format!("DateTime 须为 YYYY-MM-DDTHH:MM，收到 {raw}")))?;
        Ok(DateTime {
            date: Date::parse(d)?,
            time: TimeOfDay::parse(t)?,
        })
    }

    /// 是否严格晚于某个预约的 (date, time)。用于 no-show 判定。
    pub fn is_after(&self, date: &Date, time: &TimeOfDay) -> bool {
        self.date > *date || (self.date == *date && self.time.is_after(time))
    }
}
