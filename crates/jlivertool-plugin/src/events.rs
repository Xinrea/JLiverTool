use jlivertool_core::events::Event;
use jlivertool_core::messages::{DanmuMessage, GiftMessage, GuardMessage, InteractMessage, SuperChatMessage};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SerializableDanmu {
    pub uid: u64,
    pub uname: String,
    pub msg: String,
    pub timestamp: i64,
    pub medal_name: Option<String>,
    pub medal_level: Option<u32>,
    pub medal_room_id: Option<u64>,
}

impl From<&DanmuMessage> for SerializableDanmu {
    fn from(msg: &DanmuMessage) -> Self {
        let medal_name = if msg.sender.medal_info.medal_name.is_empty() {
            None
        } else {
            Some(msg.sender.medal_info.medal_name.clone())
        };
        let medal_level = if msg.sender.medal_info.medal_level > 0 {
            Some(msg.sender.medal_info.medal_level as u32)
        } else {
            None
        };
        let medal_room_id = if msg.sender.medal_info.anchor_roomid > 0 {
            Some(msg.sender.medal_info.anchor_roomid)
        } else {
            None
        };

        Self {
            uid: msg.sender.uid,
            uname: msg.sender.uname.clone(),
            msg: msg.content.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            medal_name,
            medal_level,
            medal_room_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializableGift {
    pub uid: u64,
    pub uname: String,
    pub gift_name: String,
    pub num: u32,
    pub price: u32,
    pub timestamp: i64,
}

impl From<&GiftMessage> for SerializableGift {
    fn from(msg: &GiftMessage) -> Self {
        Self {
            uid: msg.sender.uid,
            uname: msg.sender.uname.clone(),
            gift_name: msg.gift_info.name.clone(),
            num: msg.num,
            price: (msg.gift_info.price * msg.num as u64 / 1000) as u32,
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializableGuard {
    pub uid: u64,
    pub uname: String,
    pub guard_level: u32,
    pub num: u32,
    pub price: u32,
    pub timestamp: i64,
}

impl From<&GuardMessage> for SerializableGuard {
    fn from(msg: &GuardMessage) -> Self {
        Self {
            uid: msg.sender.uid,
            uname: msg.sender.uname.clone(),
            guard_level: msg.guard_level as u32,
            num: msg.num,
            price: (msg.price / 1000) as u32,
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializableSuperChat {
    pub uid: u64,
    pub uname: String,
    pub message: String,
    pub price: u32,
    pub timestamp: i64,
}

impl From<&SuperChatMessage> for SerializableSuperChat {
    fn from(msg: &SuperChatMessage) -> Self {
        Self {
            uid: msg.sender.uid,
            uname: msg.sender.uname.clone(),
            message: msg.message.clone(),
            price: msg.price as u32,
            timestamp: msg.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SerializableInteract {
    pub uid: u64,
    pub uname: String,
    pub msg_type: u32,
    pub timestamp: i64,
}

impl From<&InteractMessage> for SerializableInteract {
    fn from(msg: &InteractMessage) -> Self {
        Self {
            uid: msg.sender.uid,
            uname: msg.sender.uname.clone(),
            msg_type: msg.action as u32,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PluginEvent {
    NewDanmu(SerializableDanmu),
    NewGift(SerializableGift),
    NewGuard(SerializableGuard),
    NewSuperChat(SerializableSuperChat),
    NewInteract(SerializableInteract),
    UpdateRoom {
        room_id: u64,
        title: String,
        live_status: u8,
    },
    UpdateOnline {
        count: u64,
    },
    LiveStart,
    LiveEnd,
}

impl PluginEvent {
    pub fn channel(&self) -> &'static str {
        match self {
            PluginEvent::NewDanmu(_) => "new_danmu",
            PluginEvent::NewGift(_) => "new_gift",
            PluginEvent::NewGuard(_) => "new_guard",
            PluginEvent::NewSuperChat(_) => "new_superchat",
            PluginEvent::NewInteract(_) => "new_interact",
            PluginEvent::UpdateRoom { .. } => "update_room",
            PluginEvent::UpdateOnline { .. } => "update_online",
            PluginEvent::LiveStart => "live_start",
            PluginEvent::LiveEnd => "live_end",
        }
    }

    /// Convert a core Event to a PluginEvent if applicable
    pub fn from_core_event(event: &Event) -> Option<Self> {
        match event {
            Event::NewDanmu(msg) => Some(PluginEvent::NewDanmu(msg.into())),
            Event::NewGift(msg) => Some(PluginEvent::NewGift(msg.into())),
            Event::NewGuard(msg) => Some(PluginEvent::NewGuard(msg.into())),
            Event::NewSuperChat(msg) => Some(PluginEvent::NewSuperChat(msg.into())),
            Event::NewInteract(msg) => Some(PluginEvent::NewInteract(msg.into())),
            Event::UpdateRoom { room_id, title, live_status, .. } => Some(PluginEvent::UpdateRoom {
                room_id: room_id.real_id(),
                title: title.clone(),
                live_status: *live_status,
            }),
            Event::UpdateOnline { count } => Some(PluginEvent::UpdateOnline { count: *count }),
            Event::LiveStart => Some(PluginEvent::LiveStart),
            Event::LiveEnd => Some(PluginEvent::LiveEnd),
            _ => None,
        }
    }
}
