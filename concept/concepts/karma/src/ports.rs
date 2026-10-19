//! 【小白】小黑板存取说明书：每人一个数字。

use restaurant_shared_kernel::OpaqueId;

pub trait KarmaStore<U: OpaqueId> {
    fn strikes_of(&self, user: &U) -> u32;
    fn set_strikes(&mut self, user: &U, strikes: u32);
}
