//! `identity.sqlite`。只有 users / sessions。没有 karma，没有 booked_dates。

use std::path::Path;

use restaurant_ddd_kernel::{OpaqueId, SessionId, UserId};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::User;
use crate::ports::IdentityStore;

pub struct SqliteIdentityStore {
    conn: Connection,
}

impl SqliteIdentityStore {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }

    pub fn memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
    }

    fn parse_user(id: &UserId) -> i64 {
        id.as_str()
            .strip_prefix('U')
            .and_then(|s| s.parse().ok())
            .expect("user id")
    }

    fn parse_session(id: &SessionId) -> i64 {
        id.as_str()
            .strip_prefix('T')
            .and_then(|s| s.parse().ok())
            .expect("session id")
    }
}

impl IdentityStore for SqliteIdentityStore {
    fn username_taken(&self, username: &str) -> bool {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .expect("identity username_taken");
        n > 0
    }

    fn insert_user(&mut self, user: &User) {
        self.conn
            .execute(
                "INSERT INTO users (id, username, password) VALUES (?1, ?2, ?3)",
                params![Self::parse_user(&user.id), user.username, user.password],
            )
            .expect("identity insert_user");
    }

    fn find_by_username(&self, username: &str) -> Option<User> {
        self.conn
            .query_row(
                "SELECT id, username, password FROM users WHERE username = ?1",
                params![username],
                |row| {
                    Ok(User {
                        id: UserId::from_string(format!("U{}", row.get::<_, i64>(0)?)),
                        username: row.get(1)?,
                        password: row.get(2)?,
                    })
                },
            )
            .optional()
            .expect("identity find_by_username")
    }

    fn next_user_id(&mut self) -> UserId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM users", [], |row| {
                row.get(0)
            })
            .expect("identity next_user_id");
        UserId::from_string(format!("U{n}"))
    }

    fn next_session_id(&mut self) -> SessionId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM sessions", [], |row| {
                row.get(0)
            })
            .expect("identity next_session_id");
        SessionId::from_string(format!("T{n}"))
    }

    fn insert_session(&mut self, session: &SessionId, user: &UserId) {
        self.conn
            .execute(
                "INSERT INTO sessions (id, user_id) VALUES (?1, ?2)",
                params![Self::parse_session(session), Self::parse_user(user)],
            )
            .expect("identity insert_session");
    }

    fn user_of_session(&self, session: &SessionId) -> Option<UserId> {
        self.conn
            .query_row(
                "SELECT user_id FROM sessions WHERE id = ?1",
                params![Self::parse_session(session)],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .expect("identity user_of_session")
            .map(|id| UserId::from_string(format!("U{id}")))
    }
}
