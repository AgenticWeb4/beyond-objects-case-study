use restaurant_ddd_kernel::{ReservationId, SlotId, UserId};

use crate::domain::{Guest, Reservation, SlotOccupancy};

pub trait BookingStore {
    fn get_or_create_guest(&mut self, user: &UserId) -> Guest;
    fn save_guest(&mut self, guest: &Guest);
    fn get_guest(&self, user: &UserId) -> Option<Guest>;

    fn get_or_create_occupancy(&mut self, slot: &SlotId) -> SlotOccupancy;
    fn save_occupancy(&mut self, occupancy: &SlotOccupancy);

    fn save_reservation(&mut self, reservation: &Reservation);
    fn get_reservation(&self, id: &ReservationId) -> Option<Reservation>;
    fn next_reservation_id(&mut self) -> ReservationId;
}
