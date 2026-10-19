//! 【小白】这是「店」这个类，单独一个 crate。
//!
//! 位子仍住在店里，`Slot.reserved` 仍把「有人订了」和「搜索别显示」焊在一起。
//! 分包没有把预测和承诺拆开。

use restaurant_oop_shared::{Date, TimeOfDay};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RestaurantId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotId(pub String);

impl std::fmt::Display for RestaurantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Restaurant {
    pub id: RestaurantId,
    pub name: String,
    pub slots: Vec<Slot>,
}

/// `reserved=true` 同时表示承诺占用和搜索隐藏。Alice 一订，Bob 就搜不到。
#[derive(Clone, Debug)]
pub struct Slot {
    pub id: SlotId,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub date: Date,
    pub min_party: u32,
    pub max_party: u32,
    pub reserved: bool,
}

impl Restaurant {
    pub fn add_slot(&mut self, slot: Slot) {
        self.slots.push(slot);
    }

    pub fn find_available_mut(
        &mut self,
        time: &TimeOfDay,
        date: &Date,
        party: u32,
    ) -> Option<&mut Slot> {
        self.slots.iter_mut().find(|s| {
            !s.reserved
                && s.start == *time
                && s.date == *date
                && s.min_party <= party
                && s.max_party >= party
        })
    }

    pub fn available_ids(&self, time: &TimeOfDay, date: &Date, party: u32) -> Vec<SlotId> {
        self.slots
            .iter()
            .filter(|s| {
                !s.reserved
                    && s.start == *time
                    && s.date == *date
                    && s.min_party <= party
                    && s.max_party >= party
            })
            .map(|s| s.id.clone())
            .collect()
    }

    pub fn slot_mut(&mut self, id: &SlotId) -> Option<&mut Slot> {
        self.slots.iter_mut().find(|s| s.id == *id)
    }
}
