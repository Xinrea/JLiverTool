//! Core types for JLiverTool

use serde::{Deserialize, Serialize};

/// Bilibili authentication cookies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cookies {
    #[serde(rename = "DedeUserID")]
    pub dede_user_id: String,
    #[serde(rename = "DedeUserID__ckMd5")]
    pub dede_user_id_ck_md5: String,
    #[serde(rename = "Expires")]
    pub expires: String,
    #[serde(rename = "SESSDATA")]
    pub sessdata: String,
    pub bili_jct: String,
    #[serde(default)]
    pub gourl: String,
}

impl Cookies {
    /// Parse cookies from URL query string (used after QR login)
    pub fn from_query_string(query: &str) -> Self {
        let mut cookies = Cookies::default();

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                let decoded = urlencoding::decode(value).unwrap_or_default().to_string();
                match key {
                    "DedeUserID" => cookies.dede_user_id = decoded,
                    "DedeUserID__ckMd5" => cookies.dede_user_id_ck_md5 = decoded,
                    "Expires" => cookies.expires = decoded,
                    "SESSDATA" => cookies.sessdata = decoded,
                    "bili_jct" => cookies.bili_jct = decoded,
                    "gourl" => cookies.gourl = decoded,
                    _ => {}
                }
            }
        }

        cookies
    }

    /// Convert cookies to HTTP Cookie header string
    pub fn to_cookie_string(&self) -> String {
        if self.sessdata.is_empty() {
            return String::new();
        }

        let sessdata = if self.sessdata.contains('%') {
            urlencoding::encode(&self.sessdata).to_string()
        } else {
            self.sessdata.clone()
        };

        format!(
            "SESSDATA={}; DedeUserID={}; DedeUserID_ckMd5={}; bili_jct={}; Expires={}",
            sessdata,
            self.dede_user_id,
            self.dede_user_id_ck_md5,
            self.bili_jct,
            self.expires
        )
    }

    pub fn is_valid(&self) -> bool {
        !self.sessdata.is_empty() && !self.bili_jct.is_empty()
    }

    /// Get user ID as u64
    pub fn user_id(&self) -> Option<u64> {
        self.dede_user_id.parse().ok()
    }
}

/// Room ID wrapper that handles short_id vs real room_id
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomId {
    short_id: u64,
    room_id: u64,
    owner_uid: u64,
}

impl RoomId {
    pub fn new(short_id: u64, room_id: u64, owner_uid: u64) -> Self {
        Self {
            short_id,
            room_id,
            owner_uid,
        }
    }

    /// Check if the given room_id matches either short_id or real room_id
    pub fn matches(&self, room_id: u64) -> bool {
        self.short_id == room_id || self.room_id == room_id
    }

    /// Get the display ID (prefer short_id if available)
    pub fn display_id(&self) -> u64 {
        if self.short_id != 0 {
            self.short_id
        } else {
            self.room_id
        }
    }

    /// Get the real room ID
    pub fn real_id(&self) -> u64 {
        self.room_id
    }

    /// Get the owner's UID
    pub fn owner_uid(&self) -> u64 {
        self.owner_uid
    }
}

/// Default room ID for testing
pub fn default_room_id() -> RoomId {
    RoomId::new(0, 21484828, 61639371)
}

/// Merge room user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeUserInfo {
    pub index: usize,
    pub uid: String,
    pub name: String,
}

/// User/Sender information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sender {
    pub uid: u64,
    pub uname: String,
    pub face: String,
    #[serde(default)]
    pub medal_info: MedalInfo,
}

/// Fan medal information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MedalInfo {
    pub anchor_roomid: u64,
    pub anchor_uname: String,
    pub guard_level: u8,
    pub medal_color: u32,
    pub medal_color_border: u32,
    pub medal_color_start: u32,
    pub medal_color_end: u32,
    pub medal_level: u8,
    pub medal_name: String,
    pub is_lighted: bool,
}

impl MedalInfo {
    /// Convert color number to hex string
    pub fn color_to_hex(color: u32) -> String {
        format!("#{:06x}", color)
    }

    /// Get medal color as hex string
    pub fn medal_color_hex(&self) -> String {
        Self::color_to_hex(self.medal_color)
    }

    /// Get medal border color as hex string
    pub fn medal_color_border_hex(&self) -> String {
        Self::color_to_hex(self.medal_color_border)
    }
}

/// Emoji content in danmaku
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiContent {
    pub bulge_display: i32,
    pub emoticon_unique: String,
    pub height: u32,
    pub in_player_area: i32,
    pub is_dynamic: i32,
    pub url: String,
    pub width: u32,
}

/// Window types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowType {
    Invalid,
    Main,
    Gift,
    SuperChat,
    Setting,
    Detail,
    Rank,
}

impl Default for WindowType {
    fn default() -> Self {
        Self::Invalid
    }
}

/// Record types for danmaku history
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RecordType {
    Danmu = 0,
    Gift = 1,
    SuperChat = 2,
    Guard = 3,
    Interact = 4,
    EntryEffect = 5,
}

/// Danmaku record for user history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanmuRecord {
    pub record_type: RecordType,
    pub content: String,
    pub timestamp: i64,
}

/// Detail info for user popup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailInfo {
    pub sender: Sender,
    pub danmus: Vec<DanmuRecord>,
}

/// Guard level names
pub fn guard_level_name(level: u8) -> &'static str {
    match level {
        1 => "总督",
        2 => "提督",
        3 => "舰长",
        _ => "",
    }
}

/// Get guard icon URL
pub fn guard_icon_url(level: u8) -> &'static str {
    match level {
        1 => "https://i0.hdslb.com/bfs/live/143f5ec3003b4080d1b5f817a9efdca46d631945.png@44w_44h.webp",
        2 => "https://i0.hdslb.com/bfs/live/98a201c14a64e860a758f089144dcf3f42e7038c.png@44w_44h.webp",
        3 => "https://i0.hdslb.com/bfs/live/334f567013c5a9e7b93c04a0e80e31dd2af07b52.png@44w_44h.webp",
        _ => "",
    }
}

/// Interaction action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractAction {
    Enter = 1,
    Follow = 2,
    Share = 3,
    SpecialFollow = 4,
    MutualFollow = 5,
}

impl InteractAction {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Enter),
            2 => Some(Self::Follow),
            3 => Some(Self::Share),
            4 => Some(Self::SpecialFollow),
            5 => Some(Self::MutualFollow),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Enter => "进入直播间",
            Self::Follow => "关注了直播间",
            Self::Share => "分享了直播间",
            Self::SpecialFollow => "特别关注了直播间",
            Self::MutualFollow => "与主播互粉了",
        }
    }
}
