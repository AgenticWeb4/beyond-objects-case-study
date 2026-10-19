use crate::domain::ids::{Date, RestaurantId, SlotId, TimeOfDay};

/// 【小白】餐厅对象里面装着所有位子。因为网页要「打开餐厅看时段」，位子就住进店里。
/// 概念侧：餐厅只是一张标签，位子在空位牌那本账上。
#[derive(Clone, Debug)]
pub struct Restaurant {
    pub id: RestaurantId,
    pub name: String,
    pub slots: Vec<Slot>,
}

/// 【小白】一个位子。`reserved=true` 同时表示「有人订了」和「搜索别再显示」。
/// 两件事焊在一起，所以 Alice 一订，Bob 就搜不到。
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

    /// 【小白】可订 = 还没被标占用。概念侧空位牌不看订座本，所以不会用这一列。
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
