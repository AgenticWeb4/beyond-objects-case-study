//! 【小白】预测时段。`listed=true` 只表示「牌还挂着」，不是「没人订」。

use restaurant_ddd_kernel::{Date, PartySize, SlotId, TimeOfDay, VenueId};

/// 一个可订预测单元。不是 Restaurant 聚合里的小孩。
/// `ddd-clean` 把 Slot 塞进 Restaurant，还焊上 `reserved`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: SlotId,
    pub venue: VenueId,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub date: Date,
    pub min_party: PartySize,
    pub max_party: PartySize,
    /// 【小白】true = 门口还挂着。跟 Booking 有没有占用无关。
    pub listed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateError {
    InvalidWindow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToggleError {
    NotFound,
}
