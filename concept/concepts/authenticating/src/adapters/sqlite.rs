//! `authenticating.sqlite`。密码列是演示用明文。

use std::path::Path;

use restaurant_shared_kernel::OpaqueId;
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{SessionId, UserId};
use crate::ports::AuthStore;

pub struct SqliteAuthStore {
    conn: Connection,
}

impl SqliteAuthStore {
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

    fn user_id(row: i64) -> UserId {
        UserId::from_string(format!("U{row}"))
    }

    fn session_id(row: i64) -> SessionId {
        SessionId::from_string(format!("T{row}"))
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

impl AuthStore for SqliteAuthStore {
    fn username_taken(&self, username: &str) -> bool {
        let n: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .expect("auth username_taken");
        n > 0
    }

    fn insert_user(&mut self, id: &UserId, username: &str, password: &str) {
        self.conn
            .execute(
                "INSERT INTO users (id, username, password) VALUES (?1, ?2, ?3)",
                params![Self::parse_user(id), username, password],
            )
            .expect("auth insert_user");
    }

    fn find_user(&self, username: &str, password: &str) -> Option<UserId> {
        self.conn
            .query_row(
                "SELECT id FROM users WHERE username = ?1 AND password = ?2",
                params![username, password],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .expect("auth find_user")
            .map(Self::user_id)
    }

    fn next_user_id(&mut self) -> UserId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM users", [], |row| {
                row.get(0)
            })
            .expect("auth next_user_id");
        Self::user_id(n)
    }

    fn next_session_id(&mut self) -> SessionId {
        let n: i64 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM sessions", [], |row| {
                row.get(0)
            })
            .expect("auth next_session_id");
        Self::session_id(n)
    }

    fn insert_session(&mut self, id: &SessionId, user: &UserId) {
        self.conn
            .execute(
                "INSERT INTO sessions (id, user_id) VALUES (?1, ?2)",
                params![Self::parse_session(id), Self::parse_user(user)],
            )
            .expect("auth insert_session");
    }

    fn user_of_session(&self, session: &SessionId) -> Option<UserId> {
        self.conn
            .query_row(
                "SELECT user_id FROM sessions WHERE id = ?1",
                params![Self::parse_session(session)],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .expect("auth user_of_session")
            .map(Self::user_id)
    }

    fn delete_session(&mut self, session: &SessionId) -> bool {
        let n = self
            .conn
            .execute(
                "DELETE FROM sessions WHERE id = ?1",
                params![Self::parse_session(session)],
            )
            .expect("auth delete_session");
        n > 0
    }
}
