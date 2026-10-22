//! Anonymous PowerLive feedback. A draft owns a fixed snapshot of its attachments;
//! callers must retain it, including its submission ID, whenever a result is unknown.

use chrono::{DateTime, Utc};
use image::{ImageFormat, ImageReader};
use reqwest::{header, multipart, Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io::Cursor, path::Path, time::Duration};
use uuid::Uuid;

pub const FEEDBACK_APP_ID: &str = "jlivertool";
pub const FEEDBACK_BASE_URL: &str = "https://api.powerlive.io";
pub const MAX_FEEDBACK_FILE_BYTES: usize = 5 * 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackPhase {
    Validation,
    Prepare,
    Submit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackErrorCode {
    ValidationError,
    FeedbackAppDisabled,
    InvalidUploadToken,
    ChallengeRequired,
    ChallengeFailed,
    SubmissionConflict,
    SubmissionInProgress,
    RequestTooLarge,
    RateLimitExceeded,
    UploadBusy,
    UploadQuotaExceeded,
    FeedbackUnavailable,
    NetworkError,
    InvalidResponse,
    HttpError,
    Other(String),
}

impl FeedbackErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ValidationError => "validation_error",
            Self::FeedbackAppDisabled => "feedback_app_disabled",
            Self::InvalidUploadToken => "invalid_upload_token",
            Self::ChallengeRequired => "challenge_required",
            Self::ChallengeFailed => "challenge_failed",
            Self::SubmissionConflict => "submission_conflict",
            Self::SubmissionInProgress => "submission_in_progress",
            Self::RequestTooLarge => "request_too_large",
            Self::RateLimitExceeded => "rate_limit_exceeded",
            Self::UploadBusy => "upload_busy",
            Self::UploadQuotaExceeded => "upload_quota_exceeded",
            Self::FeedbackUnavailable => "feedback_unavailable",
            Self::NetworkError => "network_error",
            Self::InvalidResponse => "invalid_response",
            Self::HttpError => "http_error",
            Self::Other(code) => code,
        }
    }
}

impl From<&str> for FeedbackErrorCode {
    fn from(value: &str) -> Self {
        match value {
            "validation_error" => Self::ValidationError,
            "feedback_app_disabled" => Self::FeedbackAppDisabled,
            "invalid_upload_token" => Self::InvalidUploadToken,
            "challenge_required" => Self::ChallengeRequired,
            "challenge_failed" => Self::ChallengeFailed,
            "submission_conflict" => Self::SubmissionConflict,
            "submission_in_progress" => Self::SubmissionInProgress,
            "request_too_large" => Self::RequestTooLarge,
            "rate_limit_exceeded" => Self::RateLimitExceeded,
            "upload_busy" => Self::UploadBusy,
            "upload_quota_exceeded" => Self::UploadQuotaExceeded,
            "feedback_unavailable" => Self::FeedbackUnavailable,
            "network_error" => Self::NetworkError,
            "invalid_response" => Self::InvalidResponse,
            "http_error" => Self::HttpError,
            other => Self::Other(other.to_owned()),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("{message}")]
pub struct FeedbackError {
    pub phase: FeedbackPhase,
    /// Zero means the outcome could not be confirmed from a valid response.
    pub status: u16,
    pub code: FeedbackErrorCode,
    pub message: String,
    pub retry_after_seconds: Option<u64>,
    pub site_key: Option<String>,
    pub action: Option<String>,
}

impl FeedbackError {
    fn new(phase: FeedbackPhase, code: FeedbackErrorCode, message: impl Into<String>) -> Self {
        Self {
            phase,
            status: 0,
            code,
            message: message.into(),
            retry_after_seconds: None,
            site_key: None,
            action: None,
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self::new(
            FeedbackPhase::Validation,
            FeedbackErrorCode::ValidationError,
            message,
        )
    }

    fn network(phase: FeedbackPhase) -> Self {
        Self::new(
            phase,
            FeedbackErrorCode::NetworkError,
            "网络异常或请求超时，请保留原反馈后重试；服务端可能已经收到请求",
        )
    }

    fn invalid_response(phase: FeedbackPhase) -> Self {
        Self::new(
            phase,
            FeedbackErrorCode::InvalidResponse,
            "无法确认服务器响应，请保留原反馈后重试",
        )
    }

    /// Safe guidance chosen by error code; no raw proxy HTML or reflected input.
    pub fn user_message(&self) -> String {
        let message = match self.code {
            FeedbackErrorCode::ValidationError if self.phase == FeedbackPhase::Validation => &self.message,
            FeedbackErrorCode::ValidationError => "反馈格式校验失败，请检查正文、邮箱和附件后重新提交",
            FeedbackErrorCode::FeedbackAppDisabled => "反馈服务尚未启用，请联系软件维护者检查配置",
            FeedbackErrorCode::InvalidUploadToken => "提交凭证已失效，请保持网络稳定后重试原反馈",
            FeedbackErrorCode::ChallengeRequired | FeedbackErrorCode::ChallengeFailed => "反馈服务需要人机验证，请稍后再试",
            FeedbackErrorCode::SubmissionConflict => "原反馈编号与当前信息或网络不一致，请恢复原网络后重试；如需新建反馈，请先确认原反馈结果",
            FeedbackErrorCode::SubmissionInProgress => "原反馈仍在提交中，请稍后重试确认结果",
            FeedbackErrorCode::RequestTooLarge => "反馈超过服务端大小限制，请缩小或移除附件后重新提交",
            FeedbackErrorCode::RateLimitExceeded => "提交过于频繁，请稍后再试",
            FeedbackErrorCode::UploadBusy => "反馈服务正忙，请稍后重试",
            FeedbackErrorCode::UploadQuotaExceeded => "反馈服务今日额度已用尽，请稍后再试",
            FeedbackErrorCode::FeedbackUnavailable => "反馈服务暂时不可用，请保留原反馈后重试；服务端可能已经收到请求",
            FeedbackErrorCode::NetworkError => "网络异常或请求超时，请保留原反馈后重试；服务端可能已经收到请求",
            FeedbackErrorCode::InvalidResponse => "无法确认服务器响应，请保留原反馈后重试",
            _ => "反馈请求失败，请保留原反馈后重试",
        };
        match self.retry_after_seconds {
            Some(seconds) => format!("{message}（至少等待 {seconds} 秒）"),
            None => message.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum AttachmentKind {
    Screenshot,
    Log,
}

/// Owns final attachment bytes rather than a mutable filesystem path. Its Debug
/// output is deliberately omitted because attachments may contain private data.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedbackAttachment {
    filename: String,
    bytes: Vec<u8>,
    kind: AttachmentKind,
}

impl FeedbackAttachment {
    pub fn screenshot(filename: impl Into<String>, bytes: Vec<u8>) -> Result<Self, FeedbackError> {
        Self::new(filename.into(), bytes, AttachmentKind::Screenshot)
    }

    pub fn log(filename: impl Into<String>, bytes: Vec<u8>) -> Result<Self, FeedbackError> {
        Self::new(filename.into(), bytes, AttachmentKind::Log)
    }

    fn new(filename: String, bytes: Vec<u8>, kind: AttachmentKind) -> Result<Self, FeedbackError> {
        // Never send a local directory, even if a caller passes a complete path.
        let filename = filename.rsplit(['/', '\\']).next().unwrap_or("").to_owned();
        let attachment = Self {
            filename,
            bytes,
            kind,
        };
        attachment.validate()?;
        Ok(attachment)
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    fn validate(&self) -> Result<(), FeedbackError> {
        if self.bytes.is_empty() {
            return Err(FeedbackError::validation("附件不能为空文件"));
        }
        if self.size() > MAX_FEEDBACK_FILE_BYTES {
            return Err(FeedbackError::validation("截图和日志各自最大为 5 MiB"));
        }
        if self.filename.is_empty()
            || self.filename.len() > 255
            || self.filename.chars().any(char::is_control)
            || self.filename.contains(['/', '\\'])
        {
            return Err(FeedbackError::validation("附件文件名无效"));
        }
        let ext = Path::new(&self.filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match self.kind {
            AttachmentKind::Log => {
                let text = std::str::from_utf8(&self.bytes).map_err(|_| {
                    FeedbackError::validation("日志必须为 UTF-8 纯文本 .txt 或 .log 文件")
                })?;
                if !matches!(ext.as_str(), "txt" | "log") || !valid_text(text) {
                    return Err(FeedbackError::validation(
                        "日志必须为 UTF-8 纯文本 .txt 或 .log 文件，不能包含控制字符",
                    ));
                }
            }
            AttachmentKind::Screenshot => {
                let format = image::guess_format(&self.bytes).ok();
                if !matches!(
                    (format, ext.as_str()),
                    (Some(ImageFormat::Png), "png") | (Some(ImageFormat::Jpeg), "jpg" | "jpeg")
                ) {
                    return Err(FeedbackError::validation(
                        "截图必须为有效的 PNG 或 JPEG 图片",
                    ));
                }
                let format = format.unwrap();
                let (width, height) = ImageReader::with_format(Cursor::new(&self.bytes), format)
                    .into_dimensions()
                    .map_err(|_| FeedbackError::validation("截图内容损坏"))?;
                if width == 0
                    || height == 0
                    || width > 16384
                    || height > 16384
                    || u64::from(width) * u64::from(height) > 10_000_000
                {
                    return Err(FeedbackError::validation(
                        "截图最多 1000 万像素，单边最多 16384 像素",
                    ));
                }
                ImageReader::with_format(Cursor::new(&self.bytes), format)
                    .decode()
                    .map_err(|_| FeedbackError::validation("截图内容损坏"))?;
            }
        }
        Ok(())
    }

    fn mime_type(&self) -> &'static str {
        match self.kind {
            AttachmentKind::Log => "text/plain; charset=utf-8",
            AttachmentKind::Screenshot if self.bytes.starts_with(b"\x89PNG\r\n\x1a\n") => {
                "image/png"
            }
            AttachmentKind::Screenshot => "image/jpeg",
        }
    }
}

/// Persist this complete snapshot before contacting the server. Changing any
/// content or metadata requires constructing a new draft with a new UUID.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedbackDraft {
    submission_id: String,
    message: String,
    email: String,
    app_version: String,
    os: String,
    screenshot: Option<FeedbackAttachment>,
    log: Option<FeedbackAttachment>,
}

impl FeedbackDraft {
    /// Validate form fields without decoding attachments or allocating a draft ID.
    pub fn validate_input(message: &str, email: &str) -> Result<(), FeedbackError> {
        let trimmed = message.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 5000 || !valid_text(message) {
            return Err(FeedbackError::validation(
                "反馈内容需要 1 到 5000 字的有效文字，不能包含控制字符",
            ));
        }
        if !email.trim().is_empty() && !valid_email(email.trim()) {
            return Err(FeedbackError::validation(
                "请输入有效邮箱地址，例如 user@example.com",
            ));
        }
        Ok(())
    }

    pub fn new(
        message: impl Into<String>,
        email: impl Into<String>,
        app_version: impl Into<String>,
        os: impl Into<String>,
        screenshot: Option<FeedbackAttachment>,
        log: Option<FeedbackAttachment>,
    ) -> Result<Self, FeedbackError> {
        let message = message.into();
        let email = email.into();
        Self::validate_input(&message, &email)?;
        let draft = Self {
            submission_id: Uuid::new_v4().to_string(),
            message: message.trim().to_owned(),
            email: email.trim().to_owned(),
            app_version: app_version.into(),
            os: os.into(),
            screenshot,
            log,
        };
        draft.validate()?;
        Ok(draft)
    }

    pub fn submission_id(&self) -> &str {
        &self.submission_id
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn app_version(&self) -> &str {
        &self.app_version
    }
    pub fn os(&self) -> &str {
        &self.os
    }
    pub fn screenshot(&self) -> Option<&FeedbackAttachment> {
        self.screenshot.as_ref()
    }
    pub fn log(&self) -> Option<&FeedbackAttachment> {
        self.log.as_ref()
    }

    /// Also check drafts loaded from disk instead of trusting deserialized input.
    pub fn validate(&self) -> Result<(), FeedbackError> {
        if !valid_uuid(&self.submission_id) {
            return Err(FeedbackError::validation("反馈提交编号无效"));
        }
        Self::validate_input(&self.message, &self.email)?;
        if self.email != self.email.trim() {
            return Err(FeedbackError::validation(
                "请输入有效邮箱地址，例如 user@example.com",
            ));
        }
        if self.app_version.len() > 100
            || self.os.len() > 200
            || !valid_text(&self.app_version)
            || !valid_text(&self.os)
        {
            return Err(FeedbackError::validation(
                "软件版本或系统信息过长或包含无效字符",
            ));
        }
        for (attachment, kind) in [
            (&self.screenshot, AttachmentKind::Screenshot),
            (&self.log, AttachmentKind::Log),
        ] {
            if let Some(attachment) = attachment {
                if attachment.kind != kind {
                    return Err(FeedbackError::validation("附件类型不匹配"));
                }
                attachment.validate()?;
            }
        }
        Ok(())
    }
}

fn valid_text(value: &str) -> bool {
    value
        .chars()
        .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}

fn valid_uuid(value: &str) -> bool {
    value.len() == 36 && Uuid::parse_str(value).is_ok_and(|id| !id.is_nil())
}

fn valid_email(value: &str) -> bool {
    if value.len() > 254
        || !value.is_ascii()
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty()
        || local.len() > 64
        || local.starts_with('.')
        || local.ends_with('.')
        || local.contains("..")
    {
        return false;
    }
    if !local
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&b))
    {
        return false;
    }
    !domain.is_empty()
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedbackReceipt {
    pub id: String,
    pub message: String,
}

/// The temporary token is private and has no Debug or serialization output.
pub struct FeedbackUpload {
    token: String,
    submission_id: String,
    pub expires_at: DateTime<Utc>,
    pub max_file_bytes: u64,
    pub max_request_bytes: u64,
}

pub enum PreparedFeedback {
    Upload(FeedbackUpload),
    AlreadySubmitted(FeedbackReceipt),
}

#[derive(Clone)]
pub struct FeedbackClient {
    client: Client,
    base_url: String,
}

impl FeedbackClient {
    pub fn new() -> Result<Self, FeedbackError> {
        Self::with_base_url(FEEDBACK_BASE_URL)
    }

    /// Alternate bases are useful for integration tests. Production callers use
    /// `new()`. This client is separate from the authenticated Bilibili client.
    pub fn with_base_url(base_url: &str) -> Result<Self, FeedbackError> {
        let client = Client::builder()
            .cookie_store(false)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .user_agent("JLiverTool-feedback")
            .build()
            .map_err(|_| FeedbackError::network(FeedbackPhase::Prepare))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }

    pub async fn prepare(&self, draft: &FeedbackDraft) -> Result<PreparedFeedback, FeedbackError> {
        draft.validate()?;
        let request = serde_json::json!({
            "app_id": FEEDBACK_APP_ID,
            "submission_id": draft.submission_id,
            "app_version": draft.app_version,
            "os": draft.os,
            "screenshot_size": draft.screenshot.as_ref().map_or(0, FeedbackAttachment::size),
            "log_size": draft.log.as_ref().map_or(0, FeedbackAttachment::size),
        });
        let response = self
            .client
            .post(format!("{}/feedback/prepare", self.base_url))
            .timeout(Duration::from_secs(25))
            .json(&request)
            .send()
            .await
            .map_err(|_| FeedbackError::network(FeedbackPhase::Prepare))?;
        let data = read_response(response, FeedbackPhase::Prepare).await?;
        if data.get("already_submitted").and_then(Value::as_bool) == Some(true) {
            let id = data
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| valid_uuid(id))
                .ok_or_else(|| FeedbackError::invalid_response(FeedbackPhase::Prepare))?;
            return Ok(PreparedFeedback::AlreadySubmitted(FeedbackReceipt {
                id: id.to_owned(),
                message: "反馈已收到".to_owned(),
            }));
        }
        let invalid = || FeedbackError::invalid_response(FeedbackPhase::Prepare);
        let token = data
            .get("upload_token")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(invalid)?;
        header::HeaderValue::from_str(token).map_err(|_| invalid())?;
        let expires_at = data
            .get("expires_at")
            .and_then(Value::as_str)
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .ok_or_else(invalid)?
            .with_timezone(&Utc);
        let max_file_bytes = data
            .get("max_file_bytes")
            .and_then(Value::as_u64)
            .filter(|v| *v > 0)
            .ok_or_else(invalid)?;
        let max_request_bytes = data
            .get("max_request_bytes")
            .and_then(Value::as_u64)
            .filter(|v| *v > 0)
            .ok_or_else(invalid)?;
        Ok(PreparedFeedback::Upload(FeedbackUpload {
            token: token.to_owned(),
            submission_id: draft.submission_id.clone(),
            expires_at,
            max_file_bytes,
            max_request_bytes,
        }))
    }

    pub async fn upload(
        &self,
        draft: &FeedbackDraft,
        upload: &FeedbackUpload,
    ) -> Result<FeedbackReceipt, FeedbackError> {
        draft.validate()?;
        if upload.submission_id != draft.submission_id || upload.expires_at <= Utc::now() {
            return Err(FeedbackError::new(
                FeedbackPhase::Submit,
                FeedbackErrorCode::InvalidUploadToken,
                "提交凭证已失效，请重新准备原反馈",
            ));
        }
        let mut form = multipart::Form::new().text("message", draft.message.clone());
        if !draft.email.is_empty() {
            form = form.text("email", draft.email.clone());
        }
        for (name, attachment) in [("screenshot", &draft.screenshot), ("log", &draft.log)] {
            if let Some(attachment) = attachment {
                if attachment.size() as u64 > upload.max_file_bytes {
                    return Err(FeedbackError::new(
                        FeedbackPhase::Submit,
                        FeedbackErrorCode::RequestTooLarge,
                        "附件超过服务端大小限制",
                    ));
                }
                let part = multipart::Part::bytes(attachment.bytes.clone())
                    .file_name(attachment.filename.clone())
                    .mime_str(attachment.mime_type())
                    .map_err(|_| FeedbackError::validation("附件类型无效"))?;
                form = form.part(name, part);
            }
        }
        // reqwest computes Content-Length from our in-memory parts, including
        // its own generated boundary and all multipart headers.
        let request = self
            .client
            .post(format!("{}/feedback", self.base_url))
            .timeout(Duration::from_secs(60))
            .header("X-Feedback-Token", &upload.token)
            .multipart(form)
            .build()
            .map_err(|_| FeedbackError::invalid_response(FeedbackPhase::Submit))?;
        let length = request
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| FeedbackError::invalid_response(FeedbackPhase::Submit))?;
        if length > upload.max_request_bytes {
            return Err(FeedbackError::new(
                FeedbackPhase::Submit,
                FeedbackErrorCode::RequestTooLarge,
                "反馈超过服务端总请求大小限制",
            ));
        }
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|_| FeedbackError::network(FeedbackPhase::Submit))?;
        let data = read_response(response, FeedbackPhase::Submit).await?;
        let id = data
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| valid_uuid(id))
            .ok_or_else(|| FeedbackError::invalid_response(FeedbackPhase::Submit))?;
        let message = data
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| FeedbackError::invalid_response(FeedbackPhase::Submit))?;
        Ok(FeedbackReceipt {
            id: id.to_owned(),
            message: message.to_owned(),
        })
    }
}

async fn read_response(
    mut response: Response,
    phase: FeedbackPhase,
) -> Result<Value, FeedbackError> {
    let status = response.status();
    let retry_after_seconds = response
        .headers()
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|v| v.parse().ok());
    let mut bytes = Vec::new();
    let mut oversized = false;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| FeedbackError::network(phase))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            oversized = true;
            break;
        }
        bytes.extend_from_slice(&chunk);
    }
    let data = if oversized {
        Value::Null
    } else {
        serde_json::from_slice::<Value>(&bytes)
            .ok()
            .filter(Value::is_object)
            .unwrap_or(Value::Null)
    };
    if !status.is_success() {
        let mut error = FeedbackError::new(
            phase,
            data.get("error")
                .and_then(Value::as_str)
                .map(FeedbackErrorCode::from)
                .unwrap_or(FeedbackErrorCode::HttpError),
            data.get("message")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("反馈请求失败（HTTP {}）", status.as_u16())),
        );
        error.status = status.as_u16();
        error.retry_after_seconds = retry_after_seconds;
        error.site_key = data
            .get("site_key")
            .and_then(Value::as_str)
            .map(str::to_owned);
        error.action = data
            .get("action")
            .and_then(Value::as_str)
            .map(str::to_owned);
        return Err(error);
    }
    if (phase == FeedbackPhase::Prepare && status.as_u16() != 200)
        || (phase == FeedbackPhase::Submit && !matches!(status.as_u16(), 200 | 201))
        || data.is_null()
    {
        return Err(FeedbackError::invalid_response(phase));
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        task::JoinHandle,
    };

    const RECEIPT_ID: &str = "73652d4a-60ea-4435-93fd-b22fa2609b79";

    fn draft(
        screenshot: Option<FeedbackAttachment>,
        log: Option<FeedbackAttachment>,
    ) -> FeedbackDraft {
        FeedbackDraft::new(
            "  无法播放 @a;\n请协助排查。  ",
            " user@example.com ",
            "3.2.0",
            "macOS arm64",
            screenshot,
            log,
        )
        .unwrap()
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut buffer = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(width, height)
            .write_to(&mut buffer, ImageFormat::Png)
            .unwrap();
        buffer.into_inner()
    }

    fn prepared(max_file_bytes: u64, max_request_bytes: u64) -> String {
        serde_json::json!({
            "upload_token": "temporary-test-token",
            "expires_at": "2099-10-21T08:05:00Z",
            "max_file_bytes": max_file_bytes,
            "max_request_bytes": max_request_bytes,
        })
        .to_string()
    }

    struct MockResponse {
        status: u16,
        headers: &'static str,
        body: String,
    }

    impl MockResponse {
        fn json(status: u16, body: String) -> Self {
            Self {
                status,
                headers: "",
                body,
            }
        }
    }

    struct CapturedRequest {
        headers: String,
        body: Vec<u8>,
    }

    async fn mock_server(
        responses: Vec<MockResponse>,
    ) -> (String, JoinHandle<Vec<CapturedRequest>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();
            for response in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0u8; 4096];
                let headers_end = loop {
                    let read = socket.read(&mut buffer).await.unwrap();
                    assert!(read > 0);
                    bytes.extend_from_slice(&buffer[..read]);
                    if let Some(position) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break position + 4;
                    }
                };
                let headers = String::from_utf8(bytes[..headers_end].to_vec()).unwrap();
                let length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .and_then(|value| value.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                while bytes.len() - headers_end < length {
                    let read = socket.read(&mut buffer).await.unwrap();
                    assert!(read > 0);
                    bytes.extend_from_slice(&buffer[..read]);
                }
                requests.push(CapturedRequest {
                    headers,
                    body: bytes[headers_end..].to_vec(),
                });
                let wire = format!("HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                    response.status, response.body.len(), response.headers, response.body);
                socket.write_all(wire.as_bytes()).await.unwrap();
            }
            requests
        });
        (base, handle)
    }

    #[test]
    fn validates_unicode_text_metadata_and_addresses() {
        let make = |message: &str, email: &str| {
            FeedbackDraft::new(message, email, "3.2.0", "macOS", None, None)
        };
        assert!(make(&"😀".repeat(5000), "").is_ok());
        assert!(make(&"😀".repeat(5001), "").is_err());
        for message in [" \n\t", "hello\0", "hello\u{0085}"] {
            assert!(make(message, "").is_err());
            assert!(FeedbackDraft::validate_input(message, "").is_err());
        }
        assert!(FeedbackDraft::validate_input(&"😀".repeat(5000), "").is_ok());
        assert!(FeedbackDraft::validate_input(&"😀".repeat(5001), "").is_err());
        assert!(make("中文\n@user;\t排查", "").is_ok());
        for email in [
            "foo",
            "Name <user@example.com>",
            "user@@example.com",
            "a..b@example.com",
            "u@-host.com",
            "user\n@example.com",
        ] {
            assert!(make("hello", email).is_err(), "accepted {email}");
            assert!(FeedbackDraft::validate_input("hello", email).is_err());
        }
        assert!(make("hello", " first.last+feedback@example.com ").is_ok());
        assert!(FeedbackDraft::new("hello", "", "版".repeat(34), "macOS", None, None).is_err());
        let first = make("hello", "").unwrap();
        let second = make("hello", "").unwrap();
        assert_ne!(first.submission_id(), second.submission_id());
        assert_eq!(
            Uuid::parse_str(first.submission_id())
                .unwrap()
                .get_version_num(),
            4
        );
    }

    #[test]
    fn validates_final_attachment_bytes_and_image_dimensions() {
        let screenshot =
            FeedbackAttachment::screenshot("/private/user/screenshot.PNG", png(2, 3)).unwrap();
        assert_eq!(screenshot.filename(), "screenshot.PNG");
        assert_eq!(screenshot.mime_type(), "image/png");
        assert!(FeedbackAttachment::screenshot("screen.jpg", png(2, 3)).is_err());
        assert!(FeedbackAttachment::screenshot("screen.gif", b"GIF89a".to_vec()).is_err());
        assert!(
            FeedbackAttachment::screenshot("screen.png", b"\x89PNG\r\n\x1a\n".to_vec()).is_err()
        );
        assert!(FeedbackAttachment::screenshot("screen.png", png(16385, 1)).is_err());
        assert!(FeedbackAttachment::log("app.log", "中文\r\n\t日志".as_bytes().to_vec()).is_ok());
        assert!(FeedbackAttachment::log("app.txt", vec![0xff]).is_err());
        assert!(FeedbackAttachment::log("app.log", vec![0]).is_err());
        assert!(FeedbackAttachment::log("app.zip", b"hello".to_vec()).is_err());
        assert!(FeedbackAttachment::log("app.log", Vec::new()).is_err());
        assert!(
            FeedbackAttachment::log("app.log", vec![b'a'; MAX_FEEDBACK_FILE_BYTES + 1]).is_err()
        );
    }

    #[tokio::test]
    async fn sends_anonymous_frozen_multipart_and_accepts_both_receipt_statuses() {
        for attachment_bits in 0..4 {
            let screenshot = (attachment_bits & 1 != 0)
                .then(|| FeedbackAttachment::screenshot("screen.png", png(1, 1)).unwrap());
            let log = (attachment_bits & 2 != 0).then(|| {
                FeedbackAttachment::log("app.log", "固定日志\n".as_bytes().to_vec()).unwrap()
            });
            let draft = draft(screenshot, log);
            let receipt =
                serde_json::json!({"id": RECEIPT_ID, "message": "反馈已收到"}).to_string();
            let (base, server) = mock_server(vec![
                MockResponse {
                    status: 200,
                    headers: "Set-Cookie: account=should-not-be-sent\r\n",
                    body: prepared(5242880, 11534336),
                },
                MockResponse::json(if attachment_bits % 2 == 0 { 201 } else { 200 }, receipt),
            ])
            .await;
            let client = FeedbackClient::with_base_url(&base).unwrap();
            let PreparedFeedback::Upload(upload) = client.prepare(&draft).await.unwrap() else {
                panic!("expected token");
            };
            let received = client.upload(&draft, &upload).await.unwrap();
            assert_eq!(received.id, RECEIPT_ID);
            let captured = server.await.unwrap();
            let metadata: Value = serde_json::from_slice(&captured[0].body).unwrap();
            assert_eq!(metadata["app_id"], "jlivertool");
            assert_eq!(metadata["submission_id"], draft.submission_id());
            assert_eq!(
                metadata["screenshot_size"],
                draft.screenshot().map_or(0, FeedbackAttachment::size)
            );
            assert_eq!(
                metadata["log_size"],
                draft.log().map_or(0, FeedbackAttachment::size)
            );
            assert!(metadata.get("message").is_none());
            assert!(metadata.get("email").is_none());
            assert_eq!(metadata.as_object().unwrap().len(), 6);
            for request in &captured {
                let headers = request.headers.to_ascii_lowercase();
                assert!(!headers.contains("\r\ncookie:"));
                assert!(!headers.contains("\r\nauthorization:"));
                assert!(!headers.contains("\r\nx-forwarded-for:"));
            }
            assert!(captured[0].headers.starts_with("POST /feedback/prepare "));
            assert!(captured[1].headers.starts_with("POST /feedback "));
            assert!(captured[1]
                .headers
                .to_ascii_lowercase()
                .contains("x-feedback-token: temporary-test-token"));
            assert!(captured[1]
                .headers
                .to_ascii_lowercase()
                .contains("content-type: multipart/form-data; boundary="));
            let body = String::from_utf8_lossy(&captured[1].body);
            assert!(body.contains(draft.message()));
            assert!(body.contains("user@example.com"));
            assert_eq!(
                body.matches("name=\"screenshot\"").count(),
                usize::from(draft.screenshot().is_some())
            );
            assert_eq!(
                body.matches("name=\"log\"").count(),
                usize::from(draft.log().is_some())
            );
            assert!(!body.contains("name=\"app_id\""));
            assert!(!body.contains("name=\"submission_id\""));
        }
    }

    #[tokio::test]
    async fn repeated_prepare_recovers_same_receipt_and_keeps_submission_id() {
        let draft = draft(None, None);
        let (base, server) = mock_server(vec![
            MockResponse::json(503, "<html>upstream temporarily failed</html>".to_owned()),
            MockResponse::json(
                200,
                serde_json::json!({"already_submitted":true,"id":RECEIPT_ID}).to_string(),
            ),
        ])
        .await;
        let client = FeedbackClient::with_base_url(&base).unwrap();
        let error = client.prepare(&draft).await.err().unwrap();
        assert_eq!(error.status, 503);
        assert_eq!(error.code, FeedbackErrorCode::HttpError);
        let PreparedFeedback::AlreadySubmitted(receipt) = client.prepare(&draft).await.unwrap()
        else {
            panic!("expected receipt");
        };
        assert_eq!(receipt.id, RECEIPT_ID);
        let captured = server.await.unwrap();
        assert_eq!(captured[0].body, captured[1].body);
    }

    #[tokio::test]
    async fn preserves_error_codes_challenge_fields_and_retry_after() {
        let (base, server) = mock_server(vec![MockResponse {
            status: 403, headers: "Retry-After: 19\r\n",
            body: serde_json::json!({"error":"challenge_required", "message":"请先完成人机验证", "site_key":"public-key", "action":"feedback"}).to_string(),
        }]).await;
        let error = FeedbackClient::with_base_url(&base)
            .unwrap()
            .prepare(&draft(None, None))
            .await
            .err()
            .unwrap();
        assert_eq!(error.code, FeedbackErrorCode::ChallengeRequired);
        assert_eq!(error.status, 403);
        assert_eq!(error.phase, FeedbackPhase::Prepare);
        assert_eq!(error.retry_after_seconds, Some(19));
        assert_eq!(error.site_key.as_deref(), Some("public-key"));
        assert_eq!(error.action.as_deref(), Some("feedback"));
        assert!(error.user_message().contains("19 秒"));
        assert_eq!(server.await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn rejects_malformed_success_instead_of_losing_draft() {
        for body in [
            "not JSON",
            "{}",
            "{\"already_submitted\":true,\"id\":\"\"}",
            "{\"upload_token\":\"token\"}",
        ] {
            let (base, server) = mock_server(vec![MockResponse::json(200, body.to_owned())]).await;
            let error = FeedbackClient::with_base_url(&base)
                .unwrap()
                .prepare(&draft(None, None))
                .await
                .err()
                .unwrap();
            assert_eq!(error.code, FeedbackErrorCode::InvalidResponse);
            assert_eq!(error.status, 0);
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn accepts_only_protocol_success_statuses_even_with_valid_response_bodies() {
        let draft = draft(None, None);
        let (base, server) =
            mock_server(vec![MockResponse::json(201, prepared(5242880, 11534336))]).await;
        let error = FeedbackClient::with_base_url(&base)
            .unwrap()
            .prepare(&draft)
            .await
            .err()
            .unwrap();
        assert_eq!(error.code, FeedbackErrorCode::InvalidResponse);
        assert_eq!(error.phase, FeedbackPhase::Prepare);
        server.await.unwrap();

        let (base, server) = mock_server(vec![
            MockResponse::json(200, prepared(5242880, 11534336)),
            MockResponse::json(
                202,
                serde_json::json!({"id": RECEIPT_ID, "message": "反馈已收到"}).to_string(),
            ),
        ])
        .await;
        let client = FeedbackClient::with_base_url(&base).unwrap();
        let PreparedFeedback::Upload(upload) = client.prepare(&draft).await.unwrap() else {
            panic!("expected token");
        };
        let error = client.upload(&draft, &upload).await.err().unwrap();
        assert_eq!(error.code, FeedbackErrorCode::InvalidResponse);
        assert_eq!(error.phase, FeedbackPhase::Submit);
        assert_eq!(error.status, 0);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn honors_server_file_and_exact_multipart_request_limits() {
        for (max_file, max_request, log) in [
            (
                1,
                11534336,
                Some(FeedbackAttachment::log("app.log", b"text".to_vec()).unwrap()),
            ),
            (5242880, 5, None),
        ] {
            let (base, server) = mock_server(vec![MockResponse::json(
                200,
                prepared(max_file, max_request),
            )])
            .await;
            let draft = draft(None, log);
            let client = FeedbackClient::with_base_url(&base).unwrap();
            let PreparedFeedback::Upload(upload) = client.prepare(&draft).await.unwrap() else {
                panic!("expected token");
            };
            let error = client.upload(&draft, &upload).await.err().unwrap();
            assert_eq!(error.code, FeedbackErrorCode::RequestTooLarge);
            assert_eq!(server.await.unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn network_failure_is_an_unknown_outcome_without_automatic_retry() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            drop(socket);
        });
        let error = FeedbackClient::with_base_url(&base)
            .unwrap()
            .prepare(&draft(None, None))
            .await
            .err()
            .unwrap();
        assert_eq!(error.code, FeedbackErrorCode::NetworkError);
        assert_eq!(error.status, 0);
        server.await.unwrap();
    }
}
