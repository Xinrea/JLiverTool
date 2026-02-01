//! Statistics window view with line chart

use crate::theme::Colors;
use chrono::{Local, TimeZone};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::chart::LineChart;
use gpui_component::h_flex;
use gpui_component::v_flex;
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

/// Statistics view state
pub struct StatisticsView {
    database: Option<Arc<Database>>,
    room_id: Option<u64>,
    period: StatsPeriod,
    stats: TimeBasedStats,
    time_series: Vec<TimeSeriesPoint>,
    opacity: f32,
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
        let period = self.period;

        // Convert time series data to chart data points
        let chart_data: Vec<ChartDataPoint> = self
            .time_series
            .iter()
            .map(|point| ChartDataPoint {
                time_label: format_time_label(point.timestamp, period),
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let opacity = self.opacity;

        #[cfg(target_os = "macos")]
        let left_padding = px(78.0);
        #[cfg(not(target_os = "macos"))]
        let left_padding = px(12.0);

        v_flex()
            .size_full()
            .bg(Colors::bg_primary_with_opacity(opacity))
            .text_color(Colors::text_primary())
            // Header
            .child(
                h_flex()
                    .w_full()
                    .h(px(32.0))
                    .pl(left_padding)
                    .pr_2()
                    .items_center()
                    .bg(Colors::bg_secondary_with_opacity(opacity))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(Colors::text_primary())
                            .child("数据统计"),
                    ),
            )
            // Content
            .child(
                v_flex()
                    .flex_1()
                    .w_full()
                    .p_3()
                    .gap_3()
                    .overflow_hidden()
                    // Period selector
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(Colors::text_muted())
                                    .child("统计区间 (最近)"),
                            )
                            .child(self.render_period_selector(cx)),
                    )
                    // Summary stats
                    .child(self.render_summary())
                    // Charts
                    .child(self.render_charts()),
            )
    }
}
