use super::DashboardDanmuView;
use super::panel::DashboardPanel;
use crate::app::UiCommand;
use crate::theme::Colors;
use crate::views::{AudienceView, GiftView, StatisticsView, SuperChatView};
use gpui::*;
use gpui_component::dock::{DockArea, DockAreaState, DockEvent, DockItem, register_panel};
use gpui_component::v_flex;
use jlivertool_core::config::ConfigStore;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

const LAYOUT_VERSION: usize = 2;
#[derive(Clone, Copy)]
pub(super) struct PanelSpec {
    pub(super) name: &'static str,
    pub(super) title: &'static str,
}

const DANMU_PANEL: PanelSpec = PanelSpec {
    name: "dashboard.danmu",
    title: "弹幕",
};
const SUPERCHAT_PANEL: PanelSpec = PanelSpec {
    name: "dashboard.superchat",
    title: "醒目留言",
};
const GIFT_PANEL: PanelSpec = PanelSpec {
    name: "dashboard.gift",
    title: "礼物记录",
};
const STATISTICS_PANEL: PanelSpec = PanelSpec {
    name: "dashboard.statistics",
    title: "数据统计",
};
const AUDIENCE_PANEL: PanelSpec = PanelSpec {
    name: "dashboard.audience",
    title: "观众列表",
};

pub(crate) struct DashboardViews {
    danmu: Entity<DashboardDanmuView>,
    gift: Entity<GiftView>,
    superchat: Entity<SuperChatView>,
    statistics: Entity<StatisticsView>,
    audience: Entity<AudienceView>,
}

impl DashboardViews {
    pub(crate) fn new(
        danmu: Entity<DashboardDanmuView>,
        gift: Entity<GiftView>,
        superchat: Entity<SuperChatView>,
        statistics: Entity<StatisticsView>,
        audience: Entity<AudienceView>,
    ) -> Self {
        Self {
            danmu,
            gift,
            superchat,
            statistics,
            audience,
        }
    }
}

pub(crate) struct DashboardView {
    dock_area: Entity<DockArea>,
    command_tx: mpsc::Sender<UiCommand>,
    _layout_subscription: Subscription,
    _layout_save_task: Task<()>,
}

impl DashboardView {
    pub(crate) fn new(
        views: DashboardViews,
        config: ConfigStore,
        command_tx: mpsc::Sender<UiCommand>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::register_panels(&views, cx);

        let dock_area =
            cx.new(|cx| DockArea::new("jlivertool-dashboard", Some(LAYOUT_VERSION), window, cx));

        let saved_layout = config.get_dashboard_layout();
        let loaded = saved_layout
            .and_then(|value| serde_json::from_value::<DockAreaState>(value).ok())
            .filter(|state| state.version == Some(LAYOUT_VERSION))
            .map(|state| {
                dock_area
                    .update(cx, |dock, cx| dock.load(state, window, cx))
                    .is_ok()
            })
            .unwrap_or(false);

        if !loaded {
            Self::apply_default_layout(&dock_area, views, window, cx);
        }

        let command_tx_for_layout = command_tx.clone();
        let layout_subscription =
            cx.subscribe(&dock_area, move |this, dock_area, event: &DockEvent, cx| {
                if matches!(event, DockEvent::LayoutChanged) {
                    let state = dock_area.read(cx).dump(cx);
                    if let Ok(value) = serde_json::to_value(state) {
                        let command_tx = command_tx_for_layout.clone();
                        this._layout_save_task = cx.background_executor().spawn(async move {
                            Timer::after(Duration::from_millis(400)).await;
                            let _ = command_tx.send(UiCommand::SaveDashboardLayout(value));
                        });
                    }
                }
            });

        Self {
            dock_area,
            command_tx,
            _layout_subscription: layout_subscription,
            _layout_save_task: Task::ready(()),
        }
    }

    pub(crate) fn save_layout(&self, cx: &App) {
        let state = self.dock_area.read(cx).dump(cx);
        if let Ok(value) = serde_json::to_value(state) {
            let _ = self.command_tx.send(UiCommand::SaveDashboardLayout(value));
        }
    }

    fn register_panels(views: &DashboardViews, cx: &mut Context<Self>) {
        let content = views.danmu.clone();
        register_panel(cx, DANMU_PANEL.name, move |_, _, _, _, cx| {
            Box::new(cx.new(|cx| DashboardPanel::new(DANMU_PANEL, content.clone(), cx)))
        });

        let content = views.superchat.clone();
        register_panel(cx, SUPERCHAT_PANEL.name, move |_, _, _, _, cx| {
            Box::new(cx.new(|cx| DashboardPanel::new(SUPERCHAT_PANEL, content.clone(), cx)))
        });

        let content = views.gift.clone();
        register_panel(cx, GIFT_PANEL.name, move |_, _, _, _, cx| {
            Box::new(cx.new(|cx| DashboardPanel::new(GIFT_PANEL, content.clone(), cx)))
        });

        let content = views.statistics.clone();
        register_panel(cx, STATISTICS_PANEL.name, move |_, _, _, _, cx| {
            Box::new(cx.new(|cx| DashboardPanel::new(STATISTICS_PANEL, content.clone(), cx)))
        });

        let content = views.audience.clone();
        register_panel(cx, AUDIENCE_PANEL.name, move |_, _, _, _, cx| {
            Box::new(cx.new(|cx| DashboardPanel::new(AUDIENCE_PANEL, content.clone(), cx)))
        });
    }

    fn apply_default_layout(
        dock_area: &Entity<DockArea>,
        views: DashboardViews,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let weak_dock = dock_area.downgrade();

        let danmu_panel = cx.new(|cx| DashboardPanel::new(DANMU_PANEL, views.danmu, cx));
        let gift_panel = cx.new(|cx| DashboardPanel::new(GIFT_PANEL, views.gift, cx));
        let superchat_panel =
            cx.new(|cx| DashboardPanel::new(SUPERCHAT_PANEL, views.superchat, cx));
        let statistics_panel =
            cx.new(|cx| DashboardPanel::new(STATISTICS_PANEL, views.statistics, cx));
        let audience_panel = cx.new(|cx| DashboardPanel::new(AUDIENCE_PANEL, views.audience, cx));

        let danmu = DockItem::tabs(vec![Arc::new(danmu_panel)], &weak_dock, window, cx);

        let gift = DockItem::tabs(vec![Arc::new(gift_panel)], &weak_dock, window, cx);
        let superchat = DockItem::tabs(vec![Arc::new(superchat_panel)], &weak_dock, window, cx);
        let middle =
            DockItem::v_split(vec![gift, superchat], &weak_dock, window, cx).size(px(400.0));

        let statistics = DockItem::tabs(vec![Arc::new(statistics_panel)], &weak_dock, window, cx);
        let audience = DockItem::tabs(vec![Arc::new(audience_panel)], &weak_dock, window, cx);
        let right =
            DockItem::v_split(vec![statistics, audience], &weak_dock, window, cx).size(px(400.0));

        let root = DockItem::h_split(vec![danmu, middle, right], &weak_dock, window, cx);

        dock_area.update(cx, |dock, cx| dock.set_center(root, window, cx));
    }
}

impl Render for DashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .overflow_hidden()
            .bg(Colors::bg_primary())
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(self.dock_area.clone()),
            )
    }
}
