use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, AppMode};
use crate::scanner::format_bytes;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    match app.mode {
        AppMode::Select => draw_select_view(f, app, size),
        AppMode::Executing | AppMode::Done => draw_exec_view(f, app, size),
    }
}

fn draw_select_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Table of targets
            Constraint::Length(4), // Footer / Action
        ])
        .split(area);

    // 1. Header
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = header_block.inner(chunks[0]);
    f.render_widget(header_block, chunks[0]);

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(12),
        ])
        .split(inner);

    let left = Paragraph::new(Span::styled(
        " rensa",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(left, header_layout[0]);

    let right = Paragraph::new(Span::styled(
        "v0.1.0 ",
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Right);
    f.render_widget(right, header_layout[1]);

    // 2. Table of Targets
    let items_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Targets ", Style::default().fg(Color::Gray)));

    let spinner_char = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];

    let mut rows = Vec::new();

    for (idx, item) in app.items.iter().enumerate() {
        let is_current = idx == app.selected_index;

        let box_span = if item.is_scanning {
            Span::styled(format!(" [ {} ]", spinner_char), Style::default().fg(Color::Yellow))
        } else if item.selected {
            Span::styled(" [ x ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" [   ]", Style::default().fg(Color::DarkGray))
        };

        let size_span = if item.is_scanning {
            Span::styled(" [  scanning  ]", Style::default().fg(Color::DarkGray))
        } else if item.selected {
            Span::styled(
                format!(" [ {:>10} ]", format_bytes(item.bytes)),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                format!(" [ {:>10} ]", format_bytes(item.bytes)),
                Style::default().fg(Color::DarkGray),
            )
        };

        let title_style = if is_current {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let details_style = if is_current {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let scope_badge = if item.needs_root {
            Span::styled("[root]", Style::default().fg(Color::LightRed))
        } else {
            Span::styled("[user]", Style::default().fg(Color::DarkGray))
        };

        let cursor_symbol = if is_current {
            Span::styled(">", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            Span::raw(" ")
        };

        let status_line = Line::from(vec![cursor_symbol, box_span]);

        let mut row = Row::new(vec![
            Cell::from(status_line),
            Cell::from(Span::styled(&item.title, title_style)),
            Cell::from(Span::styled(&item.details, details_style)),
            Cell::from(scope_badge),
            Cell::from(size_span),
        ]);

        if is_current {
            row = row.style(Style::default().bg(Color::Rgb(36, 40, 48)));
        }

        rows.push(row);
    }

    let header_row = Row::new(vec![
        Cell::from(Span::styled("  STATUS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("TARGET", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("DETAILS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("SCOPE", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("     EST. SIZE", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .bottom_margin(1);

    let widths = [
        Constraint::Length(8),  // Cursor + Checkbox
        Constraint::Length(32), // Target Title
        Constraint::Min(26),    // Details / Description
        Constraint::Length(8),  // Scope badge ([root] / [user])
        Constraint::Length(16), // Size [ 14.11 GiB ]
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(items_block);

    f.render_widget(table, chunks[1]);

    // 3. Footer / Action bar
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let total_str = format_bytes(app.total_selected_bytes());
    let action_str = " Rensa skräp [Enter] ";

    let row1 = Line::from(vec![
        Span::raw("  Total selected to purge: "),
        Span::styled(total_str, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("    "),
        Span::styled(
            action_str,
            Style::default()
                .bg(Color::Rgb(40, 80, 60))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let row2 = Line::from(vec![
        Span::styled("  [Space]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" Toggle   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[a]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" All   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[j/k / ↑/↓]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" Navigate   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[r]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" Rescan   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
    ]);

    let footer = Paragraph::new(vec![row1, row2]).block(footer_block);
    f.render_widget(footer, chunks[2]);
}

fn draw_exec_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(10), // Steps table
            Constraint::Min(6),    // Logs
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // 1. Header
    let title_status = if app.mode == AppMode::Done {
        "Purge Complete"
    } else {
        "Purging Selected Targets..."
    };

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = header_block.inner(chunks[0]);
    f.render_widget(header_block, chunks[0]);

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(12),
        ])
        .split(inner);

    let left = Paragraph::new(Line::from(vec![
        Span::styled(" rensa ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("· "),
        Span::styled(
            title_status,
            Style::default()
                .fg(if app.mode == AppMode::Done { Color::Green } else { Color::Yellow })
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    f.render_widget(left, header_layout[0]);

    let right = Paragraph::new(Span::styled(
        "v0.1.0 ",
        Style::default().fg(Color::DarkGray),
    ))
    .alignment(Alignment::Right);
    f.render_widget(right, header_layout[1]);

    // 2. Steps summary (Table layout)
    let steps_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Execution Plan ", Style::default().fg(Color::Gray)));

    let spinner_char = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];
    let mut step_rows = Vec::new();

    for item in app.items.iter().filter(|i| i.selected) {
        let is_done = app.completed_steps.iter().find(|(id, _, _)| id == &item.id);
        let is_current = app.current_executing_id.as_deref() == Some(&item.id);

        let (icon, style) = if let Some((_, success, _)) = is_done {
            if *success {
                (Span::styled(" [ ✓ ]", Style::default().fg(Color::Green)), Style::default().fg(Color::Gray))
            } else {
                (Span::styled(" [ ✗ ]", Style::default().fg(Color::Red)), Style::default().fg(Color::Red))
            }
        } else if is_current {
            (
                Span::styled(format!(" [ {} ]", spinner_char), Style::default().fg(Color::Yellow)),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )
        } else {
            (Span::styled(" [   ]", Style::default().fg(Color::DarkGray)), Style::default().fg(Color::DarkGray))
        };

        let msg_detail = if let Some((_, _, msg)) = is_done {
            msg.clone()
        } else if is_current {
            "in progress...".to_string()
        } else {
            "pending".to_string()
        };

        let step_row = Row::new(vec![
            Cell::from(icon),
            Cell::from(Span::styled(&item.title, style)),
            Cell::from(Span::styled(msg_detail, Style::default().fg(Color::DarkGray))),
        ]);

        step_rows.push(step_row);
    }

    let step_widths = [
        Constraint::Length(8),
        Constraint::Length(32),
        Constraint::Min(30),
    ];

    let steps_table = Table::new(step_rows, step_widths).block(steps_block);
    f.render_widget(steps_table, chunks[1]);

    // 3. Execution Log stream
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Live Output ", Style::default().fg(Color::Gray)));

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .rev()
        .take(15)
        .rev()
        .map(|l| {
            if l.starts_with('$') {
                Line::from(Span::styled(l, Style::default().fg(Color::Cyan)))
            } else if l.to_lowercase().contains("error") {
                Line::from(Span::styled(l, Style::default().fg(Color::Red)))
            } else {
                Line::from(Span::styled(l, Style::default().fg(Color::DarkGray)))
            }
        })
        .collect();

    let logs_widget = Paragraph::new(log_lines).block(logs_block);
    f.render_widget(logs_widget, chunks[2]);

    // 4. Footer
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let footer_text = if app.mode == AppMode::Done {
        Line::from(vec![
            Span::styled("  Done! ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Freed approx "),
            Span::styled(
                format_bytes(app.total_freed_bytes),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::raw(".  Press "),
            Span::styled("[Enter / Esc]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" to return to menu, "),
            Span::styled("[q]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" to quit."),
        ])
    } else {
        Line::from(vec![
            Span::raw("  Working... Press "),
            Span::styled("[Ctrl+C]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" to abort."),
        ])
    };

    let footer = Paragraph::new(footer_text).block(footer_block);
    f.render_widget(footer, chunks[3]);
}
