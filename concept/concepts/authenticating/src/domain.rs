//! 用户与会话身份。Username 全局唯一。

use restaurant_shared_kernel::opaque_id;

opaque_id!(UserId);
opaque_id!(SessionId);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterError {
    UsernameTaken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoginError {
    InvalidCredentials,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogoutError {
    NotFound,
}
