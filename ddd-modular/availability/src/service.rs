//! Availability 应用服务：挂牌、翻牌、按条件查找。没有「订位」按钮。

use restaurant_ddd_kernel::{Date, PartySize, SlotId, TimeOfDay, VenueId};

use crate::domain::{CreateError, Slot, ToggleError};
use crate::ports::SlotStore;

pub struct AvailabilityService<Store> {
    store: Store,
}

impl<Store> AvailabilityService<Store> {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store: SlotStore> AvailabilityService<Store> {
    pub fn publish(
        &mut self,
        venue: VenueId,
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
            listed: true,
        });
        Ok(id)
    }

    /// 【小白】把牌子翻一面。若产品想「订成功就摘牌」，该由 app 用例来按，
    /// 而不是 Availability 去读 Booking。本演示默认不按。
    pub fn toggle(&mut self, slot: &SlotId) -> Result<(), ToggleError> {
        let rec = self.store.get(slot).ok_or(ToggleError::NotFound)?;
        self.store.update_listed(slot, !rec.listed);
        Ok(())
    }

    pub fn find_listed(
        &self,
        venue: &VenueId,
        time: &TimeOfDay,
        date: &Date,
        party_size: PartySize,
    ) -> Vec<SlotId> {
        self.store.listed_matching(venue, time, date, party_size)
    }
}
