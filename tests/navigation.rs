use operator_console::app::{App, Panel};

#[test]
fn tab_navigation_switches_to_exchanges_panel() {
    let mut app = App::default();

    assert_eq!(app.active_panel(), Panel::Dashboard);

    app.next_panel();
    assert_eq!(app.active_panel(), Panel::Exchanges);

    app.previous_panel();
    assert_eq!(app.active_panel(), Panel::Dashboard);
}
