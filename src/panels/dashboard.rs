use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::domain::ExchangePanelSnapshot;

pub fn render(frame: &mut Frame<'_>, area: Rect, snapshot: &ExchangePanelSnapshot) {
    let connected_count = snapshot
        .venues
        .iter()
        .filter(|venue| matches!(venue.status, crate::domain::VenueStatus::Connected))
        .count();

    let body = Paragraph::new(vec![
        Line::raw(format!("Worker: {}", snapshot.worker.name)),
        Line::raw(format!("Worker status: {:?}", snapshot.worker.status)),
        Line::raw(format!("Venues loaded: {}", snapshot.venues.len())),
        Line::raw(format!("Connected venues: {}", connected_count)),
        Line::raw(format!(
            "Selected venue: {}",
            snapshot
                .selected_venue
                .map(|venue| venue.as_str().to_string())
                .unwrap_or_else(|| String::from("none"))
        )),
        Line::raw(String::from(
            "The exchanges panel is the live venue workspace.",
        )),
    ])
    .block(Block::default().title("Dashboard").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(body, area);
}
