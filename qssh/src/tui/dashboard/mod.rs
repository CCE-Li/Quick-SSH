//! Dashboard 框架：配置 / 布局引擎 / Widget / 命令面板
//!
//! Phase 1 目标：让 UI 具备可配置的模块化布局骨架（Mock 数据），
//! 后续 Phase 将各模块替换为真实监控数据，并演进为独立 crate。

pub mod config;
pub mod layout;
pub mod palette;
pub mod ui;
pub mod widgets;

// 统一对外 re-export：当前 crate 为 bin，部分项尚未被内部消费，
// 保留公共 API 便于后续拆分为独立 crate 时直接复用。
#[allow(unused_imports)]
pub use config::{
    default_dashboard_config, default_layout_for, load_dashboard_config, save_dashboard_config,
    DashboardConfig, LayoutDirection, LayoutNode, ProfileConfig, SplitChild, WidgetId, ALL_WIDGETS,
};
#[allow(unused_imports)]
pub use layout::compute_layout;
#[allow(unused_imports)]
pub use palette::{filter_palette, PaletteAction, PaletteItem, PALETTE_ACTIONS};
#[allow(unused_imports)]
pub use widgets::{mock_widget, WidgetModule};
