use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;

use crate::app::{App, OddsMatcherFocus};
use crate::horse_matcher::HorseMatcherField;
use crate::oddsmatcher::OddsMatcherRow;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(11),
        Constraint::Min(18),
    ])
    .split(area);
    let lower = Layout::horizontal([Constraint::Min(74), Constraint::Length(40)]).split(layout[2]);
    let rows = app.horse_matcher_rows().to_vec();
    let focus = app.horse_matcher_focus();

    render_header(frame, layout[0], app);
    render_filters(frame, layout[1], app);
    render_table(
        frame,
        lower[0],
        &rows,
        app.horse_matcher_table_state(),
        focus,
    );
    render_details(
        frame,
        lower[1],
        app.selected_horse_matcher_row(),
        app.status_message(),
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let selected = app
        .selected_horse_matcher_row()
        .map(|row| row.selection_name.as_str())
        .unwrap_or("-");
    let body = Paragraph::new(vec![
        Line::from(vec![
            badge(
                "Rows",
                &app.horse_matcher_rows().len().to_string(),
                accent_blue(),
            ),
            Span::raw(" "),
            badge(
                "Focus",
                match app.horse_matcher_focus() {
                    OddsMatcherFocus::Filters => "filters",
                    OddsMatcherFocus::Results => "results",
                },
                accent_green(),
            ),
            Span::raw(" "),
            badge("Selected", selected, accent_gold()),
        ]),
        Line::from(vec![
            hint("h/l", accent_blue()),
            Span::raw(" focus  "),
            hint("j/k", accent_blue()),
            Span::raw(" move  "),
            hint("enter", accent_blue()),
            Span::raw(" edit/open calc  "),
            hint("p", accent_blue()),
            Span::raw(" place  "),
            hint("r", accent_blue()),
            Span::raw(" refresh"),
        ]),
        Line::from(Span::styled(
            app.horse_matcher_query_note(),
            Style::default().fg(muted_text()),
        )),
    ])
    .block(section_block("HorseMatcher", accent_blue()))
    .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_filters(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .split(area);

    render_filter_card(
        frame,
        columns[0],
        "Coverage",
        &[
            (HorseMatcherField::Mode, "Mode"),
            (HorseMatcherField::Bookmaker, "Book"),
            (HorseMatcherField::Exchange, "Exch"),
        ],
        app,
        accent_blue(),
    );
    render_filter_card(
        frame,
        columns[1],
        "Search",
        &[
            (HorseMatcherField::Search, "Races"),
            (HorseMatcherField::Offer, "Offers"),
            (HorseMatcherField::OfferType, "Types"),
        ],
        app,
        accent_gold(),
    );
    render_filter_card(
        frame,
        columns[2],
        "Pricing",
        &[
            (HorseMatcherField::RatingType, "Type"),
            (HorseMatcherField::MinRating, "Min Rt"),
            (HorseMatcherField::MinOdds, "Min Odds"),
        ],
        app,
        accent_green(),
    );
    render_filter_card(
        frame,
        columns[3],
        "Window",
        &[
            (HorseMatcherField::Limit, "Limit"),
            (HorseMatcherField::DateFrom, "From"),
            (HorseMatcherField::DateTo, "To"),
        ],
        app,
        accent_pink(),
    );
}

fn render_filter_card(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &'static str,
    fields: &[(HorseMatcherField, &'static str)],
    app: &App,
    accent: Color,
) {
    let selected = fields
        .iter()
        .any(|(field, _)| *field == app.horse_matcher_selected_field());
    let lines = fields
        .iter()
        .map(|(field, label)| {
            let value =
                if app.horse_matcher_is_editing() && app.horse_matcher_selected_field() == *field {
                    app.horse_matcher_edit_buffer()
                        .unwrap_or_default()
                        .to_string()
                } else {
                    field.display_value(app.horse_matcher_query())
                };
            Line::from(vec![
                Span::styled(
                    format!("{label}: "),
                    Style::default()
                        .fg(muted_text())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if value.is_empty() {
                        String::from("-")
                    } else {
                        value
                    },
                    if app.horse_matcher_selected_field() == *field {
                        Style::default().fg(on_color(accent)).bg(accent)
                    } else {
                        Style::default().fg(text_color())
                    },
                ),
            ])
        })
        .collect::<Vec<_>>();

    let body = Paragraph::new(lines)
        .block(card_block(title, accent, selected))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn render_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[OddsMatcherRow],
    state: &mut TableState,
    focus: OddsMatcherFocus,
) {
    let header = Row::new(vec![
        "Date", "Race", "Runner", "Bookie", "Back", "Exchange", "Lay", "Rating", "Avail",
    ])
    .style(
        Style::default()
            .fg(accent_blue())
            .add_modifier(Modifier::BOLD),
    );

    let table_rows = rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(start_date_label(&row.start_at)),
            Cell::from(row.event_name.clone()),
            Cell::from(row.selection_name.clone()),
            Cell::from(row.back.bookmaker.display_name.clone()),
            Cell::from(format!("{:.2}", row.back.odds)),
            Cell::from(row.lay.bookmaker.display_name.clone()),
            Cell::from(format!("{:.2}", row.lay.odds)),
            Cell::from(format!("{:.1}%", row.rating)),
            Cell::from(row.availability_label()),
        ])
    });

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(28),
            Constraint::Percentage(18),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(section_block("Racing Rows", accent_green()))
    .row_highlight_style(if focus == OddsMatcherFocus::Results {
        Style::default()
            .fg(on_color(accent_gold()))
            .bg(accent_gold())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::DIM)
    });

    frame.render_stateful_widget(table, area, state);
}

fn render_details(frame: &mut Frame<'_>, area: Rect, row: Option<&OddsMatcherRow>, status: &str) {
    let lines = if let Some(row) = row {
        vec![
            Line::from(vec![label("Race  "), value(&row.event_name)]),
            Line::from(vec![label("Runner"), value(&row.selection_name)]),
            Line::from(vec![
                label("Book  "),
                value(&row.back.bookmaker.display_name),
            ]),
            Line::from(vec![
                label("Lay   "),
                value(&row.lay.bookmaker.display_name),
            ]),
            Line::from(vec![
                label("Back  "),
                value(&format!("{:.2}", row.back.odds)),
            ]),
            Line::from(vec![
                label("Lay   "),
                value(&format!("{:.2}", row.lay.odds)),
            ]),
            Line::from(vec![label("Avail "), value(&row.availability_label())]),
            Line::from(vec![label("Info  "), value(status)]),
        ]
    } else {
        vec![
            Line::raw("No Horse Matcher row selected."),
            Line::raw(status),
        ]
    };

    let body = Paragraph::new(lines)
        .block(section_block("Details", accent_gold()))
        .wrap(Wrap { trim: true });
    frame.render_widget(body, area);
}

fn badge(label: &str, value: &str, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {label}: {value} "),
        Style::default()
            .fg(on_color(color))
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn hint(label: &str, color: Color) -> Span<'static> {
    Span::styled(
        label.to_string(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn label(text: &str) -> Span<'static> {
    Span::styled(
        text.to_string(),
        Style::default()
            .fg(muted_text())
            .add_modifier(Modifier::BOLD),
    )
}

fn value(text: &str) -> Span<'static> {
    Span::styled(text.to_string(), Style::default().fg(text_color()))
}

fn section_block(title: &'static str, color: Color) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(panel_background()).fg(text_color()))
        .border_style(Style::default().fg(border_color()))
}

fn card_block(title: &'static str, accent: Color, selected: bool) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(if selected { accent } else { muted_text() })
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(panel_background()).fg(text_color()))
        .border_style(Style::default().fg(if selected { accent } else { border_color() }))
}

fn panel_background() -> Color {
    crate::theme::panel_background()
}

fn border_color() -> Color {
    crate::theme::border_color()
}

fn text_color() -> Color {
    crate::theme::text_color()
}

fn muted_text() -> Color {
    crate::theme::muted_text()
}

fn accent_blue() -> Color {
    crate::theme::accent_blue()
}

fn accent_green() -> Color {
    crate::theme::accent_green()
}

fn accent_gold() -> Color {
    crate::theme::accent_gold()
}

fn accent_pink() -> Color {
    crate::theme::accent_pink()
}

fn on_color(color: Color) -> Color {
    crate::theme::contrast_text(color)
}

fn start_date_label(start_at: &str) -> String {
    start_at
        .split_once('T')
        .map(|(date, _)| date.to_string())
        .unwrap_or_else(|| start_at.to_string())
}
