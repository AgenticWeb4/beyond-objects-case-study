//! 值对象式的身份与时刻。和概念侧 shared-kernel 类似，但本 crate 自给自足，
//! 故意不依赖 `restaurant-shared-kernel`——两套实现要能分开编译。

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RestaurantId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReservationId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeOfDay(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub date: Date,
    pub time: TimeOfDay,
}

impl Date {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() == 10 {
            Ok(Date(raw.to_string()))
        } else {
            Err(format!("date 须为 YYYY-MM-DD，收到 {raw}"))
        }
    }
}

impl TimeOfDay {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() == 5 {
            Ok(TimeOfDay(raw.to_string()))
        } else {
            Err(format!("time 须为 HH:MM，收到 {raw}"))
        }
    }
}

impl DateTime {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (d, t) = raw
            .split_once('T')
            .ok_or_else(|| format!("DateTime 须为 YYYY-MM-DDTHH:MM，收到 {raw}"))?;
        Ok(DateTime {
            date: Date::parse(d)?,
            time: TimeOfDay::parse(t)?,
        })
    }

    pub fn is_after(&self, date: &Date, time: &TimeOfDay) -> bool {
        self.date > *date || (self.date == *date && self.time.0 > time.0)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for RestaurantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for ReservationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
