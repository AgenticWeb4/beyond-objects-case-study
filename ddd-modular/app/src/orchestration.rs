//! 【小白】跨上下文的用例编排。这里像 `concept/syncs`，但不是。
//!
//! 门口的人（syncs）问的是四台概念机的**动作完成**。
//! 这边问的是四个限界上下文的**应用服务 / 仓储**：
//! 先问 Identity 要 UserId，再问 Reputation 信誉，再问 Availability 有没有挂牌，
//! 最后才叫 Booking 去改 Guest / Occupancy / Reservation。
//!
//! `when / where / then` 不在这里。没有完成事件。就是一段 use case。

use restaurant_ddd_availability::{AvailabilityService, SlotId, SqliteSlotStore, VenueId};
use restaurant_ddd_booking::{BookingFacade, ReservationId, SqliteBookingStore};
use restaurant_ddd_identity::{IdentityService, SessionId, SqliteIdentityStore};
use restaurant_ddd_kernel::{Date, DateTime, PartySize, TimeOfDay};
use restaurant_ddd_reputation::{ReputationService, SqliteReputationStore};

pub struct RestaurantApp {
    pub identity: IdentityService<SqliteIdentityStore>,
    pub availability: AvailabilityService<SqliteSlotStore>,
    pub booking: BookingFacade<SqliteBookingStore>,
    pub reputation: ReputationService<SqliteReputationStore>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReserveOutcome {
    Reserved(ReservationId),
    Unauthorized,
    BadStanding,
    NoSlot,
    Conflict,
}

impl RestaurantApp {
    pub fn new(
        identity: IdentityService<SqliteIdentityStore>,
        availability: AvailabilityService<SqliteSlotStore>,
        booking: BookingFacade<SqliteBookingStore>,
        reputation: ReputationService<SqliteReputationStore>,
    ) -> Self {
        Self {
            identity,
            availability,
            booking,
            reputation,
        }
    }

    pub fn register(&mut self, username: &str, password: &str) -> Result<String, String> {
        self.identity
            .register(username, password)
            .map(|id| id.0)
            .map_err(|e| format!("{e:?}"))
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<SessionId, String> {
        self.identity
            .login(username, password)
            .map_err(|e| format!("{e:?}"))
    }

    pub fn publish(
        &mut self,
        venue: VenueId,
        start: TimeOfDay,
        end: TimeOfDay,
        date: Date,
        min_party: PartySize,
        max_party: PartySize,
    ) -> Result<SlotId, String> {
        self.availability
            .publish(venue, start, end, date, min_party, max_party)
            .map_err(|e| format!("{e:?}"))
    }

    /// 【小白】只问 Availability。Booking 占了位也不会从这张名单上摘掉。
    pub fn available(
        &self,
        venue: &VenueId,
        time: &TimeOfDay,
        date: &Date,
        party: PartySize,
    ) -> Vec<SlotId> {
        self.availability.find_listed(venue, time, date, party)
    }

    /// 【小白】看起来像 syncs 的 reserve：问三句再写。但问的是 BC API，不是概念动作。
    /// 写成功后**不会**叫 Availability.toggle——占用只记在 Booking。
    pub fn reserve(
        &mut self,
        session: &SessionId,
        venue: &VenueId,
        time: TimeOfDay,
        date: Date,
        party: PartySize,
    ) -> ReserveOutcome {
        let Some(user) = self.identity.user_of(session) else {
            return ReserveOutcome::Unauthorized;
        };
        if !self.reputation.in_good_standing(&user) {
            return ReserveOutcome::BadStanding;
        }
        let mut slots = self.availability.find_listed(venue, &time, &date, party);
        slots.sort();
        let Some(slot) = slots.into_iter().next() else {
            return ReserveOutcome::NoSlot;
        };
        match self.booking.reserve(user, slot, time, date, party) {
            Ok(id) => ReserveOutcome::Reserved(id),
            Err(restaurant_ddd_booking::ReserveError::Conflict) => ReserveOutcome::Conflict,
        }
    }

    pub fn cancel(&mut self, reservation: &ReservationId) -> Result<(), String> {
        self.booking
            .cancel(reservation)
            .map_err(|e| format!("{e:?}"))
    }

    pub fn grant(&mut self, reservation: &ReservationId, now: &DateTime) -> Result<(), String> {
        self.booking
            .grant(reservation, now)
            .map_err(|e| format!("{e:?}"))
    }

    /// 【小白】放鸽子：Booking 划掉订单并释放占用，再另走 Reputation.punish。
    /// 两步都在本用例里顺序调用，不是 punishNoShow 那条 sync。
    pub fn no_show(&mut self, reservation: &ReservationId, now: &DateTime) -> Result<(), String> {
        let user = self
            .booking
            .no_show(reservation, now)
            .map_err(|e| format!("{e:?}"))?;
        self.reputation.punish(&user);
        Ok(())
    }
}
