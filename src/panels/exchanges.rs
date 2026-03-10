use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::domain::{ExchangePanelSnapshot, VenueSummary};

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &ExchangePanelSnapshot,
    list_state: &mut ListState,
) {
    let layout =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).split(area);

    let items = snapshot.venues.iter().map(render_venue_item);
    let venue_list = List::new(items)
        .block(Block::default().title("Venues").borders(Borders::ALL))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(venue_list, layout[0], list_state);

    let selected_details = selected_details(snapshot, list_state.selected());
    let details = Paragraph::new(selected_details)
        .block(
            Block::default()
                .title("Venue Details")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(details, layout[1]);
}

fn render_venue_item(venue: &VenueSummary) -> ListItem<'static> {
    ListItem::new(format!(
        "{} [{}] events={} markets={}",
        venue.label,
        venue.id.as_str(),
        venue.event_count,
        venue.market_count,
    ))
}

fn selected_details(
    snapshot: &ExchangePanelSnapshot,
    selected_index: Option<usize>,
) -> Vec<Line<'static>> {
    let venue = selected_index
        .and_then(|index| snapshot.venues.get(index))
        .or_else(|| {
            snapshot
                .selected_venue
                .and_then(|selected| snapshot.venues.iter().find(|venue| venue.id == selected))
        });

    let Some(venue) = venue else {
        return vec![
            Line::raw("No venue selected."),
            Line::raw("Press j/k or arrow keys in the Exchanges panel."),
        ];
    };

    let latest_event = snapshot
        .events
        .first()
        .map(|event| format!("{} ({})", event.label, event.competition))
        .unwrap_or_else(|| String::from("No event selected"));
    let latest_market = snapshot
        .markets
        .first()
        .map(|market| format!("{} ({} contracts)", market.name, market.contract_count))
        .unwrap_or_else(|| String::from("No market snapshot loaded"));

    vec![
        Line::raw(format!("Venue: {}", venue.label)),
        Line::raw(format!("Status: {:?}", venue.status)),
        Line::raw(format!("Detail: {}", venue.detail)),
        Line::raw(format!("Latest event: {}", latest_event)),
        Line::raw(format!("Latest market: {}", latest_market)),
        Line::raw(format!("Worker: {}", snapshot.worker.detail)),
    ]
}
