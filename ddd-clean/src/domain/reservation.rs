//! 【小白】一张订单。但「去订」这个动作不在它身上，而在 BookingService 里。

use crate::domain::ids::{Date, ReservationId, RestaurantId, SlotId, TimeOfDay, UserId};

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

#[derive(Clone, Debug)]
pub struct Reservation {
    pub id: ReservationId,
    pub user_id: UserId,
    pub restaurant_id: RestaurantId,
    pub slot_id: SlotId,
    pub time: TimeOfDay,
    pub date: Date,
    pub party_size: u32,
    pub status: ReservationStatus,
}
