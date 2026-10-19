//! 【小白】订座本能做的几件事：盖章、取消、到店兑现、记放鸽子。
//!
//! `U` 和 `S` 只是编号标签（谁、哪个时段）。本文件不打开用户资料，也不看空位牌。

use std::marker::PhantomData;

use restaurant_shared_kernel::{Date, DateTime, OpaqueId, PartySize, TimeOfDay};

use crate::domain::{
    CancelError, GrantError, NoShowError, Reservation, ReservationId, ReserveError, Status,
};
use crate::ports::ReservationStore;

/// 【小白】订座本本身。所有「能不能订、怎么取消」的规矩都在这本里。
pub struct Reserving<Store, U, S> {
    store: Store,
    _u: PhantomData<U>,
    _s: PhantomData<S>,
}

impl<Store, U, S> Reserving<Store, U, S> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            _u: PhantomData,
            _s: PhantomData,
        }
    }
}

impl<U, S, Store> Reserving<Store, U, S>
where
    U: OpaqueId,
    S: OpaqueId,
    Store: ReservationStore<U, S>,
{
    /// 【小白】盖章。这个时段已经有人订了，或你今天已经订过 → 拒绝，本子一字不改。
    pub fn reserve(
        &mut self,
        user: U,
        slot: S,
        time: TimeOfDay,
        date: Date,
        party_size: PartySize,
    ) -> Result<ReservationId, ReserveError> {
        if self.store.has_booked_slot(&slot) || self.store.has_booked_user_date(&user, &date) {
            return Err(ReserveError::Conflict);
        }
        let id = self.store.next_id();
        self.store.insert(Reservation {
            id: id.clone(),
            user,
            slot,
            time,
            date,
            party_size,
            status: Status::Booked,
        });
        Ok(id)
    }

    /// 用户主动放弃承诺。canceled 后该 slot / 该日可再被 reserve。
    pub fn cancel(&mut self, reservation: &ReservationId) -> Result<(), CancelError> {
        let mut rec = self.store.get(reservation).ok_or(CancelError::NotBooked)?;
        if rec.status != Status::Booked {
            return Err(CancelError::NotBooked);
        }
        rec.status = Status::Canceled;
        self.store.update(rec);
        Ok(())
    }

    /// 【小白】客人来了，把「已预订」改成「已入座」。只有预约当天能盖。
    pub fn grant(&mut self, reservation: &ReservationId, now: &DateTime) -> Result<(), GrantError> {
        let mut rec = self.store.get(reservation).ok_or(GrantError::NotGrantable)?;
        if rec.status != Status::Booked || now.date != rec.date {
            return Err(GrantError::NotGrantable);
        }
        rec.status = Status::Granted;
        self.store.update(rec);
        Ok(())
    }

    /// 【小白】过点没来，把承诺划掉。扣不扣信誉不归本本管，门口的人再去小黑板。
    pub fn no_show(
        &mut self,
        reservation: &ReservationId,
        now: &DateTime,
    ) -> Result<(), NoShowError> {
        let mut rec = self.store.get(reservation).ok_or(NoShowError::NotNoShow)?;
        if rec.status != Status::Booked || !now.is_after(&rec.date, &rec.time) {
            return Err(NoShowError::NotNoShow);
        }
        rec.status = Status::Canceled;
        self.store.update(rec);
        Ok(())
    }

    /// 只读 query `_user`：给 punishNoShow 绑定「这个预约是谁的」。
    pub fn user_of(&self, reservation: &ReservationId) -> Option<U> {
        self.store.get(reservation).map(|r| r.user)
    }
}
