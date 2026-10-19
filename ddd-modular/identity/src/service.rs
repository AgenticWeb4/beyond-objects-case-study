//! Identity 应用服务：注册、登录、用 session 换 UserId。

use restaurant_ddd_kernel::{SessionId, UserId};

use crate::domain::{LoginError, RegisterError, User};
use crate::ports::IdentityStore;

pub struct IdentityService<Store> {
    store: Store,
}

impl<Store> IdentityService<Store> {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store: IdentityStore> IdentityService<Store> {
    pub fn register(&mut self, username: &str, password: &str) -> Result<UserId, RegisterError> {
        if self.store.username_taken(username) {
            return Err(RegisterError::UsernameTaken);
        }
        let id = self.store.next_user_id();
        self.store.insert_user(&User {
            id: id.clone(),
            username: username.to_string(),
            password: password.to_string(),
        });
        Ok(id)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<SessionId, LoginError> {
        let user = self
            .store
            .find_by_username(username)
            .ok_or(LoginError::InvalidCredentials)?;
        if !user.check_password(password) {
            return Err(LoginError::InvalidCredentials);
        }
        let session = self.store.next_session_id();
        self.store.insert_session(&session, &user.id);
        Ok(session)
    }

    /// 【小白】工牌换编号。查不到 = 没登录。信誉和订位不在这本里查。
    pub fn user_of(&self, session: &SessionId) -> Option<UserId> {
        self.store.user_of_session(session)
    }
}
