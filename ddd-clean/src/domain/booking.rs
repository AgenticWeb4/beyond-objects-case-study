//! 【小白】「预约」这个动作没地方放，就另开一个 BookingService 同时改三个对象。
//!
//! 一次订位会：在人身上记下「这天订过」、把位子标占用、再建一张订单。
//! 概念侧这三件事不在同一个函数里。

use crate::domain::ids::{Date, ReservationId, TimeOfDay};
use crate::domain::reservation::{Reservation, ReservationStatus};
use crate::domain::restaurant::Restaurant;
use crate::domain::user::User;

pub struct BookingService;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReserveError {
    BadStanding,
    NoSlot,
    Conflict,
}

impl BookingService {
    pub fn reserve(
        user: &mut User,
        restaurant: &mut Restaurant,
        time: TimeOfDay,
        date: Date,
        party_size: u32,
        new_id: ReservationId,
    ) -> Result<Reservation, ReserveError> {
        // 【小白】信誉写在人身上。概念侧这是另一块小黑板。
        if !user.in_good_standing() {
            return Err(ReserveError::BadStanding);
        }
        if user.has_booked_on(&date) {
            return Err(ReserveError::Conflict);
        }
        let restaurant_id = restaurant.id.clone();
        let slot = restaurant
            .find_available_mut(&time, &date, party_size)
            .ok_or(ReserveError::NoSlot)?;
        if slot.reserved {
            return Err(ReserveError::Conflict);
        }
        // 【小白】位子一标占用，门口的空位牌等于被摘掉。下一步搜索就是空的。
        slot.reserved = true;
        let slot_id = slot.id.clone();
        user.note_booked(date.clone());
        Ok(Reservation {
            id: new_id,
            user_id: user.id.clone(),
            restaurant_id,
            slot_id,
            time,
            date,
            party_size,
            status: ReservationStatus::Booked,
        })
    }
}
