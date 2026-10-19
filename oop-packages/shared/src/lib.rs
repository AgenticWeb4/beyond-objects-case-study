//! 【小白】钟表零件。分包之后日期仍是共用的；业务对象不在这里。

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
