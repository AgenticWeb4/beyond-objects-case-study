//! 【小白】订座本上有哪些格子。
//!
//! 两条死规矩：同一个时段不能两份有效预约；同一个人同一天不能两份有效预约。
//! 「有效」= 状态还是 Booked（已预订）。取消或已入座就不占坑了。

use restaurant_shared_kernel::{opaque_id, Date, PartySize, TimeOfDay};

opaque_id!(ReservationId);

/// 【小白】预订 / 取消 / 已入座。只有「预订」占着名额。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Booked,
    Canceled,
    Granted,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Booked => "booked",
            Status::Canceled => "canceled",
            Status::Granted => "granted",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "booked" => Some(Status::Booked),
            "canceled" => Some(Status::Canceled),
            "granted" => Some(Status::Granted),
            _ => None,
        }
    }
}

/// 【小白】订座本上的一行。几点几人写在这一行上，可以和空位牌印的时间不一样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation<U, S> {
    pub id: ReservationId,
    pub user: U,
    pub slot: S,
    pub time: TimeOfDay,
    pub date: Date,
    pub party_size: PartySize,
    pub status: Status,
}

/// 该 slot 已有 booked，或该用户当天已有 booked。状态不变。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReserveError {
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CancelError {
    NotBooked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GrantError {
    NotGrantable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoShowError {
    NotNoShow,
}
