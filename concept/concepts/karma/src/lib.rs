//! 【小白】信誉小黑板：放鸽子就记一笔。满两笔就不能再订。
//! 小黑板不认识「预约」这两个字；是门口的人在划掉订座之后，过来记一笔。

pub mod actions;
pub mod adapters;
pub mod domain;
pub mod ports;

pub use actions::Karma;
pub use adapters::sqlite::SqliteKarmaStore;
pub use domain::STANDING_THRESHOLD;
