//! 【小白】登录用的人。口袋里只有用户名和密码。

use restaurant_ddd_kernel::UserId;

/// Identity 上下文的 User 聚合。对比 `ddd-clean` 的 User：那边还装着 strikes 和 booked_dates。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub username: String,
    /// 演示用明文。不要从这里学存密码。
    pub password: String,
}

impl User {
    pub fn check_password(&self, password: &str) -> bool {
        self.password == password
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterError {
    UsernameTaken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoginError {
    InvalidCredentials,
}
