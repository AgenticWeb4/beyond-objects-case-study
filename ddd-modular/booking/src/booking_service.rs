//! 【小白】「预约」仍然没有自然的家，所以另开一个领域服务同时改三块碎片。
//!
//! 看起来有点像概念侧的 `Reserving.reserve`（也会 Conflict），但打开函数就不是：
//! 这里改的是 Guest.booked_dates、SlotOccupancy.occupied、再 new 一张 Reservation。
//! 概念侧是一台机器、一张表、两条不变量。
//!
//! 也不要把它当成 `syncs`：本服务看不见 Identity / Availability / Reputation。

use restaurant_ddd_kernel::{Date, PartySize, ReservationId, SlotId, TimeOfDay, UserId};

use crate::domain::{Guest, Reservation, ReservationStatus, ReserveError, SlotOccupancy};

pub struct BookingService;

impl BookingService {
    pub fn reserve(
        guest: &mut Guest,
        occupancy: &mut SlotOccupancy,
        user_id: UserId,
        slot_id: SlotId,
        time: TimeOfDay,
        date: Date,
        party_size: PartySize,
        new_id: ReservationId,
    ) -> Result<Reservation, ReserveError> {
        // 【小白】同日约束写在客人对象上。概念侧写在订座本的一条索引里。
        if guest.has_booked_on(&date) {
            return Err(ReserveError::Conflict);
        }
        // 【小白】一槽一约写在占用对象上。不是写在 Availability 的 listed 上。
        if occupancy.occupied {
            return Err(ReserveError::Conflict);
        }
        occupancy.occupy();
        guest.note_booked(date.clone());
        Ok(Reservation {
            id: new_id,
            user_id,
            slot_id,
            time,
            date,
            party_size,
            status: ReservationStatus::Booked,
        })
    }
}
