//! Identity 仓储。本上下文自己的表，不引用 booking / reputation。

use restaurant_ddd_kernel::{SessionId, UserId};

use crate::domain::User;

pub trait IdentityStore {
    fn username_taken(&self, username: &str) -> bool;
    fn insert_user(&mut self, user: &User);
    fn find_by_username(&self, username: &str) -> Option<User>;
    fn next_user_id(&mut self) -> UserId;
    fn next_session_id(&mut self) -> SessionId;
    fn insert_session(&mut self, session: &SessionId, user: &UserId);
    fn user_of_session(&self, session: &SessionId) -> Option<UserId>;
}
