//! 【小白】「请帮订座本存一行 / 查一行」的说明书。不管柜子是 SQLite 还是内存。

use restaurant_shared_kernel::{Date, OpaqueId};

use crate::domain::{Reservation, ReservationId};

pub trait ReservationStore<U: OpaqueId, S: OpaqueId> {
    fn insert(&mut self, rec: Reservation<U, S>);
    fn get(&self, id: &ReservationId) -> Option<Reservation<U, S>>;
    fn update(&mut self, rec: Reservation<U, S>);
    fn next_id(&mut self) -> ReservationId;
    fn has_booked_slot(&self, slot: &S) -> bool;
    fn has_booked_user_date(&self, user: &U, date: &Date) -> bool;
}
