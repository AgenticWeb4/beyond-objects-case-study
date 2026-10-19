//! 【小白】空位牌对应的 SQLite 文件。没有「已预订」这一列。
//! DDD 那边把 reserved 焊在 slot 上，所以搜索会立刻变空。

use std::marker::PhantomData;
use std::path::Path;

use restaurant_shared_kernel::{Date, OpaqueId, TimeOfDay};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{Slot, SlotId};
use crate::ports::SlotStore;

pub struct SqliteSlotStore<V> {
    conn: Connection,
    _v: PhantomData<V>,
}

impl<V: OpaqueId> SqliteSlotStore<V> {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    pub fn memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS slots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                venue TEXT NOT NULL,
                start TEXT NOT NULL,
                end TEXT NOT NULL,
                date TEXT NOT NULL,
                min_party INTEGER NOT NULL,
                max_party INTEGER NOT NULL,
                available INTEGER NOT NULL
            );
            ",
        )?;
        Ok(Self {
            conn,
            _v: PhantomData,
        })
    }

    fn row_id(id: &SlotId) -> i64 {
        id.as_str()
            .strip_prefix('S')
            .and_then(|s| s.parse().ok())
            .expect("slot id")
    }

    fn to_id(row: i64) -> SlotId {
        SlotId::from_string(format!("S{row}"))
    }
}

impl<V: OpaqueId> SlotStore<V> for SqliteSlotStore<V> {
    fn insert(&mut self, slot: Slot<V>) {
        self.conn
            .execute(
                "INSERT INTO slots (id, venue, start, end, date, min_party, max_party, available)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    Self::row_id(&slot.id),
                    slot.venue.as_str(),
                    slot.start.0,
                    slot.end.0,
                    slot.date.0,
                    slot.min_party as i64,
                    slot.max_party as i64,
                    if slot.available { 1 } else { 0 },
                ],
            )
            .expect("availability insert");
    }

    fn get(&self, id: &SlotId) -> Option<Slot<V>> {
        self.conn
            .query_row(
                "SELECT id, venue, start, end, date, min_party, max_party, available FROM slots WHERE id = ?1",
                params![Self::row_id(id)],
                |row| {
                    let id: i64 = row.get(0)?;
                    let venue: String = row.get(1)?;
                    let start: String = row.get(2)?;
                    let end: String = row.get(3)?;
                    let date: String = row.get(4)?;
                    let min_party: i64 = row.get(5)?;
                    let max_party: i64 = row.get(6)?;
                    let available: i64 = row.get(7)?;
                    Ok(Slot {
                        id: Self::to_id(id),
                        venue: V::from_string(venue),
                        start: TimeOfDay(start),
                        end: TimeOfDay(end),
                        date: Date(date),
                        min_party: min_party as u32,
                        max_party: max_party as u32,
                        available: available != 0,
                    })
                },
            )
            .optional()
            .expect("availability get")
    }

    fn update_available(&mut self, id: &SlotId, available: bool) {
        self.conn
            .execute(
                "UPDATE slots SET available = ?1 WHERE id = ?2",
                params![if available { 1 } else { 0 }, Self::row_id(id)],
            )
            .expect("availability toggle");
    }

    fn next_id(&mut self) -> SlotId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM slots", [], |row| {
                row.get(0)
            })
            .expect("availability next_id");
        Self::to_id(n)
    }

    fn available_matching(
        &self,
        venue: &V,
        time: &TimeOfDay,
        date: &Date,
        party_size: restaurant_shared_kernel::PartySize,
    ) -> Vec<SlotId> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id FROM slots
                 WHERE venue = ?1 AND start = ?2 AND date = ?3
                   AND available = 1 AND min_party <= ?4 AND max_party >= ?4
                 ORDER BY id",
            )
            .expect("availability query");
        let rows = stmt
            .query_map(
                params![venue.as_str(), time.0, date.0, party_size as i64],
                |row| {
                    let id: i64 = row.get(0)?;
                    Ok(Self::to_id(id))
                },
            )
            .expect("availability rows");
        rows.map(|r| r.expect("slot row")).collect()
    }
}
