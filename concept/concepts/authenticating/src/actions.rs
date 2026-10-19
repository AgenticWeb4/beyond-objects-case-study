//! 【小白】工牌账能做的事：办工牌、刷工牌进门、问「这张临时卡是谁」。

use crate::domain::{LoginError, LogoutError, RegisterError, SessionId, UserId};
use crate::ports::AuthStore;

pub struct Authenticating<Store> {
    store: Store,
}

impl<Store> Authenticating<Store> {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store: AuthStore> Authenticating<Store> {
    pub fn register(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<UserId, RegisterError> {
        if self.store.username_taken(username) {
            return Err(RegisterError::UsernameTaken);
        }
        let id = self.store.next_user_id();
        self.store.insert_user(&id, username, password);
        Ok(id)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<SessionId, LoginError> {
        let user = self
            .store
            .find_user(username, password)
            .ok_or(LoginError::InvalidCredentials)?;
        let session = self.store.next_session_id();
        self.store.insert_session(&session, &user);
        Ok(session)
    }

    pub fn logout(&mut self, session: &SessionId) -> Result<(), LogoutError> {
        if self.store.delete_session(session) {
            Ok(())
        } else {
            Err(LogoutError::NotFound)
        }
    }

    /// 【小白】拿着登录后拿到的那串 session，查出用户编号。查不到就当没登录。
    pub fn get_user(&self, session: &SessionId) -> Option<UserId> {
        self.store.user_of_session(session)
    }
}
