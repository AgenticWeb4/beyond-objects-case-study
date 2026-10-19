//! 【小白】Booking 内部的三块碎片。没有一台「订座本」把规矩收齐。

use restaurant_ddd_kernel::{Date, PartySize, ReservationId, SlotId, TimeOfDay, UserId};

/// 【小白】订位关心的「人」。只有编号和哪天订过。
///
/// 不是 Identity 的 User（没有密码），也不是 Reputation 的 Standing（没有 karma）。
/// `ddd-clean` 把这三样塞进同一个 User；这边拆开了，但订位侧仍是一个客人对象。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guest {
    pub user_id: UserId,
    pub booked_dates: Vec<Date>,
}

impl Guest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            booked_dates: vec![],
        }
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

/// 【小白】「这个时段被占了」。住在 Booking，不住在 Availability 的 Slot 上。
///
/// 所以搜索还可以列出该时段；拒绝发生在这里，不发生在空位牌。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotOccupancy {
    pub slot_id: SlotId,
    pub occupied: bool,
}

impl SlotOccupancy {
    pub fn vacant(slot_id: SlotId) -> Self {
        Self {
            slot_id,
            occupied: false,
        }
    }

    pub fn occupy(&mut self) {
        self.occupied = true;
    }

    pub fn release(&mut self) {
        self.occupied = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationStatus {
    Booked,
    Canceled,
    Granted,
}

impl ReservationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ReservationStatus::Booked => "booked",
            ReservationStatus::Canceled => "canceled",
            ReservationStatus::Granted => "granted",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "booked" => Some(Self::Booked),
            "canceled" => Some(Self::Canceled),
            "granted" => Some(Self::Granted),
            _ => None,
        }
    }
}

/// 【小白】一张订单。但「去订」这个动作不在它身上，而在 BookingService 里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub id: ReservationId,
    pub user_id: UserId,
    pub slot_id: SlotId,
    pub time: TimeOfDay,
    pub date: Date,
    pub party_size: PartySize,
    pub status: ReservationStatus,
}

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
