//! 【小白】订座本对应的那个 SQLite 文件。
//!
//! 里面没有「餐厅表」「用户表」。user_id / slot_id 只是抄下来的编号，
//! 不能 JOIN 到另外三本账——四本账本来就不许互相翻。

use std::marker::PhantomData;
use std::path::Path;

use restaurant_shared_kernel::{Date, OpaqueId, TimeOfDay};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{Reservation, ReservationId, Status};
use crate::ports::ReservationStore;

pub struct SqliteReservationStore<U, S> {
    conn: Connection,
    _u: PhantomData<U>,
    _s: PhantomData<S>,
}

impl<U: OpaqueId, S: OpaqueId> SqliteReservationStore<U, S> {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    pub fn memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS reservations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                slot_id TEXT NOT NULL,
                time TEXT NOT NULL,
                date TEXT NOT NULL,
                party_size INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            -- 只约束 booked 行：canceled/granted 不占坑，与 actions 里的不变量一致。
            CREATE UNIQUE INDEX IF NOT EXISTS idx_booked_slot
                ON reservations(slot_id) WHERE status = 'booked';
            CREATE UNIQUE INDEX IF NOT EXISTS idx_booked_user_date
                ON reservations(user_id, date) WHERE status = 'booked';
            ",
        )?;
        Ok(Self {
            conn,
            _u: PhantomData,
            _s: PhantomData,
        })
    }

    fn row_id(id: &ReservationId) -> i64 {
        id.as_str()
            .strip_prefix('R')
            .and_then(|s| s.parse().ok())
            .expect("reservation id")
    }

    fn to_id(row: i64) -> ReservationId {
        ReservationId::from_string(format!("R{row}"))
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Reservation<U, S>> {
        let id: i64 = row.get(0)?;
        let user: String = row.get(1)?;
        let slot: String = row.get(2)?;
        let time: String = row.get(3)?;
        let date: String = row.get(4)?;
        let party_size: i64 = row.get(5)?;
        let status: String = row.get(6)?;
        Ok(Reservation {
            id: Self::to_id(id),
            user: U::from_string(user),
            slot: S::from_string(slot),
            time: TimeOfDay(time),
            date: Date(date),
            party_size: party_size as u32,
            status: Status::parse(&status).expect("status"),
        })
    }
}

impl<U: OpaqueId, S: OpaqueId> ReservationStore<U, S> for SqliteReservationStore<U, S> {
    fn insert(&mut self, rec: Reservation<U, S>) {
        let expected = Self::row_id(&rec.id);
        self.conn
            .execute(
                "INSERT INTO reservations (id, user_id, slot_id, time, date, party_size, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    expected,
                    rec.user.as_str(),
                    rec.slot.as_str(),
                    rec.time.0,
                    rec.date.0,
                    rec.party_size as i64,
                    rec.status.as_str(),
                ],
            )
            .expect("reserving insert");
    }

    fn get(&self, id: &ReservationId) -> Option<Reservation<U, S>> {
        self.conn
            .query_row(
                "SELECT id, user_id, slot_id, time, date, party_size, status
                 FROM reservations WHERE id = ?1",
                params![Self::row_id(id)],
                Self::map_row,
            )
            .optional()
            .expect("reserving get")
    }

    fn update(&mut self, rec: Reservation<U, S>) {
        self.conn
            .execute(
                "UPDATE reservations SET status = ?1 WHERE id = ?2",
                params![rec.status.as_str(), Self::row_id(&rec.id)],
            )
            .expect("reserving update");
    }

    fn next_id(&mut self) -> ReservationId {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(id), 0) + 1 FROM reservations",
                [],
                |row| row.get(0),
            )
            .expect("reserving next_id");
        Self::to_id(n)
    }

    fn has_booked_slot(&self, slot: &S) -> bool {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM reservations WHERE slot_id = ?1 AND status = 'booked'",
                params![slot.as_str()],
                |row| row.get(0),
            )
            .expect("reserving has_booked_slot");
        n > 0
    }

    fn has_booked_user_date(&self, user: &U, date: &Date) -> bool {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM reservations WHERE user_id = ?1 AND date = ?2 AND status = 'booked'",
                params![user.as_str(), date.0],
                |row| row.get(0),
            )
            .expect("reserving has_booked_user_date");
        n > 0
    }
}
