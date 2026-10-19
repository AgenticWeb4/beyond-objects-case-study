//! 【小白】空位牌上有哪些格子。`available=true` 只表示「牌还挂着」，不是「没人订」。

use restaurant_shared_kernel::{opaque_id, Date, PartySize, TimeOfDay};

opaque_id!(SlotId);

/// 一个可订预测单元。`V` 是场所身份（应用里通常是餐厅名/ID）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot<V> {
    pub id: SlotId,
    pub venue: V,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub date: Date,
    pub min_party: PartySize,
    pub max_party: PartySize,
    /// 【小白】true = 门口还挂着。跟订座本有没有盖章无关。
    pub available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateError {
    InvalidWindow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToggleError {
    NotFound,
}
