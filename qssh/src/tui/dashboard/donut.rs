//! ASCII 甜甜圈连接加载动画（donut.c 移植）
//!
//! 经典 a1k0n donut.c 的 Rust 移植：按终端区域尺寸参数化投影，z 缓冲逐点
//! 渲染旋转的甜甜圈，并将光照亮度映射到 `.,-~:;=!*#$@` 字符集。
//! 供嵌入式终端「连接中」加载动画使用（样式由 `~/.qsshrc` 的
//! `connect_animation` 选择）。

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

/// 亮度字符集（由暗到亮）
pub(crate) const DONUT_CHARS: [char; 12] =
    ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];

/// 旋转角速度（弧度/秒）：A / B 两轴保持经典 donut.c 的 2:1 比例
const SPIN_A_SPEED: f32 = 2.4;
const SPIN_B_SPEED: f32 = 1.2;

/// 表面参数（i, j）采样步长（基准密度）与密度参考半径（格）
const BASE_STEP_I: f32 = 0.02;
const BASE_STEP_J: f32 = 0.07;
const BASE_RADIUS: f32 = 20.0;

/// 在 `area` 内渲染甜甜圈动画；`elapsed` 为进入连接状态的时长（驱动旋转相位）
pub fn render_donut(area: Rect, buf: &mut Buffer, elapsed: Duration) {
    if area.width < 3 || area.height < 2 {
        return;
    }
    let (w, h) = (area.width as usize, area.height as usize);

    // 经典参数（画布 80x22、振幅 30/15）下甜甜圈投影半径恒为振幅的 0.75 倍，
    // 取 kx = 2*ky 抵消终端单元格约 1:2 的宽高比以保证投影为圆形；0.9 留边。
    let ky = ((w as f32 / 3.0).min(h as f32 / 1.5)) * 0.9;
    let kx = ky * 2.0;
    let (ox, oy) = (area.x as f32, area.y as f32);
    let (cx, cy) = (ox + w as f32 / 2.0, oy + h as f32 / 2.0);

    // 区域越大采样越密，避免投影出现空洞
    let density = (ky / BASE_RADIUS).max(1.0);
    let step_i = BASE_STEP_I / density;
    let step_j = BASE_STEP_J / density;

    let ta = elapsed.as_secs_f32() * SPIN_A_SPEED;
    let tb = elapsed.as_secs_f32() * SPIN_B_SPEED;
    let (e, g) = (ta.sin(), ta.cos());
    let (m, n) = (tb.sin(), tb.cos());

    let mut zbuf = vec![0.0f32; w * h];
    let mut j = 0.0f32;
    while j < std::f32::consts::TAU {
        let (cos_j, sin_j) = (j.cos(), j.sin());
        let ring = cos_j + 2.0;
        let mut i = 0.0f32;
        while i < std::f32::consts::TAU {
            let (cos_i, sin_i) = (i.cos(), i.sin());
            // 与 donut.c 一致的投影 / 光照公式
            let depth = 1.0 / (sin_i * ring * e + sin_j * g + 5.0);
            let t = sin_i * ring * g - sin_j * e;
            let x = cx + kx * depth * (cos_i * ring * m - t * n);
            let y = cy + ky * depth * (cos_i * ring * n + t * m);
            let (rx, ry) = (x - ox, y - oy);
            if (0.0..w as f32).contains(&rx) && (0.0..h as f32).contains(&ry) {
                let (px, py) = (rx as usize, ry as usize);
                let idx = py * w + px;
                if depth > zbuf[idx] {
                    zbuf[idx] = depth;
                    let lum = 8.0
                        * ((sin_j * e - sin_i * cos_j * g) * m
                            - sin_i * cos_j * e
                            - sin_j * g
                            - cos_i * cos_j * n);
                    let level = if lum > 0.0 {
                        (lum as usize).min(DONUT_CHARS.len() - 1)
                    } else {
                        0
                    };
                    if let Some(cell) = buf.cell_mut((area.x + px as u16, area.y + py as u16)) {
                        cell.set_char(DONUT_CHARS[level]);
                        cell.set_style(level_style(level));
                    }
                }
            }
            i += step_i;
        }
        j += step_j;
    }
}

/// 按亮度分级着色：暗部灰色、中部亮灰、高光黄色加粗
fn level_style(level: usize) -> Style {
    match level {
        0..=3 => Style::default().fg(Color::DarkGray),
        4..=7 => Style::default().fg(Color::Gray),
        _ => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    fn render(area: Rect, elapsed: Duration) -> Buffer {
        let mut buf = Buffer::empty(area);
        render_donut(area, &mut buf, elapsed);
        buf
    }

    fn text_of(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    /// 图案包围盒（绝对坐标 (min_x, min_y, max_x, max_y)；空画面返回 None）
    fn glyph_bounds(buf: &Buffer) -> Option<(usize, usize, usize, usize)> {
        let (ax, ay) = (buf.area.x as usize, buf.area.y as usize);
        let w = buf.area.width as usize;
        let (mut min_x, mut min_y) = (usize::MAX, usize::MAX);
        let (mut max_x, mut max_y) = (0usize, 0usize);
        let mut found = false;
        for (idx, cell) in buf.content().iter().enumerate() {
            if cell.symbol() != " " {
                let (x, y) = (ax + idx % w, ay + idx / w);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
        found.then_some((min_x, min_y, max_x, max_y))
    }

    /// 画面只由甜甜圈字符集构成，且有实际内容
    #[test]
    fn renders_donut_charset_only() {
        for elapsed in [Duration::ZERO, Duration::from_millis(700)] {
            let buf = render(Rect::new(0, 0, 60, 20), elapsed);
            let text = text_of(&buf);
            assert!(
                text.chars().all(|c| c == ' ' || DONUT_CHARS.contains(&c)),
                "出现字符集外的字符: {text:?}"
            );
            assert!(text.chars().any(|c| DONUT_CHARS.contains(&c)), "画面为空");
        }
    }

    /// 同一时刻输出确定；不同时刻画面变化（动画确实在转）
    #[test]
    fn animation_is_time_driven_and_deterministic() {
        let area = Rect::new(0, 0, 60, 20);
        let a = text_of(&render(area, Duration::from_millis(300)));
        let b = text_of(&render(area, Duration::from_millis(300)));
        assert_eq!(a, b);
        let c = text_of(&render(area, Duration::from_millis(900)));
        assert_ne!(a, c);
    }

    /// 各相位下图案都环绕区域中心（donut.c 透视投影本身带轻微摆动）
    #[test]
    fn donut_is_centered_in_area() {
        let area = Rect::new(0, 0, 80, 24);
        for ms in [0u64, 300, 600, 900, 1200] {
            let buf = render(area, Duration::from_millis(ms));
            let (min_x, min_y, max_x, max_y) = glyph_bounds(&buf).expect("应有内容");
            let center_x = (min_x + max_x) as f32 / 2.0;
            let center_y = (min_y + max_y) as f32 / 2.0;
            assert!(
                (center_x - 39.5).abs() <= 8.0,
                "ms={ms} 水平偏离过大: {center_x}"
            );
            assert!(
                (center_y - 11.5).abs() <= 5.0,
                "ms={ms} 垂直偏离过大: {center_y}"
            );
        }
    }

    /// 区域变大时图形等比放大
    #[test]
    fn donut_scales_with_area() {
        let small = glyph_bounds(&render(Rect::new(0, 0, 40, 12), Duration::from_millis(500)))
            .expect("小区域应有内容");
        let big = glyph_bounds(&render(Rect::new(0, 0, 80, 24), Duration::from_millis(500)))
            .expect("大区域应有内容");
        assert!(
            big.2 - big.0 > small.2 - small.0 && big.3 - big.1 > small.3 - small.1,
            "图形未随区域放大: small={small:?} big={big:?}"
        );
    }

    /// 偏移区域：图案仍绘制在区域自身的坐标系内（字符不越界）
    #[test]
    fn renders_inside_offset_area() {
        let area = Rect::new(10, 5, 50, 16);
        for ms in [100u64, 400, 700] {
            let buf = render(area, Duration::from_millis(ms));
            let (ax, ay) = (area.x as usize, area.y as usize);
            let w = buf.area.width as usize;
            for (idx, cell) in buf.content().iter().enumerate() {
                if cell.symbol() != " " {
                    let (x, y) = (ax + idx % w, ay + idx / w);
                    assert!(
                        x >= ax && x < ax + 50 && y >= ay && y < ay + 16,
                        "字符越界: ({x}, {y})"
                    );
                }
            }
        }
    }

    /// 极小 / 空区域不 panic
    #[test]
    fn tiny_areas_are_skipped_without_panic() {
        for (w, h) in [(0, 0), (1, 1), (2, 3), (3, 2), (5, 5)] {
            let area = Rect::new(0, 0, w, h);
            let mut buf = Buffer::empty(area);
            render_donut(area, &mut buf, Duration::from_millis(123));
        }
    }
}
