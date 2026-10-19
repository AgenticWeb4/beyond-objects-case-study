use restaurant_ddd_kernel::{Date, PartySize, SlotId, TimeOfDay, VenueId};

use crate::domain::Slot;

pub trait SlotStore {
    fn insert(&mut self, slot: Slot);
    fn get(&self, id: &SlotId) -> Option<Slot>;
    fn update_listed(&mut self, id: &SlotId, listed: bool);
    fn next_id(&mut self) -> SlotId;
    fn listed_matching(
        &self,
        venue: &VenueId,
        time: &TimeOfDay,
        date: &Date,
        party_size: PartySize,
    ) -> Vec<SlotId>;
}
