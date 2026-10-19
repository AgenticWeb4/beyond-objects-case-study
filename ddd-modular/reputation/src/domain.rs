//! 阈值 2 与另两栏演示一致。论文只说 “hasn’t been punished too many times”。

/// good standing 当且仅当 strikes < 2。
pub const STANDING_THRESHOLD: u32 = 2;

use restaurant_ddd_kernel::UserId;

/// 【小白】一块小黑板上的一行。只有「这个人编号 + 被记了几次」。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Standing {
    pub user_id: UserId,
    pub strikes: u32,
}

impl Standing {
    pub fn in_good_standing(&self) -> bool {
        self.strikes < STANDING_THRESHOLD
    }

    pub fn punish(&mut self) {
        self.strikes += 1;
    }
}
