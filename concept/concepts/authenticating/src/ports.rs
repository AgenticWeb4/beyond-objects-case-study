//! 【小白】工牌账的存取说明书。不要在这里加信誉或订座列表。

use crate::domain::{SessionId, UserId};

pub trait AuthStore {
    fn username_taken(&self, username: &str) -> bool;
    fn insert_user(&mut self, id: &UserId, username: &str, password: &str);
    fn find_user(&self, username: &str, password: &str) -> Option<UserId>;
    fn next_user_id(&mut self) -> UserId;
    fn next_session_id(&mut self) -> SessionId;
    fn insert_session(&mut self, id: &SessionId, user: &UserId);
    fn user_of_session(&self, session: &SessionId) -> Option<UserId>;
    fn delete_session(&mut self, session: &SessionId) -> bool;
}
