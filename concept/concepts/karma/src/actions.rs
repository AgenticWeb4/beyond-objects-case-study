//! 【小白】小黑板：记一笔、问还能不能订。

use std::marker::PhantomData;

use restaurant_shared_kernel::OpaqueId;

use crate::domain::STANDING_THRESHOLD;
use crate::ports::KarmaStore;

pub struct Karma<Store, U> {
    store: Store,
    _u: PhantomData<U>,
}

impl<Store, U> Karma<Store, U> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            _u: PhantomData,
        }
    }
}

impl<U, Store> Karma<Store, U>
where
    U: OpaqueId,
    Store: KarmaStore<U>,
{
    pub fn punish(&mut self, user: &U) {
        let next = self.store.strikes_of(user) + 1;
        self.store.set_strikes(user, next);
    }

    /// 【小白】问：这个人还能订吗？答不上来（None）= 资格不够。
    pub fn in_good_standing(&self, user: &U) -> Option<U> {
        if self.store.strikes_of(user) < STANDING_THRESHOLD {
            Some(user.clone())
        } else {
            None
        }
    }
}
