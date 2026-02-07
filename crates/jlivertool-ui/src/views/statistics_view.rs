//! Statistics window view with line chart

use crate::components::{draggable_area, render_window_controls};
use crate::theme::Colors;
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::calendar::Date;
use gpui_component::chart::LineChart;
use gpui_component::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_component::h_flex;
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_component::v_flex;
use gpui_component::Sizable;
use jlivertool_core::database::{Database, TimeBasedStats, TimeSeriesPoint};
use std::sync::Arc;
use std::time::Duration;

/// Statistics period for data aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsPeriod {
    FiveMinutes,
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    EightHours,
    OneDay,
    OneWeek,
    OneMonth,
}

impl StatsPeriod {
    fn label(&self) -> &'static str {
        match self {
            StatsPeriod::FiveMinutes => "5分钟",
            StatsPeriod::TenMinutes => "10分钟",
            StatsPeriod::ThirtyMinutes => "30分钟",
            StatsPeriod::OneHour => "1小时",
            StatsPeriod::TwoHours => "2小时",
            StatsPeriod::FourHours => "4小时",
            StatsPeriod::EightHours => "8小时",
            StatsPeriod::OneDay => "1天",
            StatsPeriod::OneWeek => "1周",
            StatsPeriod::OneMonth => "1月",
        }
    }

    /// Total seconds for this period
    fn total_seconds(&self) -> i64 {
        match self {
            StatsPeriod::FiveMinutes => 5 * 60,
            StatsPeriod::TenMinutes => 10 * 60,
            StatsPeriod::ThirtyMinutes => 30 * 60,
            StatsPeriod::OneHour => 60 * 60,
            StatsPeriod::TwoHours => 2 * 60 * 60,
            StatsPeriod::FourHours => 4 * 60 * 60,
            StatsPeriod::EightHours => 8 * 60 * 60,
            StatsPeriod::OneDay => 24 * 60 * 60,
            StatsPeriod::OneWeek => 7 * 24 * 60 * 60,
            StatsPeriod::OneMonth => 30 * 24 * 60 * 60,
        }
    }

    /// Bucket size in seconds - determines granularity of data points
    fn bucket_seconds(&self) -> i64 {
        match self {
            StatsPeriod::FiveMinutes => 10,           // 10 seconds per bucket (30 points)
            StatsPeriod::TenMinutes => 20,            // 20 seconds per bucket (30 points)
            StatsPeriod::ThirtyMinutes => 60,         // 1 minute per bucket (30 points)
            StatsPeriod::OneHour => 2 * 60,           // 2 minutes per bucket (30 points)
            StatsPeriod::TwoHours => 4 * 60,          // 4 minutes per bucket (30 points)
            StatsPeriod::FourHours => 8 * 60,         // 8 minutes per bucket (30 points)
            StatsPeriod::EightHours => 16 * 60,       // 16 minutes per bucket (30 points)
            StatsPeriod::OneDay => 60 * 60,           // 1 hour per bucket (24 points)
            StatsPeriod::OneWeek => 24 * 60 * 60,     // 1 day per bucket (7 points)
            StatsPeriod::OneMonth => 24 * 60 * 60,    // 1 day per bucket (30 points)
        }
    }

    fn all() -> &'static [StatsPeriod] {
        &[
            StatsPeriod::FiveMinutes,
            StatsPeriod::TenMinutes,
            StatsPeriod::ThirtyMinutes,
            StatsPeriod::OneHour,
            StatsPeriod::TwoHours,
            StatsPeriod::FourHours,
            StatsPeriod::EightHours,
            StatsPeriod::OneDay,
            StatsPeriod::OneWeek,
            StatsPeriod::OneMonth,
        ]
    }
}

/// Which data series to display on the chart
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartSeries {
    Danmu,
    Gift,
    SuperChat,
}

impl ChartSeries {
    fn label(&self) -> &'static str {
        match self {
            ChartSeries::Danmu => "弹幕",
            ChartSeries::Gift => "礼物",
            ChartSeries::SuperChat => "SC",
        }
    }

    fn color(&self) -> Hsla {
        match self {
            ChartSeries::Danmu => Colors::accent(),
            ChartSeries::Gift => Colors::warning(),
            ChartSeries::SuperChat => hsla(200.0 / 360.0, 0.8, 0.6, 1.0),
        }
    }
}

/// Chart data point with formatted time label and all values
#[derive(Clone)]
struct ChartDataPoint {
    time_label: String,
    danmu_count: f64,
    gift_value: f64,
    superchat_value: f64,
}

/// Format timestamp to time label based on period
fn format_time_label(timestamp: i64, period: StatsPeriod) -> String {
    let dt = Local.timestamp_opt(timestamp, 0).unwrap();
    match period {
        StatsPeriod::FiveMinutes | StatsPeriod::TenMinutes => {
            dt.format("%H:%M:%S").to_string()
        }
        StatsPeriod::ThirtyMinutes | StatsPeriod::OneHour | StatsPeriod::TwoHours => {
            dt.format("%H:%M").to_string()
        }
        StatsPeriod::FourHours | StatsPeriod::EightHours | StatsPeriod::OneDay => {
            dt.format("%m/%d %H:%M").to_string()
        }
        StatsPeriod::OneWeek | StatsPeriod::OneMonth => {
            dt.format("%m/%d").to_string()
        }
    }
}

/// Format timestamp to time label based on duration in seconds
fn format_time_label_by_duration(timestamp: i64, duration_seconds: i64) -> String {
    let dt = Local.timestamp_opt(timestamp, 0).unwrap();
    if duration_seconds <= 10 * 60 {
        // <= 10 minutes
        dt.format("%H:%M:%S").to_string()
    } else if duration_seconds <= 2 * 60 * 60 {
        // <= 2 hours
        dt.format("%H:%M").to_string()
    } else if duration_seconds <= 7 * 24 * 60 * 60 {
        // <= 7 days: show date and time to distinguish points on same day
        dt.format("%m/%d %H:%M").to_string()
    } else {
        // > 7 days: show only date
        dt.format("%m/%d").to_string()
    }
}

/// Calculate appropriate bucket size based on time range duration
fn calculate_bucket_seconds(duration_seconds: i64) -> i64 {
    // Aim for approximately 30 data points
    let target_points = 30;
    let bucket = duration_seconds / target_points;

    // Round to nice values
    if bucket <= 10 {
        10
    } else if bucket <= 30 {
        30
    } else if bucket <= 60 {
        60
    } else if bucket <= 2 * 60 {
        2 * 60
    } else if bucket <= 5 * 60 {
        5 * 60
    } else if bucket <= 10 * 60 {
        10 * 60
    } else if bucket <= 30 * 60 {
        30 * 60
    } else if bucket <= 60 * 60 {
        60 * 60
    } else if bucket <= 2 * 60 * 60 {
        2 * 60 * 60
    } else if bucket <= 6 * 60 * 60 {
        6 * 60 * 60
    } else if bucket <= 12 * 60 * 60 {
        12 * 60 * 60
    } else {
        24 * 60 * 60
    }
}

/// Convert NaiveDate with hour and minute to Unix timestamp
fn naive_date_time_to_timestamp(date: NaiveDate, hour: u32, minute: u32) -> i64 {
    Local
        .from_local_datetime(&date.and_hms_opt(hour, minute, 0).unwrap())
        .single()
        .map(|d| d.timestamp())
        .unwrap_or(0)
}

/// Generate hour options (00-23)
fn generate_hour_options() -> Vec<String> {
    (0..24).map(|h| format!("{:02}", h)).collect()
}

/// Generate minute options (00, 15, 30, 45 for quick selection, or 00-59)
fn generate_minute_options() -> Vec<String> {
    (0..60).map(|m| format!("{:02}", m)).collect()
}

/// Statistics view state
pub struct StatisticsView {
    database: Option<Arc<Database>>,
    room_id: Option<u64>,
    period: StatsPeriod,
    stats: TimeBasedStats,
    time_series: Vec<TimeSeriesPoint>,
    opacity: f32,
    // Custom time range mode
    use_custom_range: bool,
    custom_start_picker: Option<Entity<DatePickerState>>,
    custom_end_picker: Option<Entity<DatePickerState>>,
    // Time selectors (hour and minute)
    start_hour_select: Option<Entity<SelectState<Vec<String>>>>,
    start_minute_select: Option<Entity<SelectState<Vec<String>>>>,
    end_hour_select: Option<Entity<SelectState<Vec<String>>>>,
    end_minute_select: Option<Entity<SelectState<Vec<String>>>>,
    // Cached values
    custom_start_date: Option<NaiveDate>,
    custom_end_date: Option<NaiveDate>,
    custom_start_hour: u32,
    custom_start_minute: u32,
    custom_end_hour: u32,
    custom_end_minute: u32,
}

impl StatisticsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Start auto-refresh timer (every 10 seconds)
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                Timer::after(Duration::from_secs(10)).await;
                let result = cx.update(|cx| {
                    this.update(cx, |view, cx| {
                        view.refresh_stats();
                        cx.notify();
                    })
                });
                if result.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            database: None,
            room_id: None,
            period: StatsPeriod::OneHour,
            stats: TimeBasedStats::default(),
            time_series: Vec::new(),
            opacity: 1.0,
            use_custom_range: false,
            custom_start_picker: None,
            custom_end_picker: None,
            start_hour_select: None,
            start_minute_select: None,
            end_hour_select: None,
            end_minute_select: None,
            custom_start_date: None,
            custom_end_date: None,
            custom_start_hour: 0,
            custom_start_minute: 0,
            custom_end_hour: 23,
            custom_end_minute: 59,
        }
    }

    /// Set the database reference
    pub fn set_database(&mut self, db: Arc<Database>) {
        self.database = Some(db);
    }

    /// Set the current room ID
    pub fn set_room_id(&mut self, room_id: Option<u64>, cx: &mut Context<Self>) {
        self.room_id = room_id;
        self.refresh_stats();
        cx.notify();
    }

    /// Set window opacity
    pub fn set_opacity(&mut self, opacity: f32, cx: &mut Context<Self>) {
        self.opacity = opacity;
        cx.notify();
    }

    /// Refresh statistics from database
    pub fn refresh_stats(&mut self) {
        if let (Some(db), Some(room_id)) = (&self.database, self.room_id) {
            if self.use_custom_range {
                // Custom time range mode - calculate timestamps from date and time
                if let (Some(start_date), Some(end_date)) = (self.custom_start_date, self.custom_end_date) {
                    let start = naive_date_time_to_timestamp(
                        start_date,
                        self.custom_start_hour,
                        self.custom_start_minute,
                    );
                    let end = naive_date_time_to_timestamp(
                        end_date,
                        self.custom_end_hour,
                        self.custom_end_minute,
                    );

                    if end > start {
                        // Get summary stats for the custom time range
                        if let Ok(stats) = db.get_time_based_stats_range(room_id, start, end) {
                            self.stats = stats;
                        }

                        // Calculate bucket size based on duration
                        let duration = end - start;
                        let bucket_seconds = calculate_bucket_seconds(duration);

                        // Get time series data
                        if let Ok(series) = db.get_time_series_stats_range(room_id, start, end, bucket_seconds) {
                            self.time_series = series;
                        }
                    }
                }
            } else {
                // Preset period mode
                let since = chrono::Utc::now().timestamp() - self.period.total_seconds();

                // Get summary stats for the displayed time range
                if let Ok(stats) = db.get_time_based_stats(room_id, since) {
                    self.stats = stats;
                }

                // Get time series data with bucket size based on period
                if let Ok(series) = db.get_time_series_stats(room_id, since, self.period.bucket_seconds()) {
                    self.time_series = series;
                }
            }
        }
    }

    /// Render period selector
    fn render_period_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_period = self.period;

        h_flex()
            .gap_1()
            .flex_wrap()
            .children(StatsPeriod::all().iter().map(|&period| {
                let is_selected = period == current_period;
                div()
                    .id(SharedString::from(format!("period-{:?}", period)))
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .when(is_selected, |this| {
                        this.bg(Colors::accent())
                            .text_color(Colors::button_text())
                    })
                    .when(!is_selected, |this| {
                        this.bg(Colors::bg_hover())
                            .text_color(Colors::text_secondary())
                            .hover(|s| s.bg(Colors::bg_secondary()))
                    })
                    .child(period.label())
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.period = period;
                        this.refresh_stats();
                        cx.notify();
                    }))
            }))
    }

    /// Render mode selector (preset vs custom)
    fn render_mode_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let use_custom = self.use_custom_range;

        h_flex()
            .gap_1()
            .child(
                div()
                    .id("mode-preset")
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .when(!use_custom, |this| {
                        this.bg(Colors::accent())
                            .text_color(Colors::button_text())
                    })
                    .when(use_custom, |this| {
                        this.bg(Colors::bg_hover())
                            .text_color(Colors::text_secondary())
                            .hover(|s| s.bg(Colors::bg_secondary()))
                    })
                    .child("预设")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.use_custom_range = false;
                        this.refresh_stats();
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("mode-custom")
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .text_size(px(11.0))
                    .when(use_custom, |this| {
                        this.bg(Colors::accent())
                            .text_color(Colors::button_text())
                    })
                    .when(!use_custom, |this| {
                        this.bg(Colors::bg_hover())
                            .text_color(Colors::text_secondary())
                            .hover(|s| s.bg(Colors::bg_secondary()))
                    })
                    .child("自定义")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.use_custom_range = true;
                        cx.notify();
                    })),
            )
    }

    /// Render custom time range inputs
    fn render_custom_range_inputs(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // Initialize start date picker if not already done
        if self.custom_start_picker.is_none() {
            let now = Local::now();
            let one_day_ago = now - chrono::Duration::days(1);
            let default_date = NaiveDate::from_ymd_opt(
                one_day_ago.year(),
                one_day_ago.month(),
                one_day_ago.day(),
            );

            let picker = cx.new(|cx| {
                let mut state = DatePickerState::new(window, cx).date_format("%Y/%m/%d");
                if let Some(date) = default_date {
                    state.set_date(Date::Single(Some(date)), window, cx);
                    self.custom_start_date = Some(date);
                }
                state
            });

            // Subscribe to date changes
            cx.subscribe_in(&picker, window, |this, _, event: &DatePickerEvent, _window, cx| {
                let DatePickerEvent::Change(date) = event;
                if let Date::Single(Some(d)) = date {
                    this.custom_start_date = Some(*d);
                    cx.notify();
                }
            })
            .detach();

            self.custom_start_picker = Some(picker);
        }

        // Initialize end date picker if not already done
        if self.custom_end_picker.is_none() {
            let now = Local::now();
            let default_date = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day());

            let picker = cx.new(|cx| {
                let mut state = DatePickerState::new(window, cx).date_format("%Y/%m/%d");
                if let Some(date) = default_date {
                    state.set_date(Date::Single(Some(date)), window, cx);
                    self.custom_end_date = Some(date);
                }
                state
            });

            // Subscribe to date changes
            cx.subscribe_in(&picker, window, |this, _, event: &DatePickerEvent, _window, cx| {
                let DatePickerEvent::Change(date) = event;
                if let Date::Single(Some(d)) = date {
                    this.custom_end_date = Some(*d);
                    cx.notify();
                }
            })
            .detach();

            self.custom_end_picker = Some(picker);
        }

        // Initialize hour/minute selectors
        if self.start_hour_select.is_none() {
            let hours = generate_hour_options();
            let select = cx.new(|cx| {
                SelectState::new(hours, Some(gpui_component::IndexPath::new(0)), window, cx)
            });
            cx.subscribe_in(&select, window, |this, _, event: &SelectEvent<Vec<String>>, _window, cx| {
                if let SelectEvent::Confirm(Some(hour_str)) = event {
                    if let Ok(hour) = hour_str.parse::<u32>() {
                        this.custom_start_hour = hour;
                        cx.notify();
                    }
                }
            })
            .detach();
            self.start_hour_select = Some(select);
        }

        if self.start_minute_select.is_none() {
            let minutes = generate_minute_options();
            let select = cx.new(|cx| {
                SelectState::new(minutes, Some(gpui_component::IndexPath::new(0)), window, cx)
            });
            cx.subscribe_in(&select, window, |this, _, event: &SelectEvent<Vec<String>>, _window, cx| {
                if let SelectEvent::Confirm(Some(minute_str)) = event {
                    if let Ok(minute) = minute_str.parse::<u32>() {
                        this.custom_start_minute = minute;
                        cx.notify();
                    }
                }
            })
            .detach();
            self.start_minute_select = Some(select);
        }

        if self.end_hour_select.is_none() {
            let hours = generate_hour_options();
            let select = cx.new(|cx| {
                SelectState::new(hours, Some(gpui_component::IndexPath::new(23)), window, cx)
            });
            cx.subscribe_in(&select, window, |this, _, event: &SelectEvent<Vec<String>>, _window, cx| {
                if let SelectEvent::Confirm(Some(hour_str)) = event {
                    if let Ok(hour) = hour_str.parse::<u32>() {
                        this.custom_end_hour = hour;
                        cx.notify();
                    }
                }
            })
            .detach();
            self.end_hour_select = Some(select);
        }

        if self.end_minute_select.is_none() {
            let minutes = generate_minute_options();
            let select = cx.new(|cx| {
                SelectState::new(minutes, Some(gpui_component::IndexPath::new(59)), window, cx)
            });
            cx.subscribe_in(&select, window, |this, _, event: &SelectEvent<Vec<String>>, _window, cx| {
                if let SelectEvent::Confirm(Some(minute_str)) = event {
                    if let Ok(minute) = minute_str.parse::<u32>() {
                        this.custom_end_minute = minute;
                        cx.notify();
                    }
                }
            })
            .detach();
            self.end_minute_select = Some(select);
        }

        let start_picker = self.custom_start_picker.as_ref().unwrap().clone();
        let end_picker = self.custom_end_picker.as_ref().unwrap().clone();
        let start_hour = self.start_hour_select.as_ref().unwrap().clone();
        let start_minute = self.start_minute_select.as_ref().unwrap().clone();
        let end_hour = self.end_hour_select.as_ref().unwrap().clone();
        let end_minute = self.end_minute_select.as_ref().unwrap().clone();

        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .flex_wrap()
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_muted())
                                    .child("开始:"),
                            )
                            .child(
                                div()
                                    .min_w(px(95.0))
                                    .child(
                                        DatePicker::new(&start_picker)
                                            .xsmall()
                                            .appearance(false),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(50.0))
                                    .child(Select::new(&start_hour).xsmall().appearance(false)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_muted())
                                    .child(":"),
                            )
                            .child(
                                div()
                                    .w(px(50.0))
                                    .child(Select::new(&start_minute).xsmall().appearance(false)),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_muted())
                                    .child("结束:"),
                            )
                            .child(
                                div()
                                    .min_w(px(95.0))
                                    .child(
                                        DatePicker::new(&end_picker)
                                            .xsmall()
                                            .appearance(false),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(50.0))
                                    .child(Select::new(&end_hour).xsmall().appearance(false)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_muted())
                                    .child(":"),
                            )
                            .child(
                                div()
                                    .w(px(50.0))
                                    .child(Select::new(&end_minute).xsmall().appearance(false)),
                            ),
                    )
                    .child(
                        div()
                            .id("apply-range")
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .bg(Colors::accent())
                            .text_color(Colors::button_text())
                            .hover(|s| s.opacity(0.8))
                            .child("应用")
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.refresh_stats();
                                cx.notify();
                            })),
                    ),
            )
    }

    /// Render summary stats
    fn render_summary(&self) -> impl IntoElement {
        h_flex()
            .w_full()
            .gap_2()
            .child(
                v_flex()
                    .flex_1()
                    .p_2()
                    .rounded_md()
                    .bg(Colors::bg_secondary())
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Colors::text_muted())
                            .child("弹幕"),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Colors::accent())
                            .child(self.stats.danmu_count.to_string()),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .p_2()
                    .rounded_md()
                    .bg(Colors::bg_secondary())
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Colors::text_muted())
                            .child("礼物"),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Colors::warning())
                            .child(format!("¥{:.2}", self.stats.gift_value_cny())),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .p_2()
                    .rounded_md()
                    .bg(Colors::bg_secondary())
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(Colors::text_muted())
                            .child("SC"),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(hsla(200.0 / 360.0, 0.8, 0.6, 1.0))
                            .child(format!("¥{:.2}", self.stats.superchat_value_cny())),
                    ),
            )
    }

    /// Render a single chart for a specific series
    fn render_single_chart(
        &self,
        series: ChartSeries,
        chart_data: &[ChartDataPoint],
        tick_margin: usize,
    ) -> impl IntoElement {
        let chart_data_clone = chart_data.to_vec();
        let color = series.color();
        let label = series.label();

        // Calculate max value for Y axis labels (must match LineChart's ScaleLinear calculation)
        // The chart includes 0 in the domain, so scale is always [0, max_value]
        let max_value = chart_data
            .iter()
            .map(|d| match series {
                ChartSeries::Danmu => d.danmu_count,
                ChartSeries::Gift => d.gift_value,
                ChartSeries::SuperChat => d.superchat_value,
            })
            .fold(0.0_f64, f64::max);

        // If max is 0, show 1 as max to avoid division by zero in labels
        let display_max = if max_value == 0.0 { 1.0 } else { max_value };

        // Format Y axis label
        let format_y_label = |value: f64| -> String {
            if value >= 1000.0 {
                format!("{:.1}k", value / 1000.0)
            } else if value >= 10.0 {
                format!("{:.0}", value)
            } else if value >= 1.0 {
                format!("{:.1}", value)
            } else if value > 0.0 {
                format!("{:.2}", value)
            } else {
                "0".to_string()
            }
        };

        // Build chart based on series type
        let chart = match series {
            ChartSeries::Danmu => LineChart::new(chart_data_clone)
                .x(|d: &ChartDataPoint| d.time_label.clone())
                .y(|d: &ChartDataPoint| d.danmu_count)
                .stroke(color)
                .dot()
                .tick_margin(tick_margin),
            ChartSeries::Gift => LineChart::new(chart_data_clone)
                .x(|d: &ChartDataPoint| d.time_label.clone())
                .y(|d: &ChartDataPoint| d.gift_value)
                .stroke(color)
                .dot()
                .tick_margin(tick_margin),
            ChartSeries::SuperChat => LineChart::new(chart_data_clone)
                .x(|d: &ChartDataPoint| d.time_label.clone())
                .y(|d: &ChartDataPoint| d.superchat_value)
                .stroke(color)
                .dot()
                .tick_margin(tick_margin),
        };

        v_flex()
            .w_full()
            .gap_1()
            // Chart label
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded(px(4.0))
                            .bg(color),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(Colors::text_secondary())
                            .child(label),
                    ),
            )
            // Chart with Y axis
            .child(
                h_flex()
                    .w_full()
                    .h(px(120.0))
                    .gap_1()
                    // Y axis labels
                    .child(
                        v_flex()
                            .h_full()
                            .w(px(35.0))
                            .justify_between()
                            .pb(px(18.0)) // Account for X axis labels
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .text_color(Colors::text_muted())
                                    .child(format_y_label(display_max)),
                            )
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .text_color(Colors::text_muted())
                                    .child(format_y_label(display_max * 0.5)),
                            )
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .text_color(Colors::text_muted())
                                    .child("0"),
                            ),
                    )
                    // Chart
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .rounded_md()
                            .bg(Colors::bg_hover())
                            .p_2()
                            .child(chart),
                    ),
            )
    }

    /// Render all charts
    fn render_charts(&self) -> impl IntoElement {
        // Calculate duration for time label formatting
        let duration_seconds = if self.use_custom_range {
            if let (Some(start_date), Some(end_date)) = (self.custom_start_date, self.custom_end_date) {
                let start = naive_date_time_to_timestamp(start_date, self.custom_start_hour, self.custom_start_minute);
                let end = naive_date_time_to_timestamp(end_date, self.custom_end_hour, self.custom_end_minute);
                end - start
            } else {
                self.period.total_seconds()
            }
        } else {
            self.period.total_seconds()
        };

        let period = self.period;
        let use_custom = self.use_custom_range;

        // Convert time series data to chart data points
        let chart_data: Vec<ChartDataPoint> = self
            .time_series
            .iter()
            .map(|point| ChartDataPoint {
                time_label: if use_custom {
                    format_time_label_by_duration(point.timestamp, duration_seconds)
                } else {
                    format_time_label(point.timestamp, period)
                },
                danmu_count: point.danmu_count as f64,
                gift_value: point.gift_value_cny(),
                superchat_value: point.superchat_value_cny(),
            })
            .collect();

        // Calculate tick margin based on data length to avoid overcrowding
        let tick_margin = match chart_data.len() {
            0..=10 => 1,
            11..=20 => 2,
            21..=30 => 3,
            _ => 5,
        };

        v_flex()
            .w_full()
            .gap_2()
            .child(self.render_single_chart(ChartSeries::Danmu, &chart_data, tick_margin))
            .child(self.render_single_chart(ChartSeries::Gift, &chart_data, tick_margin))
            .child(self.render_single_chart(ChartSeries::SuperChat, &chart_data, tick_margin))
    }
}

impl Render for StatisticsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = self.opacity;
        let use_custom_range = self.use_custom_range;

        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        let is_maximized = window.is_maximized();

        // Pre-render custom range inputs if in custom mode
        let custom_range_inputs = if use_custom_range {
            Some(self.render_custom_range_inputs(window, cx))
        } else {
            None
        };

        v_flex()
            .size_full()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .text_color(Colors::text_primary())
            // Header
            .child(
                h_flex()
                    .w_full()
                    .h(px(32.0))
                    .items_center()
                    .bg(Colors::bg_secondary_with_opacity(opacity))
                    .child(
                        draggable_area()
                            .flex_1()
                            .h_full()
                            .pl(left_padding)
                            .pr_2()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(Colors::text_primary())
                                    .child("数据统计"),
                            ),
                    )
                    .child(render_window_controls(is_maximized)),
            )
            // Content
            .child(
                v_flex()
                    .flex_1()
                    .w_full()
                    .p_3()
                    .gap_3()
                    .overflow_hidden()
                    // Mode selector row
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(Colors::text_muted())
                                            .child("统计区间"),
                                    )
                                    .child(self.render_mode_selector(cx)),
                            ),
                    )
                    // Period selector or custom range inputs
                    .child(
                        v_flex()
                            .w_full()
                            .gap_2()
                            .when(!use_custom_range, |this| {
                                this.child(self.render_period_selector(cx))
                            })
                            .when(use_custom_range, |this| {
                                this.children(custom_range_inputs)
                            }),
                    )
                    // Summary stats
                    .child(self.render_summary())
                    // Charts
                    .child(self.render_charts()),
            )
    }
}
