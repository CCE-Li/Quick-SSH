//! Layout Engine：布局树 → 屏幕区域
//!
//! 将 [`LayoutNode`] 递归计算为每个 Widget 的 `Rect`，
//! 使用权重比例（`Constraint::Ratio`）精确切分，避免百分比取整误差。

use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};

use super::config::{LayoutDirection, LayoutNode, WidgetId};

/// 计算布局树，返回 `(WidgetId, Rect)` 列表（树遍历顺序）
pub fn compute_layout(root: &LayoutNode, area: Rect) -> Vec<(WidgetId, Rect)> {
    let mut out = Vec::new();
    walk(root, area, &mut out);
    out
}

/// 计算 Split 节点的子区域（按权重比例切分）
pub fn split_areas(node: &LayoutNode, area: Rect) -> Vec<Rect> {
    let LayoutNode::Split {
        children,
        direction,
    } = node
    else {
        return Vec::new();
    };
    if children.is_empty() {
        return Vec::new();
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
    Layout::default()
        .direction(ratatui_dir)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

fn walk(node: &LayoutNode, area: Rect, out: &mut Vec<(WidgetId, Rect)>) {
    match node {
        LayoutNode::Widget { id } => {
            out.push((*id, area));
        }
        LayoutNode::Split { children, .. } => {
            let areas = split_areas(node, area);
            for (child, child_area) in children.iter().zip(areas.iter()) {
                walk(&child.node, *child_area, out);
            }
        }
    }
}

/// 收集布局树中出现的全部 WidgetId（含重复，树遍历顺序）
pub fn collect_widget_ids(node: &LayoutNode, out: &mut Vec<WidgetId>) {
    match node {
        LayoutNode::Widget { id } => out.push(*id),
        LayoutNode::Split { children, .. } => {
            for child in children {
                collect_widget_ids(&child.node, out);
            }
        }
    }
}

/// 鼠标命中测试：返回命中的分隔线所属 Split 的路径（根 → Split 的子节点下标序列）
/// 以及分隔线左侧/上方子节点下标 `pivot`。未命中返回 `None`。
pub fn hit_test_divider(
    root: &LayoutNode,
    area: Rect,
    pos: Position,
) -> Option<(Vec<usize>, usize)> {
    let mut path = Vec::new();
    hit_test_rec(root, area, pos, &mut path)
}

fn hit_test_rec(
    node: &LayoutNode,
    area: Rect,
    pos: Position,
    path: &mut Vec<usize>,
) -> Option<(Vec<usize>, usize)> {
    match node {
        LayoutNode::Widget { .. } => None,
        LayoutNode::Split {
            children,
            direction,
        } => {
            let areas = split_areas(node, area);
            for i in 0..children.len().saturating_sub(1) {
                if divider_hit(*direction, areas[i], areas[i + 1], pos) {
                    return Some((path.clone(), i));
                }
            }
            for (i, child_area) in areas.iter().enumerate() {
                if child_area.contains(pos) {
                    path.push(i);
                    if let Some(hit) = hit_test_rec(&children[i].node, *child_area, pos, path) {
                        return Some(hit);
                    }
                    path.pop();
                }
            }
            None
        }
    }
}

/// 分隔线命中判断：Horizontal 为左右子节点之间的竖线，Vertical 为上下之间的横线
fn divider_hit(direction: LayoutDirection, left: Rect, right: Rect, pos: Position) -> bool {
    match direction {
        LayoutDirection::Horizontal => {
            let boundary = left.x + left.width;
            let top = left.y.min(right.y);
            let bottom = (left.y + left.height).max(right.y + right.height);
            pos.x.saturating_add(1) >= boundary
                && pos.x <= boundary
                && pos.y >= top
                && pos.y < bottom
        }
        LayoutDirection::Vertical => {
            let boundary = left.y + left.height;
            let start = left.x.min(right.x);
            let end = (left.x + left.width).max(right.x + right.width);
            pos.y.saturating_add(1) >= boundary
                && pos.y <= boundary
                && pos.x >= start
                && pos.x < end
        }
    }
}

/// 沿路径定位到 Split 节点，返回其可变引用、所在区域、以及各子节点当前像素尺寸
/// （Horizontal 取宽，Vertical 取高）。路径终点必须是 Split 节点。
pub fn split_metrics<'a>(
    root: &'a mut LayoutNode,
    area: Rect,
    path: &[usize],
) -> Option<(&'a mut LayoutNode, Rect, Vec<u16>)> {
    fn rec<'a>(
        node: &'a mut LayoutNode,
        area: Rect,
        path: &[usize],
    ) -> Option<(&'a mut LayoutNode, Rect, Vec<u16>)> {
        if matches!(node, LayoutNode::Widget { .. }) {
            return None;
        }
        // 先计算子区域（不可变借用返回 owned，借用随即结束）
        let areas = split_areas(node, area);
        if path.is_empty() {
            let sizes = match node {
                LayoutNode::Split { direction, .. } => match direction {
                    LayoutDirection::Horizontal => areas.iter().map(|r| r.width).collect(),
                    LayoutDirection::Vertical => areas.iter().map(|r| r.height).collect(),
                },
                LayoutNode::Widget { .. } => unreachable!(),
            };
            return Some((node, area, sizes));
        }
        let idx = path[0];
        let child_area = *areas.get(idx)?;
        let LayoutNode::Split { children, .. } = node else {
            return None;
        };
        rec(&mut children.get_mut(idx)?.node, child_area, &path[1..])
    }
    rec(root, area, path)
}

/// 沿路径定位 Split 节点并返回各子节点当前像素尺寸（只读，Horizontal 取宽，Vertical 取高）
pub fn split_sizes(node: &LayoutNode, area: Rect, path: &[usize]) -> Option<Vec<u16>> {
    let mut node = node;
    let mut current_area = area;
    for &idx in path {
        let areas = split_areas(node, current_area);
        let child_area = *areas.get(idx)?;
        let LayoutNode::Split { children, .. } = node else {
            return None;
        };
        node = &children.get(idx)?.node;
        current_area = child_area;
    }
    let LayoutNode::Split { direction, .. } = node else {
        return None;
    };
    let areas = split_areas(node, current_area);
    Some(match direction {
        LayoutDirection::Horizontal => areas.iter().map(|r| r.width).collect(),
        LayoutDirection::Vertical => areas.iter().map(|r| r.height).collect(),
    })
}

/// 计算拖动分隔线后各子节点应更新的权重（像素尺寸）。
///
/// `fixed` 为拖动开始时各子节点的像素尺寸；拖动相邻两个子节点之间的分隔线时，
/// 两个被拖动子节点按其像素尺寸重新分配，其余子节点保持 `fixed` 不变，
/// 所有权重之和恒等于 split 区域尺寸，保证 ratatui `Ratio` 精确取整。
pub fn apply_split_resize(
    children_count: usize,
    direction: LayoutDirection,
    area: Rect,
    pivot: usize,
    fixed: &[u16],
    col: u16,
    row: u16,
) -> Option<Vec<u16>> {
    if pivot + 1 >= children_count {
        return None;
    }
    let (total, boundary, origin) = match direction {
        LayoutDirection::Horizontal => (area.width as i64, col as i64, area.x as i64),
        LayoutDirection::Vertical => (area.height as i64, row as i64, area.y as i64),
    };
    let before: i64 = fixed[..pivot].iter().map(|&w| w as i64).sum();
    let after: i64 = fixed[pivot + 2..].iter().map(|&w| w as i64).sum();
    let remaining = total - before - after;
    if remaining < 2 {
        return None;
    }
    let offset = boundary - origin;
    let pi = (offset - before).clamp(1, remaining - 1);
    let pj = remaining - pi;
    let mut weights: Vec<u16> = fixed.to_vec();
    weights[pivot] = pi as u16;
    weights[pivot + 1] = pj as u16;
    Some(weights)
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

    #[test]
    fn hit_test_divider_finds_vertical_divider() {
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
        // 分隔线位于 x=40（两侧边框 39/40 均命中）
        assert!(hit_test_divider(&node, area, Position::new(39, 10)).is_some());
        assert!(hit_test_divider(&node, area, Position::new(40, 10)).is_some());
        // Widget 内部不命中
        assert!(hit_test_divider(&node, area, Position::new(10, 10)).is_none());
        assert!(hit_test_divider(&node, area, Position::new(70, 10)).is_none());
    }

    #[test]
    fn split_metrics_returns_child_sizes() {
        let area = Rect::new(0, 0, 80, 24);
        let mut node = LayoutNode::Split {
            direction: LayoutDirection::Horizontal,
            children: vec![
                SplitChild::new(1, LayoutNode::Widget { id: WidgetId::Cpu }),
                SplitChild::new(
                    3,
                    LayoutNode::Widget {
                        id: WidgetId::Terminal,
                    },
                ),
            ],
        };
        let (_, split_area, sizes) = split_metrics(&mut node, area, &[]).unwrap();
        assert_eq!(split_area, area);
        assert_eq!(sizes, vec![20, 60]);
    }

    #[test]
    fn apply_split_resize_redistributes_weights() {
        let area = Rect::new(0, 0, 80, 24);
        let weights =
            apply_split_resize(2, LayoutDirection::Horizontal, area, 0, &[20, 60], 50, 0).unwrap();
        assert_eq!(weights, vec![50, 30]);
        // 权重和保持区域尺寸，Ratio 精确取整
        assert_eq!(weights.iter().sum::<u16>(), 80);
    }

    #[test]
    fn apply_split_resize_clamps_to_valid_range() {
        let area = Rect::new(0, 0, 80, 24);
        // 拖出区域外时钳制到最小 1px
        let weights =
            apply_split_resize(2, LayoutDirection::Horizontal, area, 0, &[20, 60], 0, 0).unwrap();
        assert_eq!(weights, vec![1, 79]);
    }

    #[test]
    fn apply_split_resize_keeps_third_child_fixed() {
        let area = Rect::new(0, 0, 120, 24);
        // [left(1) | terminal(3) | agent(1)] 拖 terminal/agent 分隔线
        let weights = apply_split_resize(
            3,
            LayoutDirection::Horizontal,
            area,
            1,
            &[24, 72, 24],
            96,
            0,
        )
        .unwrap();
        assert_eq!(weights, vec![24, 72, 24]);
        // 拖动到 terminal 只剩 10px
        let weights = apply_split_resize(
            3,
            LayoutDirection::Horizontal,
            area,
            1,
            &[24, 72, 24],
            35,
            0,
        )
        .unwrap();
        assert_eq!(weights, vec![24, 11, 85]);
    }

    #[test]
    fn collect_widget_ids_lists_all_widgets() {
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
        let mut ids = Vec::new();
        collect_widget_ids(&node, &mut ids);
        assert_eq!(ids, vec![WidgetId::Cpu, WidgetId::Agent]);
    }
}
