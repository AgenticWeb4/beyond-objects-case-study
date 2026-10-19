//! `availability.sqlite`。没有 occupied / reserved 列。

use std::path::Path;

use restaurant_ddd_kernel::{Date, OpaqueId, PartySize, SlotId, TimeOfDay, VenueId};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::Slot;
use crate::ports::SlotStore;

pub struct SqliteSlotStore {
    conn: Connection,
}

impl SqliteSlotStore {
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
                listed INTEGER NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
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

impl SlotStore for SqliteSlotStore {
    fn insert(&mut self, slot: Slot) {
        self.conn
            .execute(
                "INSERT INTO slots (id, venue, start, end, date, min_party, max_party, listed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    Self::row_id(&slot.id),
                    slot.venue.as_str(),
                    slot.start.0,
                    slot.end.0,
                    slot.date.0,
                    slot.min_party as i64,
                    slot.max_party as i64,
                    if slot.listed { 1 } else { 0 },
                ],
            )
            .expect("availability insert");
    }

    fn get(&self, id: &SlotId) -> Option<Slot> {
        self.conn
            .query_row(
                "SELECT id, venue, start, end, date, min_party, max_party, listed FROM slots WHERE id = ?1",
                params![Self::row_id(id)],
                |row| {
                    Ok(Slot {
                        id: Self::to_id(row.get(0)?),
                        venue: VenueId::from_string(row.get(1)?),
                        start: TimeOfDay(row.get(2)?),
                        end: TimeOfDay(row.get(3)?),
                        date: Date(row.get(4)?),
                        min_party: row.get::<_, i64>(5)? as u32,
                        max_party: row.get::<_, i64>(6)? as u32,
                        listed: row.get::<_, i64>(7)? != 0,
                    })
                },
            )
            .optional()
            .expect("availability get")
    }

    fn update_listed(&mut self, id: &SlotId, listed: bool) {
        self.conn
            .execute(
                "UPDATE slots SET listed = ?1 WHERE id = ?2",
                params![if listed { 1 } else { 0 }, Self::row_id(id)],
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

    fn listed_matching(
        &self,
        venue: &VenueId,
        time: &TimeOfDay,
        date: &Date,
        party_size: PartySize,
    ) -> Vec<SlotId> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id FROM slots
                 WHERE venue = ?1 AND start = ?2 AND date = ?3
                   AND listed = 1 AND min_party <= ?4 AND max_party >= ?4
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
