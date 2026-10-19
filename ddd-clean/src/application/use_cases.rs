//! 【小白】一次订位：先把人和店从库里取出，交给 BookingService 同时改，再存回去。
//! 对比概念侧门口的人：先只读问三句，最后只写订座本。

use crate::application::ports::{
    ReservationRepository, RestaurantRepository, SessionRepository, UserRepository,
};
use crate::domain::booking::{BookingService, ReserveError};
use crate::domain::ids::{Date, DateTime, ReservationId, RestaurantId, SessionId, TimeOfDay};
use crate::domain::reservation::ReservationStatus;
use crate::domain::restaurant::Slot;
use crate::domain::user::User;

pub struct RestaurantApp<Db> {
    pub db: Db,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReserveOutcome {
    Reserved(ReservationId),
    Unauthorized,
    BadStanding,
    NoSlot,
    Conflict,
}

impl From<ReserveError> for ReserveOutcome {
    fn from(err: ReserveError) -> Self {
        match err {
            ReserveError::BadStanding => ReserveOutcome::BadStanding,
            ReserveError::NoSlot => ReserveOutcome::NoSlot,
            ReserveError::Conflict => ReserveOutcome::Conflict,
        }
    }
}

impl<Db> RestaurantApp<Db>
where
    Db: UserRepository + SessionRepository + RestaurantRepository + ReservationRepository,
{
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn register(&mut self, username: &str, password: &str) -> Result<String, String> {
        if self.db.by_username(username).is_some() {
            return Err("UsernameTaken".into());
        }
        let id = self.db.next_user_id();
        self.db.save_user(&User {
            id: id.clone(),
            username: username.to_string(),
            password: password.to_string(),
            karma_strikes: 0,
            booked_dates: vec![],
        });
        Ok(id.0)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<SessionId, String> {
        let user = self.db.by_username(username).ok_or("InvalidCredentials")?;
        if !user.check_password(password) {
            return Err("InvalidCredentials".into());
        }
        let session = self.db.next_session_id();
        self.db.save_session(&session, &user.id);
        Ok(session)
    }

    pub fn publish(
        &mut self,
        venue: &str,
        start: TimeOfDay,
        end: TimeOfDay,
        date: Date,
        min_party: u32,
        max_party: u32,
    ) -> Result<String, String> {
        if end.0 <= start.0 || max_party < min_party {
            return Err("InvalidWindow".into());
        }
        let mut restaurant = self.db.upsert_named(venue);
        let n = restaurant.slots.len() + 1;
        restaurant.add_slot(Slot {
            id: crate::domain::ids::SlotId(format!("{}-S{n}", restaurant.id.0)),
            start,
            end,
            date,
            min_party,
            max_party,
            reserved: false,
        });
        let id = restaurant.slots.last().unwrap().id.clone();
        self.db.save_restaurant(&restaurant);
        Ok(id.0)
    }

    pub fn available(&self, venue: &str, time: &TimeOfDay, date: &Date, party: u32) -> Vec<String> {
        let Some(restaurant) = self.db.get_restaurant(&RestaurantId(venue.to_string())) else {
            return vec![];
        };
        restaurant
            .available_ids(time, date, party)
            .into_iter()
            .map(|s| s.0)
            .collect()
    }

    /// 【小白】加载人和店 → 一次改三处 → 分别存盘。
    pub fn reserve(
        &mut self,
        session: &SessionId,
        venue: &str,
        time: TimeOfDay,
        date: Date,
        party: u32,
    ) -> ReserveOutcome {
        let Some(user_id) = self.db.user_of(session) else {
            return ReserveOutcome::Unauthorized;
        };
        let mut user = match self.db.get_user(&user_id) {
            Some(u) => u,
            None => return ReserveOutcome::Unauthorized,
        };
        let mut restaurant = match self.db.get_restaurant(&RestaurantId(venue.to_string())) {
            Some(r) => r,
            None => return ReserveOutcome::NoSlot,
        };
        let new_id = self.db.next_reservation_id();
        match BookingService::reserve(&mut user, &mut restaurant, time, date, party, new_id) {
            Ok(reservation) => {
                self.db.save_user(&user);
                self.db.save_restaurant(&restaurant);
                let id = reservation.id.clone();
                self.db.save_reservation(&reservation);
                ReserveOutcome::Reserved(id)
            }
            Err(err) => err.into(),
        }
    }

    pub fn grant(&mut self, reservation: &ReservationId, now: &DateTime) -> Result<(), String> {
        let mut rec = self.db.get_reservation(reservation).ok_or("NotGrantable")?;
        if rec.status != ReservationStatus::Booked || now.date != rec.date {
            return Err("NotGrantable".into());
        }
        rec.status = ReservationStatus::Granted;
        if let Some(mut user) = self.db.get_user(&rec.user_id) {
            user.release_date(&rec.date);
            self.db.save_user(&user);
        }
        self.db.save_reservation(&rec);
        Ok(())
    }

    /// 【小白】放鸽子：划掉订单、放出位子、在人身上记一笔失信。又是三处一起改。
    pub fn no_show(&mut self, reservation: &ReservationId, now: &DateTime) -> Result<(), String> {
        let mut rec = self.db.get_reservation(reservation).ok_or("NotNoShow")?;
        if rec.status != ReservationStatus::Booked || !now.is_after(&rec.date, &rec.time) {
            return Err("NotNoShow".into());
        }
        rec.status = ReservationStatus::Canceled;
        if let Some(mut user) = self.db.get_user(&rec.user_id) {
            user.release_date(&rec.date);
            user.punish();
            self.db.save_user(&user);
        }
        if let Some(mut restaurant) = self.db.get_restaurant(&rec.restaurant_id) {
            if let Some(slot) = restaurant.slot_mut(&rec.slot_id) {
                slot.reserved = false;
            }
            self.db.save_restaurant(&restaurant);
        }
        self.db.save_reservation(&rec);
        Ok(())
    }

    pub fn cancel(&mut self, reservation: &ReservationId) -> Result<(), String> {
        let mut rec = self.db.get_reservation(reservation).ok_or("NotBooked")?;
        if rec.status != ReservationStatus::Booked {
            return Err("NotBooked".into());
        }
        rec.status = ReservationStatus::Canceled;
        if let Some(mut user) = self.db.get_user(&rec.user_id) {
            user.release_date(&rec.date);
            self.db.save_user(&user);
        }
        if let Some(mut restaurant) = self.db.get_restaurant(&rec.restaurant_id) {
            if let Some(slot) = restaurant.slot_mut(&rec.slot_id) {
                slot.reserved = false;
            }
            self.db.save_restaurant(&restaurant);
        }
        self.db.save_reservation(&rec);
        Ok(())
    }
}
