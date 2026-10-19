use crate::domain::ids::{Date, UserId};

/// 【小白】一个「用户对象」装了太多无关的事：登录密码、放鸽子次数、哪天订过。
///
/// 像把工牌、信誉小黑板、订座记录全塞进同一个人的口袋。
/// 换一个产品，这个口袋里的东西就不一样，User 类很难搬走复用。
#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password: String,
    /// 【小白】放鸽子次数。概念侧这列根本不在 users 表。
    pub karma_strikes: u32,
    /// 【小白】「你这天订过了」记在人身上。另一条规矩（一个位子只能订一次）记在位子上。
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
