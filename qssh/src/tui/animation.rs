use std::time::{Duration, Instant};

use ratatui::layout::Rect;

use crate::tui::action::{Mode, View};

/// Mode/View 的逻辑快照，用于在帧边界捕获所有直接字段赋值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiState {
    pub mode: Mode,
    pub view: View,
}

/// 动画缓动曲线。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseOutCubic,
    EaseInOutCubic,
}

impl Easing {
    pub fn sample(self, value: f32) -> f32 {
        let value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        match self {
            Self::Linear => value,
            Self::EaseOutCubic => 1.0 - (1.0 - value).powi(3),
            Self::EaseInOutCubic if value < 0.5 => 4.0 * value.powi(3),
            Self::EaseInOutCubic => 1.0 - (-2.0 * value + 2.0).powi(3) / 2.0,
        }
    }
}

/// 一次 Mode/View 状态过渡。
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: UiState,
    pub to: UiState,
    started_at: Instant,
    duration: Duration,
    easing: Easing,
}

impl Transition {
    pub fn new(
        from: UiState,
        to: UiState,
        started_at: Instant,
        duration: Duration,
        easing: Easing,
    ) -> Self {
        Self {
            from,
            to,
            started_at,
            duration,
            easing,
        }
    }

    pub fn raw_progress_at(&self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        let elapsed = now.saturating_duration_since(self.started_at);
        (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    pub fn progress_at(&self, now: Instant) -> f32 {
        self.easing.sample(self.raw_progress_at(now))
    }

    pub fn is_finished_at(&self, now: Instant) -> bool {
        self.raw_progress_at(now) >= 1.0
    }

    #[allow(dead_code)]
    pub fn mode_changed(&self) -> bool {
        self.from.mode != self.to.mode
    }

    pub fn view_changed(&self) -> bool {
        self.from.view != self.to.view
    }
}

/// 将目标区域从中心轻微内缩状态平滑展开到完整大小。
pub fn expand_rect(target: Rect, progress: f32) -> Rect {
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if progress >= 1.0 {
        return target;
    }
    let max_inset_x = (target.width / 12).min(6);
    let max_inset_y = (target.height / 12).min(2);
    let inset_x = ((1.0 - progress) * f32::from(max_inset_x)).round() as u16;
    let inset_y = ((1.0 - progress) * f32::from(max_inset_y)).round() as u16;
    Rect::new(
        target.x.saturating_add(inset_x),
        target.y.saturating_add(inset_y),
        target.width.saturating_sub(inset_x.saturating_mul(2)),
        target.height.saturating_sub(inset_y.saturating_mul(2)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn states() -> (UiState, UiState) {
        (
            UiState {
                mode: Mode::Normal,
                view: View::HostList,
            },
            UiState {
                mode: Mode::Help,
                view: View::Dashboard,
            },
        )
    }

    #[test]
    fn easing_endpoints_and_clamping() {
        for easing in [Easing::Linear, Easing::EaseOutCubic, Easing::EaseInOutCubic] {
            assert_eq!(easing.sample(-1.0), 0.0);
            assert_eq!(easing.sample(0.0), 0.0);
            assert_eq!(easing.sample(1.0), 1.0);
            assert_eq!(easing.sample(2.0), 1.0);
            assert_eq!(easing.sample(f32::NAN), 0.0);
        }
        assert!(Easing::EaseOutCubic.sample(0.5) > 0.5);
    }

    #[test]
    fn transition_progress_is_time_driven_and_clamped() {
        let now = Instant::now();
        let (from, to) = states();
        let transition = Transition::new(from, to, now, Duration::from_millis(200), Easing::Linear);
        assert_eq!(transition.raw_progress_at(now), 0.0);
        assert_eq!(
            transition.raw_progress_at(now + Duration::from_millis(100)),
            0.5
        );
        assert!(transition.is_finished_at(now + Duration::from_millis(200)));
        assert_eq!(
            transition.raw_progress_at(now + Duration::from_secs(1)),
            1.0
        );
        assert!(transition.mode_changed());
        assert!(transition.view_changed());
    }

    #[test]
    fn zero_duration_transition_finishes_immediately() {
        let now = Instant::now();
        let (from, to) = states();
        let transition = Transition::new(from, to, now, Duration::ZERO, Easing::Linear);
        assert_eq!(transition.progress_at(now), 1.0);
        assert!(transition.is_finished_at(now));
    }

    #[test]
    fn expanded_rect_grows_monotonically_and_reaches_target() {
        let target = Rect::new(10, 5, 120, 40);
        let start = expand_rect(target, 0.0);
        let middle = expand_rect(target, 0.5);
        let end = expand_rect(target, 1.0);
        assert!(start.x >= middle.x && middle.x >= end.x);
        assert!(start.y >= middle.y && middle.y >= end.y);
        assert!(start.width <= middle.width && middle.width <= end.width);
        assert!(start.height <= middle.height && middle.height <= end.height);
        assert_eq!(end, target);
    }

    #[test]
    fn expanded_rect_handles_tiny_areas() {
        for target in [
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 0, 1, 1),
            Rect::new(3, 4, 2, 2),
        ] {
            let rect = expand_rect(target, f32::NAN);
            assert!(rect.x >= target.x);
            assert!(rect.y >= target.y);
            assert!(rect.right() <= target.right());
            assert!(rect.bottom() <= target.bottom());
        }
    }
}
