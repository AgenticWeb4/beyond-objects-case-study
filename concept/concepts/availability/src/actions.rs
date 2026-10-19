//! 【小白】空位牌能做的事：挂上去、翻面、按时间人数查找。没有「订位」按钮。

use std::marker::PhantomData;

use restaurant_shared_kernel::{Date, OpaqueId, PartySize, TimeOfDay};

use crate::domain::{CreateError, Slot, SlotId, ToggleError};
use crate::ports::SlotStore;

pub struct Availability<Store, V> {
    store: Store,
    _v: PhantomData<V>,
}

impl<Store, V> Availability<Store, V> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            _v: PhantomData,
        }
    }
}

impl<V, Store> Availability<Store, V>
where
    V: OpaqueId,
    Store: SlotStore<V>,
{
    /// 创建一个对外提供的时段。end 必须晚于 start，人数上下限合法。
    pub fn create(
        &mut self,
        venue: V,
        start: TimeOfDay,
        end: TimeOfDay,
        date: Date,
        min_party: PartySize,
        max_party: PartySize,
    ) -> Result<SlotId, CreateError> {
        if !end.is_after(&start) || max_party < min_party {
            return Err(CreateError::InvalidWindow);
        }
        let id = self.store.next_id();
        self.store.insert(Slot {
            id: id.clone(),
            venue,
            start,
            end,
            date,
            min_party,
            max_party,
            available: true,
        });
        Ok(id)
    }

    /// 【小白】把牌子翻一面：还挂着 ↔ 摘下来。
    /// 若产品想「订成功就摘牌」，应该让门口的人来按这个按钮，而不是空位牌去偷看订座本。
    pub fn toggle_available(&mut self, slot: &SlotId) -> Result<(), ToggleError> {
        let rec = self.store.get(slot).ok_or(ToggleError::NotFound)?;
        self.store.update_available(slot, !rec.available);
        Ok(())
    }

    /// 【小白】按餐厅、时间、人数把还挂着的牌找出来。可能一张都没有，也可能好几张。
    pub fn get_available_slot(
        &self,
        venue: &V,
        time: &TimeOfDay,
        date: &Date,
        party_size: PartySize,
    ) -> Vec<SlotId> {
        self.store
            .available_matching(venue, time, date, party_size)
    }
}
