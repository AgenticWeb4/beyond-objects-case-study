//! 【小白】预约仍没有自然的家，所以另开一个包同时改三个对象包里的实例。
//!
//! 这不是 syncs，也不是 Reserving。打开函数：改 User.karma / booked_dates、
//! 改 Slot.reserved、再 new Reservation。三个 crate 只是三个文件夹。

use restaurant_oop_reservation::{Reservation, ReservationId, ReservationStatus};
use restaurant_oop_restaurant::Restaurant;
use restaurant_oop_shared::{Date, TimeOfDay};
use restaurant_oop_user::User;

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
        // 【小白】位子一标占用，搜索跟着变空。分包没有拆开这两件事。
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
