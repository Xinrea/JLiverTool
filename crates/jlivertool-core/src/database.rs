//! SQLite database for storing danmus, gifts, guards, and superchats

use crate::messages::{DanmuMessage, GiftMessage, GuardMessage, SuperChatMessage};
use crate::types::{MedalInfo, Sender};
use anyhow::Result;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

/// Database store for JLiverTool
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// Initialize database tables
    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock();

        // Danmu messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS danmus (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id INTEGER NOT NULL,
                sender_uid INTEGER NOT NULL,
                sender_uname TEXT NOT NULL,
                sender_face TEXT,
                medal_level INTEGER,
                medal_name TEXT,
                medal_anchor_uname TEXT,
                medal_anchor_roomid INTEGER,
                medal_guard_level INTEGER,
                content TEXT NOT NULL,
                is_special INTEGER DEFAULT 0,
                timestamp INTEGER NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Gift messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS gifts (
                id TEXT PRIMARY KEY,
                room_id INTEGER NOT NULL,
                sender_uid INTEGER NOT NULL,
                sender_uname TEXT NOT NULL,
                sender_face TEXT,
                medal_level INTEGER,
                medal_name TEXT,
                gift_id INTEGER NOT NULL,
                gift_name TEXT NOT NULL,
                gift_price INTEGER NOT NULL,
                coin_type TEXT NOT NULL,
                action TEXT NOT NULL,
                num INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                archived INTEGER DEFAULT 0,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Add archived column if it doesn't exist (migration for existing databases)
        let _ = conn.execute("ALTER TABLE gifts ADD COLUMN archived INTEGER DEFAULT 0", []);

        // Guard messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS guards (
                id TEXT PRIMARY KEY,
                room_id INTEGER NOT NULL,
                sender_uid INTEGER NOT NULL,
                sender_uname TEXT NOT NULL,
                sender_face TEXT,
                num INTEGER NOT NULL,
                unit TEXT NOT NULL,
                guard_level INTEGER NOT NULL,
                price INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                archived INTEGER DEFAULT 0,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Add archived column if it doesn't exist (migration for existing databases)
        let _ = conn.execute("ALTER TABLE guards ADD COLUMN archived INTEGER DEFAULT 0", []);

        // SuperChat messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS superchats (
                id TEXT PRIMARY KEY,
                room_id INTEGER NOT NULL,
                sender_uid INTEGER NOT NULL,
                sender_uname TEXT NOT NULL,
                sender_face TEXT,
                medal_level INTEGER,
                medal_name TEXT,
                message TEXT NOT NULL,
                price INTEGER NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                background_color TEXT,
                background_bottom_color TEXT,
                timestamp INTEGER NOT NULL,
                archived INTEGER DEFAULT 0,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Add archived column if it doesn't exist (migration for existing databases)
        let _ = conn.execute("ALTER TABLE superchats ADD COLUMN archived INTEGER DEFAULT 0", []);

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_danmus_room_timestamp ON danmus(room_id, timestamp DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_gifts_room_timestamp ON gifts(room_id, timestamp DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_guards_room_timestamp ON guards(room_id, timestamp DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_superchats_room_timestamp ON superchats(room_id, timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    /// Insert a danmu message
    pub fn insert_danmu(&self, room_id: u64, danmu: &DanmuMessage) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO danmus (
                room_id, sender_uid, sender_uname, sender_face,
                medal_level, medal_name, medal_anchor_uname, medal_anchor_roomid, medal_guard_level,
                content, is_special, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                room_id as i64,
                danmu.sender.uid as i64,
                danmu.sender.uname,
                danmu.sender.face,
                danmu.sender.medal_info.medal_level as i64,
                danmu.sender.medal_info.medal_name,
                danmu.sender.medal_info.anchor_uname,
                danmu.sender.medal_info.anchor_roomid as i64,
                danmu.sender.medal_info.guard_level as i64,
                danmu.content,
                danmu.is_special as i64,
                chrono::Utc::now().timestamp(),
            ],
        )?;
        Ok(())
    }

    /// Insert multiple danmu messages in a single transaction (batch insert)
    pub fn insert_danmus_batch(&self, room_id: u64, danmus: &[DanmuMessage]) -> Result<()> {
        if danmus.is_empty() {
            return Ok(());
        }

        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let timestamp = chrono::Utc::now().timestamp();

        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO danmus (
                    room_id, sender_uid, sender_uname, sender_face,
                    medal_level, medal_name, medal_anchor_uname, medal_anchor_roomid, medal_guard_level,
                    content, is_special, timestamp
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
            )?;

            for danmu in danmus {
                stmt.execute(params![
                    room_id as i64,
                    danmu.sender.uid as i64,
                    danmu.sender.uname,
                    danmu.sender.face,
                    danmu.sender.medal_info.medal_level as i64,
                    danmu.sender.medal_info.medal_name,
                    danmu.sender.medal_info.anchor_uname,
                    danmu.sender.medal_info.anchor_roomid as i64,
                    danmu.sender.medal_info.guard_level as i64,
                    danmu.content,
                    danmu.is_special as i64,
                    timestamp,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Insert a gift message
    pub fn insert_gift(&self, gift: &GiftMessage) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO gifts (
                id, room_id, sender_uid, sender_uname, sender_face,
                medal_level, medal_name,
                gift_id, gift_name, gift_price, coin_type, action, num, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                gift.id,
                gift.room as i64,
                gift.sender.uid as i64,
                gift.sender.uname,
                gift.sender.face,
                gift.sender.medal_info.medal_level as i64,
                gift.sender.medal_info.medal_name,
                gift.gift_info.id as i64,
                gift.gift_info.name,
                gift.gift_info.price as i64,
                gift.gift_info.coin_type,
                gift.action,
                gift.num as i64,
                gift.timestamp,
            ],
        )?;
        Ok(())
    }

    /// Insert multiple gift messages in a single transaction (batch insert)
    pub fn insert_gifts_batch(&self, gifts: &[GiftMessage]) -> Result<()> {
        if gifts.is_empty() {
            return Ok(());
        }

        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO gifts (
                    id, room_id, sender_uid, sender_uname, sender_face,
                    medal_level, medal_name,
                    gift_id, gift_name, gift_price, coin_type, action, num, timestamp
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
            )?;

            for gift in gifts {
                stmt.execute(params![
                    gift.id,
                    gift.room as i64,
                    gift.sender.uid as i64,
                    gift.sender.uname,
                    gift.sender.face,
                    gift.sender.medal_info.medal_level as i64,
                    gift.sender.medal_info.medal_name,
                    gift.gift_info.id as i64,
                    gift.gift_info.name,
                    gift.gift_info.price as i64,
                    gift.gift_info.coin_type,
                    gift.action,
                    gift.num as i64,
                    gift.timestamp,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Insert a guard message
    pub fn insert_guard(&self, guard: &GuardMessage) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO guards (
                id, room_id, sender_uid, sender_uname, sender_face,
                num, unit, guard_level, price, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                guard.id,
                guard.room as i64,
                guard.sender.uid as i64,
                guard.sender.uname,
                guard.sender.face,
                guard.num as i64,
                guard.unit,
                guard.guard_level as i64,
                guard.price as i64,
                guard.timestamp,
            ],
        )?;
        Ok(())
    }

    /// Insert a superchat message
    pub fn insert_superchat(&self, sc: &SuperChatMessage) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO superchats (
                id, room_id, sender_uid, sender_uname, sender_face,
                medal_level, medal_name,
                message, price, start_time, end_time,
                background_color, background_bottom_color, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                sc.id,
                sc.room as i64,
                sc.sender.uid as i64,
                sc.sender.uname,
                sc.sender.face,
                sc.sender.medal_info.medal_level as i64,
                sc.sender.medal_info.medal_name,
                sc.message,
                sc.price as i64,
                sc.start_time,
                sc.end_time,
                sc.background_color,
                sc.background_bottom_color,
                sc.timestamp,
            ],
        )?;
        Ok(())
    }

    /// Get recent gifts for a room
    pub fn get_recent_gifts(&self, room_id: u64, limit: usize) -> Result<Vec<GiftMessage>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, sender_uid, sender_uname, sender_face,
                    medal_level, medal_name,
                    gift_id, gift_name, gift_price, coin_type, action, num, timestamp,
                    COALESCE(archived, 0) as archived
             FROM gifts
             WHERE room_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let gifts = stmt
            .query_map(params![room_id as i64, limit as i64], |row| {
                Ok(GiftMessage {
                    id: row.get(0)?,
                    room: row.get::<_, i64>(1)? as u64,
                    sender: Sender {
                        uid: row.get::<_, i64>(2)? as u64,
                        uname: row.get(3)?,
                        face: row.get(4)?,
                        medal_info: MedalInfo {
                            medal_level: row.get::<_, i64>(5)? as u8,
                            medal_name: row.get(6)?,
                            ..Default::default()
                        },
                    },
                    gift_info: crate::messages::GiftInfo {
                        id: row.get::<_, i64>(7)? as u64,
                        name: row.get(8)?,
                        price: row.get::<_, i64>(9)? as u64,
                        coin_type: row.get(10)?,
                        img_basic: String::new(),
                        img_dynamic: String::new(),
                        gif: String::new(),
                        webp: String::new(),
                    },
                    action: row.get(11)?,
                    num: row.get::<_, i64>(12)? as u32,
                    timestamp: row.get(13)?,
                    archived: row.get::<_, i64>(14)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Reverse to get chronological order
        Ok(gifts.into_iter().rev().collect())
    }

    /// Get recent guards for a room
    pub fn get_recent_guards(&self, room_id: u64, limit: usize) -> Result<Vec<GuardMessage>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, sender_uid, sender_uname, sender_face,
                    num, unit, guard_level, price, timestamp,
                    COALESCE(archived, 0) as archived
             FROM guards
             WHERE room_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let guards = stmt
            .query_map(params![room_id as i64, limit as i64], |row| {
                Ok(GuardMessage {
                    id: row.get(0)?,
                    room: row.get::<_, i64>(1)? as u64,
                    sender: Sender {
                        uid: row.get::<_, i64>(2)? as u64,
                        uname: row.get(3)?,
                        face: row.get(4)?,
                        medal_info: MedalInfo::default(),
                    },
                    num: row.get::<_, i64>(5)? as u32,
                    unit: row.get(6)?,
                    guard_level: row.get::<_, i64>(7)? as u8,
                    price: row.get::<_, i64>(8)? as u64,
                    timestamp: row.get(9)?,
                    archived: row.get::<_, i64>(10)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(guards.into_iter().rev().collect())
    }

    /// Get recent superchats for a room
    pub fn get_recent_superchats(&self, room_id: u64, limit: usize) -> Result<Vec<SuperChatMessage>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, sender_uid, sender_uname, sender_face,
                    medal_level, medal_name,
                    message, price, start_time, end_time,
                    background_color, background_bottom_color, timestamp,
                    COALESCE(archived, 0) as archived
             FROM superchats
             WHERE room_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let scs = stmt
            .query_map(params![room_id as i64, limit as i64], |row| {
                Ok(SuperChatMessage {
                    id: row.get(0)?,
                    room: row.get::<_, i64>(1)? as u64,
                    sender: Sender {
                        uid: row.get::<_, i64>(2)? as u64,
                        uname: row.get(3)?,
                        face: row.get(4)?,
                        medal_info: MedalInfo {
                            medal_level: row.get::<_, i64>(5)? as u8,
                            medal_name: row.get(6)?,
                            ..Default::default()
                        },
                    },
                    message: row.get(7)?,
                    price: row.get::<_, i64>(8)? as u64,
                    start_time: row.get(9)?,
                    end_time: row.get(10)?,
                    background_color: row.get(11)?,
                    background_bottom_color: row.get(12)?,
                    timestamp: row.get(13)?,
                    archived: row.get::<_, i64>(14)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(scs.into_iter().rev().collect())
    }

    /// Get recent danmus for a room
    pub fn get_recent_danmus(&self, room_id: u64, limit: usize) -> Result<Vec<DanmuMessage>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT sender_uid, sender_uname, sender_face,
                    medal_level, medal_name, medal_anchor_uname, medal_anchor_roomid, medal_guard_level,
                    content, is_special
             FROM danmus
             WHERE room_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let danmus = stmt
            .query_map(params![room_id as i64, limit as i64], |row| {
                Ok(DanmuMessage {
                    sender: Sender {
                        uid: row.get::<_, i64>(0)? as u64,
                        uname: row.get(1)?,
                        face: row.get(2)?,
                        medal_info: MedalInfo {
                            medal_level: row.get::<_, i64>(3)? as u8,
                            medal_name: row.get(4)?,
                            anchor_uname: row.get(5)?,
                            anchor_roomid: row.get::<_, i64>(6)? as u64,
                            guard_level: row.get::<_, i64>(7)? as u8,
                            ..Default::default()
                        },
                    },
                    content: row.get(8)?,
                    is_special: row.get::<_, i64>(9)? != 0,
                    is_generated: false,
                    emoji_content: None,
                    side_index: -1,
                    reply_uname: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(danmus.into_iter().rev().collect())
    }

    /// Get danmus from the last N minutes
    pub fn get_danmus_since(&self, room_id: u64, minutes: i64) -> Result<Vec<DanmuMessage>> {
        let conn = self.conn.lock();
        let since_timestamp = chrono::Utc::now().timestamp() - (minutes * 60);
        let mut stmt = conn.prepare(
            "SELECT sender_uid, sender_uname, sender_face,
                    medal_level, medal_name, medal_anchor_uname, medal_anchor_roomid, medal_guard_level,
                    content, is_special
             FROM danmus
             WHERE room_id = ?1 AND timestamp >= ?2
             ORDER BY timestamp ASC",
        )?;

        let danmus = stmt
            .query_map(params![room_id as i64, since_timestamp], |row| {
                Ok(DanmuMessage {
                    sender: Sender {
                        uid: row.get::<_, i64>(0)? as u64,
                        uname: row.get(1)?,
                        face: row.get(2)?,
                        medal_info: MedalInfo {
                            medal_level: row.get::<_, i64>(3)? as u8,
                            medal_name: row.get(4)?,
                            anchor_uname: row.get(5)?,
                            anchor_roomid: row.get::<_, i64>(6)? as u64,
                            guard_level: row.get::<_, i64>(7)? as u8,
                            ..Default::default()
                        },
                    },
                    content: row.get(8)?,
                    is_special: row.get::<_, i64>(9)? != 0,
                    is_generated: false,
                    emoji_content: None,
                    side_index: -1,
                    reply_uname: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(danmus)
    }

    /// Get danmus by user UID with timestamp
    pub fn get_danmus_by_user(
        &self,
        room_id: u64,
        uid: u64,
        limit: usize,
    ) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT content, timestamp
             FROM danmus
             WHERE room_id = ?1 AND sender_uid = ?2
             ORDER BY timestamp DESC
             LIMIT ?3",
        )?;

        let danmus = stmt
            .query_map(params![room_id as i64, uid as i64, limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(danmus)
    }

    /// Clear all data for a room
    #[allow(dead_code)]
    pub fn clear_room_data(&self, room_id: u64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM danmus WHERE room_id = ?1", params![room_id as i64])?;
        conn.execute("DELETE FROM gifts WHERE room_id = ?1", params![room_id as i64])?;
        conn.execute("DELETE FROM guards WHERE room_id = ?1", params![room_id as i64])?;
        conn.execute("DELETE FROM superchats WHERE room_id = ?1", params![room_id as i64])?;
        Ok(())
    }

    /// Update archived status for a gift
    pub fn set_gift_archived(&self, id: &str, archived: bool) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE gifts SET archived = ?1 WHERE id = ?2",
            params![archived as i64, id],
        )?;
        Ok(())
    }

    /// Update archived status for a guard
    pub fn set_guard_archived(&self, id: &str, archived: bool) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE guards SET archived = ?1 WHERE id = ?2",
            params![archived as i64, id],
        )?;
        Ok(())
    }

    /// Update archived status for a superchat
    pub fn set_superchat_archived(&self, id: &str, archived: bool) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE superchats SET archived = ?1 WHERE id = ?2",
            params![archived as i64, id],
        )?;
        Ok(())
    }

    /// Delete a gift by ID
    pub fn delete_gift(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM gifts WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete a guard by ID
    pub fn delete_guard(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM guards WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Delete a superchat by ID
    pub fn delete_superchat(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM superchats WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Clear all gifts for a room
    pub fn clear_gifts(&self, room_id: u64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM gifts WHERE room_id = ?1", params![room_id as i64])?;
        Ok(())
    }

    /// Clear all guards for a room
    pub fn clear_guards(&self, room_id: u64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM guards WHERE room_id = ?1", params![room_id as i64])?;
        Ok(())
    }

    /// Clear all superchats for a room
    pub fn clear_superchats(&self, room_id: u64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM superchats WHERE room_id = ?1", params![room_id as i64])?;
        Ok(())
    }

    /// Get gift statistics for a room
    pub fn get_gift_stats(&self, room_id: u64) -> Result<GiftStats> {
        let conn = self.conn.lock();

        // Total paid gifts value
        let total_paid: i64 = conn.query_row(
            "SELECT COALESCE(SUM(gift_price * num), 0) FROM gifts WHERE room_id = ?1 AND coin_type = 'gold'",
            params![room_id as i64],
            |row| row.get(0),
        )?;

        // Total guard value
        let total_guard: i64 = conn.query_row(
            "SELECT COALESCE(SUM(price), 0) FROM guards WHERE room_id = ?1",
            params![room_id as i64],
            |row| row.get(0),
        )?;

        // Total superchat value
        let total_sc: i64 = conn.query_row(
            "SELECT COALESCE(SUM(price), 0) FROM superchats WHERE room_id = ?1",
            params![room_id as i64],
            |row| row.get(0),
        )?;

        Ok(GiftStats {
            total_paid_gifts: total_paid as u64,
            total_guards: total_guard as u64,
            total_superchats: total_sc as u64,
        })
    }
}

/// Gift statistics
#[derive(Debug, Clone, Default)]
pub struct GiftStats {
    pub total_paid_gifts: u64,
    pub total_guards: u64,
    pub total_superchats: u64,
}

impl GiftStats {
    /// Get total value in CNY (price is in 1/1000 yuan for gifts/guards, 1/100 for SC)
    pub fn total_value_cny(&self) -> f64 {
        (self.total_paid_gifts as f64 / 1000.0)
            + (self.total_guards as f64 / 1000.0)
            + (self.total_superchats as f64 / 100.0)
    }
}

/// Time-based statistics
#[derive(Debug, Clone, Default)]
pub struct TimeBasedStats {
    pub danmu_count: u64,
    pub gift_count: u64,
    pub gift_value: u64,  // in 1/1000 yuan
    pub superchat_count: u64,
    pub superchat_value: u64,  // in yuan (no division needed)
}

impl TimeBasedStats {
    /// Get gift value in CNY
    pub fn gift_value_cny(&self) -> f64 {
        self.gift_value as f64 / 1000.0
    }

    /// Get superchat value in CNY (already in yuan)
    pub fn superchat_value_cny(&self) -> f64 {
        self.superchat_value as f64
    }
}

impl Database {
    /// Get statistics for a specific time period
    /// `since_timestamp` is the Unix timestamp to start counting from
    pub fn get_time_based_stats(&self, room_id: u64, since_timestamp: i64) -> Result<TimeBasedStats> {
        let conn = self.conn.lock();

        // Danmu count
        let danmu_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM danmus WHERE room_id = ?1 AND timestamp >= ?2",
            params![room_id as i64, since_timestamp],
            |row| row.get(0),
        )?;

        // Gift count and value (only paid gifts)
        let (gift_count, gift_value): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(gift_price * num), 0) FROM gifts WHERE room_id = ?1 AND timestamp >= ?2 AND coin_type = 'gold'",
            params![room_id as i64, since_timestamp],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Guard count and value (guards are always paid)
        let (guard_count, guard_value): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(price), 0) FROM guards WHERE room_id = ?1 AND timestamp >= ?2",
            params![room_id as i64, since_timestamp],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Superchat count and value
        let (sc_count, sc_value): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(price), 0) FROM superchats WHERE room_id = ?1 AND timestamp >= ?2",
            params![room_id as i64, since_timestamp],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok(TimeBasedStats {
            danmu_count: danmu_count as u64,
            gift_count: (gift_count + guard_count) as u64,
            gift_value: (gift_value + guard_value) as u64,
            superchat_count: sc_count as u64,
            superchat_value: sc_value as u64,
        })
    }

    /// Get time-series statistics for charting
    /// Returns a vector of (timestamp, danmu_count, gift_value, sc_value) for each time bucket
    /// `bucket_seconds` determines the granularity of the data points
    pub fn get_time_series_stats(
        &self,
        room_id: u64,
        since_timestamp: i64,
        bucket_seconds: i64,
    ) -> Result<Vec<TimeSeriesPoint>> {
        use std::collections::HashMap;

        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();

        // Calculate number of buckets
        let total_seconds = now - since_timestamp;
        let num_buckets = (total_seconds / bucket_seconds).max(1) as usize;

        // Use GROUP BY queries to aggregate all buckets at once (4 queries instead of 4*num_buckets)

        // Danmu counts by bucket
        let mut danmu_buckets: HashMap<i64, u64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COUNT(*) as cnt
                 FROM danmus WHERE room_id = ?2 AND timestamp >= ?3
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, since_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, count)) = row {
                    danmu_buckets.insert(bucket, count as u64);
                }
            }
        }

        // Gift values by bucket (only paid gifts)
        let mut gift_buckets: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COALESCE(SUM(gift_price * num), 0) as val
                 FROM gifts WHERE room_id = ?2 AND timestamp >= ?3 AND coin_type = 'gold'
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, since_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, value)) = row {
                    gift_buckets.insert(bucket, value);
                }
            }
        }

        // Guard values by bucket
        let mut guard_buckets: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COALESCE(SUM(price), 0) as val
                 FROM guards WHERE room_id = ?2 AND timestamp >= ?3
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, since_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, value)) = row {
                    guard_buckets.insert(bucket, value);
                }
            }
        }

        // Superchat values by bucket
        let mut sc_buckets: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COALESCE(SUM(price), 0) as val
                 FROM superchats WHERE room_id = ?2 AND timestamp >= ?3
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, since_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, value)) = row {
                    sc_buckets.insert(bucket, value);
                }
            }
        }

        // Build result vector from aggregated data
        let mut points: Vec<TimeSeriesPoint> = Vec::with_capacity(num_buckets);
        for i in 0..num_buckets {
            let bucket_start = since_timestamp + (i as i64 * bucket_seconds);
            // Align bucket_start to bucket boundary for lookup
            let bucket_key = (bucket_start / bucket_seconds) * bucket_seconds;

            let danmu_count = danmu_buckets.get(&bucket_key).copied().unwrap_or(0);
            let gift_value = gift_buckets.get(&bucket_key).copied().unwrap_or(0);
            let guard_value = guard_buckets.get(&bucket_key).copied().unwrap_or(0);
            let sc_value = sc_buckets.get(&bucket_key).copied().unwrap_or(0);

            points.push(TimeSeriesPoint {
                timestamp: bucket_start,
                danmu_count,
                gift_value: (gift_value + guard_value) as u64,
                superchat_value: sc_value as u64,
            });
        }

        Ok(points)
    }
}

/// A single point in time-series data
#[derive(Debug, Clone, Default)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub danmu_count: u64,
    pub gift_value: u64,      // in 1/1000 yuan
    pub superchat_value: u64, // in yuan (no division needed)
}

impl TimeSeriesPoint {
    /// Get gift value in CNY
    pub fn gift_value_cny(&self) -> f64 {
        self.gift_value as f64 / 1000.0
    }

    /// Get superchat value in CNY (already in yuan)
    pub fn superchat_value_cny(&self) -> f64 {
        self.superchat_value as f64
    }
}

impl Database {
    /// Get statistics for a specific time range
    pub fn get_time_based_stats_range(
        &self,
        room_id: u64,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<TimeBasedStats> {
        let conn = self.conn.lock();

        // Danmu count
        let danmu_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM danmus WHERE room_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3",
            params![room_id as i64, start_timestamp, end_timestamp],
            |row| row.get(0),
        )?;

        // Gift count and value (only paid gifts)
        let (gift_count, gift_value): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(gift_price * num), 0) FROM gifts WHERE room_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3 AND coin_type = 'gold'",
            params![room_id as i64, start_timestamp, end_timestamp],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Guard count and value (guards are always paid)
        let (guard_count, guard_value): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(price), 0) FROM guards WHERE room_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3",
            params![room_id as i64, start_timestamp, end_timestamp],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Superchat count and value
        let (sc_count, sc_value): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(price), 0) FROM superchats WHERE room_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3",
            params![room_id as i64, start_timestamp, end_timestamp],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok(TimeBasedStats {
            danmu_count: danmu_count as u64,
            gift_count: (gift_count + guard_count) as u64,
            gift_value: (gift_value + guard_value) as u64,
            superchat_count: sc_count as u64,
            superchat_value: sc_value as u64,
        })
    }

    /// Get time-series statistics for a specific time range
    pub fn get_time_series_stats_range(
        &self,
        room_id: u64,
        start_timestamp: i64,
        end_timestamp: i64,
        bucket_seconds: i64,
    ) -> Result<Vec<TimeSeriesPoint>> {
        use std::collections::HashMap;

        let conn = self.conn.lock();

        // Calculate number of buckets
        let total_seconds = end_timestamp - start_timestamp;
        let num_buckets = (total_seconds / bucket_seconds).max(1) as usize;

        // Danmu counts by bucket
        let mut danmu_buckets: HashMap<i64, u64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COUNT(*) as cnt
                 FROM danmus WHERE room_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, start_timestamp, end_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, count)) = row {
                    danmu_buckets.insert(bucket, count as u64);
                }
            }
        }

        // Gift values by bucket (only paid gifts)
        let mut gift_buckets: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COALESCE(SUM(gift_price * num), 0) as val
                 FROM gifts WHERE room_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4 AND coin_type = 'gold'
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, start_timestamp, end_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, value)) = row {
                    gift_buckets.insert(bucket, value);
                }
            }
        }

        // Guard values by bucket
        let mut guard_buckets: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COALESCE(SUM(price), 0) as val
                 FROM guards WHERE room_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, start_timestamp, end_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, value)) = row {
                    guard_buckets.insert(bucket, value);
                }
            }
        }

        // Superchat values by bucket
        let mut sc_buckets: HashMap<i64, i64> = HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / ?1) * ?1 as bucket, COALESCE(SUM(price), 0) as val
                 FROM superchats WHERE room_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                 GROUP BY bucket"
            )?;
            let rows = stmt.query_map(
                params![bucket_seconds, room_id as i64, start_timestamp, end_timestamp],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            )?;
            for row in rows {
                if let Ok((bucket, value)) = row {
                    sc_buckets.insert(bucket, value);
                }
            }
        }

        // Build result vector from aggregated data
        let mut points: Vec<TimeSeriesPoint> = Vec::with_capacity(num_buckets);
        for i in 0..num_buckets {
            let bucket_start = start_timestamp + (i as i64 * bucket_seconds);
            let bucket_key = (bucket_start / bucket_seconds) * bucket_seconds;

            let danmu_count = danmu_buckets.get(&bucket_key).copied().unwrap_or(0);
            let gift_value = gift_buckets.get(&bucket_key).copied().unwrap_or(0);
            let guard_value = guard_buckets.get(&bucket_key).copied().unwrap_or(0);
            let sc_value = sc_buckets.get(&bucket_key).copied().unwrap_or(0);

            points.push(TimeSeriesPoint {
                timestamp: bucket_start,
                danmu_count,
                gift_value: (gift_value + guard_value) as u64,
                superchat_value: sc_value as u64,
            });
        }

        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().unwrap();
        assert!(db.get_recent_gifts(12345, 10).unwrap().is_empty());
    }
}
