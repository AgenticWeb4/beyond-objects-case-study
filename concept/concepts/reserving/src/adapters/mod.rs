//! SQLite adapter。本文件是 Reserving 的私人账本。
//!
//! `user_id` / `slot_id` 存 TEXT，**没有** `REFERENCES` 到认证库或时段库。
//! 部分唯一索引落实「一槽一约 / 同日一人一约」，与动作里的检查双保险。

pub mod sqlite;
