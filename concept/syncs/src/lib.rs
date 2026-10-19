//! 【小白】这是「门口的人」。
//!
//! 四本账（登录、空位、订座、信誉）彼此不打电话。客人说「我要订位」时，
//! 由本文件按顺序问三句，全通过才去订座本盖章：
//! 1. 工牌还有效吗？
//! 2. 信誉还够吗？
//! 3. 空位牌上还有这个点吗？
//!
//! 问句任何一个答不上来：不去改订座本，直接告诉客人原因。
//! DDD 那边是打开 User、打开 Restaurant，一次改三个对象——见 `ddd-clean` 的 `BookingService`。
//!
//! 【进阶】论文 Fig.3 `makeReservation` / `punishNoShow`；规格 [`SYNCS.md`](../SYNCS.md)。

use restaurant_authenticating::{Authenticating, LoginError, RegisterError, SessionId, UserId};
use restaurant_availability::{Availability, CreateError, SlotId};
use restaurant_karma::Karma;
use restaurant_reserving::{
    CancelError, GrantError, NoShowError, ReservationId, ReserveError, Reserving,
};
use restaurant_shared_kernel::{opaque_id, Date, DateTime, OpaqueId, PartySize, TimeOfDay};

/// 【小白】餐厅的名字/编号。这里不当成「餐厅对象」，只当一张标签。
opaque_id!(VenueId);

/// 【小白】门口的人手里拿着四本账的钥匙。他自己没有第五本账。
pub struct RestaurantSyncs<AvailStore, ReserveStore, AuthStore, KarmaStore> {
    pub availability: Availability<AvailStore, VenueId>,
    pub reserving: Reserving<ReserveStore, UserId, SlotId>,
    pub authenticating: Authenticating<AuthStore>,
    pub karma: Karma<KarmaStore, UserId>,
}

/// 【小白】订位这件事可能的几种答复。都是正常业务结果，不是程序崩溃。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReserveOutcome {
    /// 盖章成功，这是预约编号
    Reserved(ReservationId),
    /// 工牌无效（没登录或 session 过期）
    Unauthorized,
    /// 信誉不够（放鸽子太多次）
    BadStanding,
    /// 空位牌上没有这个点
    NoSlot,
    /// 订座本上已经有人占了（或你当天已经订过）
    Conflict,
}

impl<AvailStore, ReserveStore, AuthStoreImpl, KarmaStore>
    RestaurantSyncs<AvailStore, ReserveStore, AuthStoreImpl, KarmaStore>
where
    AvailStore: restaurant_availability::ports::SlotStore<VenueId>,
    ReserveStore: restaurant_reserving::ports::ReservationStore<UserId, SlotId>,
    AuthStoreImpl: restaurant_authenticating::ports::AuthStore,
    KarmaStore: restaurant_karma::ports::KarmaStore<UserId>,
{
    pub fn new(
        availability: Availability<AvailStore, VenueId>,
        reserving: Reserving<ReserveStore, UserId, SlotId>,
        authenticating: Authenticating<AuthStoreImpl>,
        karma: Karma<KarmaStore, UserId>,
    ) -> Self {
        Self {
            availability,
            reserving,
            authenticating,
            karma,
        }
    }

    pub fn register(&mut self, username: &str, password: &str) -> Result<UserId, RegisterError> {
        self.authenticating.register(username, password)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<SessionId, LoginError> {
        self.authenticating.login(username, password)
    }

    pub fn publish(
        &mut self,
        venue: VenueId,
        start: TimeOfDay,
        end: TimeOfDay,
        date: Date,
        min_party: PartySize,
        max_party: PartySize,
    ) -> Result<SlotId, CreateError> {
        self.availability
            .create(venue, start, end, date, min_party, max_party)
    }

    pub fn available(
        &self,
        venue: &VenueId,
        time: &TimeOfDay,
        date: &Date,
        party_size: PartySize,
    ) -> Vec<SlotId> {
        self.availability
            .get_available_slot(venue, time, date, party_size)
    }

    /// 【小白】客人说「我要订」。门口的人问三句，再决定去不去订座本。
    ///
    /// 订成功后，**空位牌默认不摘下来**。所以再搜一次往往还能看见。
    /// DDD 那边会立刻把位子标成占用，搜索就空了。这是两套结构最大的可见差别。
    pub fn reserve(
        &mut self,
        session: &SessionId,
        venue: &VenueId,
        time: TimeOfDay,
        date: Date,
        party_size: PartySize,
    ) -> ReserveOutcome {
        // 1. 工牌
        let Some(user) = self.authenticating.get_user(session) else {
            return ReserveOutcome::Unauthorized;
        };
        // 2. 小黑板
        if self.karma.in_good_standing(&user).is_none() {
            return ReserveOutcome::BadStanding;
        }
        // 3. 空位牌（可能 0 个、1 个、多个；多个时按编号挑一个，好复现）
        let mut slots = self
            .availability
            .get_available_slot(venue, &time, &date, party_size);
        slots.sort();
        let Some(slot) = slots.into_iter().next() else {
            return ReserveOutcome::NoSlot;
        };
        // 4. 只有现在才写订座本。冲突是订座本自己的规矩，不是空位牌的规矩。
        match self
            .reserving
            .reserve(user, slot, time, date, party_size)
        {
            Ok(id) => ReserveOutcome::Reserved(id),
            Err(ReserveError::Conflict) => ReserveOutcome::Conflict,
        }
    }

    pub fn cancel(&mut self, reservation: &ReservationId) -> Result<(), CancelError> {
        self.reserving.cancel(reservation)
    }

    pub fn grant(
        &mut self,
        reservation: &ReservationId,
        now: &DateTime,
    ) -> Result<(), GrantError> {
        self.reserving.grant(reservation, now)
    }

    /// 【小白】放鸽子：先在订座本上划掉，再去小黑板记一笔。
    /// 如果订座本拒绝划（还没到点、或已经取消），小黑板不动。
    pub fn no_show(
        &mut self,
        reservation: &ReservationId,
        now: &DateTime,
    ) -> Result<(), NoShowError> {
        self.reserving.no_show(reservation, now)?;
        if let Some(user) = self.reserving.user_of(reservation) {
            self.karma.punish(&user);
        }
        Ok(())
    }
}

impl VenueId {
    pub fn new(raw: &str) -> Self {
        VenueId::from_string(raw.to_string())
    }
}
