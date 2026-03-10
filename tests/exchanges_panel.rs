use operator_console::app::{App, Panel};

#[test]
fn exchanges_panel_tracks_selected_row() {
    let mut app = App::default();
    app.set_active_panel(Panel::Exchanges);

    assert_eq!(app.selected_exchange_row(), None);

    app.select_next_exchange_row();
    assert_eq!(app.selected_exchange_row(), Some(0));
}
