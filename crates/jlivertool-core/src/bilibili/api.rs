//! Bilibili REST API client

use crate::bilibili::wbi::WbiSigner;
use crate::types::{Cookies, RoomId};
use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Base URLs
const LIVE_API_BASE: &str = "https://api.live.bilibili.com";
const WEB_API_BASE: &str = "https://api.bilibili.com";

/// Common API response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> Result<T> {
        if self.code == 0 {
            self.data.ok_or_else(|| anyhow!("API returned no data"))
        } else {
            Err(anyhow!("API error {}: {}", self.code, self.message))
        }
    }
}

/// Room init response
#[derive(Debug, Deserialize)]
pub struct RoomInitData {
    pub room_id: u64,
    pub short_id: u64,
    pub uid: u64,
    pub live_status: u8,
    pub live_time: i64,
}

/// Room info response
#[derive(Debug, Deserialize)]
pub struct RoomInfoData {
    pub room_id: u64,
    pub short_id: u64,
    pub uid: u64,
    pub title: String,
    pub live_status: u8,
    pub area_name: String,
    pub parent_area_name: String,
    pub keyframe: String,
    pub tags: String,
    pub description: String,
}

/// Danmu info (WebSocket connection info)
#[derive(Debug, Deserialize)]
pub struct DanmuInfoData {
    pub token: String,
    pub host_list: Vec<DanmuHost>,
}

#[derive(Debug, Deserialize)]
pub struct DanmuHost {
    pub host: String,
    pub port: u16,
    pub wss_port: u16,
    pub ws_port: u16,
}

/// Gift config item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GiftConfigItem {
    pub id: u64,
    pub name: String,
    pub price: u64,
    pub coin_type: String,
    pub img_basic: String,
    pub img_dynamic: String,
    pub gif: String,
    pub webp: String,
}

#[derive(Debug, Deserialize)]
pub struct GiftConfigData {
    pub list: Vec<GiftConfigItem>,
}

/// Online gold rank item (from queryContributionRank API)
#[derive(Debug, Clone, Deserialize)]
pub struct OnlineGoldRankItem {
    pub uid: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub face: String,
    #[serde(default)]
    pub rank: u32,
    #[serde(default)]
    pub score: u64,
    #[serde(default)]
    pub guard_level: u8,
    #[serde(default)]
    pub medal_info: Option<OnlineRankMedalInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OnlineRankMedalInfo {
    #[serde(default)]
    pub guard_level: u8,
    #[serde(default)]
    pub medal_name: String,
    #[serde(default)]
    pub level: u8,
}

#[derive(Debug, Deserialize)]
pub struct OnlineGoldRankData {
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub item: Vec<OnlineGoldRankItem>,
}

/// Guard list item
#[derive(Debug, Clone, Deserialize)]
pub struct GuardListItem {
    pub uid: u64,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub face: String,
    #[serde(default)]
    pub guard_level: u8,
    #[serde(default)]
    pub ruid: u64,
    #[serde(default)]
    pub medal_info: Option<GuardMedalInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GuardMedalInfo {
    #[serde(default)]
    pub medal_name: String,
    #[serde(default)]
    pub medal_level: u8,
}

/// Guard list response - the API returns top3 and list separately
#[derive(Debug, Deserialize)]
pub struct GuardListData {
    #[serde(default)]
    pub top3: Vec<GuardListItem>,
    #[serde(default)]
    pub list: Vec<GuardListItem>,
    #[serde(default)]
    pub info: GuardListInfo,
}

#[derive(Debug, Default, Deserialize)]
pub struct GuardListInfo {
    #[serde(default)]
    pub num: u32,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub now: u32,
}

/// Nav response (for login status and WBI keys)
#[derive(Debug, Deserialize)]
pub struct NavData {
    #[serde(rename = "isLogin")]
    pub is_login: bool,
    pub mid: Option<u64>,
    pub uname: Option<String>,
    pub face: Option<String>,
    pub wbi_img: Option<WbiImg>,
}

#[derive(Debug, Deserialize)]
pub struct WbiImg {
    pub img_url: String,
    pub sub_url: String,
}

/// User info response
#[derive(Debug, Clone, Deserialize)]
pub struct UserInfoData {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub level: u8,
    #[serde(default)]
    pub sex: String,
    #[serde(default)]
    pub birthday: String,
    #[serde(default)]
    pub top_photo: String,
    #[serde(default)]
    pub fans_badge: bool,
    #[serde(default)]
    pub official: UserOfficialInfo,
    #[serde(default)]
    pub vip: UserVipInfo,
    #[serde(default)]
    pub live_room: Option<UserLiveRoom>,
}

/// User official verification info
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserOfficialInfo {
    #[serde(default)]
    pub role: u8,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub desc: String,
    #[serde(rename = "type", default)]
    pub official_type: i8,
}

/// User VIP info
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserVipInfo {
    #[serde(rename = "type", default)]
    pub vip_type: u8,
    #[serde(default)]
    pub status: u8,
    #[serde(default)]
    pub label: UserVipLabel,
}

/// User VIP label
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserVipLabel {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub label_theme: String,
}

/// User live room info
#[derive(Debug, Clone, Deserialize)]
pub struct UserLiveRoom {
    #[serde(default)]
    pub roomid: u64,
    #[serde(rename = "liveStatus", default)]
    pub live_status: u8,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cover: String,
}

/// QR code login response
#[derive(Debug, Deserialize)]
pub struct QrCodeData {
    pub url: String,
    pub qrcode_key: String,
}

/// QR code poll response
#[derive(Debug, Deserialize)]
pub struct QrCodePollData {
    pub code: i32,
    pub message: String,
    pub url: Option<String>,
    pub refresh_token: Option<String>,
}

/// QR code status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrCodeStatus {
    /// Waiting for scan
    NeedScan,
    /// Scanned, waiting for confirm
    NeedConfirm,
    /// Login success
    Success,
    /// QR code expired
    Expired,
    /// Unknown error
    Error,
}

/// Bilibili API client
#[derive(Clone)]
pub struct BiliApi {
    client: Client,
    cookies: Option<Cookies>,
    wbi_signer: Arc<WbiSigner>,
}

impl BiliApi {
    /// Create a new API client
    pub fn new() -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );
        headers.insert(
            header::REFERER,
            header::HeaderValue::from_static("https://live.bilibili.com/"),
        );

        let client = Client::builder()
            .default_headers(headers)
            // Connection pool configuration for better performance
            .pool_max_idle_per_host(5)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            cookies: None,
            wbi_signer: Arc::new(WbiSigner::new()),
        })
    }

    /// Set cookies for authenticated requests
    pub fn with_cookies(mut self, cookies: Cookies) -> Self {
        self.cookies = Some(cookies);
        self
    }

    /// Update cookies
    pub fn set_cookies(&mut self, cookies: Option<Cookies>) {
        self.cookies = cookies;
    }

    /// Get reference to WBI signer
    pub fn wbi_signer(&self) -> &Arc<WbiSigner> {
        &self.wbi_signer
    }

    /// Make a GET request
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<ApiResponse<T>> {
        let mut req = self.client.get(url);

        if let Some(ref cookies) = self.cookies {
            req = req.header(header::COOKIE, cookies.to_cookie_string());
        }

        let resp = req.send().await?;
        let body = resp.json::<ApiResponse<T>>().await?;
        Ok(body)
    }

    /// Make a POST request with form data
    async fn post_form<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &HashMap<String, String>,
    ) -> Result<ApiResponse<T>> {
        let mut req = self.client.post(url).form(form);

        if let Some(ref cookies) = self.cookies {
            req = req.header(header::COOKIE, cookies.to_cookie_string());
        }

        let resp = req.send().await?;
        let body = resp.json::<ApiResponse<T>>().await?;
        Ok(body)
    }

    /// Initialize room info (get real room_id from short_id)
    pub async fn room_init(&self, room_id: u64) -> Result<RoomInitData> {
        let url = format!("{}/room/v1/Room/room_init?id={}", LIVE_API_BASE, room_id);
        self.get(&url).await?.into_result()
    }

    /// Get room info
    pub async fn get_room_info(&self, room_id: u64) -> Result<RoomInfoData> {
        let url = format!(
            "{}/room/v1/Room/get_info?room_id={}",
            LIVE_API_BASE, room_id
        );
        self.get(&url).await?.into_result()
    }

    /// Get danmu info (WebSocket connection info with token)
    pub async fn get_danmu_info(&self, room_id: u64) -> Result<DanmuInfoData> {
        // Ensure WBI keys are loaded
        if !self.wbi_signer.is_cache_valid() {
            self.nav().await?;
        }

        let mut params = HashMap::new();
        params.insert("id".to_string(), room_id.to_string());
        params.insert("type".to_string(), "0".to_string());

        let query = self.wbi_signer.get_signed_query(&mut params)?;
        let url = format!(
            "{}/xlive/web-room/v1/index/getDanmuInfo?{}",
            LIVE_API_BASE, query
        );

        self.get(&url).await?.into_result()
    }

    /// Get gift config for a room
    pub async fn get_gift_config(&self, room_id: u64) -> Result<GiftConfigData> {
        let url = format!(
            "{}/xlive/web-room/v1/giftPanel/giftConfig?platform=pc&room_id={}",
            LIVE_API_BASE, room_id
        );
        self.get(&url).await?.into_result()
    }

    /// Get online gold rank (contribution rank / audience list)
    pub async fn get_online_gold_rank(&self, room_id: u64, ruid: u64, page: u32, page_size: u32) -> Result<OnlineGoldRankData> {
        let url = format!(
            "{}/xlive/general-interface/v1/rank/queryContributionRank?ruid={}&room_id={}&page={}&page_size={}&type=online_rank&switch=contribution_rank",
            LIVE_API_BASE, ruid, room_id, page, page_size
        );
        self.get(&url).await?.into_result()
    }

    /// Get guard list (舰长列表)
    pub async fn get_guard_list(&self, room_id: u64, ruid: u64, page: u32) -> Result<GuardListData> {
        let url = format!(
            "{}/xlive/app-room/v2/guardTab/topList?roomid={}&ruid={}&page={}&page_size=30",
            LIVE_API_BASE, room_id, ruid, page
        );
        self.get(&url).await?.into_result()
    }

    /// Get nav info (login status + WBI keys)
    pub async fn nav(&self) -> Result<NavData> {
        let url = format!("{}/x/web-interface/nav", WEB_API_BASE);
        let data = self.get::<NavData>(&url).await?.into_result()?;

        // Update WBI keys
        if let Some(ref wbi_img) = data.wbi_img {
            self.wbi_signer
                .update_keys(&wbi_img.img_url, &wbi_img.sub_url);
        }

        Ok(data)
    }

    /// Get user info
    pub async fn get_user_info(&self, mid: u64) -> Result<UserInfoData> {
        // Ensure WBI keys are loaded
        if !self.wbi_signer.is_cache_valid() {
            self.nav().await?;
        }

        let mut params = HashMap::new();
        params.insert("mid".to_string(), mid.to_string());

        let query = self.wbi_signer.get_signed_query(&mut params)?;
        let url = format!("{}/x/space/wbi/acc/info?{}", WEB_API_BASE, query);

        self.get(&url).await?.into_result()
    }

    /// Send danmaku to room
    pub async fn send_danmu(
        &self,
        room_id: u64,
        msg: &str,
        mode: u8,
        color: u32,
        fontsize: u8,
    ) -> Result<()> {
        let cookies = self
            .cookies
            .as_ref()
            .ok_or_else(|| anyhow!("Not logged in"))?;

        let mut form = HashMap::new();
        form.insert("bubble".to_string(), "0".to_string());
        form.insert("msg".to_string(), msg.to_string());
        form.insert("color".to_string(), color.to_string());
        form.insert("mode".to_string(), mode.to_string());
        form.insert("fontsize".to_string(), fontsize.to_string());
        form.insert("rnd".to_string(), chrono::Utc::now().timestamp().to_string());
        form.insert("roomid".to_string(), room_id.to_string());
        form.insert("csrf".to_string(), cookies.bili_jct.clone());
        form.insert("csrf_token".to_string(), cookies.bili_jct.clone());

        let url = format!("{}/msg/send", LIVE_API_BASE);
        let resp: ApiResponse<serde_json::Value> = self.post_form(&url, &form).await?;

        if resp.code == 0 {
            Ok(())
        } else {
            Err(anyhow!("Failed to send danmaku: {}", resp.message))
        }
    }

    /// Update room title
    pub async fn update_room_title(&self, room_id: u64, title: &str) -> Result<()> {
        let cookies = self
            .cookies
            .as_ref()
            .ok_or_else(|| anyhow!("Not logged in"))?;

        let mut form = HashMap::new();
        form.insert("room_id".to_string(), room_id.to_string());
        form.insert("title".to_string(), title.to_string());
        form.insert("csrf".to_string(), cookies.bili_jct.clone());
        form.insert("csrf_token".to_string(), cookies.bili_jct.clone());

        let url = format!("{}/room/v1/Room/update", LIVE_API_BASE);
        let resp: ApiResponse<serde_json::Value> = self.post_form(&url, &form).await?;

        if resp.code == 0 {
            Ok(())
        } else {
            Err(anyhow!("Failed to update room title: {}", resp.message))
        }
    }

    /// Start live stream
    pub async fn start_live(&self, room_id: u64, area_v2: u64) -> Result<serde_json::Value> {
        let cookies = self
            .cookies
            .as_ref()
            .ok_or_else(|| anyhow!("Not logged in"))?;

        let mut form = HashMap::new();
        form.insert("room_id".to_string(), room_id.to_string());
        form.insert("platform".to_string(), "pc_link".to_string());
        form.insert("area_v2".to_string(), area_v2.to_string());
        form.insert("csrf".to_string(), cookies.bili_jct.clone());
        form.insert("csrf_token".to_string(), cookies.bili_jct.clone());

        let url = format!("{}/room/v1/Room/startLive", LIVE_API_BASE);
        self.post_form(&url, &form).await?.into_result()
    }

    /// Stop live stream
    pub async fn stop_live(&self, room_id: u64) -> Result<serde_json::Value> {
        let cookies = self
            .cookies
            .as_ref()
            .ok_or_else(|| anyhow!("Not logged in"))?;

        let mut form = HashMap::new();
        form.insert("room_id".to_string(), room_id.to_string());
        form.insert("csrf".to_string(), cookies.bili_jct.clone());
        form.insert("csrf_token".to_string(), cookies.bili_jct.clone());

        let url = format!("{}/room/v1/Room/stopLive", LIVE_API_BASE);
        self.post_form(&url, &form).await?.into_result()
    }

    /// Helper: Get full room info with RoomId
    pub async fn get_room(&self, room_id: u64) -> Result<RoomId> {
        let init = self.room_init(room_id).await?;
        Ok(RoomId::new(init.short_id, init.room_id, init.uid))
    }

    /// Generate QR code for login
    pub async fn qr_generate(&self) -> Result<QrCodeData> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
        self.get::<QrCodeData>(url).await?.into_result()
    }

    /// Poll QR code login status
    pub async fn qr_poll(&self, qrcode_key: &str) -> Result<(QrCodeStatus, Option<Cookies>)> {
        let url = format!(
            "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}",
            qrcode_key
        );
        let resp: ApiResponse<QrCodePollData> = self.get(&url).await?;

        if let Some(data) = resp.data {
            match data.code {
                0 => {
                    // Success - parse cookies from URL
                    if let Some(url) = data.url {
                        if let Some(query) = url.split('?').nth(1) {
                            let cookies = Cookies::from_query_string(query);
                            return Ok((QrCodeStatus::Success, Some(cookies)));
                        }
                    }
                    Ok((QrCodeStatus::Error, None))
                }
                86101 => Ok((QrCodeStatus::NeedScan, None)),
                86090 => Ok((QrCodeStatus::NeedConfirm, None)),
                86038 => Ok((QrCodeStatus::Expired, None)),
                _ => Ok((QrCodeStatus::Error, None)),
            }
        } else {
            Ok((QrCodeStatus::Error, None))
        }
    }

    /// Logout
    pub async fn logout(&self) -> Result<()> {
        let cookies = self
            .cookies
            .as_ref()
            .ok_or_else(|| anyhow!("Not logged in"))?;

        let mut form = HashMap::new();
        form.insert("biliCSRF".to_string(), cookies.bili_jct.clone());

        let url = "https://passport.bilibili.com/login/exit/v2";
        let _: ApiResponse<serde_json::Value> = self.post_form(url, &form).await?;
        Ok(())
    }
}

impl Default for BiliApi {
    fn default() -> Self {
        Self::new().expect("Failed to create BiliApi client")
    }
}
