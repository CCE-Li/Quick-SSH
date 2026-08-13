//! Layout Engine：布局树 → 屏幕区域
//!
//! 将 [`LayoutNode`] 递归计算为每个 Widget 的 `Rect`，
//! 使用权重比例（`Constraint::Ratio`）精确切分，避免百分比取整误差。

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::config::{LayoutDirection, LayoutNode, WidgetId};

/// 计算布局树，返回 `(WidgetId, Rect)` 列表（树遍历顺序）
pub fn compute_layout(root: &LayoutNode, area: Rect) -> Vec<(WidgetId, Rect)> {
    let mut out = Vec::new();
    walk(root, area, &mut out);
    out
}

fn walk(node: &LayoutNode, area: Rect, out: &mut Vec<(WidgetId, Rect)>) {
    match node {
        LayoutNode::Widget { id } => {
            out.push((*id, area));
        }
        LayoutNode::Split {
            direction,
            children,
        } => {
            if children.is_empty() {
                return;
            }
            let total: u32 = children.iter().map(|c| c.weight as u32).sum::<u32>().max(1);
            let ratatui_dir = match direction {
                LayoutDirection::Vertical => Direction::Vertical,
                LayoutDirection::Horizontal => Direction::Horizontal,
            };
            let constraints: Vec<Constraint> = children
                .iter()
                .map(|c| Constraint::Ratio(c.weight as u32, total))
                .collect();

            let areas = Layout::default()
                .direction(ratatui_dir)
                .constraints(constraints)
                .split(area);

            for (child, child_area) in children.iter().zip(areas.iter()) {
                walk(&child.node, *child_area, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::dashboard::config::{LayoutDirection, LayoutNode, SplitChild, WidgetId};

    #[test]
    fn single_widget_takes_whole_area() {
        let area = Rect::new(0, 0, 80, 24);
        let node = LayoutNode::Widget { id: WidgetId::Cpu };
        let areas = compute_layout(&node, area);
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].0, WidgetId::Cpu);
        assert_eq!(areas[0].1, area);
    }

    #[test]
    fn split_horizontal_respects_weights() {
        let area = Rect::new(0, 0, 80, 24);
        let node = LayoutNode::Split {
            direction: LayoutDirection::Horizontal,
            children: vec![
                SplitChild::new(1, LayoutNode::Widget { id: WidgetId::Cpu }),
                SplitChild::new(
                    1,
                    LayoutNode::Widget {
                        id: WidgetId::Agent,
                    },
                ),
            ],
        };
        let areas = compute_layout(&node, area);
        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].1.x, 0);
        assert_eq!(areas[1].1.x, 40);
    }

    #[test]
    fn split_vertical_stacks_areas() {
        let area = Rect::new(0, 0, 80, 24);
        let node = LayoutNode::Split {
            direction: LayoutDirection::Vertical,
            children: vec![
                SplitChild::new(1, LayoutNode::Widget { id: WidgetId::Cpu }),
                SplitChild::new(
                    1,
                    LayoutNode::Widget {
                        id: WidgetId::Memory,
                    },
                ),
            ],
        };
        let areas = compute_layout(&node, area);
        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].1.y, 0);
        assert_eq!(areas[1].1.y, 12);
    }

    #[test]
    fn nested_split_three_to_one_ratio() {
        let area = Rect::new(0, 0, 100, 24);
        let node = LayoutNode::Split {
            direction: LayoutDirection::Horizontal,
            children: vec![
                SplitChild::new(3, LayoutNode::Widget { id: WidgetId::Cpu }),
                SplitChild::new(
                    1,
                    LayoutNode::Widget {
                        id: WidgetId::Agent,
                    },
                ),
            ],
        };
        let areas = compute_layout(&node, area);
        assert_eq!(areas[0].1.width, 75);
        assert_eq!(areas[1].1.width, 25);
    }
}
