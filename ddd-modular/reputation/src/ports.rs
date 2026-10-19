use restaurant_ddd_kernel::UserId;

pub trait ReputationStore {
    fn strikes_of(&self, user: &UserId) -> u32;
    fn set_strikes(&mut self, user: &UserId, strikes: u32);
}
