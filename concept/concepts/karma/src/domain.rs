//! 阈值 2 是演示确认值。论文只说 “hasn’t been punished too many times”。

/// good standing 当且仅当 strikes < 2。
pub const STANDING_THRESHOLD: u32 = 2;
