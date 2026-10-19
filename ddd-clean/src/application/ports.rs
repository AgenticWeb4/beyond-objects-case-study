//! 【小白】「请帮我存/取 User、Restaurant…」的说明书。下面由 SQLite 兑现。

use crate::domain::ids::{ReservationId, RestaurantId, SessionId, UserId};
use crate::domain::reservation::Reservation;
use crate::domain::restaurant::Restaurant;
use crate::domain::user::User;

pub trait UserRepository {
    fn save_user(&mut self, user: &User);
    fn get_user(&self, id: &UserId) -> Option<User>;
    fn by_username(&self, username: &str) -> Option<User>;
    fn next_user_id(&mut self) -> UserId;
}

pub trait SessionRepository {
    fn save_session(&mut self, session: &SessionId, user: &UserId);
    fn user_of(&self, session: &SessionId) -> Option<UserId>;
    fn next_session_id(&mut self) -> SessionId;
}

pub trait RestaurantRepository {
    fn save_restaurant(&mut self, restaurant: &Restaurant);
    fn get_restaurant(&self, id: &RestaurantId) -> Option<Restaurant>;
    fn upsert_named(&mut self, name: &str) -> Restaurant;
}

pub trait ReservationRepository {
    fn save_reservation(&mut self, reservation: &Reservation);
    fn get_reservation(&self, id: &ReservationId) -> Option<Reservation>;
    fn next_reservation_id(&mut self) -> ReservationId;
}
