//! Bilibili REST API client

use crate::bilibili::wbi::WbiSigner;
use crate::types::{Cookies, RoomId};
use anyhow::{anyhow, Result};
use reqwest::{header, redirect, Client, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Base URLs
const LIVE_API_BASE: &str = "https://api.live.bilibili.com";
const WEB_API_BASE: &str = "https://api.bilibili.com";
const QR_LOGIN_MAX_REDIRECTS: usize = 5;

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

/// Start live response data
#[derive(Debug, Clone, Deserialize)]
pub struct StartLiveData {
    /// Whether face auth is needed
    #[serde(default)]
    pub need_face_auth: bool,
    /// QR code URL for face auth (if needed)
    #[serde(default)]
    pub qr: String,
    /// RTMP info (if live started successfully)
    pub rtmp: Option<RtmpData>,
    /// Change status (1 = started, 0 = already live)
    #[serde(default)]
    pub change: i32,
}

/// RTMP connection info
#[derive(Debug, Clone, Deserialize)]
pub struct RtmpData {
    pub addr: String,
    pub code: String,
}

/// Start live response (includes code for face auth check)
#[derive(Debug, Clone)]
pub struct StartLiveResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<StartLiveData>,
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
    pub area_id: u64,
    pub area_name: String,
    pub parent_area_id: u64,
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

#[derive(Debug, Clone, Deserialize)]
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
    /// Returns StartLiveResponse which includes code for checking face auth requirement
    pub async fn start_live(&self, room_id: u64, area_v2: u64) -> Result<StartLiveResponse> {
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
        let resp: ApiResponse<StartLiveData> = self.post_form(&url, &form).await?;

        Ok(StartLiveResponse {
            code: resp.code,
            message: resp.message,
            data: resp.data,
        })
    }

    /// Stop live stream
    pub async fn stop_live(&self, room_id: u64) -> Result<serde_json::Value> {
        let cookies = self
            .cookies
            .as_ref()
            .ok_or_else(|| anyhow!("Not logged in"))?;

        let mut form = HashMap::new();
        form.insert("room_id".to_string(), room_id.to_string());
        form.insert("platform".to_string(), "pc_link".to_string());
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
                    let login_url = data
                        .url
                        .ok_or_else(|| anyhow!("QR login response did not contain a login URL"))?;
                    let cookies = self.resolve_qr_login(&login_url).await?;
                    Ok((QrCodeStatus::Success, Some(cookies)))
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

    /// Exchange the ticket URL returned by QR polling for account cookies.
    async fn resolve_qr_login(&self, login_url: &str) -> Result<Cookies> {
        let client = Client::builder()
            .redirect(redirect::Policy::none())
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()?;
        let mut current_url =
            Url::parse(login_url).map_err(|e| anyhow!("QR login returned an invalid URL: {e}"))?;
        let mut cookie_values = HashMap::new();

        for redirect_count in 0..=QR_LOGIN_MAX_REDIRECTS {
            if current_url.scheme() != "https" {
                return Err(anyhow!("QR login redirected to a non-HTTPS URL"));
            }

            let cookie_header = cookie_values
                .iter()
                .map(|(name, value)| format!("{name}={value}"))
                .collect::<Vec<_>>()
                .join("; ");
            let mut request = client
                .get(current_url.clone())
                .header(header::REFERER, "https://passport.bilibili.com/");
            if !cookie_header.is_empty() {
                request = request.header(header::COOKIE, cookie_header);
            }

            let response = request.send().await?;
            for set_cookie in response.headers().get_all(header::SET_COOKIE) {
                let set_cookie = set_cookie
                    .to_str()
                    .map_err(|_| anyhow!("QR login returned an invalid Set-Cookie header"))?;
                if let Some((name, value)) = parse_set_cookie(set_cookie) {
                    cookie_values.insert(name.to_string(), value.to_string());
                }
            }

            if let Some(cookies) = login_cookies_from_map(&cookie_values) {
                return Ok(cookies);
            }

            let location = response.headers().get(header::LOCATION).cloned();
            drop(response);
            let Some(location) = location else {
                return Err(anyhow!(
                    "QR login succeeded but did not return complete account cookies"
                ));
            };
            if redirect_count == QR_LOGIN_MAX_REDIRECTS {
                return Err(anyhow!("QR login redirected too many times"));
            }
            current_url = current_url
                .join(location.to_str()?)
                .map_err(|e| anyhow!("QR login returned an invalid redirect URL: {e}"))?;
        }

        Err(anyhow!("QR login redirected too many times"))
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

fn parse_set_cookie(header: &str) -> Option<(&str, &str)> {
    let cookie = header.split(';').next()?;
    let (name, value) = cookie.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some((name, value.trim()))
    }
}

fn login_cookies_from_map(values: &HashMap<String, String>) -> Option<Cookies> {
    let dede_user_id = values.get("DedeUserID")?.clone();
    let sessdata = values.get("SESSDATA")?.clone();
    let bili_jct = values.get("bili_jct")?.clone();

    Some(Cookies {
        dede_user_id,
        dede_user_id_ck_md5: values.get("DedeUserID__ckMd5").cloned().unwrap_or_default(),
        sessdata,
        bili_jct,
        sid: values.get("sid").cloned().unwrap_or_default(),
        ..Cookies::default()
    })
}

#[cfg(test)]
mod tests {
    use super::{login_cookies_from_map, parse_set_cookie};
    use std::collections::HashMap;

    #[test]
    fn parses_set_cookie_without_attributes() {
        assert_eq!(
            parse_set_cookie("SESSDATA=value%2Cwith%2Ccommas; Path=/; HttpOnly"),
            Some(("SESSDATA", "value%2Cwith%2Ccommas"))
        );
    }

    #[test]
    fn requires_complete_login_cookies() {
        let mut values = HashMap::from([
            ("DedeUserID".to_string(), "123".to_string()),
            ("SESSDATA".to_string(), "session".to_string()),
        ]);
        assert!(login_cookies_from_map(&values).is_none());

        values.insert("bili_jct".to_string(), "csrf".to_string());
        let cookies = login_cookies_from_map(&values).expect("complete login cookies");
        assert_eq!(cookies.dede_user_id, "123");
        assert_eq!(cookies.sessdata, "session");
        assert_eq!(cookies.bili_jct, "csrf");
    }
}

impl Default for BiliApi {
    fn default() -> Self {
        Self::new().expect("Failed to create BiliApi client")
    }
}
