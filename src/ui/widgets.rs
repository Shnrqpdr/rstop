use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as TLayout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};

use crate::app::AppState;

const COLOR_OK: Color = Color::Green;
const COLOR_WARN: Color = Color::Yellow;
const COLOR_CRIT: Color = Color::Red;

fn usage_color(pct: f64) -> Color {
    if pct >= 85.0 {
        COLOR_CRIT
    } else if pct >= 65.0 {
        COLOR_WARN
    } else {
        COLOR_OK
    }
}

/// One named series for [`line_chart`].
struct Series {
    label: String,
    color: Color,
    points: Vec<(f64, f64)>,
}

fn series(label: impl Into<String>, color: Color, values: &[f64]) -> Series {
    Series {
        label: label.into(),
        color,
        points: values
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v))
            .collect(),
    }
}

/// Render one or more time series as a line chart with X (time) / Y axes.
fn line_chart(
    frame: &mut Frame,
    area: Rect,
    title: String,
    series: &[Series],
    y_max: f64,
    y_unit: &str,
) {
    let block = Block::default().borders(Borders::ALL).title(title);

    let x_len = series
        .iter()
        .map(|s| s.points.len())
        .max()
        .unwrap_or(0)
        .max(2);
    let x_bounds = [0.0, (x_len - 1) as f64];

    let datasets: Vec<Dataset> = series
        .iter()
        .map(|s| {
            Dataset::default()
                .name(s.label.clone())
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(s.color))
                .data(&s.points)
        })
        .collect();

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds(x_bounds)
                .labels(vec![Span::raw("older"), Span::raw("now")]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, y_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.0}", y_max / 2.0)),
                    Span::raw(format!("{:.0}{}", y_max, y_unit)),
                ]),
        );
    frame.render_widget(chart, area);
}

fn empty_panel(frame: &mut Frame, area: Rect, title: &str, msg: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray))),
        inner,
    );
}

// ---------------------------------------------------------------------------
// CPU
// ---------------------------------------------------------------------------

pub fn cpu_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.config.detail.cpu {
        cpu_gauges(frame, area, state);
    } else {
        let data = state.history.cpu_total.as_slice_vec();
        let now = state.history.cpu_total.last().unwrap_or(0.0);
        let title = format!(
            " CPU  •  now {:.1}%  •  load {:.2} {:.2} {:.2} ",
            now,
            state.snapshot.cpu.load_avg.0,
            state.snapshot.cpu.load_avg.1,
            state.snapshot.cpu.load_avg.2,
        );
        line_chart(
            frame,
            area,
            title,
            &[series("cpu%", usage_color(now), &data)],
            100.0,
            "%",
        );
    }
}

fn cpu_gauges(frame: &mut Frame, area: Rect, state: &AppState) {
    use ratatui::widgets::Gauge;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " CPU cores  •  total {:.1}%  •  load {:.2} {:.2} {:.2} ",
            state.snapshot.cpu.total_usage,
            state.snapshot.cpu.load_avg.0,
            state.snapshot.cpu.load_avg.1,
            state.snapshot.cpu.load_avg.2,
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cores = &state.snapshot.cpu.per_core;
    if cores.is_empty() {
        return;
    }

    let cols = (cores.len() as f32).sqrt().ceil() as usize;
    let cols = cols.max(1).min(8);
    let rows = (cores.len() + cols - 1) / cols;

    let row_constraints: Vec<Constraint> =
        (0..rows).map(|_| Constraint::Ratio(1, rows as u32)).collect();
    let row_areas = TLayout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner);

    for r in 0..rows {
        let col_constraints: Vec<Constraint> =
            (0..cols).map(|_| Constraint::Ratio(1, cols as u32)).collect();
        let col_areas = TLayout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(row_areas[r]);

        for c in 0..cols {
            let idx = r * cols + c;
            if let Some(core) = cores.get(idx) {
                let pct = core.usage.clamp(0.0, 100.0);
                let gauge = Gauge::default()
                    .block(Block::default().title(format!("{} {} MHz", idx, core.frequency_mhz)))
                    .gauge_style(Style::default().fg(usage_color(pct as f64)))
                    .ratio((pct / 100.0) as f64)
                    .label(format!("{:.0}%", pct));
                frame.render_widget(gauge, col_areas[c]);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

pub fn mem_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.config.detail.mem {
        mem_block(frame, area, state);
    } else {
        let ram = state.history.mem_used_pct.as_slice_vec();
        let swap = state.history.swap_used_pct.as_slice_vec();
        let ram_now = state.snapshot.mem.ram_used_pct();
        let title = format!(
            " Memory  •  RAM {} / {} ({:.1}%)  •  Swap {:.1}% ",
            human_bytes(state.snapshot.mem.ram_used),
            human_bytes(state.snapshot.mem.ram_total),
            ram_now,
            state.snapshot.mem.swap_used_pct(),
        );
        line_chart(
            frame,
            area,
            title,
            &[
                series("ram%", Color::Cyan, &ram),
                series("swap%", Color::Magenta, &swap),
            ],
            100.0,
            "%",
        );
    }
}

fn mem_block(frame: &mut Frame, area: Rect, state: &AppState) {
    use ratatui::widgets::{Gauge, Sparkline};

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Memory (detailed) ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let v = TLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);

    let ram_pct = state.snapshot.mem.ram_used_pct();
    let ram_gauge = Gauge::default()
        .block(Block::default().title(format!(
            "RAM {} / {}",
            human_bytes(state.snapshot.mem.ram_used),
            human_bytes(state.snapshot.mem.ram_total),
        )))
        .gauge_style(Style::default().fg(usage_color(ram_pct)))
        .ratio((ram_pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.1}%", ram_pct));
    frame.render_widget(ram_gauge, v[0]);

    let swap_pct = state.snapshot.mem.swap_used_pct();
    let swap_gauge = Gauge::default()
        .block(Block::default().title(format!(
            "Swap {} / {}",
            human_bytes(state.snapshot.mem.swap_used),
            human_bytes(state.snapshot.mem.swap_total),
        )))
        .gauge_style(Style::default().fg(usage_color(swap_pct)))
        .ratio((swap_pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.1}%", swap_pct));
    frame.render_widget(swap_gauge, v[1]);

    let mem_hist = state.history.mem_used_pct.as_u64_vec();
    let sp = Sparkline::default()
        .block(Block::default().title("RAM history"))
        .data(&mem_hist)
        .max(100)
        .style(Style::default().fg(usage_color(ram_pct)));
    frame.render_widget(sp, v[2]);
}

// ---------------------------------------------------------------------------
// GPU
// ---------------------------------------------------------------------------

pub fn gpu_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.snapshot.gpus.is_empty() {
        empty_panel(
            frame,
            area,
            "GPU",
            "No GPU detected (NVML unavailable or no NVIDIA device).",
        );
        return;
    }

    if state.config.detail.gpu {
        gpu_block(frame, area, state);
        return;
    }

    let color = |i: usize| match i % 3 {
        0 => Color::Green,
        1 => Color::Cyan,
        _ => Color::Magenta,
    };

    let util_series: Vec<Series> = state
        .snapshot
        .gpus
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let data = state
                .history
                .gpu_util
                .get(i)
                .map(|h| h.as_slice_vec())
                .unwrap_or_default();
            series(format!("{} util%", g.name), color(i), &data)
        })
        .collect();

    let vram_series: Vec<Series> = state
        .snapshot
        .gpus
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let data = state
                .history
                .gpu_vram_pct
                .get(i)
                .map(|h| h.as_slice_vec())
                .unwrap_or_default();
            series(format!("{} vram%", g.name), color(i), &data)
        })
        .collect();

    let first = &state.snapshot.gpus[0];
    let temp = first
        .temperature_c
        .map(|t| format!("{:.0}°C", t))
        .unwrap_or_else(|| "—".into());

    let halves = TLayout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    line_chart(
        frame,
        halves[0],
        format!(
            " GPU util  •  {} {:.0}%  •  {} ",
            first.name,
            first.utilization_pct.unwrap_or(0.0),
            temp,
        ),
        &util_series,
        100.0,
        "%",
    );
    line_chart(
        frame,
        halves[1],
        format!(
            " VRAM  •  {:.0}% ",
            first.vram_used_pct().unwrap_or(0.0)
        ),
        &vram_series,
        100.0,
        "%",
    );
}

fn gpu_block(frame: &mut Frame, area: Rect, state: &AppState) {
    use ratatui::widgets::{Gauge, Sparkline};

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" GPU (detailed) ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let count = state.snapshot.gpus.len();
    let constraints: Vec<Constraint> = (0..count)
        .map(|_| Constraint::Ratio(1, count as u32))
        .collect();
    let slots = TLayout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, gpu) in state.snapshot.gpus.iter().enumerate() {
        let slot = slots[i];
        let v = TLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(slot);

        let temp_str = gpu
            .temperature_c
            .map(|t| format!("{:.0}°C", t))
            .unwrap_or_else(|| "—".into());
        let power_str = gpu
            .power_w
            .map(|p| format!("{:.0}W", p))
            .unwrap_or_else(|| "—".into());
        let header = Line::from(vec![
            Span::styled(
                format!("{} {}", gpu.vendor.label(), gpu.name),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   temp {}   power {}", temp_str, power_str)),
        ]);
        frame.render_widget(Paragraph::new(header), v[0]);

        let util = gpu.utilization_pct.unwrap_or(0.0) as f64;
        let util_gauge = Gauge::default()
            .block(Block::default().title("Utilization"))
            .gauge_style(Style::default().fg(usage_color(util)))
            .ratio((util / 100.0).clamp(0.0, 1.0))
            .label(format!("{:.0}%", util));
        frame.render_widget(util_gauge, v[1]);

        let vram_pct = gpu.vram_used_pct().unwrap_or(0.0);
        let vram_label = match (gpu.vram_used, gpu.vram_total) {
            (Some(u), Some(t)) => format!("VRAM {} / {}", human_bytes(u), human_bytes(t)),
            _ => "VRAM —".to_string(),
        };
        let vram_gauge = Gauge::default()
            .block(Block::default().title(vram_label))
            .gauge_style(Style::default().fg(usage_color(vram_pct)))
            .ratio((vram_pct / 100.0).clamp(0.0, 1.0))
            .label(format!("{:.1}%", vram_pct));
        frame.render_widget(vram_gauge, v[2]);

        let hist = state
            .history
            .gpu_util
            .get(i)
            .map(|h| h.as_u64_vec())
            .unwrap_or_default();
        let sp = Sparkline::default()
            .block(Block::default().title("GPU util history"))
            .data(&hist)
            .max(100)
            .style(Style::default().fg(usage_color(util)));
        frame.render_widget(sp, v[3]);
    }
}

// ---------------------------------------------------------------------------
// Thermals
// ---------------------------------------------------------------------------

pub fn temp_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.config.detail.temp {
        temp_block(frame, area, state);
        return;
    }

    let data = state.history.cpu_temp_max.as_slice_vec();
    if data.is_empty() {
        empty_panel(
            frame,
            area,
            "Thermals",
            "No hwmon CPU sensors available (common under WSL2).",
        );
        return;
    }
    let now = state.history.cpu_temp_max.last().unwrap_or(0.0);
    let color = if now >= 85.0 {
        COLOR_CRIT
    } else if now >= 70.0 {
        COLOR_WARN
    } else {
        COLOR_OK
    };
    let title = format!(" Thermals  •  CPU max {:.1}°C ", now);
    line_chart(
        frame,
        area,
        title,
        &[series("cpu °C", color, &data)],
        100.0,
        "°C",
    );
}

fn temp_block(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Thermals (detailed) ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    if !state.snapshot.temp.cpu_sensors.is_empty() {
        lines.push(Line::from(Span::styled(
            "CPU",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for s in &state.snapshot.temp.cpu_sensors {
            lines.push(temp_line(&s.label, s.temp_c));
        }
    }
    if !state.snapshot.temp.other_sensors.is_empty() {
        lines.push(Line::from(Span::styled(
            "Other",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for s in &state.snapshot.temp.other_sensors {
            lines.push(temp_line(&s.label, s.temp_c));
        }
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No hwmon sensors available.",
            Style::default().fg(Color::DarkGray),
        )));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn temp_line(label: &str, temp_c: f32) -> Line<'static> {
    let color = if temp_c >= 85.0 {
        COLOR_CRIT
    } else if temp_c >= 70.0 {
        COLOR_WARN
    } else {
        COLOR_OK
    };
    Line::from(vec![
        Span::raw(format!("  {:<28}", label.to_string())),
        Span::styled(format!("{:>6.1} °C", temp_c), Style::default().fg(color)),
    ])
}

fn human_bytes(b: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", b, UNITS[0])
    } else {
        format!("{:.1} {}", v, UNITS[i])
    }
}
