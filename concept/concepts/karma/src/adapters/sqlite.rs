//! `karma.sqlite`。user_id 仍是字符串，不引用 authenticating 库。

use std::marker::PhantomData;
use std::path::Path;

use restaurant_shared_kernel::OpaqueId;
use rusqlite::{params, Connection, OptionalExtension};

use crate::ports::KarmaStore;

pub struct SqliteKarmaStore<U> {
    conn: Connection,
    _u: PhantomData<U>,
}

impl<U: OpaqueId> SqliteKarmaStore<U> {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    pub fn memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS strikes (
                user_id TEXT PRIMARY KEY,
                count INTEGER NOT NULL
            );
            ",
        )?;
        Ok(Self {
            conn,
            _u: PhantomData,
        })
    }
}

impl<U: OpaqueId> KarmaStore<U> for SqliteKarmaStore<U> {
    fn strikes_of(&self, user: &U) -> u32 {
        self.conn
            .query_row(
                "SELECT count FROM strikes WHERE user_id = ?1",
                params![user.as_str()],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .expect("karma strikes")
            .map(|n| n as u32)
            .unwrap_or(0)
    }

    fn set_strikes(&mut self, user: &U, strikes: u32) {
        self.conn
            .execute(
                "INSERT INTO strikes (user_id, count) VALUES (?1, ?2)
                 ON CONFLICT(user_id) DO UPDATE SET count = excluded.count",
                params![user.as_str(), strikes as i64],
            )
            .expect("karma set_strikes");
    }
}
