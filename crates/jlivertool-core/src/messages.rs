//! Message types for danmaku, gifts, superchat, etc.

use crate::protobuf::{decode_message, PbMessage};
use crate::types::{EmojiContent, MedalInfo, MergeUserInfo, Sender};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::warn;

/// Dynamic emoji map: maps emoji text to WebP URL for animated emojis
static DYNAMIC_EMOJI_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    // 轴伊Joi收藏集动态表情包
    map.insert("[轴伊Joi收藏集动态表情包_跑了]", "https://i0.hdslb.com/bfs/garb/3c6551d38c726273798e1cc1e488a3d0633bad5e.webp");
    map.insert("[轴伊Joi收藏集动态表情包_鞠躬]", "https://i0.hdslb.com/bfs/garb/13fc84be85e70ccd8b12acc8e96e49203e96b1a9.webp");
    map.insert("[轴伊Joi收藏集动态表情包_摇你]", "https://i0.hdslb.com/bfs/garb/ec27426348fafa3286412d04e8ea851a7a84ce6e.webp");
    map.insert("[轴伊Joi收藏集动态表情包_愤怒]", "https://i0.hdslb.com/bfs/garb/5fbc1bda6a6d73b83e51cba2119ea65d0dc74689.webp");
    map.insert("[轴伊Joi收藏集动态表情包_猴]", "https://i0.hdslb.com/bfs/garb/4c7be23a801094ea00452a2da5ce3184336df331.webp");
    map.insert("[轴伊Joi收藏集动态表情包_NO]", "https://i0.hdslb.com/bfs/garb/5d124a80db426608c48c76b27d790c711d39b7b7.webp");
    map.insert("[轴伊Joi收藏集动态表情包_贴贴]", "https://i0.hdslb.com/bfs/garb/b02b9ebd543b2ee63573daffa191e8a76b7bb384.webp");
    map.insert("[轴伊Joi收藏集动态表情包_kksk]", "https://i0.hdslb.com/bfs/garb/518c5b00cdb1329ae8e26d50c52369394d7a2394.webp");
    map.insert("[轴伊Joi收藏集动态表情包_这辈子完了]", "https://i0.hdslb.com/bfs/garb/85464aa487412c66ab24543a86218fa82fff7f23.webp");
    map.insert("[轴伊Joi收藏集动态表情包_呆]", "https://i0.hdslb.com/bfs/garb/09a0bd1deaa02a4255dc886642123dfe2606ddb8.webp");
    map.insert("[轴伊Joi收藏集动态表情包_唔唔]", "https://i0.hdslb.com/bfs/garb/c82df0783d3cba6af8cebaa43c1f36b1ef3cc0f4.webp");
    map.insert("[轴伊Joi收藏集动态表情包_啊这]", "https://i0.hdslb.com/bfs/garb/03e899fb3c823cf337b50100718247f25955ee4b.webp");
    map.insert("[轴伊Joi收藏集动态表情包_失落]", "https://i0.hdslb.com/bfs/garb/dc085a527f49bedf897d08de2bb07fa7c779641b.webp");
    map.insert("[轴伊Joi收藏集动态表情包_神气]", "https://i0.hdslb.com/bfs/garb/8ed028a87af6c2ef79cdd654f3076d7f7f836514.webp");
    map.insert("[轴伊Joi收藏集动态表情包_怎么这样]", "https://i0.hdslb.com/bfs/garb/7e239e9ee33a8e168df4bba58cd9a18c40d9af45.webp");
    map.insert("[轴伊Joi收藏集动态表情包_尼嘻嘻]", "https://i0.hdslb.com/bfs/garb/278ebc1d82a220a127aa60a10b461dbbc71040c6.webp");
    map.insert("[轴伊Joi收藏集动态表情包_惊]", "https://i0.hdslb.com/bfs/garb/82f222eebba1160d81244d578dda1c4724276056.webp");
    map.insert("[轴伊Joi收藏集动态表情包_害怕]", "https://i0.hdslb.com/bfs/garb/4c00a15ab814a8855e3b7106cdc43c1f0a8fa640.webp");
    map.insert("[轴伊Joi收藏集动态表情包_睡觉]", "https://i0.hdslb.com/bfs/garb/dbec05c6adfd3c56c4fa484b00e4095b4d17cec7.webp");
    map.insert("[轴伊Joi收藏集动态表情包_爆]", "https://i0.hdslb.com/bfs/garb/b7e50a289001583cd06eb1149304149ba5ccc78b.webp");
    map
});

/// Danmaku message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanmuMessage {
    pub sender: Sender,
    pub content: String,
    #[serde(default)]
    pub is_generated: bool,
    #[serde(default)]
    pub is_special: bool,
    #[serde(default)]
    pub is_mirror: bool,
    pub emoji_content: Option<EmojiContent>,
    #[serde(default = "default_side_index")]
    pub side_index: i32,
    pub reply_uname: Option<String>,
}

fn default_side_index() -> i32 {
    -1
}

impl DanmuMessage {
    /// Parse a danmaku message from raw WebSocket body
    pub fn from_raw(body: &Value, user_info: Option<&MergeUserInfo>) -> Option<Self> {
        let info = body.get("info")?.as_array()?;

        let mut sender = Sender::default();

        // Get side index from merge user info
        let side_index = user_info.map(|u| u.index as i32).unwrap_or(-1);

        // Basic sender info: info[2] = [uid, uname, ...]
        if let Some(user_info_arr) = info.get(2).and_then(|v| v.as_array()) {
            sender.uid = user_info_arr.first()?.as_u64()?;
            sender.uname = user_info_arr.get(1)?.as_str()?.to_string();
        }

        // Parse extra info for reply: info[0][15]
        let reply_uname = info
            .first()
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(15))
            .and_then(|extra| {
                let show_reply = extra.get("show_reply")?.as_bool()?;
                if show_reply {
                    extra
                        .get("reply_uname")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            });

        // Parse medal info: info[3]
        if let Some(medal_arr) = info.get(3).and_then(|v| v.as_array()) {
            if medal_arr.len() >= 12 {
                sender.medal_info = MedalInfo {
                    medal_level: medal_arr.first().and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                    medal_name: medal_arr
                        .get(1)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    anchor_uname: medal_arr
                        .get(2)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    anchor_roomid: medal_arr.get(3).and_then(|v| v.as_u64()).unwrap_or(0),
                    medal_color: medal_arr.get(4).and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    medal_color_border: medal_arr.get(7).and_then(|v| v.as_u64()).unwrap_or(0)
                        as u32,
                    medal_color_start: medal_arr.get(8).and_then(|v| v.as_u64()).unwrap_or(0)
                        as u32,
                    medal_color_end: medal_arr.get(9).and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    guard_level: medal_arr.get(10).and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                    is_lighted: medal_arr.get(11).and_then(|v| v.as_u64()).unwrap_or(0) == 1,
                };
            }
        }

        // Content: info[1]
        let content = info
            .get(1)?
            .as_str()?
            .trim()
            .replace(['\r', '\n'], "")
            .to_string();

        // Emoji content: info[0][12] == 1 means emoji, info[0][13] is content
        let emoji_content = info.first().and_then(|v| v.as_array()).and_then(|arr| {
            if arr.get(12).and_then(|v| v.as_i64()) == Some(1) {
                arr.get(13)
                    .and_then(|v| serde_json::from_value::<EmojiContent>(v.clone()).ok())
                    .map(|mut emoji| {
                        // Check if this is a dynamic emoji and replace URL with WebP version
                        // emoticon_unique format: "upower_[轴伊Joi收藏集动态表情包_贴贴]"
                        // We need to remove "upower_" prefix, keeping the brackets
                        let emoji_key = emoji.emoticon_unique.replace("upower_", "");
                        if let Some(webp_url) = DYNAMIC_EMOJI_MAP.get(emoji_key.as_str()) {
                            emoji.url = webp_url.to_string();
                        }
                        emoji
                    })
            } else {
                None
            }
        });

        // Generated message: info[0][9] > 0
        let is_generated = info
            .first()
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(9))
            .and_then(|v| v.as_i64())
            .map(|v| v > 0)
            .unwrap_or(false);

        // Special user: info[2][2] > 0
        let is_special = info
            .get(2)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(2))
            .and_then(|v| v.as_i64())
            .map(|v| v > 0)
            .unwrap_or(false);

        Some(Self {
            sender,
            content,
            is_generated,
            is_special,
            is_mirror: false,
            emoji_content,
            side_index,
            reply_uname,
        })
    }
}

/// Gift information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftInfo {
    pub id: u64,
    pub name: String,
    pub price: u64,
    pub coin_type: String,
    pub img_basic: String,
    pub img_dynamic: String,
    pub gif: String,
    pub webp: String,
}

/// Gift message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftMessage {
    pub id: String,
    pub room: u64,
    pub gift_info: GiftInfo,
    pub sender: Sender,
    pub action: String,
    pub num: u32,
    pub timestamp: i64,
    #[serde(default)]
    pub archived: bool,
}

impl GiftMessage {
    /// Parse a gift message from raw WebSocket body
    pub fn from_raw(body: &Value, room_id: u64) -> Option<Self> {
        let Some(data) = body.get("data") else {
            warn!(
                "GiftMessage parse failed: missing `data` field, body={}",
                body
            );
            return None;
        };

        let Some(uid) = data.get("uid").and_then(|v| v.as_u64()) else {
            warn!(
                "GiftMessage parse failed: invalid/missing `uid` (got {:?}), data={}",
                data.get("uid"),
                data
            );
            return None;
        };
        let Some(uname) = data.get("uname").and_then(|v| v.as_str()) else {
            warn!(
                "GiftMessage parse failed: invalid/missing `uname` (got {:?}), data={}",
                data.get("uname"),
                data
            );
            return None;
        };

        let mut sender = Sender {
            uid,
            uname: uname.to_string(),
            face: data
                .get("face")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ..Default::default()
        };

        // Parse medal info
        if let Some(medal) = data.get("medal_info") {
            sender.medal_info = MedalInfo {
                medal_level: medal
                    .get("medal_level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                medal_name: medal
                    .get("medal_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                anchor_uname: medal
                    .get("anchor_uname")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                anchor_roomid: medal
                    .get("anchor_roomid")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                guard_level: medal
                    .get("guard_level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                ..Default::default()
            };
        }

        let Some(gift_id) = data.get("giftId").and_then(|v| v.as_u64()) else {
            warn!(
                "GiftMessage parse failed: invalid/missing `giftId` \
                 (giftId={:?}, gift_id={:?}), data keys={:?}, data={}",
                data.get("giftId"),
                data.get("gift_id"),
                data.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
                data
            );
            return None;
        };

        let Some(gift_name) = data.get("giftName").and_then(|v| v.as_str()) else {
            warn!(
                "GiftMessage parse failed: invalid/missing `giftName` \
                 (giftName={:?}, gift_name={:?}), data keys={:?}, data={}",
                data.get("giftName"),
                data.get("gift_name"),
                data.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
                data
            );
            return None;
        };

        let price = data.get("price").and_then(|v| v.as_u64()).unwrap_or(0);
        let coin_type = data
            .get("coin_type")
            .and_then(|v| v.as_str())
            .unwrap_or("gold")
            .to_string();

        let gift_info = GiftInfo {
            id: gift_id,
            name: gift_name.to_string(),
            price,
            coin_type,
            img_basic: String::new(),
            img_dynamic: String::new(),
            gif: String::new(),
            webp: String::new(),
        };

        let action = data
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("投喂")
            .to_string();
        let num = data.get("num").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        let timestamp = data
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        Some(Self {
            id: uuid::Uuid::new_v4().to_string(),
            room: room_id,
            gift_info,
            sender,
            action,
            num,
            timestamp,
            archived: false,
        })
    }

    /// Parse SEND_GIFT_V2 (base64 protobuf in `data.pb`).
    ///
    /// Schema based on community reverse of `bilibili.live.gift.v1.SendGiftBroadcast`
    /// (see sjh8130/bili_danmaku `SEND_GIFT_V2.proto`).
    ///
    /// One WS packet may contain multiple gift items (`gift_list`); this returns one
    /// `GiftMessage` per item.
    pub fn from_raw_v2(body: &Value, room_id: u64) -> Vec<Self> {
        let Some(data) = body.get("data") else {
            warn!("SEND_GIFT_V2 parse failed: missing `data`, body={}", body);
            return Vec::new();
        };

        // Observed as `data.pb`; some docs mention `data.data.pb`.
        let pb_b64 = data
            .get("pb")
            .and_then(|v| v.as_str())
            .or_else(|| data.pointer("/data/pb").and_then(|v| v.as_str()));

        let Some(pb_b64) = pb_b64 else {
            warn!(
                "SEND_GIFT_V2 parse failed: missing `pb`, data keys={:?}, data={}",
                data.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
                data
            );
            return Vec::new();
        };

        let pb_bytes = match BASE64.decode(pb_b64) {
            Ok(b) => b,
            Err(e) => {
                warn!("SEND_GIFT_V2 base64 decode failed: {}", e);
                return Vec::new();
            }
        };

        let root = match decode_message(&pb_bytes) {
            Ok(m) => m,
            Err(e) => {
                warn!("SEND_GIFT_V2 protobuf decode failed: {}", e);
                return Vec::new();
            }
        };

        Self::from_pb_root(&root, room_id)
    }

    fn from_pb_root(root: &PbMessage, room_id: u64) -> Vec<Self> {
        let Some(uid) = root.get_u64(1) else {
            warn!("SEND_GIFT_V2 missing uid (field 1)");
            return Vec::new();
        };
        let Some(uname) = root.get_str(2).map(str::to_string) else {
            warn!("SEND_GIFT_V2 missing uname (field 2)");
            return Vec::new();
        };
        let face = root.get_str(3).unwrap_or("").to_string();

        let mut sender = Sender {
            uid,
            uname,
            face,
            ..Default::default()
        };

        // MedalInfo (field 8)
        if let Some(medal) = root.get_message(8) {
            sender.medal_info = MedalInfo {
                medal_level: medal.get_u64(5).unwrap_or(0) as u8,
                medal_name: medal.get_str(6).unwrap_or("").to_string(),
                anchor_uname: medal.get_str(3).unwrap_or("").to_string(),
                anchor_roomid: medal.get_u64(4).unwrap_or(0),
                guard_level: medal.get_u64(12).unwrap_or(0) as u8,
                ..Default::default()
            };
        }

        // GiftItem list (field 10, repeated)
        let gift_items = root.get_messages(10);
        if gift_items.is_empty() {
            warn!("SEND_GIFT_V2 missing gift_list (field 10)");
            return Vec::new();
        }

        gift_items
            .into_iter()
            .filter_map(|item| Self::from_pb_gift_item(&item, room_id, &sender))
            .collect()
    }

    fn from_pb_gift_item(item: &PbMessage, room_id: u64, sender: &Sender) -> Option<Self> {
        let gift_id = item.get_u64(1).or_else(|| {
            warn!("SEND_GIFT_V2 gift item missing gift_id (field 1)");
            None
        })?;
        let gift_name = item.get_str(2).map(str::to_string).or_else(|| {
            warn!("SEND_GIFT_V2 gift item missing gift_name (field 2)");
            None
        })?;

        let num = item.get_u64(3).unwrap_or(1) as u32;
        let price = item.get_u64(5).unwrap_or(0);
        let coin_type = item
            .get_str(8)
            .unwrap_or("gold")
            .to_string();
        let action = item
            .get_str(18)
            .unwrap_or("投喂")
            .to_string();
        let timestamp = item
            .get_i64(10)
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        // GiftMaterialSnapShot (field 35) for images when present
        let (img_basic, webp, gif) = item
            .get_message(35)
            .map(|snap| {
                (
                    snap.get_str(1).unwrap_or("").to_string(),
                    snap.get_str(2).unwrap_or("").to_string(),
                    snap.get_str(5).unwrap_or("").to_string(),
                )
            })
            .unwrap_or_default();

        Some(Self {
            id: uuid::Uuid::new_v4().to_string(),
            room: room_id,
            gift_info: GiftInfo {
                id: gift_id,
                name: gift_name,
                price,
                coin_type,
                img_basic,
                img_dynamic: String::new(),
                gif,
                webp,
            },
            sender: sender.clone(),
            action,
            num,
            timestamp,
            archived: false,
        })
    }
}

#[cfg(test)]
mod gift_v2_tests {
    use super::*;
    use serde_json::json;

    // Captured SEND_GIFT_V2 pb (Xinrea / 人气票)
    const SAMPLE_PB: &str = "CMqAHRIGWGlucmVhGkpodHRwczovL2kxLmhkc2xiLmNvbS9iZnMvZmFjZS80NDMxMmNmNWExODQwNWE5NzUyNzQyZjRlODdmYThlNmFiODE1YzAyLmpwZyIHIzAwRDFGMSgDQicIy5WyHSgqMgbovbToiq84i8L9B0CLwv0HSISh/wdQ/9GfA1gBYANSjAUIxIkCEgnkurrmsJTnpagYASABKGQwZDhkQgRnb2xkShM0Nzk3MDQ2ODc4OTcxNjYzMzYwULuu/tIGWAFiOWJhdGNoOmdpZnQ6Y29tYm9faWQ6NDc1MjEwOjYxNjM5MzcxOjMzOTg4OjE3ODQ2NDk1MzEuODExOWgKcGR4BYUBAACAP4gBAZIBBuaKleWWgsABhoSoDuoBGAoR6L205LyKSm9pX0NoYW5uZWwQy5WyHYoC6QEIy5WyHRLhAQoR6L205LyKSm9pX0NoYW5uZWwSSmh0dHBzOi8vaTAuaGRzbGIuY29tL2Jmcy9mYWNlLzZjOGIzMTMwYWE5YmVkMGU0NjI1YWEwOWEzY2U1M2Q4NWVhMTJmM2UuanBnMl8KEei9tOS8ikpvaV9DaGFubmVsEkpodHRwczovL2kwLmhkc2xiLmNvbS9iZnMvZmFjZS82YzhiMzEzMGFhOWJlZDBlNDYyNWFhMDlhM2NlNTNkODVlYTEyZjNlLmpwZzofCAcSG2JpbGliaWxpIOebtOaSremrmOiDveS4u+aSrZICAJoC5QEKSmh0dHBzOi8vczEuaGRzbGIuY29tL2Jmcy9saXZlLzcxNjRjOTU1ZWMwZWQ3NTM3NDkxZDE4OWI4MjFjYzY4ZjFiZWEyMGQucG5nEktodHRwczovL2kwLmhkc2xiLmNvbS9iZnMvbGl2ZS81YTIwMTIwOTRiYjg3NWEwNGVhN2QwMzZlZGRlM2U5YzAwMTI3ZWMyLndlYnAqSmh0dHBzOi8vaTAuaGRzbGIuY29tL2Jmcy9saXZlL2E5ZjVkMWY5MDM1ODJjZmYxOGY5NDAxNGY1MWFmYWYwMTJmMGUyZjkuZ2lmqgIAWAFqAggpeu8CCMqAHRK3AQoGWGlucmVhEkpodHRwczovL2kxLmhkc2xiLmNvbS9iZnMvZmFjZS80NDMxMmNmNWExODQwNWE5NzUyNzQyZjRlODdmYThlNmFiODE1YzAyLmpwZzJUCgZYaW5yZWESSmh0dHBzOi8vaTEuaGRzbGIuY29tL2Jmcy9mYWNlLzQ0MzEyY2Y1YTE4NDA1YTk3NTI3NDJmNGU4N2ZhOGU2YWI4MTVjMDIuanBnOgsg////////////ARquAQoG6L206IqvECoYi8L9ByCEof8HKP/RnwMwi8L9B0gBUMuVsh1YA2Dm+yNqSmh0dHBzOi8vaTAuaGRzbGIuY29tL2Jmcy9saXZlLzQ4MzYwYzhmM2I3ZGU4MDMxZTg2ZmYxZWY0YTJkZmMwZWMyYTYxYzIucG5negkjQTc3M0YxOTmCAQkjQTc3M0YxOTmKAQcjRDQ3QUZGkgEHI0ZGRkZGRpoBCSNBNzczRjFFNg==";

    #[test]
    fn parse_send_gift_v2_sample() {
        let body = json!({
            "cmd": "SEND_GIFT_V2",
            "data": { "dmscore": 1190, "pb": SAMPLE_PB }
        });
        let gifts = GiftMessage::from_raw_v2(&body, 12345);
        assert_eq!(gifts.len(), 1);
        let g = &gifts[0];
        assert_eq!(g.room, 12345);
        assert_eq!(g.sender.uid, 475210);
        assert_eq!(g.sender.uname, "Xinrea");
        assert_eq!(g.gift_info.id, 33988);
        assert_eq!(g.gift_info.name, "人气票");
        assert_eq!(g.gift_info.price, 100);
        assert_eq!(g.gift_info.coin_type, "gold");
        assert_eq!(g.num, 1);
        assert_eq!(g.action, "投喂");
        assert_eq!(g.timestamp, 1784649531);
        assert_eq!(g.sender.medal_info.medal_name, "轴芯");
        assert_eq!(g.sender.medal_info.medal_level, 42);
        assert!(
            !g.gift_info.img_basic.is_empty() || !g.gift_info.webp.is_empty(),
            "expected gift image urls from snapshot"
        );
    }
}

/// Guard (舰长/提督/总督) message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardMessage {
    pub id: String,
    pub room: u64,
    pub sender: Sender,
    pub num: u32,
    pub unit: String,
    pub guard_level: u8,
    pub price: u64,
    pub timestamp: i64,
    #[serde(default)]
    pub archived: bool,
}

impl GuardMessage {
    /// Parse a guard message from raw WebSocket body (USER_TOAST_MSG)
    pub fn from_raw(body: &Value, room_id: u64) -> Option<Self> {
        let data = body.get("data")?;

        let sender = Sender {
            uid: data.get("uid")?.as_u64()?,
            uname: data.get("username")?.as_str()?.to_string(),
            face: data
                .get("face")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ..Default::default()
        };

        let guard_level = data.get("guard_level")?.as_u64()? as u8;
        let num = data.get("num").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        let unit = data
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("月")
            .to_string();
        let price = data.get("price").and_then(|v| v.as_u64()).unwrap_or(0);
        let timestamp = chrono::Utc::now().timestamp();

        Some(Self {
            id: uuid::Uuid::new_v4().to_string(),
            room: room_id,
            sender,
            num,
            unit,
            guard_level,
            price,
            timestamp,
            archived: false,
        })
    }
}

/// SuperChat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperChatMessage {
    pub id: String,
    pub room: u64,
    pub sender: Sender,
    pub message: String,
    pub price: u64,
    pub timestamp: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub background_color: String,
    pub background_bottom_color: String,
    #[serde(default)]
    pub archived: bool,
}

impl SuperChatMessage {
    /// Parse a superchat message from raw WebSocket body
    pub fn from_raw(body: &Value, room_id: u64) -> Option<Self> {
        let data = body.get("data")?;

        let mut sender = Sender::default();
        let user_info = data.get("user_info")?;
        sender.uid = data.get("uid")?.as_u64()?;
        sender.uname = user_info.get("uname")?.as_str()?.to_string();
        sender.face = user_info
            .get("face")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Parse medal info
        if let Some(medal) = data.get("medal_info") {
            sender.medal_info = MedalInfo {
                medal_level: medal
                    .get("medal_level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                medal_name: medal
                    .get("medal_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                anchor_uname: medal
                    .get("anchor_uname")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                anchor_roomid: medal
                    .get("anchor_roomid")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                guard_level: medal
                    .get("guard_level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                ..Default::default()
            };
        }

        let message = data.get("message")?.as_str()?.to_string();
        let price = data.get("price")?.as_u64()?;
        let start_time = data.get("start_time")?.as_i64()?;
        let end_time = data.get("end_time")?.as_i64()?;

        let background_color = data
            .get("background_color")
            .and_then(|v| v.as_str())
            .unwrap_or("#EDF5FF")
            .to_string();
        let background_bottom_color = data
            .get("background_bottom_color")
            .and_then(|v| v.as_str())
            .unwrap_or("#2A60B2")
            .to_string();

        Some(Self {
            id: data.get("id")?.as_u64()?.to_string(),
            room: room_id,
            sender,
            message,
            price,
            timestamp: start_time,
            start_time,
            end_time,
            background_color,
            background_bottom_color,
            archived: false,
        })
    }
}

/// Interact (enter/follow) message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractMessage {
    pub sender: Sender,
    pub action: i32,
}

impl InteractMessage {
    /// Parse an interact message from raw WebSocket body
    pub fn from_raw(body: &Value) -> Option<Self> {
        let data = body.get("data")?;

        let mut sender = Sender {
            uid: data.get("uid")?.as_u64()?,
            uname: data.get("uname")?.as_str()?.to_string(),
            ..Default::default()
        };

        // Parse medal info
        if let Some(medal) = data.get("fans_medal") {
            sender.medal_info = MedalInfo {
                medal_level: medal
                    .get("medal_level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                medal_name: medal
                    .get("medal_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                anchor_roomid: medal
                    .get("anchor_roomid")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                guard_level: medal
                    .get("guard_level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                ..Default::default()
            };
        }

        let action = data.get("msg_type")?.as_i64()? as i32;

        Some(Self { sender, action })
    }
}

/// Entry effect message (guard/level entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryEffectMessage {
    pub sender: Sender,
    pub privilege_type: u8,
}

impl EntryEffectMessage {
    /// Parse an entry effect message from raw WebSocket body
    pub fn from_raw(body: &Value) -> Option<Self> {
        let data = body.get("data")?;

        let mut sender = Sender {
            uid: data.get("uid")?.as_u64()?,
            ..Default::default()
        };

        // Extract uname from copy_writing (format: "<%{uname}%> ...")
        let copy_writing = data.get("copy_writing")?.as_str()?;
        if let Some(start) = copy_writing.find("<%") {
            if let Some(end) = copy_writing.find("%>") {
                sender.uname = copy_writing[start + 2..end].to_string();
            }
        }

        // Check if this is a guard entry by looking for guard titles in copy_writing
        let is_guard_entry = copy_writing.contains("舰长")
            || copy_writing.contains("提督")
            || copy_writing.contains("总督");

        // Get privilege_type from data, but set to 0 if not a guard entry
        let mut privilege_type = data.get("privilege_type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        if !is_guard_entry {
            privilege_type = 0;
        }

        Some(Self {
            sender,
            privilege_type,
        })
    }
}

/// Room change message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomChangeMessage {
    pub title: String,
    pub area_name: String,
    pub parent_area_name: String,
}

impl RoomChangeMessage {
    pub fn from_raw(body: &Value) -> Option<Self> {
        let data = body.get("data")?;

        Some(Self {
            title: data
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            area_name: data
                .get("area_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            parent_area_name: data
                .get("parent_area_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

/// Online rank count message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineRankCountMessage {
    pub count: u64,
}

impl OnlineRankCountMessage {
    pub fn from_raw(body: &Value) -> Option<Self> {
        let data = body.get("data")?;
        let count = data.get("count")?.as_u64()?;
        Some(Self { count })
    }
}

/// Warning message from Bilibili (live stream warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningMessage {
    pub msg: String,
}

impl WarningMessage {
    pub fn from_raw(body: &Value) -> Option<Self> {
        let msg = body
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Some(Self { msg })
    }
}

/// Cut off message from Bilibili (live stream cut off)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutOffMessage {
    pub msg: String,
}

impl CutOffMessage {
    pub fn from_raw(body: &Value) -> Option<Self> {
        let msg = body
            .get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Some(Self { msg })
    }
}
