//! Reputation 应用服务：记一笔、问还能不能订。

use restaurant_ddd_kernel::UserId;

use crate::domain::STANDING_THRESHOLD;
use crate::ports::ReputationStore;

pub struct ReputationService<Store> {
    store: Store,
}

impl<Store> ReputationService<Store> {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store: ReputationStore> ReputationService<Store> {
    pub fn punish(&mut self, user: &UserId) {
        let next = self.store.strikes_of(user) + 1;
        self.store.set_strikes(user, next);
    }

    /// 【小白】问：这个人信誉还够吗？不够就不让订。次数记在本上下文，不在 User 口袋。
    pub fn in_good_standing(&self, user: &UserId) -> bool {
        self.store.strikes_of(user) < STANDING_THRESHOLD
    }
}
