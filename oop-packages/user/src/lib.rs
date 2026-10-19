//! 【小白】这是「人」这个类，只是单独放进了一个 crate。
//!
//! 口袋没变：密码、放鸽子次数、哪天订过，还在同一个 User 上。
//! 换文件夹 ≠ 换成概念。概念侧登录和信誉根本不在同一本账。

use restaurant_oop_shared::Date;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 一个「用户对象」装了太多无关的事：登录密码、放鸽子次数、哪天订过。
#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password: String,
    pub karma_strikes: u32,
    pub booked_dates: Vec<Date>,
}

pub const STANDING_THRESHOLD: u32 = 2;

impl User {
    pub fn in_good_standing(&self) -> bool {
        self.karma_strikes < STANDING_THRESHOLD
    }

    pub fn check_password(&self, password: &str) -> bool {
        self.password == password
    }

    pub fn punish(&mut self) {
        self.karma_strikes += 1;
    }

    pub fn has_booked_on(&self, date: &Date) -> bool {
        self.booked_dates.iter().any(|d| d == date)
    }

    pub fn note_booked(&mut self, date: Date) {
        self.booked_dates.push(date);
    }

    pub fn release_date(&mut self, date: &Date) {
        if let Some(i) = self.booked_dates.iter().position(|d| d == date) {
            self.booked_dates.remove(i);
        }
    }
}
