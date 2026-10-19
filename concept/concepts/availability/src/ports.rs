//! 【小白】空位牌的存取说明书。查找时不会去问订座本。

use restaurant_shared_kernel::{Date, OpaqueId, PartySize, TimeOfDay};

use crate::domain::{Slot, SlotId};

pub trait SlotStore<V: OpaqueId> {
    fn insert(&mut self, slot: Slot<V>);
    fn get(&self, id: &SlotId) -> Option<Slot<V>>;
    fn update_available(&mut self, id: &SlotId, available: bool);
    fn next_id(&mut self) -> SlotId;
    fn available_matching(
        &self,
        venue: &V,
        time: &TimeOfDay,
        date: &Date,
        party_size: PartySize,
    ) -> Vec<SlotId>;
}
