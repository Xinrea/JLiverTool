//! Authenticated received-gift history from the creator center.

use super::*;
use chrono::{Datelike, FixedOffset, NaiveDate, Utc};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct GiftStreamCategory {
    pub category_type: u64,
    pub category_name: String,
    pub items: Vec<GiftStreamType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GiftStreamType {
    pub goods_id: u64,
    pub gift_id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GiftStreamFilter {
    pub goods_ids: Vec<u64>,
    pub gift_id: u64,
}

impl GiftStreamCategory {
    pub fn query_filter(&self) -> GiftStreamFilter {
        let mut goods_ids = Vec::new();
        for item in &self.items {
            if !goods_ids.contains(&item.goods_id) {
                goods_ids.push(item.goods_id);
            }
        }
        GiftStreamFilter {
            goods_ids,
            gift_id: 0,
        }
    }
}

impl GiftStreamType {
    pub fn query_filter(&self) -> GiftStreamFilter {
        GiftStreamFilter {
            goods_ids: vec![self.goods_id],
            gift_id: self.gift_id,
        }
    }
}

#[derive(Deserialize)]
struct GiftStreamTypesData {
    list: Vec<GiftStreamCategory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GiftStreamRecord {
    pub uid: Value,
    pub uname: String,
    pub name: String,
    pub time: String,
    #[serde(default)]
    pub goods_id: Value,
    #[serde(default, alias = "gift_num", alias = "quantity")]
    pub num: Value,
}

impl GiftStreamRecord {
    pub fn is_guard(&self) -> bool {
        matches!(
            self.goods_id
                .as_u64()
                .or_else(|| self.goods_id.as_str()?.parse().ok()),
            Some(5..=7)
        )
    }
    pub fn gift_name(&self) -> &str {
        match self
            .goods_id
            .as_u64()
            .or_else(|| self.goods_id.as_str()?.parse().ok())
        {
            Some(5) => "总督",
            Some(6) => "提督",
            Some(7) => "舰长",
            _ => &self.name,
        }
    }

    pub fn quantity(&self) -> String {
        match &self.num {
            Value::String(value) => value.clone(),
            Value::Number(value) => value.to_string(),
            _ => "—".into(),
        }
    }

    pub fn sender_uid(&self) -> String {
        match &self.uid {
            Value::String(value) => value.clone(),
            Value::Number(value) => value.to_string(),
            _ => "—".into(),
        }
    }
}

/// Creator-center dates follow China Standard Time, regardless of the host timezone.
pub fn gift_stream_today() -> NaiveDate {
    Utc::now()
        .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
        .date_naive()
}

/// Current month through yesterday; on the first day, use the previous full month.
pub fn gift_stream_default_period(today: NaiveDate) -> (NaiveDate, NaiveDate) {
    let end = today.pred_opt().unwrap();
    (end.with_day(1).unwrap(), end)
}

/// The API accepts inclusive ranges within a single calendar month.
pub fn gift_stream_date_ranges(begin: &str, end: &str) -> Result<Vec<(String, String)>> {
    let parse = |text: &str| -> Result<NaiveDate> {
        let date = NaiveDate::parse_from_str(text, "%Y-%m-%d")
            .map_err(|_| anyhow!("请输入有效日期，格式为 YYYY-MM-DD"))?;
        if date.format("%Y-%m-%d").to_string() != text || date.year() < 1 {
            return Err(anyhow!("请输入有效日期，格式为 YYYY-MM-DD"));
        }
        Ok(date)
    };
    let mut begin = parse(begin)?;
    let end = parse(end)?;
    if begin > end {
        return Err(anyhow!("开始日期不能晚于结束日期"));
    }
    if end >= gift_stream_today() {
        return Err(anyhow!("在线记录仅支持查询到昨天，请勿选择当天或未来日期"));
    }
    let mut ranges = Vec::new();
    while begin <= end {
        let next_month = if begin.month() == 12 {
            NaiveDate::from_ymd_opt(begin.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(begin.year(), begin.month() + 1, 1)
        }
        .ok_or_else(|| anyhow!("日期超出支持范围"))?;
        let range_end = end.min(next_month.pred_opt().unwrap());
        ranges.push((
            begin.format("%Y%m%d").to_string(),
            range_end.format("%Y%m%d").to_string(),
        ));
        begin = next_month;
    }
    Ok(ranges)
}

impl BiliApi {
    fn gift_stream_cookies(&self) -> Result<&Cookies> {
        self.cookies
            .as_ref()
            .filter(|c| {
                !c.sessdata.is_empty() && !c.bili_jct.is_empty() && !c.dede_user_id.is_empty()
            })
            .ok_or_else(|| anyhow!("请先登录；登录信息不完整时请退出后重新扫码登录"))
    }

    pub async fn get_gift_types(&self) -> Result<Vec<GiftStreamCategory>> {
        self.gift_types_from(LIVE_API_BASE).await
    }

    async fn gift_types_from(&self, base: &str) -> Result<Vec<GiftStreamCategory>> {
        let cookies = self.gift_stream_cookies()?;
        let response = self.client
            .get(format!("{base}/xlive/revenue/v2/giftStream/getGiftTypes"))
            .timeout(Duration::from_secs(30))
            .header(header::COOKIE, cookies.to_cookie_string())
            .header(header::ORIGIN, "https://link.bilibili.com")
            .header(header::REFERER, "https://link.bilibili.com/p/center/index")
            .header(header::ACCEPT, "application/json, text/plain, */*")
            .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36")
            .send().await?.error_for_status()?.text().await?;
        if response.trim_start().starts_with('<') {
            return Err(anyhow!(
                "礼物类型接口返回了 HTML，请求被风控拦截，请稍后重试"
            ));
        }
        let response: ApiResponse<GiftStreamTypesData> =
            serde_json::from_str(&response).map_err(|_| anyhow!("礼物类型接口返回了无效数据"))?;
        if response.code == -101 {
            return Err(anyhow!("登录状态已过期，请重新登录"));
        }
        Ok(response.into_result()?.list)
    }

    pub async fn get_received_gifts(
        &self,
        begin: &str,
        end: &str,
        filter: &GiftStreamFilter,
    ) -> Result<Vec<GiftStreamRecord>> {
        self.received_gifts_from(LIVE_API_BASE, begin, end, filter)
            .await
    }

    async fn received_gifts_from(
        &self,
        base: &str,
        begin: &str,
        end: &str,
        filter: &GiftStreamFilter,
    ) -> Result<Vec<GiftStreamRecord>> {
        let ranges = gift_stream_date_ranges(begin, end)?;
        let cookies = self.gift_stream_cookies()?;
        let goods_id = filter
            .goods_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut records = Vec::new();
        for (begin_date, end_date) in ranges {
            let mut page = 0;
            let mut total_pages = 1;
            while page < total_pages {
                let data = tokio::time::timeout(Duration::from_secs(180), async {
                    let mut delay = 1;
                    loop {
                        let response = self.client
                            .post(format!("{base}/xlive/revenue/v1/giftStream/getReceivedGiftStream"))
                            .timeout(Duration::from_secs(30))
                            .header(header::COOKIE, cookies.to_cookie_string())
                            .header(header::ORIGIN, "https://link.bilibili.com")
                            .header(header::REFERER, "https://link.bilibili.com/p/center/index")
                            .header(header::ACCEPT, "application/json, text/plain, */*")
                            .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36")
                            .form(&[
                                ("page", page.to_string()), ("gift_id", filter.gift_id.to_string()),
                                ("begin_date", begin_date.clone()), ("end_date", end_date.clone()),
                                ("uname", "".into()), ("goods_id", goods_id.clone()),
                                ("csrf", cookies.bili_jct.clone()), ("csrf_token", cookies.bili_jct.clone()),
                            ])
                            .send().await?.error_for_status()?.text().await?;
                        if response.trim_start().starts_with('<') {
                            return Err(anyhow!("礼物流水接口返回了 HTML，请求被风控拦截，请稍后重试"));
                        }
                        let response: ApiResponse<Value> = serde_json::from_str(&response)
                            .map_err(|_| anyhow!("礼物流水接口返回了无效 JSON"))?;
                        if response.code == -101 {
                            return Err(anyhow!("登录状态已过期，请重新登录"));
                        }
                        let data = response.into_result()?;
                        match data.get("ready").and_then(Value::as_u64) {
                            Some(1) => return Ok(data),
                            Some(0) => {
                                tokio::time::sleep(Duration::from_secs(delay)).await;
                                delay = (delay * 2).min(10);
                            }
                            _ => return Err(anyhow!("礼物流水接口返回了无效数据")),
                        }
                    }
                }).await.map_err(|_| anyhow!("礼物流水数据查询超时，请稍后重试"))??;
                if page == 0 {
                    total_pages = data
                        .get("total_page")
                        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
                        .ok_or_else(|| anyhow!("礼物流水接口返回了无效分页"))?;
                }
                let list = data
                    .get("list")
                    .filter(|v| v.is_array())
                    .ok_or_else(|| anyhow!("礼物流水接口返回了无效列表"))?;
                records.extend(
                    serde_json::from_value::<Vec<GiftStreamRecord>>(list.clone())
                        .map_err(|_| anyhow!("礼物流水接口返回了无效记录"))?,
                );
                page += 1;
            }
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn authenticated_api() -> BiliApi {
        BiliApi {
            client: Client::builder().no_proxy().build().unwrap(),
            cookies: None,
            wbi_signer: Arc::new(WbiSigner::new()),
        }
        .with_cookies(Cookies {
            dede_user_id: "123".into(),
            dede_user_id_ck_md5: String::new(),
            expires: String::new(),
            sessdata: "test-session".into(),
            bili_jct: "test-csrf".into(),
            sid: String::new(),
            gourl: String::new(),
            buvid3: "test-device".into(),
        })
    }

    async fn server(responses: Vec<String>) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let mut requests = Vec::new();
            for response in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                loop {
                    let mut chunk = [0; 4096];
                    let count = socket.read(&mut chunk).await.unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&chunk[..count]);
                    let request = String::from_utf8_lossy(&bytes);
                    if let Some(header_end) = request.find("\r\n\r\n") {
                        let length: usize = request[..header_end]
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length: ")
                                    .map(str::to_owned)
                            })
                            .unwrap_or_else(|| "0".into())
                            .parse()
                            .unwrap();
                        if bytes.len() >= header_end + 4 + length {
                            break;
                        }
                    }
                }
                requests.push(String::from_utf8(bytes).unwrap());
                socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).as_bytes()).await.unwrap();
            }
            requests
        });
        (url, task)
    }

    fn ready(pages: u64) -> String {
        serde_json::json!({"code": 0, "message": "", "data": {
            "ready": 1, "total_page": pages,
            "list": [{"uid": "456", "uname": "test", "name": "purchase", "goods_id": "7", "time": "2024-01-01 12:00:00", "num": "1"}]
        }}).to_string()
    }

    fn gift_types_response() -> String {
        serde_json::json!({"code": 0, "message": "OK", "ttl": 1, "data": {"list": [
            {"category_type": 1, "category_name": "礼物道具", "items": [
                {"goods_id": 15, "gift_id": 31164, "name": "粉丝团灯牌"},
                {"goods_id": 15, "gift_id": 31738, "name": "粉丝团灯牌"}
            ]},
            {"category_type": 0, "category_name": "大航海", "items": [
                {"goods_id": 5, "gift_id": 0, "name": "总督"},
                {"goods_id": 6, "gift_id": 0, "name": "提督"},
                {"goods_id": 7, "gift_id": 0, "name": "舰长"}
            ]},
            {"category_type": 0, "category_name": "醒目留言", "items": [
                {"goods_id": 12, "gift_id": 0, "name": "醒目留言"}
            ]}
        ]}})
        .to_string()
    }

    #[tokio::test]
    async fn fetches_authenticated_types_and_preserves_categories_and_duplicate_names() {
        let (url, task) = server(vec![gift_types_response()]).await;
        let categories = authenticated_api().gift_types_from(&url).await.unwrap();
        let requests = task.await.unwrap();
        assert!(requests[0].starts_with("GET /xlive/revenue/v2/giftStream/getGiftTypes "));
        assert!(requests[0].contains("SESSDATA=test-session"));
        assert!(requests[0].contains("origin: https://link.bilibili.com"));
        assert!(requests[0].contains("referer: https://link.bilibili.com/p/center/index"));
        assert_eq!(categories.len(), 3);
        assert_eq!(categories[1].category_type, categories[2].category_type);
        assert_ne!(categories[1].category_name, categories[2].category_name);
        assert_eq!(categories[0].items.len(), 2);
        assert_eq!(categories[0].items[0].name, categories[0].items[1].name);
        assert_ne!(
            categories[0].items[0].query_filter(),
            categories[0].items[1].query_filter()
        );
        assert_eq!(categories[0].query_filter().goods_ids, vec![15]);
        assert_eq!(categories[1].query_filter().goods_ids, vec![5, 6, 7]);
        assert_eq!(categories[2].query_filter().goods_ids, vec![12]);
    }

    #[tokio::test]
    async fn posts_selected_gift_and_superchat_ids_instead_of_filtering_by_name() {
        let (url, task) = server(vec![ready(1), ready(1)]).await;
        for filter in [
            GiftStreamFilter {
                goods_ids: vec![15],
                gift_id: 31738,
            },
            GiftStreamFilter {
                goods_ids: vec![12],
                gift_id: 0,
            },
        ] {
            authenticated_api()
                .received_gifts_from(&url, "2024-10-10", "2024-10-10", &filter)
                .await
                .unwrap();
        }
        let requests = task.await.unwrap();
        assert!(requests[0].contains("gift_id=31738&"));
        assert!(requests[0].contains("goods_id=15&"));
        assert!(requests[1].contains("gift_id=0&"));
        assert!(requests[1].contains("goods_id=12&"));
    }

    #[tokio::test]
    async fn gift_types_reject_expired_cookies_html_and_malformed_data() {
        for (response, message) in [
            (r#"{"code":-101,"message":"expired"}"#, "登录状态已过期"),
            ("<html>blocked</html>", "HTML"),
            (
                r#"{"code":0,"message":"OK","data":{"list":null}}"#,
                "无效数据",
            ),
            (
                r#"{"code":0,"message":"OK","data":{"list":[{"category_type":0,"category_name":"大航海"}]}}"#,
                "无效数据",
            ),
        ] {
            let (url, task) = server(vec![response.into()]).await;
            let error = authenticated_api().gift_types_from(&url).await.unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
            task.await.unwrap();
        }
    }

    #[test]
    fn splits_inclusive_calendar_months_and_validates_dates() {
        assert_eq!(
            gift_stream_date_ranges("2023-12-31", "2024-03-01").unwrap(),
            vec![
                ("20231231".into(), "20231231".into()),
                ("20240101".into(), "20240131".into()),
                ("20240201".into(), "20240229".into()),
                ("20240301".into(), "20240301".into())
            ]
        );
        assert_eq!(
            gift_stream_date_ranges("2024-10-10", "2024-10-10")
                .unwrap()
                .len(),
            1
        );
        for (begin, end) in [
            ("2026-02-30", "2026-03-01"),
            ("2026-10-11", "2024-10-10"),
            ("20241010", "20241010"),
            ("2026-1-01", "2026-01-01"),
        ] {
            assert!(gift_stream_date_ranges(begin, end).is_err());
        }
    }

    #[test]
    fn default_period_excludes_today_at_month_and_year_boundaries() {
        for (today, begin, end) in [
            ("2026-10-10", "2026-10-01", "2026-10-09"),
            ("2026-10-01", "2026-09-01", "2026-09-30"),
            ("2026-01-01", "2025-12-01", "2025-12-31"),
            ("2024-03-01", "2024-02-01", "2024-02-29"),
        ] {
            let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap();
            let period = gift_stream_default_period(today);
            assert_eq!(
                (period.0.to_string(), period.1.to_string()),
                (begin.into(), end.into())
            );
            assert!(period.0 <= period.1 && period.1 < today);
        }
    }

    #[test]
    fn rejects_today_and_future_dates_but_accepts_yesterday() {
        let today = gift_stream_today();
        let yesterday = today.pred_opt().unwrap().to_string();
        assert!(gift_stream_date_ranges(&yesterday, &yesterday).is_ok());
        for end in [today, today.succ_opt().unwrap()] {
            let error = gift_stream_date_ranges(&yesterday, &end.to_string()).unwrap_err();
            assert!(error.to_string().contains("昨天"));
        }
    }

    #[tokio::test]
    async fn rejects_missing_and_incomplete_cookies_before_requesting() {
        let mut api = authenticated_api();
        api.set_cookies(None);
        assert!(api
            .gift_types_from("http://127.0.0.1:1")
            .await
            .unwrap_err()
            .to_string()
            .contains("请先登录"));
        assert!(api
            .received_gifts_from(
                "http://127.0.0.1:1",
                "2024-10-10",
                "2024-10-10",
                &GiftStreamFilter::default()
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("请先登录"));
        let mut api = authenticated_api();
        api.cookies.as_mut().unwrap().bili_jct.clear();
        assert!(api
            .received_gifts_from(
                "http://127.0.0.1:1",
                "2024-10-10",
                "2024-10-10",
                &GiftStreamFilter::default()
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("登录信息不完整"));
    }

    #[tokio::test]
    async fn posts_auth_headers_and_fetches_all_zero_based_pages_and_months() {
        let (url, task) = server(vec![ready(2), ready(0), ready(1)]).await;
        let records = authenticated_api()
            .received_gifts_from(
                &url,
                "2024-01-31",
                "2024-02-01",
                &GiftStreamFilter {
                    goods_ids: vec![5, 6, 7],
                    gift_id: 0,
                },
            )
            .await
            .unwrap();
        assert_eq!(records.len(), 3); // Repeated purchases must be preserved.
        assert_eq!(records[0].gift_name(), "舰长");
        assert_eq!(records[0].quantity(), "1");
        let requests = task.await.unwrap();
        for request in &requests {
            assert!(request.starts_with("POST /xlive/revenue/v1/giftStream/getReceivedGiftStream "));
            assert!(request.contains("SESSDATA=test-session"));
            assert!(request.contains("bili_jct=test-csrf"));
            assert!(request.contains("origin: https://link.bilibili.com"));
            assert!(request.contains("referer: https://link.bilibili.com/p/center/index"));
            let body = request.split("\r\n\r\n").nth(1).unwrap();
            let form: HashMap<_, _> = Url::parse(&format!("http://test/?{body}"))
                .unwrap()
                .query_pairs()
                .into_owned()
                .collect();
            assert_eq!(form["goods_id"], "5,6,7");
            assert_eq!(form["csrf"], "test-csrf");
            assert_eq!(form["csrf_token"], "test-csrf");
        }
        assert!(requests[0].contains("page=0&gift_id=0&begin_date=20240131&end_date=20240131"));
        assert!(requests[1].contains("page=1&gift_id=0&begin_date=20240131&end_date=20240131"));
        assert!(requests[2].contains("page=0&gift_id=0&begin_date=20240201&end_date=20240201"));
    }

    #[tokio::test]
    async fn polls_pending_pages_without_advancing_and_does_not_filter_all_gifts() {
        let (url, task) = server(vec![
            r#"{"code":0,"message":"","data":{"ready":0}}"#.into(),
            ready(1),
        ])
        .await;
        let records = authenticated_api()
            .received_gifts_from(
                &url,
                "2024-10-10",
                "2024-10-10",
                &GiftStreamFilter::default(),
            )
            .await
            .unwrap();
        assert_eq!(records.len(), 1);
        let requests = task.await.unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests
            .iter()
            .all(|r| r.contains("page=0&") && r.contains("goods_id=&")));
    }

    #[tokio::test]
    async fn handles_empty_results_and_rejects_login_errors_html_and_invalid_metadata() {
        let (url, task) = server(vec![
            r#"{"code":0,"message":"","data":{"ready":1,"total_page":0,"list":[]}}"#.into(),
        ])
        .await;
        assert!(authenticated_api()
            .received_gifts_from(
                &url,
                "2024-10-10",
                "2024-10-10",
                &GiftStreamFilter::default()
            )
            .await
            .unwrap()
            .is_empty());
        task.await.unwrap();
        for (response, message) in [
            (r#"{"code":-101,"message":"expired"}"#, "登录状态已过期"),
            ("<html>blocked</html>", "HTML"),
            ("{invalid", "JSON"),
            (
                r#"{"code":0,"message":"","data":{"ready":1,"list":[]}}"#,
                "无效分页",
            ),
            (
                r#"{"code":0,"message":"","data":{"ready":1,"total_page":0,"list":null}}"#,
                "无效列表",
            ),
            (r#"{"code":0,"message":"","data":{}}"#, "无效数据"),
        ] {
            let (url, task) = server(vec![response.into()]).await;
            let error = authenticated_api()
                .received_gifts_from(
                    &url,
                    "2024-10-10",
                    "2024-10-10",
                    &GiftStreamFilter::default(),
                )
                .await
                .unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
            task.await.unwrap();
        }
    }
}
