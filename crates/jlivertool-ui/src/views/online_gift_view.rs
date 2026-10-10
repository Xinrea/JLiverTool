//! Cookie-authenticated gift history, separate from the local live-event log.

use crate::app::UiCommand;
use crate::theme::Colors;
use crate::views::window_wrapper::WindowFrameContent;
use chrono::{Datelike, Duration, NaiveDate};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::calendar::{Date, Matcher};
use gpui_component::date_picker::{DatePicker, DatePickerEvent, DatePickerState, DateRangePreset};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::Scrollbar;
use gpui_component::{h_flex, v_flex, Disableable, Icon, IconName, Sizable, Size};
use jlivertool_core::bilibili::api::{
    gift_stream_date_ranges, gift_stream_default_period, gift_stream_today, GiftStreamCategory,
    GiftStreamFilter, GiftStreamRecord, GiftStreamType,
};
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};

static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

pub struct OnlineGiftView {
    command_tx: mpsc::Sender<UiCommand>,
    account: Option<(u64, String)>,
    period: Date,
    date_picker: Option<Entity<DatePickerState>>,
    gift_types: Vec<GiftStreamCategory>,
    types_pending: Option<u64>,
    types_loaded: bool,
    types_error: Option<String>,
    controls_window: Option<WindowId>,
    subscriptions: Vec<Subscription>,
    selected_gift: Option<GiftStreamType>,
    selected_category: Option<usize>,
    queried_period: Option<(NaiveDate, NaiveDate)>,
    has_queried: bool,
    error: bool,
    pending: Option<u64>,
    records: Rc<Vec<GiftStreamRecord>>,
    status: String,
    scroll: UniformListScrollHandle,
}

impl OnlineGiftView {
    pub fn new(command_tx: mpsc::Sender<UiCommand>) -> Self {
        Self {
            command_tx,
            account: None,
            period: gift_stream_default_period(gift_stream_today()).into(),
            date_picker: None,
            gift_types: Vec::new(),
            types_pending: None,
            types_loaded: false,
            types_error: None,
            controls_window: None,
            subscriptions: Vec::new(),
            selected_gift: None,
            selected_category: None,
            queried_period: None,
            has_queried: false,
            error: false,
            pending: None,
            records: Rc::new(Vec::new()),
            status: "选择日期范围后点击查询".into(),
            scroll: UniformListScrollHandle::new(),
        }
    }

    pub fn set_account(&mut self, account: Option<(u64, String)>, cx: &mut Context<Self>) {
        if self.account != account {
            self.account = account;
            self.pending = None;
            self.records = Rc::new(Vec::new());
            self.selected_gift = None;
            self.selected_category = None;
            self.queried_period = None;
            self.has_queried = false;
            self.error = false;
            self.gift_types.clear();
            self.types_pending = None;
            self.types_loaded = false;
            self.types_error = None;
            self.status = if self.account.is_some() {
                "选择日期范围后点击查询"
            } else {
                "请先在设置中登录后使用在线记录查询"
            }
            .into();
            cx.notify();
        }
    }

    pub fn receive(
        &mut self,
        request_id: u64,
        account_uid: u64,
        result: Result<Vec<GiftStreamRecord>, String>,
        cx: &mut Context<Self>,
    ) {
        if self.pending != Some(request_id)
            || self.account.as_ref().map(|a| a.0) != Some(account_uid)
        {
            return;
        }
        self.pending = None;
        match result {
            Ok(records) => {
                self.status = if records.is_empty() {
                    "该日期范围内没有礼物记录".into()
                } else {
                    format!("查询完成，共 {} 条记录", records.len())
                };
                self.records = Rc::new(records);
                self.has_queried = true;
                self.scroll.scroll_to_item(0, ScrollStrategy::Top);
            }
            Err(error) => {
                self.status = format!("查询失败：{error}");
                self.error = true;
            }
        }
        cx.notify();
    }

    fn query(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() || self.types_pending.is_some() {
            return;
        }
        let Some(account_uid) = self.account.as_ref().map(|account| account.0) else {
            return;
        };
        let Date::Range(Some(begin), Some(end)) = self.period else {
            self.status = "请选择完整的起止日期".into();
            self.error = true;
            cx.notify();
            return;
        };
        if let Err(error) = gift_stream_date_ranges(&begin.to_string(), &end.to_string()) {
            self.status = error.to_string();
            self.error = true;
            cx.notify();
            return;
        }
        let Some(filter) = self.selected_filter() else {
            self.status = "该分类暂无可查询的类型，请选择其他分类".into();
            self.error = true;
            cx.notify();
            return;
        };
        let request_id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        self.clear_results();
        if self
            .command_tx
            .send(UiCommand::QueryOnlineGifts {
                request_id,
                account_uid,
                begin: begin.to_string(),
                end: end.to_string(),
                filter,
            })
            .is_ok()
        {
            self.pending = Some(request_id);
            self.queried_period = Some((begin, end));
            self.status = "正在查询，服务器准备数据可能需要几分钟…".into();
        } else {
            self.status = "查询失败：服务不可用".into();
            self.error = true;
        }
        cx.notify();
    }
}

impl OnlineGiftView {
    fn clear_results(&mut self) {
        self.records = Rc::new(Vec::new());
        self.queried_period = None;
        self.has_queried = false;
        self.error = false;
        self.status = "选择条件后点击查询".into();
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
    }

    fn selected_filter(&self) -> Option<GiftStreamFilter> {
        if let Some(gift) = &self.selected_gift {
            return Some(gift.query_filter());
        }
        match self.selected_category {
            Some(index) => self
                .gift_types
                .get(index)
                .filter(|category| !category.items.is_empty())
                .map(GiftStreamCategory::query_filter),
            None => Some(GiftStreamFilter::default()),
        }
    }

    fn load_types(&mut self, cx: &mut Context<Self>) {
        if self.types_pending.is_some() || self.types_loaded {
            return;
        }
        let Some(account_uid) = self.account.as_ref().map(|account| account.0) else {
            return;
        };
        let request_id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        self.types_error = None;
        if self
            .command_tx
            .send(UiCommand::FetchOnlineGiftTypes {
                request_id,
                account_uid,
            })
            .is_ok()
        {
            self.types_pending = Some(request_id);
        } else {
            self.types_error = Some("服务不可用".into());
        }
        cx.notify();
    }

    pub fn receive_types(
        &mut self,
        request_id: u64,
        account_uid: u64,
        result: Result<Vec<GiftStreamCategory>, String>,
        cx: &mut Context<Self>,
    ) {
        if self.types_pending != Some(request_id)
            || self.account.as_ref().map(|account| account.0) != Some(account_uid)
        {
            return;
        }
        self.types_pending = None;
        match result {
            Ok(categories) => {
                self.gift_types = categories;
                self.types_loaded = true;
                self.types_error = None;
            }
            Err(error) => self.types_error = Some(error),
        }
        cx.notify();
    }

    fn gift_type_label(&self, gift: &GiftStreamType) -> String {
        let duplicates = self
            .gift_types
            .iter()
            .flat_map(|category| &category.items)
            .filter(|item| item.name == gift.name)
            .count();
        if duplicates > 1 {
            format!("{}（ID {}）", gift.name, gift.gift_id)
        } else {
            gift.name.clone()
        }
    }

    fn initialize_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let window_id = window.window_handle().window_id();
        if self.controls_window != Some(window_id) {
            self.subscriptions.clear();
            let picker = cx.new(|cx| {
                let mut picker = DatePickerState::range(window, cx)
                    .date_format("%Y-%m-%d")
                    .disabled_matcher(Matcher::custom(|date| *date >= gift_stream_today()));
                picker.set_date(self.period, window, cx);
                picker
            });
            self.subscriptions
                .push(cx.subscribe(&picker, |this, _, event, cx| {
                    let DatePickerEvent::Change(date) = event;
                    this.period = date.date();
                    this.clear_results();
                    cx.notify();
                }));
            self.date_picker = Some(picker);
            self.controls_window = Some(window_id);
        }
        if !self.types_loaded && self.types_error.is_none() {
            self.load_types(cx);
        }
    }

    fn render_category_filter(&self, enabled: bool, cx: &Context<Self>) -> impl IntoElement {
        let entity = cx.entity().downgrade();
        let selected = self.selected_category;
        let categories = self
            .gift_types
            .iter()
            .enumerate()
            .map(|(index, category)| (Some(index), category.category_name.clone()))
            .collect::<Vec<_>>();
        Button::new("online-category")
            .outline()
            .w_full()
            .justify_between()
            .dropdown_caret(true)
            .label(
                selected
                    .and_then(|index| self.gift_types.get(index))
                    .map(|category| category.category_name.clone())
                    .unwrap_or_else(|| "全部分类".into()),
            )
            .disabled(!enabled)
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu.min_w(px(140.0));
                for (value, label) in std::iter::once((None, "全部分类".to_string()))
                    .chain(categories.iter().cloned())
                {
                    let entity = entity.clone();
                    menu = menu.item(
                        PopupMenuItem::new(label)
                            .checked(selected == value)
                            .on_click(move |_, _, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.selected_category = value;
                                    this.selected_gift = None;
                                    this.clear_results();
                                    cx.notify();
                                });
                            }),
                    );
                }
                menu
            })
    }

    fn render_type_filter(&self, enabled: bool, cx: &Context<Self>) -> impl IntoElement {
        let entity = cx.entity().downgrade();
        let selected = self.selected_gift.clone();
        let type_label = selected
            .as_ref()
            .map(|gift| self.gift_type_label(gift))
            .unwrap_or_else(|| "全部类型".into());
        let button_label = selected
            .as_ref()
            .map(|gift| {
                if type_label != gift.name {
                    format!("{} · {}", gift.name, gift.gift_id)
                } else {
                    gift.name.clone()
                }
            })
            .unwrap_or_else(|| "全部类型".into());
        let types = self
            .gift_types
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                self.selected_category
                    .is_none_or(|selected| selected == *index)
            })
            .flat_map(|(_, category)| &category.items)
            .map(|gift| (Some(gift.clone()), self.gift_type_label(gift)))
            .collect::<Vec<_>>();
        Button::new("online-gift-type")
            .outline()
            .w_full()
            .justify_between()
            .dropdown_caret(true)
            .child(div().max_w(px(156.0)).truncate().child(button_label))
            .tooltip(type_label)
            .disabled(!enabled || types.is_empty())
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu.min_w(px(190.0)).scrollable(true).max_h(px(280.0));
                for (gift, label) in
                    std::iter::once((None, "全部类型".to_string())).chain(types.iter().cloned())
                {
                    let entity = entity.clone();
                    menu = menu.item(
                        PopupMenuItem::new(label)
                            .checked(selected == gift)
                            .on_click(move |_, _, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.selected_gift = gift.clone();
                                    this.clear_results();
                                    cx.notify();
                                });
                            }),
                    );
                }
                menu
            })
    }

    fn render_empty(&self) -> impl IntoElement {
        let (icon, title, hint) = if self.account.is_none() {
            (
                IconName::CircleUser,
                "登录后查询在线记录",
                "请先前往设置登录，仅能查询该账号收到的礼物记录",
            )
        } else if self.pending.is_some() {
            (
                IconName::Loader,
                "正在查询记录",
                "服务器正在准备数据，较长的日期区间可能需要几分钟",
            )
        } else if self.error {
            (
                IconName::TriangleAlert,
                "暂时无法完成查询",
                "请根据上方提示调整条件或稍后重试",
            )
        } else if !self.has_queried {
            (
                IconName::Calendar,
                "选择时间，查询收到的礼物",
                "点击时间框选择日期区间，或使用日历中的快捷选项",
            )
        } else {
            (
                IconName::Inbox,
                "这段时间没有收到礼物",
                "可以选择其他日期区间再查询",
            )
        };
        v_flex()
            .flex_1()
            .w_full()
            .items_center()
            .justify_center()
            .gap_3()
            .child(
                div()
                    .size(px(64.0))
                    .rounded(px(16.0))
                    .bg(Colors::accent().opacity(0.10))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).size(px(28.0)).text_color(Colors::accent())),
            )
            .child(
                div()
                    .text_size(px(15.0))
                    .font_weight(FontWeight::MEDIUM)
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(Colors::text_secondary())
                    .child(hint),
            )
    }
}

fn date_presets(today: NaiveDate) -> Vec<DateRangePreset> {
    let yesterday = today.pred_opt().unwrap();
    let last_month_end = today.with_day(1).unwrap().pred_opt().unwrap();
    let mut presets = vec![
        DateRangePreset::range("昨天", yesterday, yesterday),
        DateRangePreset::range("近 7 天", today - Duration::days(7), yesterday),
        DateRangePreset::range("近 30 天", today - Duration::days(30), yesterday),
        DateRangePreset::range("上月", last_month_end.with_day(1).unwrap(), last_month_end),
    ];
    if today.day() > 1 {
        presets.push(DateRangePreset::range(
            "本月至昨天",
            today.with_day(1).unwrap(),
            yesterday,
        ));
    }
    presets
}

impl Render for OnlineGiftView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.initialize_controls(window, cx);
        let enabled =
            self.account.is_some() && self.pending.is_none() && self.types_pending.is_none();
        let filters_enabled = enabled && self.types_loaded;
        let records = self.records.clone();
        let account_name = self
            .account
            .as_ref()
            .map(|(_, name)| name.as_str())
            .unwrap_or("尚未登录");
        let period_label = self
            .queried_period
            .map(|(start, end)| format!("{start} 至 {end}"))
            .unwrap_or_default();
        v_flex().size_full().bg(Colors::bg_primary()).text_color(Colors::text_primary())
            .child(v_flex().p_5().gap_4().flex_shrink_0()
                .child(h_flex().justify_between().items_center().gap_4()
                    .child(v_flex().gap_1()
                        .child(div().text_size(px(20.0)).font_weight(FontWeight::SEMIBOLD).child("在线记录查询"))
                        .child(div().text_size(px(12.0)).text_color(Colors::text_secondary()).child("仅可查询当前登录账号收到的礼物记录")))
                    .child(h_flex().gap_2().px_3().py_2().rounded(px(8.0)).bg(Colors::bg_secondary()).max_w(px(220.0))
                        .child(Icon::new(IconName::CircleUser).size(px(18.0)).text_color(Colors::accent()))
                        .child(v_flex().min_w_0()
                            .child(div().text_size(px(12.0)).truncate().child(account_name.to_string()))
                            .when_some(self.account.as_ref(), |this, (uid, _)| this.child(div().text_size(px(10.0)).text_color(Colors::text_muted()).child(format!("UID {uid}")))))))
                .child(v_flex().p_4().gap_3().rounded(px(10.0)).border_1().border_color(Colors::border()).bg(Colors::bg_secondary())
                    .child(h_flex().gap_3().items_end()
                        .child(v_flex().w(px(140.0)).flex_shrink_0().gap_2()
                            .child(div().text_size(px(12.0)).text_color(Colors::text_secondary()).child("分类"))
                            .child(self.render_category_filter(filters_enabled, cx)))
                        .child(v_flex().w(px(210.0)).flex_shrink_0().gap_2()
                            .child(div().text_size(px(12.0)).text_color(Colors::text_secondary()).child("类型"))
                            .child(self.render_type_filter(filters_enabled, cx)))
                        .child(v_flex().flex_1().min_w(px(305.0)).gap_2()
                            .child(div().text_size(px(12.0)).text_color(Colors::text_secondary()).child("时间区间"))
                            .child(DatePicker::new(self.date_picker.as_ref().unwrap()).with_size(Size::Medium).number_of_months(2).presets(date_presets(gift_stream_today())).disabled(!enabled))))
                    .child(h_flex().justify_between().items_center().gap_3()
                        .child(div().text_size(px(11.0)).text_color(Colors::text_muted())
                            .child(if self.types_pending.is_some() { "正在加载分类和礼物类型…" } else { "在线记录截至昨天，暂不支持当天数据" }))
                        .child(Button::new("query-online-gifts").primary().icon(IconName::Search)
                            .label(if self.pending.is_some() { "查询中" } else { "查询记录" }).with_size(Size::Medium).loading(self.pending.is_some()).disabled(!enabled)
                            .on_click(cx.listener(|this, _, _, cx| this.query(cx)))))
                    .when_some(self.types_error.clone(), |this, error| this.child(h_flex().gap_2().px_3().py_2().rounded(px(6.0)).bg(Colors::error().opacity(0.10))
                        .child(div().flex_1().text_size(px(12.0)).text_color(Colors::error()).child(format!("分类和类型加载失败：{error}")))
                        .child(Button::new("retry-online-gift-types").outline().small().label("重试")
                            .disabled(self.account.is_none() || self.pending.is_some())
                            .on_click(cx.listener(|this, _, _, cx| this.load_types(cx))))))
                    .when(self.error, |this| this.child(h_flex().gap_2().px_3().py_2().rounded(px(6.0)).bg(Colors::error().opacity(0.10)).text_color(Colors::error())
                        .child(Icon::new(IconName::TriangleAlert).size(px(16.0)))
                        .child(div().text_size(px(12.0)).child(self.status.clone()))))))
            .child(v_flex().mx_5().mb_5().flex_1().min_h_0().overflow_hidden().rounded(px(10.0)).border_1().border_color(Colors::border())
                .child(h_flex().px_4().h(px(46.0)).flex_shrink_0().justify_between().items_center().gap_3().bg(Colors::bg_secondary())
                    .child(h_flex().gap_2().items_center()
                        .child(div().text_size(px(13.0)).font_weight(FontWeight::SEMIBOLD).child("查询结果"))
                        .child(div().px_2().py(px(2.0)).rounded(px(4.0)).bg(Colors::accent().opacity(0.12)).text_color(Colors::accent()).text_size(px(11.0)).child(format!("{} 条", records.len()))))
                    .child(div().text_size(px(11.0)).text_color(Colors::text_muted()).child(period_label)))
                .when(records.is_empty(), |this| this.child(self.render_empty()))
                .when(!records.is_empty(), |this| this
                    .child(h_flex().px_4().h(px(34.0)).flex_shrink_0().gap_4().items_center().border_b_1().border_color(Colors::border()).text_size(px(11.0)).text_color(Colors::text_muted())
                        .child(div().w(px(165.0)).flex_shrink_0().child("时间"))
                        .child(div().flex_1().child("赠送用户"))
                        .child(div().w(px(160.0)).flex_shrink_0().child("类型"))
                        .child(div().w(px(60.0)).flex_shrink_0().text_right().child("数量")))
                    .child(h_flex().id("online-gift-results").flex_1().min_h_0().overflow_hidden()
                        .child(uniform_list("online-gift-list", records.len(), move |range, _, _| {
                            range.map(|ix| {
                                let record = &records[ix];
                                h_flex().id(("online-gift-row", ix)).h(px(58.0)).w_full().px_4().gap_4().items_center()
                                    .border_b_1().border_color(Colors::border().opacity(0.6)).text_size(px(12.0))
                                    .when(ix % 2 == 1, |this| this.bg(Colors::bg_secondary().opacity(0.35)))
                                    .hover(|style| style.bg(Colors::bg_hover()))
                                    .child(div().w(px(165.0)).flex_shrink_0().text_color(Colors::text_secondary()).child(record.time.clone()))
                                    .child(v_flex().flex_1().min_w_0().gap_1().overflow_hidden()
                                        .child(div().truncate().child(record.uname.clone()))
                                        .child(div().text_size(px(10.0)).text_color(Colors::text_muted()).child(format!("UID {}", record.sender_uid()))))
                                    .child(div().w(px(160.0)).flex_shrink_0().truncate().text_color(if record.is_guard() { Colors::accent() } else { Colors::text_primary() }).child(record.gift_name().to_string()))
                                    .child(div().w(px(60.0)).flex_shrink_0().text_right().child(record.quantity()))
                            }).collect()
                        }).flex_1().h_full().track_scroll(&self.scroll))
                        .child(Scrollbar::vertical(&self.scroll)))))
    }
}

impl WindowFrameContent for OnlineGiftView {
    fn window_opacity(&self) -> f32 {
        1.0
    }
}
