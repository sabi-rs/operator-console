use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::owls::{OwlsEndpointGroup, OwlsEndpointSummary, OwlsGroupSummary};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let layout = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(13),
        Constraint::Length(7),
    ])
    .split(area);
    let top = Layout::horizontal([
        Constraint::Percentage(34),
        Constraint::Percentage(36),
        Constraint::Percentage(30),
    ])
    .split(layout[0]);
    let body = Layout::horizontal([Constraint::Percentage(61), Constraint::Percentage(39)])
        .split(layout[1]);
    let detail = Layout::vertical([Constraint::Length(9), Constraint::Min(4)]).split(body[1]);

    let selected = app.selected_owls_endpoint().cloned();
    render_overview(frame, top[0], app);
    render_group_strip(frame, top[1], app);
    render_selection_brief(frame, top[2], selected.as_ref());
    render_endpoint_table(frame, body[0], app);
    render_selection_detail(frame, detail[0], selected.as_ref());
    render_preview(frame, detail[1], selected.as_ref());
    render_runbook(frame, layout[2], app, selected.as_ref());
}

fn render_overview(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let owls = app.owls_dashboard();
    let ready = owls
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.status == "ready")
        .count();
    let waiting = owls
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.status == "waiting")
        .count();
    let errors = owls
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.status == "error")
        .count();

    let body = Paragraph::new(vec![
        Line::from(vec![
            badge("Sport", owls.sport.as_str(), accent_blue()),
            Span::raw("  "),
            badge(
                "Ready",
                &format!("{ready}/{}", owls.endpoints.len()),
                accent_green(),
            ),
            Span::raw("  "),
            badge("Wait", &waiting.to_string(), accent_gold()),
            Span::raw("  "),
            badge("Err", &errors.to_string(), accent_red()),
        ]),
        Line::raw(truncate(&owls.status_line, 80)),
    ])
    .block(section_block("Owls Surface", accent_blue()))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_group_strip(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mut lines = Vec::new();
    let mut current = Vec::new();

    for (index, group) in app.owls_dashboard().groups.iter().enumerate() {
        current.push(group_chip(group));
        if index == 2 {
            lines.push(Line::from(current));
            current = Vec::new();
        } else {
            current.push(Span::raw(" "));
        }
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }

    let body = Paragraph::new(lines)
        .block(section_block("Coverage", accent_cyan()))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_selection_brief(
    frame: &mut Frame<'_>,
    area: Rect,
    selected: Option<&OwlsEndpointSummary>,
) {
    let (title, path, status) = match selected {
        Some(endpoint) => (
            endpoint.label.as_str(),
            endpoint.path.as_str(),
            endpoint.status.as_str(),
        ),
        None => ("No endpoint", "-", "idle"),
    };

    let body = Paragraph::new(vec![
        Line::styled(
            truncate(title, 30),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(vec![
            badge("State", status, status_color(status)),
            Span::raw("  "),
            Span::styled(truncate(path, 42), Style::default().fg(muted_text())),
        ]),
    ])
    .block(section_block("Selection", accent_gold()))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_endpoint_table(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let rows = app
        .owls_dashboard()
        .endpoints
        .iter()
        .map(|endpoint| {
            Row::new(vec![
                Cell::from(endpoint.group.short()),
                Cell::from(endpoint.label.clone()),
                Cell::from(endpoint.status.clone()).style(
                    Style::default()
                        .fg(status_color(&endpoint.status))
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(endpoint.count.to_string()),
                Cell::from(truncate(&endpoint.path, 32)).style(Style::default().fg(muted_text())),
                Cell::from(truncate(&endpoint.detail, 22)),
            ])
        })
        .collect::<Vec<_>>();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(34),
            Constraint::Min(12),
        ],
    )
    .header(
        Row::new(vec!["G", "Endpoint", "State", "Rows", "Route", "Detail"]).style(
            Style::default()
                .fg(accent_cyan())
                .add_modifier(Modifier::BOLD),
        ),
    )
    .row_highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(accent_cyan())
            .add_modifier(Modifier::BOLD),
    )
    .column_spacing(1)
    .block(section_block("Endpoint Board", accent_cyan()));
    frame.render_stateful_widget(table, area, app.owls_endpoint_table_state());
}

fn render_selection_detail(
    frame: &mut Frame<'_>,
    area: Rect,
    selected: Option<&OwlsEndpointSummary>,
) {
    let Some(endpoint) = selected else {
        let body = Paragraph::new("Select an endpoint to inspect the route and filters.")
            .block(section_block("Endpoint Detail", accent_gold()))
            .wrap(Wrap { trim: true });
        frame.render_widget(body, area);
        return;
    };

    let lines = vec![
        Line::from(vec![
            badge("Group", endpoint.group.label(), group_color(endpoint.group)),
            Span::raw("  "),
            badge("Rows", &endpoint.count.to_string(), accent_green()),
            Span::raw("  "),
            badge(
                "Updated",
                &endpoint.updated_at.as_str().if_empty("-"),
                accent_gold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Route ", Style::default().fg(accent_cyan())),
            Span::raw(format!("{} {}", endpoint.method, endpoint.path)),
        ]),
        Line::from(vec![
            Span::styled("About ", Style::default().fg(accent_cyan())),
            Span::raw(endpoint.description.clone()),
        ]),
        Line::from(vec![
            Span::styled("Filters ", Style::default().fg(accent_cyan())),
            Span::raw(endpoint.query_hint.clone()),
        ]),
        Line::from(vec![
            Span::styled("Detail ", Style::default().fg(accent_cyan())),
            Span::raw(endpoint.detail.clone()),
        ]),
    ];

    let body = Paragraph::new(lines)
        .block(section_block("Endpoint Detail", accent_gold()))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_preview(frame: &mut Frame<'_>, area: Rect, selected: Option<&OwlsEndpointSummary>) {
    let mut lines = Vec::new();

    match selected {
        Some(endpoint) if !endpoint.preview.is_empty() => {
            for row in endpoint.preview.iter().take(6) {
                lines.push(Line::styled(
                    truncate(&row.label, 34),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
                lines.push(Line::raw(format!(
                    "{} | {}",
                    truncate(&row.detail, 28),
                    truncate(&row.metric, 22)
                )));
            }
        }
        Some(endpoint) => lines.push(Line::raw(format!(
            "No preview rows returned for {}.",
            endpoint.label
        ))),
        None => lines.push(Line::raw("No endpoint selected.")),
    }

    let body = Paragraph::new(lines)
        .block(section_block("Preview", accent_pink()))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_runbook(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    selected: Option<&OwlsEndpointSummary>,
) {
    let snapshot = app.snapshot();
    let selected_label = selected
        .map(|endpoint| endpoint.label.as_str())
        .unwrap_or("none");
    let body = Paragraph::new(vec![
        Line::from(vec![
            metric("Route", accent_blue()),
            Span::raw(selected_label),
            Span::raw("  "),
            metric("Provider", accent_cyan()),
            Span::raw(truncate(&snapshot.status_line, 34)),
            Span::raw("  "),
            metric("Venue", accent_green()),
            Span::raw(
                snapshot
                    .selected_venue
                    .map(|venue| venue.as_str())
                    .unwrap_or("-"),
            ),
        ]),
        Line::from(vec![
            metric("Use", accent_gold()),
            Span::raw("j/k select  enter inspect  r/R hydrate"),
            Span::raw("  "),
            metric("Goal", accent_pink()),
            Span::raw("API-first board"),
        ]),
    ])
    .block(section_block("Runbook", accent_pink()))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn group_chip(group: &OwlsGroupSummary) -> Span<'static> {
    Span::styled(
        format!(
            "{} {} {}/{} !{} ?{}",
            group.group.short(),
            group.label,
            group.ready,
            group.total,
            group.error,
            group.waiting
        ),
        Style::default()
            .fg(group_color(group.group))
            .add_modifier(Modifier::BOLD),
    )
}

fn section_block(title: &'static str, color: Color) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
}

fn badge(label: &str, value: &str, color: Color) -> Span<'static> {
    Span::styled(
        format!("{label}:{value}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn metric(label: &str, color: Color) -> Span<'static> {
    Span::styled(format!("{label} "), Style::default().fg(color))
}

fn status_color(status: &str) -> Color {
    match status {
        "ready" => accent_green(),
        "waiting" => accent_gold(),
        "error" => accent_red(),
        _ => muted_text(),
    }
}

fn group_color(group: OwlsEndpointGroup) -> Color {
    match group {
        OwlsEndpointGroup::Odds => accent_blue(),
        OwlsEndpointGroup::Props => accent_pink(),
        OwlsEndpointGroup::Scores => accent_green(),
        OwlsEndpointGroup::Stats => accent_cyan(),
        OwlsEndpointGroup::Prediction => accent_gold(),
        OwlsEndpointGroup::History => Color::Rgb(255, 171, 145),
        OwlsEndpointGroup::Realtime => Color::Rgb(196, 181, 253),
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_string();
    }
    format!("{}...", &value[..limit.saturating_sub(3)])
}

fn accent_blue() -> Color {
    Color::Rgb(90, 169, 255)
}

fn accent_cyan() -> Color {
    Color::Rgb(78, 201, 176)
}

fn accent_green() -> Color {
    Color::Rgb(134, 239, 172)
}

fn accent_gold() -> Color {
    Color::Rgb(250, 204, 21)
}

fn accent_pink() -> Color {
    Color::Rgb(244, 143, 177)
}

fn accent_red() -> Color {
    Color::Rgb(248, 113, 113)
}

fn muted_text() -> Color {
    Color::Rgb(152, 166, 181)
}

trait EmptyFallback {
    fn if_empty(self, fallback: &str) -> String;
}

impl EmptyFallback for &str {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            return String::from(fallback);
        }
        self.to_string()
    }
}
