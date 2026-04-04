use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::app_state::MatcherView;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(10)]).split(area);
    let titles = MatcherView::ALL.map(MatcherView::label);
    let selected = MatcherView::ALL
        .iter()
        .position(|view| *view == app.matcher_view())
        .unwrap_or(0);

    let tabs = Tabs::new(titles.to_vec())
        .select(selected)
        .block(section_block("Matcher", accent_blue()))
        .style(Style::default().fg(muted_text()).bg(panel_background()))
        .highlight_style(
            Style::default()
                .fg(selected_text())
                .bg(selected_background())
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");
    frame.render_widget(tabs, layout[0]);
    register_tab_targets(layout[0], &titles)
        .into_iter()
        .enumerate()
        .for_each(|(index, rect)| app.register_matcher_view_target(rect, MatcherView::ALL[index]));

    match app.matcher_view() {
        MatcherView::Odds => crate::panels::oddsmatcher::render(frame, layout[1], app),
        MatcherView::Horse => crate::panels::horse_matcher::render(frame, layout[1], app),
        MatcherView::Acca => {
            let body = Paragraph::new(
                "Acca Matcher is scaffolded. The merged matcher shell is live, but acca ranking and execution wiring still need API-backed leg aggregation.",
            )
            .block(section_block("Acca Matcher", accent_gold()))
            .wrap(Wrap { trim: true });
            frame.render_widget(body, layout[1]);
        }
    }
}

fn register_tab_targets(area: Rect, titles: &[&str]) -> Vec<Rect> {
    let mut targets = Vec::new();
    let mut x = area.x.saturating_add(1);
    let y = area.y.saturating_add(1);
    for title in titles {
        let width = title.len() as u16;
        targets.push(Rect {
            x,
            y,
            width,
            height: 1,
        });
        x = x.saturating_add(width).saturating_add(2);
    }
    targets
}

fn section_block(title: &'static str, accent: Color) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(panel_background()).fg(text_color()))
        .border_style(Style::default().fg(border_color()))
}

fn panel_background() -> Color {
    crate::theme::panel_background()
}

fn text_color() -> Color {
    crate::theme::text_color()
}

fn muted_text() -> Color {
    crate::theme::muted_text()
}

fn border_color() -> Color {
    crate::theme::border_color()
}

fn accent_blue() -> Color {
    crate::theme::accent_blue()
}

fn accent_gold() -> Color {
    crate::theme::accent_gold()
}

fn selected_background() -> Color {
    crate::theme::selected_background()
}

fn selected_text() -> Color {
    crate::theme::selected_text()
}
