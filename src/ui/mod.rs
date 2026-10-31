use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
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
            Constraint::Min(8),    // List of items
            Constraint::Length(4), // Footer / Action
        ])
        .split(area);

    // 1. Header
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let header_text = Line::from(vec![
        Span::styled(" rensa ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("v0.1.0", Style::default().fg(Color::DarkGray)),
        Span::raw("  ·  "),
        Span::styled("NixOS Garbage Collection Helper", Style::default().fg(Color::Gray)),
    ]);

    let header = Paragraph::new(header_text)
        .block(header_block)
        .alignment(Alignment::Left);
    f.render_widget(header, chunks[0]);

    // 2. Items List
    let items_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Targets ", Style::default().fg(Color::Gray)));

    let mut list_items = Vec::new();
    let spinner_char = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];

    for (idx, item) in app.items.iter().enumerate() {
        let is_current = idx == app.selected_index;

        let (box_span, size_span) = if item.is_scanning {
            (
                Span::styled(format!("[ {} ]", spinner_char), Style::default().fg(Color::Yellow)),
                Span::styled("[  scanning  ]", Style::default().fg(Color::DarkGray)),
            )
        } else if item.selected {
            (
                Span::styled("[ x ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("[ {:>10} ]", format_bytes(item.bytes)),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            )
        } else {
            (
                Span::styled("[   ]", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("[ {:>10} ]", format_bytes(item.bytes)),
                    Style::default().fg(Color::DarkGray),
                ),
            )
        };

        let title_style = if is_current {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let details_style = Style::default().fg(Color::DarkGray);

        let root_badge = if item.needs_root {
            Span::styled(" [root]", Style::default().fg(Color::Red))
        } else {
            Span::raw("")
        };

        // Render as a single styled line
        let line = Line::from(vec![
            Span::raw("  "),
            box_span,
            Span::raw(" "),
            Span::styled(format!("{:<30}", item.title), title_style),
            Span::styled(format!(" {:<40}", item.details), details_style),
            root_badge,
            Span::raw("  "),
            size_span,
        ]);

        let mut item_widget = ListItem::new(line);
        if is_current {
            item_widget = item_widget.style(Style::default().bg(Color::Rgb(35, 38, 46)));
        }

        list_items.push(item_widget);
    }

    let list = List::new(list_items).block(items_block);
    f.render_widget(list, chunks[1]);

    // 3. Footer / Action bar
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let total_str = format_bytes(app.total_selected_bytes());
    let action_str = " [Enter] Rensa skräp ";

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
            Constraint::Length(8), // Steps
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

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" rensa ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("· "),
        Span::styled(
            title_status,
            Style::default()
                .fg(if app.mode == AppMode::Done { Color::Green } else { Color::Yellow })
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(header_block);
    f.render_widget(header, chunks[0]);

    // 2. Steps summary
    let steps_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Execution Plan ");

    let spinner_char = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];
    let mut step_lines = Vec::new();

    for item in app.items.iter().filter(|i| i.selected) {
        let is_done = app.completed_steps.iter().find(|(id, _, _)| id == &item.id);
        let is_current = app.current_executing_id.as_deref() == Some(&item.id);

        let (icon, style) = if let Some((_, success, _)) = is_done {
            if *success {
                (Span::styled("[ ✓ ]", Style::default().fg(Color::Green)), Style::default().fg(Color::Gray))
            } else {
                (Span::styled("[ ✗ ]", Style::default().fg(Color::Red)), Style::default().fg(Color::Red))
            }
        } else if is_current {
            (
                Span::styled(format!("[ {} ]", spinner_char), Style::default().fg(Color::Yellow)),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )
        } else {
            (Span::styled("[   ]", Style::default().fg(Color::DarkGray)), Style::default().fg(Color::DarkGray))
        };

        let msg_detail = if let Some((_, _, msg)) = is_done {
            format!(" · {}", msg)
        } else if is_current {
            " · working...".to_string()
        } else {
            "".to_string()
        };

        step_lines.push(Line::from(vec![
            Span::raw("  "),
            icon,
            Span::raw(" "),
            Span::styled(&item.title, style),
            Span::styled(msg_detail, Style::default().fg(Color::DarkGray)),
        ]));
    }

    let steps = Paragraph::new(step_lines).block(steps_block);
    f.render_widget(steps, chunks[1]);

    // 3. Execution Log stream
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Logs ");

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
