//! 【小白】仍然是一个 restaurant.sqlite，外键把人/店/位子/订单焊成对象图。
//! 对象类拆到了不同 crate，表还在同一本通讯录里。

use std::path::Path;

use restaurant_oop_reservation::{Reservation, ReservationId, ReservationStatus};
use restaurant_oop_restaurant::{Restaurant, RestaurantId, Slot, SlotId};
use restaurant_oop_shared::{Date, TimeOfDay};
use restaurant_oop_user::{SessionId, User, UserId};
use rusqlite::{params, Connection, OptionalExtension};

use crate::ports::{
    ReservationRepository, RestaurantRepository, SessionRepository, UserRepository,
};

pub struct SqliteDb {
    conn: Connection,
}

impl SqliteDb {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                karma_strikes INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS user_booked_dates (
                user_id INTEGER NOT NULL REFERENCES users(id),
                date TEXT NOT NULL,
                PRIMARY KEY (user_id, date)
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL REFERENCES users(id)
            );
            CREATE TABLE IF NOT EXISTS restaurants (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS slots (
                id TEXT PRIMARY KEY,
                restaurant_id TEXT NOT NULL REFERENCES restaurants(id),
                start TEXT NOT NULL,
                end TEXT NOT NULL,
                date TEXT NOT NULL,
                min_party INTEGER NOT NULL,
                max_party INTEGER NOT NULL,
                reserved INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS reservations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL REFERENCES users(id),
                restaurant_id TEXT NOT NULL REFERENCES restaurants(id),
                slot_id TEXT NOT NULL REFERENCES slots(id),
                time TEXT NOT NULL,
                date TEXT NOT NULL,
                party_size INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
    }

    fn parse_user(id: &UserId) -> i64 {
        id.0.strip_prefix('U')
            .and_then(|s| s.parse().ok())
            .expect("user id")
    }

    fn parse_session(id: &SessionId) -> i64 {
        id.0.strip_prefix('T')
            .and_then(|s| s.parse().ok())
            .expect("session id")
    }

    fn parse_res(id: &ReservationId) -> i64 {
        id.0.strip_prefix('R')
            .and_then(|s| s.parse().ok())
            .expect("reservation id")
    }

    fn load_user(&self, id: i64) -> Option<User> {
        let (username, password, strikes) = self
            .conn
            .query_row(
                "SELECT username, password, karma_strikes FROM users WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .expect("user")?;
        let mut dates = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT date FROM user_booked_dates WHERE user_id = ?1")
            .unwrap();
        let rows = stmt.query_map(params![id], |row| row.get::<_, String>(0)).unwrap();
        for d in rows {
            dates.push(Date(d.unwrap()));
        }
        Some(User {
            id: UserId(format!("U{id}")),
            username,
            password,
            karma_strikes: strikes as u32,
            booked_dates: dates,
        })
    }
}

impl UserRepository for SqliteDb {
    fn save_user(&mut self, user: &User) {
        let id = Self::parse_user(&user.id);
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO users (id, username, password, karma_strikes) VALUES (?1, ?2, ?3, ?4)",
                    params![id, user.username, user.password, user.karma_strikes as i64],
                )
                .unwrap();
        } else {
            self.conn
                .execute(
                    "UPDATE users SET password = ?1, karma_strikes = ?2 WHERE id = ?3",
                    params![user.password, user.karma_strikes as i64, id],
                )
                .unwrap();
        }
        self.conn
            .execute("DELETE FROM user_booked_dates WHERE user_id = ?1", params![id])
            .unwrap();
        for d in &user.booked_dates {
            self.conn
                .execute(
                    "INSERT INTO user_booked_dates (user_id, date) VALUES (?1, ?2)",
                    params![id, d.0],
                )
                .unwrap();
        }
    }

    fn get_user(&self, id: &UserId) -> Option<User> {
        self.load_user(Self::parse_user(id))
    }

    fn by_username(&self, username: &str) -> Option<User> {
        let id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .optional()
            .unwrap()?;
        self.load_user(id)
    }

    fn next_user_id(&mut self) -> UserId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM users", [], |row| {
                row.get(0)
            })
            .unwrap();
        UserId(format!("U{n}"))
    }
}

impl SessionRepository for SqliteDb {
    fn save_session(&mut self, session: &SessionId, user: &UserId) {
        self.conn
            .execute(
                "INSERT INTO sessions (id, user_id) VALUES (?1, ?2)",
                params![Self::parse_session(session), Self::parse_user(user)],
            )
            .unwrap();
    }

    fn user_of(&self, session: &SessionId) -> Option<UserId> {
        self.conn
            .query_row(
                "SELECT user_id FROM sessions WHERE id = ?1",
                params![Self::parse_session(session)],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .unwrap()
            .map(|id| UserId(format!("U{id}")))
    }

    fn next_session_id(&mut self) -> SessionId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM sessions", [], |row| {
                row.get(0)
            })
            .unwrap();
        SessionId(format!("T{n}"))
    }
}

impl RestaurantRepository for SqliteDb {
    fn save_restaurant(&mut self, restaurant: &Restaurant) {
        self.conn
            .execute(
                "INSERT INTO restaurants (id, name) VALUES (?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET name = excluded.name",
                params![restaurant.id.0, restaurant.name],
            )
            .unwrap();
        for slot in &restaurant.slots {
            self.conn
                .execute(
                    "INSERT INTO slots (id, restaurant_id, start, end, date, min_party, max_party, reserved)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(id) DO UPDATE SET
                        reserved = excluded.reserved,
                        start = excluded.start,
                        end = excluded.end,
                        date = excluded.date,
                        min_party = excluded.min_party,
                        max_party = excluded.max_party",
                    params![
                        slot.id.0,
                        restaurant.id.0,
                        slot.start.0,
                        slot.end.0,
                        slot.date.0,
                        slot.min_party as i64,
                        slot.max_party as i64,
                        if slot.reserved { 1 } else { 0 },
                    ],
                )
                .unwrap();
        }
    }

    fn get_restaurant(&self, id: &RestaurantId) -> Option<Restaurant> {
        let name: String = self
            .conn
            .query_row(
                "SELECT name FROM restaurants WHERE id = ?1",
                params![id.0],
                |row| row.get(0),
            )
            .optional()
            .unwrap()?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, start, end, date, min_party, max_party, reserved FROM slots WHERE restaurant_id = ?1",
            )
            .unwrap();
        let rows = stmt
            .query_map(params![id.0], |row| {
                Ok(Slot {
                    id: SlotId(row.get(0)?),
                    start: TimeOfDay(row.get(1)?),
                    end: TimeOfDay(row.get(2)?),
                    date: Date(row.get(3)?),
                    min_party: row.get::<_, i64>(4)? as u32,
                    max_party: row.get::<_, i64>(5)? as u32,
                    reserved: row.get::<_, i64>(6)? != 0,
                })
            })
            .unwrap();
        let mut slots = Vec::new();
        for s in rows {
            slots.push(s.unwrap());
        }
        Some(Restaurant {
            id: id.clone(),
            name,
            slots,
        })
    }

    fn upsert_named(&mut self, name: &str) -> Restaurant {
        let id = RestaurantId(name.to_string());
        if let Some(existing) = self.get_restaurant(&id) {
            existing
        } else {
            let r = Restaurant {
                id: id.clone(),
                name: name.to_string(),
                slots: vec![],
            };
            self.save_restaurant(&r);
            r
        }
    }
}

impl ReservationRepository for SqliteDb {
    fn save_reservation(&mut self, reservation: &Reservation) {
        let id = Self::parse_res(&reservation.id);
        let user = Self::parse_user(&reservation.user_id);
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
                    "INSERT INTO reservations (id, user_id, restaurant_id, slot_id, time, date, party_size, status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        id,
                        user,
                        reservation.restaurant_id.0,
                        reservation.slot_id.0,
                        reservation.time.0,
                        reservation.date.0,
                        reservation.party_size as i64,
                        reservation.status.as_str(),
                    ],
                )
                .unwrap();
        } else {
            self.conn
                .execute(
                    "UPDATE reservations SET status = ?1 WHERE id = ?2",
                    params![reservation.status.as_str(), id],
                )
                .unwrap();
        }
    }

    fn get_reservation(&self, id: &ReservationId) -> Option<Reservation> {
        self.conn
            .query_row(
                "SELECT id, user_id, restaurant_id, slot_id, time, date, party_size, status
                 FROM reservations WHERE id = ?1",
                params![Self::parse_res(id)],
                |row| {
                    Ok(Reservation {
                        id: ReservationId(format!("R{}", row.get::<_, i64>(0)?)),
                        user_id: UserId(format!("U{}", row.get::<_, i64>(1)?)),
                        restaurant_id: RestaurantId(row.get(2)?),
                        slot_id: SlotId(row.get(3)?),
                        time: TimeOfDay(row.get(4)?),
                        date: Date(row.get(5)?),
                        party_size: row.get::<_, i64>(6)? as u32,
                        status: ReservationStatus::parse(&row.get::<_, String>(7)?)
                            .expect("status"),
                    })
                },
            )
            .optional()
            .unwrap()
    }

    fn next_reservation_id(&mut self) -> ReservationId {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(id), 0) + 1 FROM reservations",
                [],
                |row| row.get(0),
            )
            .unwrap();
        ReservationId(format!("R{n}"))
    }
}
