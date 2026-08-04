//! Dashboard mode: render live information as dockable panels in one window.

mod danmu_view;
mod dashboard_view;
mod panel;

pub(crate) use danmu_view::DashboardDanmuView;
pub(crate) use dashboard_view::{DashboardView, DashboardViews};
