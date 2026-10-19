//! `booking.sqlite`：三张表（客人、占用、订单），本库内可以互相指。
//!
//! 【小白】概念侧订座本只有一张 reservations 表 + 两条唯一索引。
//! 这边把「同日」「占槽」「订单」拆成三张表，再靠 BookingService 对齐。
//! user_id / slot_id 是抄来的编号，**没有**指向 identity.sqlite / availability.sqlite 的外键。

use std::path::Path;

use restaurant_ddd_kernel::{
    Date, OpaqueId, ReservationId, SlotId, TimeOfDay, UserId,
};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{Guest, Reservation, ReservationStatus, SlotOccupancy};
use crate::ports::BookingStore;

pub struct SqliteBookingStore {
    conn: Connection,
}

impl SqliteBookingStore {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    pub fn memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS guests (
                user_id TEXT PRIMARY KEY
            );
            CREATE TABLE IF NOT EXISTS guest_booked_dates (
                user_id TEXT NOT NULL REFERENCES guests(user_id),
                date TEXT NOT NULL,
                PRIMARY KEY (user_id, date)
            );
            CREATE TABLE IF NOT EXISTS occupancies (
                slot_id TEXT PRIMARY KEY,
                occupied INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS reservations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                slot_id TEXT NOT NULL,
                time TEXT NOT NULL,
                date TEXT NOT NULL,
                party_size INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
    }

    fn parse_res(id: &ReservationId) -> i64 {
        id.as_str()
            .strip_prefix('R')
            .and_then(|s| s.parse().ok())
            .expect("reservation id")
    }

    fn load_guest(&self, user_id: &str) -> Option<Guest> {
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM guests WHERE user_id = ?1",
                params![user_id],
                |row| row.get(0),
            )
            .expect("booking guest exists");
        if exists == 0 {
            return None;
        }
        let mut dates = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT date FROM guest_booked_dates WHERE user_id = ?1")
            .unwrap();
        let rows = stmt
            .query_map(params![user_id], |row| row.get::<_, String>(0))
            .unwrap();
        for d in rows {
            dates.push(Date(d.unwrap()));
        }
        Some(Guest {
            user_id: UserId::from_string(user_id.to_string()),
            booked_dates: dates,
        })
    }
}

impl BookingStore for SqliteBookingStore {
    fn get_or_create_guest(&mut self, user: &UserId) -> Guest {
        if let Some(existing) = self.load_guest(user.as_str()) {
            return existing;
        }
        self.conn
            .execute(
                "INSERT INTO guests (user_id) VALUES (?1)",
                params![user.as_str()],
            )
            .expect("booking insert guest");
        Guest::new(user.clone())
    }

    fn save_guest(&mut self, guest: &Guest) {
        self.conn
            .execute(
                "INSERT INTO guests (user_id) VALUES (?1) ON CONFLICT(user_id) DO NOTHING",
                params![guest.user_id.as_str()],
            )
            .expect("booking upsert guest");
        self.conn
            .execute(
                "DELETE FROM guest_booked_dates WHERE user_id = ?1",
                params![guest.user_id.as_str()],
            )
            .unwrap();
        for d in &guest.booked_dates {
            self.conn
                .execute(
                    "INSERT INTO guest_booked_dates (user_id, date) VALUES (?1, ?2)",
                    params![guest.user_id.as_str(), d.0],
                )
                .unwrap();
        }
    }

    fn get_guest(&self, user: &UserId) -> Option<Guest> {
        self.load_guest(user.as_str())
    }

    fn get_or_create_occupancy(&mut self, slot: &SlotId) -> SlotOccupancy {
        if let Some(existing) = self
            .conn
            .query_row(
                "SELECT occupied FROM occupancies WHERE slot_id = ?1",
                params![slot.as_str()],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .expect("booking occupancy")
        {
            return SlotOccupancy {
                slot_id: slot.clone(),
                occupied: existing != 0,
            };
        }
        let vacant = SlotOccupancy::vacant(slot.clone());
        self.conn
            .execute(
                "INSERT INTO occupancies (slot_id, occupied) VALUES (?1, 0)",
                params![slot.as_str()],
            )
            .expect("booking insert occupancy");
        vacant
    }

    fn save_occupancy(&mut self, occupancy: &SlotOccupancy) {
        self.conn
            .execute(
                "INSERT INTO occupancies (slot_id, occupied) VALUES (?1, ?2)
                 ON CONFLICT(slot_id) DO UPDATE SET occupied = excluded.occupied",
                params![
                    occupancy.slot_id.as_str(),
                    if occupancy.occupied { 1 } else { 0 }
                ],
            )
            .expect("booking save occupancy");
    }

    fn save_reservation(&mut self, reservation: &Reservation) {
        let id = Self::parse_res(&reservation.id);
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM reservations WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO reservations (id, user_id, slot_id, time, date, party_size, status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        id,
                        reservation.user_id.as_str(),
                        reservation.slot_id.as_str(),
                        reservation.time.0,
                        reservation.date.0,
                        reservation.party_size as i64,
                        reservation.status.as_str(),
                    ],
                )
                .expect("booking insert reservation");
        } else {
            self.conn
                .execute(
                    "UPDATE reservations SET status = ?1 WHERE id = ?2",
                    params![reservation.status.as_str(), id],
                )
                .expect("booking update reservation");
        }
    }

    fn get_reservation(&self, id: &ReservationId) -> Option<Reservation> {
        self.conn
            .query_row(
                "SELECT id, user_id, slot_id, time, date, party_size, status
                 FROM reservations WHERE id = ?1",
                params![Self::parse_res(id)],
                |row| {
                    Ok(Reservation {
                        id: ReservationId::from_string(format!("R{}", row.get::<_, i64>(0)?)),
                        user_id: UserId::from_string(row.get(1)?),
                        slot_id: SlotId::from_string(row.get(2)?),
                        time: TimeOfDay(row.get(3)?),
                        date: Date(row.get(4)?),
                        party_size: row.get::<_, i64>(5)? as u32,
                        status: ReservationStatus::parse(&row.get::<_, String>(6)?)
                            .expect("status"),
                    })
                },
            )
            .optional()
            .expect("booking get reservation")
    }

    fn next_reservation_id(&mut self) -> ReservationId {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(id), 0) + 1 FROM reservations",
                [],
                |row| row.get(0),
            )
            .expect("booking next_id");
        ReservationId::from_string(format!("R{n}"))
    }
}
