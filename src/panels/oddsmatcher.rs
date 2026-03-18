use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;

use crate::app::{App, OddsMatcherFocus};
use crate::oddsmatcher::{OddsMatcherField, OddsMatcherRow};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(13),
        Constraint::Min(18),
    ])
    .split(area);
    let lower = Layout::horizontal([Constraint::Min(72), Constraint::Length(38)]).split(layout[2]);
    let rows = app.oddsmatcher_rows().to_vec();
    let focus = app.oddsmatcher_focus();

    render_header(frame, layout[0], app);
    render_filter_deck(frame, layout[1], app);
    render_table(frame, lower[0], &rows, app.oddsmatcher_table_state(), focus);
    render_details(
        frame,
        lower[1],
        app.selected_oddsmatcher_row(),
        app.status_message(),
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let selected = app
        .selected_oddsmatcher_row()
        .map(|row| row.selection_name.as_str())
        .unwrap_or("-");
    let row_count = app.oddsmatcher_rows().len().to_string();
    let metrics = Line::from(vec![
        badge("Rows", row_count.as_str(), accent_blue()),
        Span::raw(" "),
        badge(
            "Focus",
            match app.oddsmatcher_focus() {
                OddsMatcherFocus::Filters => "filters",
                OddsMatcherFocus::Results => "results",
            },
            accent_green(),
        ),
        Span::raw(" "),
        badge("Selected", selected, accent_gold()),
        Span::raw(" "),
        badge(
            "Edit",
            if app.oddsmatcher_is_editing() {
                "live"
            } else {
                "idle"
            },
            accent_pink(),
        ),
    ]);
    let controls = Line::from(vec![
        hint("h/l", accent_blue()),
        Span::raw(" focus  "),
        hint("j/k", accent_blue()),
        Span::raw(" move  "),
        hint("enter", accent_blue()),
        Span::raw(" edit/open calc  "),
        hint("[ ]", accent_blue()),
        Span::raw(" cycle  "),
        hint("r", accent_blue()),
        Span::raw(" refresh  "),
        hint("c", accent_blue()),
        Span::raw(" reset"),
    ]);

    let body = Paragraph::new(vec![
        metrics,
        controls,
        Line::from(Span::styled(
            app.oddsmatcher_query_note(),
            Style::default().fg(muted_text()),
        )),
    ])
    .block(section_block("OddsMatcher", accent_blue()))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_filter_deck(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows = Layout::vertical([Constraint::Length(5), Constraint::Length(8)]).split(area);
    let primary = Layout::horizontal([
        Constraint::Ratio(1, 6),
        Constraint::Ratio(1, 6),
        Constraint::Ratio(1, 6),
        Constraint::Ratio(1, 6),
        Constraint::Ratio(1, 6),
        Constraint::Ratio(1, 6),
    ])
    .split(rows[0]);
    let secondary = Layout::horizontal([
        Constraint::Length(28),
        Constraint::Length(30),
        Constraint::Min(28),
        Constraint::Length(30),
    ])
    .split(rows[1]);

    render_single_field_card(
        frame,
        primary[0],
        "Sport",
        OddsMatcherField::Sport,
        app,
        accent_blue(),
    );
    render_single_field_card(
        frame,
        primary[1],
        "Market Type",
        OddsMatcherField::MarketGroup,
        app,
        accent_blue(),
    );
    render_single_field_card(
        frame,
        primary[2],
        "Bookmaker",
        OddsMatcherField::Bookmaker,
        app,
        accent_gold(),
    );
    render_single_field_card(
        frame,
        primary[3],
        "Exchange",
        OddsMatcherField::Exchange,
        app,
        accent_cyan(),
    );
    render_rating_card(frame, primary[4], app);
    render_odds_card(frame, primary[5], app);

    render_availability_card(frame, secondary[0], app);
    render_timeframe_card(frame, secondary[1], app);
    render_multi_field_card(
        frame,
        secondary[2],
        "Scope",
        &[
            (OddsMatcherField::EventGroup, "Comp"),
            (OddsMatcherField::Country, "Country"),
            (OddsMatcherField::EventId, "Event"),
        ],
        app,
        accent_gold(),
    );
    render_status_card(frame, secondary[3], app);
}

fn render_single_field_card(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    field: OddsMatcherField,
    app: &App,
    accent: Color,
) {
    let lines = if field.suggestions().len() > 1 {
        vec![
            selected_field_value_line(app, field, "Value"),
            preset_chip_line(app, field),
        ]
    } else {
        vec![
            selected_field_value_line(app, field, "Value"),
            Line::from(Span::styled(
                suggestion_label(field),
                Style::default().fg(muted_text()),
            )),
        ]
    };
    let body = Paragraph::new(lines)
        .block(card_block(
            title,
            accent,
            app.oddsmatcher_selected_field() == field,
        ))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_rating_card(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let selected = matches!(
        app.oddsmatcher_selected_field(),
        OddsMatcherField::RatingType | OddsMatcherField::MaxRating
    );
    let body = Paragraph::new(vec![
        selected_field_value_line(app, OddsMatcherField::RatingType, "Type"),
        selected_field_value_line(app, OddsMatcherField::MaxRating, "Cap"),
        preset_chip_line(app, OddsMatcherField::MaxRating),
    ])
    .block(card_block("Rating", accent_green(), selected))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_odds_card(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let selected = matches!(
        app.oddsmatcher_selected_field(),
        OddsMatcherField::MinOdds | OddsMatcherField::MaxOdds
    );
    let body = Paragraph::new(vec![
        selected_field_value_line(app, OddsMatcherField::MinOdds, "Min"),
        preset_chip_line(app, OddsMatcherField::MinOdds),
        selected_field_value_line(app, OddsMatcherField::MaxOdds, "Max"),
        preset_chip_line(app, OddsMatcherField::MaxOdds),
    ])
    .block(card_block("Min/Max Odds", accent_pink(), selected))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_availability_card(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let selected = matches!(
        app.oddsmatcher_selected_field(),
        OddsMatcherField::MinLiquidity
            | OddsMatcherField::ExcludeDraw
            | OddsMatcherField::Limit
            | OddsMatcherField::Skip
    );
    let body = Paragraph::new(vec![
        selected_field_value_line(app, OddsMatcherField::MinLiquidity, "Min Liq"),
        preset_chip_line(app, OddsMatcherField::MinLiquidity),
        selected_field_value_line(app, OddsMatcherField::Limit, "Limit"),
        preset_chip_line(app, OddsMatcherField::Limit),
        selected_field_value_line(app, OddsMatcherField::ExcludeDraw, "No Draw"),
        preset_chip_line(app, OddsMatcherField::ExcludeDraw),
    ])
    .block(card_block("Availability", accent_cyan(), selected))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_multi_field_card(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    fields: &[(OddsMatcherField, &'static str)],
    app: &App,
    accent: Color,
) {
    let selected = fields
        .iter()
        .any(|(field, _)| *field == app.oddsmatcher_selected_field());
    let lines = fields
        .iter()
        .map(|(field, label)| selected_field_value_line(app, *field, label))
        .collect::<Vec<_>>();
    let body = Paragraph::new(lines)
        .block(card_block(title, accent, selected))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_timeframe_card(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let sections = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner_area(area));
    let ratio = timeframe_ratio(app.oddsmatcher_query().updated_within_seconds);
    let gauge = LineGauge::default()
        .filled_style(
            Style::default()
                .fg(accent_blue())
                .add_modifier(Modifier::BOLD),
        )
        .unfilled_style(Style::default().fg(border_color()))
        .ratio(ratio)
        .label(timeframe_label(
            app.oddsmatcher_query().updated_within_seconds,
        ));
    let labels = Paragraph::new(Line::from(vec![
        Span::raw("now  "),
        timeframe_span(app, 1_800, "30m"),
        Span::raw("  "),
        timeframe_span(app, 3_600, "1h"),
        Span::raw("  "),
        timeframe_span(app, 7_200, "2h"),
        Span::raw("  "),
        timeframe_span(app, 21_600, "6h"),
        Span::raw("  "),
        timeframe_span(app, 86_400, "1d"),
        Span::raw("  "),
        timeframe_span(app, 259_200, "3d"),
        Span::raw("  "),
        timeframe_span(app, 604_800, "7d"),
        Span::raw("  all"),
    ]));
    let info = Paragraph::new(vec![
        selected_field_value_line(app, OddsMatcherField::UpdatedWithinSeconds, "Window"),
        Line::from(Span::styled(
            "Set this in filters, then refresh.",
            Style::default().fg(muted_text()),
        )),
    ]);

    frame.render_widget(
        card_block(
            "Timeframe",
            accent_blue(),
            app.oddsmatcher_selected_field() == OddsMatcherField::UpdatedWithinSeconds,
        ),
        area,
    );
    frame.render_widget(gauge, sections[0]);
    frame.render_widget(labels, sections[1]);
    frame.render_widget(info, sections[2]);
}

fn render_status_card(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let status = if app.oddsmatcher_rows().is_empty() {
        "No offers cached"
    } else {
        "Offers loaded"
    };
    let body = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("State  ", Style::default().fg(muted_text())),
            Span::styled(status, Style::default().fg(accent_green())),
        ]),
        Line::from(vec![
            Span::styled("Refresh ", Style::default().fg(muted_text())),
            Span::raw("r"),
            Span::raw("   "),
            Span::styled("Reset ", Style::default().fg(muted_text())),
            Span::raw("c"),
        ]),
        Line::from(vec![
            Span::styled("Action ", Style::default().fg(muted_text())),
            Span::raw("enter seeds calculator"),
        ]),
    ])
    .block(section_block("Actions", accent_green()))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[OddsMatcherRow],
    table_state: &mut TableState,
    focus: OddsMatcherFocus,
) {
    let header = Row::new(vec![
        Cell::from("Date"),
        Cell::from("Time"),
        Cell::from("Event"),
        Cell::from("Bet"),
        Cell::from("Bookie"),
        Cell::from("Exch"),
        Cell::from("Back"),
        Cell::from("Lay"),
        Cell::from("Rating"),
        Cell::from("Avail."),
    ])
    .style(
        Style::default()
            .fg(Color::Black)
            .bg(accent_cyan())
            .add_modifier(Modifier::BOLD),
    );

    let body_rows = rows.iter().enumerate().map(|(index, row)| {
        let stripe = if index % 2 == 0 {
            panel_background()
        } else {
            elevated_background()
        };
        Row::new(vec![
            Cell::from(start_date_label(&row.start_at)),
            Cell::from(start_time_label(&row.start_at)),
            Cell::from(row.event_name.clone()),
            Cell::from(row.selection_name.clone()),
            Cell::from(short_bookmaker_name(&row.back.bookmaker.display_name)),
            Cell::from(short_bookmaker_name(&row.lay.bookmaker.display_name)),
            Cell::from(Span::styled(
                format!("{:.2}", row.back.odds),
                Style::default()
                    .fg(accent_blue())
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                format!("{:.2}", row.lay.odds),
                Style::default()
                    .fg(accent_pink())
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                format!("{:.1}%", row.rating),
                rating_style(row.rating),
            )),
            Cell::from(row.availability_label()),
        ])
        .style(Style::default().bg(stripe).fg(text_color()))
    });

    let table = Table::new(
        body_rows,
        [
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Percentage(28),
            Constraint::Percentage(18),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(section_block(
        if focus == OddsMatcherFocus::Results {
            "● Offers"
        } else {
            "Offers"
        },
        accent_green(),
    ))
    .row_highlight_style(
        Style::default()
            .bg(accent_blue())
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(if rows.is_empty() { "  " } else { "● " });

    frame.render_stateful_widget(table, area, table_state);
}

fn render_details(
    frame: &mut Frame<'_>,
    area: Rect,
    row: Option<&OddsMatcherRow>,
    status_message: &str,
) {
    let layout = Layout::vertical([
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Min(4),
    ])
    .split(area);

    if let Some(row) = row {
        let date = start_date_label(&row.start_at);
        let time = start_time_label(&row.start_at);
        let rating = format!("{:.1}%", row.rating);
        let availability = row.availability_label();
        render_key_values(
            frame,
            layout[0],
            "Selection",
            accent_pink(),
            &[
                ("Event", row.event_name.as_str()),
                ("Date", date.as_str()),
                ("Time", time.as_str()),
                ("Bet", row.selection_name.as_str()),
            ],
        );
        render_key_values(
            frame,
            layout[1],
            "Market",
            accent_blue(),
            &[
                ("Sport", row.sport.display_name.as_str()),
                ("Type", row.market_name.as_str()),
                ("Group", row.event_group.display_name.as_str()),
                ("Match", row.market_group.display_name.as_str()),
            ],
        );
        render_key_values(
            frame,
            layout[2],
            "Pricing",
            accent_green(),
            &[
                ("Bookie", row.back.bookmaker.display_name.as_str()),
                ("Exchange", row.lay.bookmaker.display_name.as_str()),
                ("Rating", rating.as_str()),
                ("Avail", availability.as_str()),
            ],
        );
        let notes = Paragraph::new(vec![
            link_line("Back", row.back.deep_link.as_deref().unwrap_or("-")),
            link_line("Lay", row.lay.deep_link.as_deref().unwrap_or("-")),
            Line::from(vec![
                Span::styled("Betslip ", Style::default().fg(muted_text())),
                Span::raw(
                    row.lay
                        .bet_slip
                        .as_ref()
                        .map(|bet_slip| format!("{}/{}", bet_slip.market_id, bet_slip.selection_id))
                        .unwrap_or_else(|| String::from("-")),
                ),
            ]),
            Line::from(Span::styled(
                status_message,
                Style::default().fg(muted_text()),
            )),
        ])
        .block(section_block("Links", accent_gold()))
        .wrap(Wrap { trim: true });
        frame.render_widget(notes, layout[3]);
    } else {
        let empty = Paragraph::new(vec![
            Line::raw("No live OddsMatcher rows loaded."),
            Line::raw("Refresh with r after adjusting filters."),
            Line::raw("Enter on a result seeds the calculator."),
            Line::raw(""),
            Line::from(Span::styled(
                status_message,
                Style::default().fg(muted_text()),
            )),
        ])
        .block(section_block("Selection", accent_pink()))
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
    }
}

fn render_key_values(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    accent: Color,
    entries: &[(&str, &str)],
) {
    let rows = entries.iter().map(|(label, value)| {
        Row::new(vec![
            Cell::from(Span::styled(
                format!("{label:>8}"),
                Style::default().fg(muted_text()),
            )),
            Cell::from(Span::styled(
                (*value).to_string(),
                Style::default()
                    .fg(text_color())
                    .add_modifier(Modifier::BOLD),
            )),
        ])
    });
    let table = Table::new(rows, [Constraint::Length(10), Constraint::Min(10)])
        .block(section_block(title, accent));
    frame.render_widget(table, area);
}

fn selected_field_value_line(app: &App, field: OddsMatcherField, label: &str) -> Line<'static> {
    let selected = app.oddsmatcher_selected_field() == field;
    let value = displayed_field_value(app, field);
    let label_style = if selected {
        selected_filter_style(app)
    } else {
        Style::default().fg(muted_text())
    };
    let value_style = if selected {
        selected_filter_style(app).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(text_color())
    };
    Line::from(vec![
        Span::styled(format!("{label:<8}"), label_style),
        Span::styled(value, value_style),
    ])
}

fn suggestion_label(field: OddsMatcherField) -> String {
    let suggestions = field.suggestions();
    if suggestions.is_empty() {
        String::from("Free text")
    } else {
        format!("Preset: {}", suggestions.join(" · "))
    }
}

fn preset_chip_line(app: &App, field: OddsMatcherField) -> Line<'static> {
    let suggestions = field.suggestions();
    if suggestions.is_empty() {
        return Line::from(Span::styled("Free text", Style::default().fg(muted_text())));
    }

    let current = field.display_value(app.oddsmatcher_query());
    let selected = app.oddsmatcher_selected_field() == field;
    let mut spans = Vec::new();
    for (index, suggestion) in suggestions.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(preset_chip_span(
            display_chip_value(field, suggestion),
            selected && app.oddsmatcher_focus() == OddsMatcherFocus::Filters,
            current == *suggestion,
        ));
    }
    Line::from(spans)
}

fn preset_chip_span(label: String, selected: bool, active: bool) -> Span<'static> {
    let style = if active && selected {
        Style::default()
            .fg(Color::Black)
            .bg(accent_gold())
            .add_modifier(Modifier::BOLD)
    } else if active {
        Style::default()
            .fg(Color::Black)
            .bg(accent_blue())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(muted_text()).bg(elevated_background())
    };
    Span::styled(format!(" {label} "), style)
}

fn display_chip_value(field: OddsMatcherField, value: &str) -> String {
    match (field, value) {
        (OddsMatcherField::ExcludeDraw, "false") => String::from("Off"),
        (OddsMatcherField::ExcludeDraw, "true") => String::from("On"),
        _ => value.to_string(),
    }
}

fn displayed_field_value(app: &App, field: OddsMatcherField) -> String {
    if app.oddsmatcher_is_editing() && app.oddsmatcher_selected_field() == field {
        let buffer = app.oddsmatcher_edit_buffer().unwrap_or_default();
        if buffer.is_empty() {
            String::from("_")
        } else {
            format!("{buffer}_")
        }
    } else {
        let value = field.display_value(app.oddsmatcher_query());
        if value.is_empty() {
            String::from("All")
        } else {
            value
        }
    }
}

fn selected_filter_style(app: &App) -> Style {
    if app.oddsmatcher_focus() == OddsMatcherFocus::Filters {
        Style::default()
            .fg(Color::Black)
            .bg(accent_gold())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(accent_gold())
    }
}

fn badge<'a>(label: &'a str, value: &'a str, accent: Color) -> Span<'a> {
    Span::styled(
        format!(" {label}: {value} "),
        Style::default()
            .fg(Color::Black)
            .bg(accent)
            .add_modifier(Modifier::BOLD),
    )
}

fn hint<'a>(value: &'a str, accent: Color) -> Span<'a> {
    Span::styled(
        value,
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    )
}

fn link_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<7}"), Style::default().fg(muted_text())),
        Span::raw(value.to_string()),
    ])
}

fn timeframe_label(seconds: u64) -> String {
    match seconds {
        0 => String::from("All"),
        1..=1_800 => String::from("30 minutes"),
        1_801..=3_600 => String::from("1 hour"),
        3_601..=7_200 => String::from("2 hours"),
        7_201..=21_600 => String::from("6 hours"),
        21_601..=86_400 => String::from("1 day"),
        86_401..=259_200 => String::from("3 days"),
        259_201..=604_800 => String::from("7 days"),
        _ => format!("{}s", seconds),
    }
}

fn timeframe_ratio(seconds: u64) -> f64 {
    let presets = [1_800_u64, 3_600, 7_200, 21_600, 86_400, 259_200, 604_800];
    let index = presets
        .iter()
        .position(|preset| seconds <= *preset)
        .unwrap_or(presets.len());
    if presets.is_empty() {
        0.0
    } else {
        index as f64 / presets.len() as f64
    }
}

fn timeframe_span(app: &App, seconds: u64, label: &'static str) -> Span<'static> {
    if app.oddsmatcher_query().updated_within_seconds == seconds {
        Span::styled(
            label,
            Style::default()
                .fg(Color::Black)
                .bg(accent_blue())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(label, Style::default().fg(muted_text()))
    }
}

fn short_bookmaker_name(name: &str) -> String {
    match name {
        "Smarkets" => String::from("Smarkets"),
        "Matchbook" => String::from("Matchbook"),
        "Betfair Exchange" => String::from("Betfair"),
        _ if name.len() > 9 => name.chars().take(9).collect(),
        _ => name.to_string(),
    }
}

fn rating_style(rating: f64) -> Style {
    if rating >= 99.0 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(214, 69, 65))
            .add_modifier(Modifier::BOLD)
    } else if rating >= 97.0 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(248, 208, 119))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(accent_green())
            .add_modifier(Modifier::BOLD)
    }
}

fn card_block(title: &'static str, accent: Color, selected: bool) -> Block<'static> {
    let border = if selected {
        accent_gold()
    } else {
        border_color()
    };
    Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(panel_background()).fg(text_color()))
        .border_style(Style::default().fg(border))
}

fn section_block(title: &'static str, accent: Color) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(panel_background()).fg(text_color()))
        .border_style(Style::default().fg(border_color()))
}

fn inner_area(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn start_date_label(value: &str) -> String {
    parse_isoish_start(value)
        .map(|(date, _)| date)
        .unwrap_or_else(|| String::from("-"))
}

fn start_time_label(value: &str) -> String {
    parse_isoish_start(value)
        .map(|(_, time)| time)
        .unwrap_or_else(|| String::from("-"))
}

fn parse_isoish_start(value: &str) -> Option<(String, String)> {
    let trimmed = value.trim();
    if trimmed.len() < 16 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    if bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
    {
        return None;
    }
    Some((
        trimmed.get(0..10)?.to_string(),
        trimmed.get(11..16)?.to_string(),
    ))
}

fn panel_background() -> Color {
    Color::Rgb(16, 22, 30)
}

fn elevated_background() -> Color {
    Color::Rgb(21, 29, 39)
}

fn text_color() -> Color {
    Color::Rgb(234, 240, 246)
}

fn muted_text() -> Color {
    Color::Rgb(156, 171, 188)
}

fn border_color() -> Color {
    Color::Rgb(74, 88, 104)
}

fn accent_blue() -> Color {
    Color::Rgb(109, 180, 255)
}

fn accent_cyan() -> Color {
    Color::Rgb(94, 234, 212)
}

fn accent_green() -> Color {
    Color::Rgb(134, 239, 172)
}

fn accent_gold() -> Color {
    Color::Rgb(248, 208, 119)
}

fn accent_pink() -> Color {
    Color::Rgb(244, 143, 177)
}
