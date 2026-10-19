//! Booking 应用服务：把三块碎片从库里取出，交给 BookingService，再分别存回去。
//!
//! 【小白】这是 DDD 的「加载聚合 → 领域服务改它们 → 仓储保存」。
//! 不是概念动作：概念动作不先拼三个对象。

use restaurant_ddd_kernel::{Date, DateTime, PartySize, ReservationId, SlotId, TimeOfDay, UserId};

use crate::booking_service::BookingService;
use crate::domain::{
    CancelError, GrantError, NoShowError, ReservationStatus, ReserveError,
};
use crate::ports::BookingStore;

pub struct BookingFacade<Store> {
    store: Store,
}

impl<Store> BookingFacade<Store> {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store: BookingStore> BookingFacade<Store> {
    pub fn reserve(
        &mut self,
        user: UserId,
        slot: SlotId,
        time: TimeOfDay,
        date: Date,
        party_size: PartySize,
    ) -> Result<ReservationId, ReserveError> {
        let mut guest = self.store.get_or_create_guest(&user);
        let mut occupancy = self.store.get_or_create_occupancy(&slot);
        let new_id = self.store.next_reservation_id();
        let reservation = BookingService::reserve(
            &mut guest,
            &mut occupancy,
            user,
            slot,
            time,
            date,
            party_size,
            new_id,
        )?;
        self.store.save_guest(&guest);
        self.store.save_occupancy(&occupancy);
        let id = reservation.id.clone();
        self.store.save_reservation(&reservation);
        Ok(id)
    }

    pub fn grant(&mut self, reservation: &ReservationId, now: &DateTime) -> Result<(), GrantError> {
        let mut rec = self
            .store
            .get_reservation(reservation)
            .ok_or(GrantError::NotGrantable)?;
        if rec.status != ReservationStatus::Booked || now.date != rec.date {
            return Err(GrantError::NotGrantable);
        }
        rec.status = ReservationStatus::Granted;
        if let Some(mut guest) = self.store.get_guest(&rec.user_id) {
            guest.release_date(&rec.date);
            self.store.save_guest(&guest);
        }
        let mut occupancy = self.store.get_or_create_occupancy(&rec.slot_id);
        occupancy.release();
        self.store.save_occupancy(&occupancy);
        self.store.save_reservation(&rec);
        Ok(())
    }

    pub fn no_show(
        &mut self,
        reservation: &ReservationId,
        now: &DateTime,
    ) -> Result<UserId, NoShowError> {
        let mut rec = self
            .store
            .get_reservation(reservation)
            .ok_or(NoShowError::NotNoShow)?;
        if rec.status != ReservationStatus::Booked || !now.is_after(&rec.date, &rec.time) {
            return Err(NoShowError::NotNoShow);
        }
        rec.status = ReservationStatus::Canceled;
        let user = rec.user_id.clone();
        if let Some(mut guest) = self.store.get_guest(&user) {
            guest.release_date(&rec.date);
            self.store.save_guest(&guest);
        }
        let mut occupancy = self.store.get_or_create_occupancy(&rec.slot_id);
        occupancy.release();
        self.store.save_occupancy(&occupancy);
        self.store.save_reservation(&rec);
        Ok(user)
    }

    pub fn cancel(&mut self, reservation: &ReservationId) -> Result<(), CancelError> {
        let mut rec = self
            .store
            .get_reservation(reservation)
            .ok_or(CancelError::NotBooked)?;
        if rec.status != ReservationStatus::Booked {
            return Err(CancelError::NotBooked);
        }
        rec.status = ReservationStatus::Canceled;
        if let Some(mut guest) = self.store.get_guest(&rec.user_id) {
            guest.release_date(&rec.date);
            self.store.save_guest(&guest);
        }
        let mut occupancy = self.store.get_or_create_occupancy(&rec.slot_id);
        occupancy.release();
        self.store.save_occupancy(&occupancy);
        self.store.save_reservation(&rec);
        Ok(())
    }
}
