use crate::app::populate_snapshot_enrichment;
use crate::domain::ExchangePanelSnapshot;
use crate::exchange_api::MatchbookAccountState;
use crate::owls::OwlsDashboard;

pub fn project_snapshot(
    base: &ExchangePanelSnapshot,
    owls_dashboard: &OwlsDashboard,
    matchbook_account_state: Option<&MatchbookAccountState>,
) -> ExchangePanelSnapshot {
    let mut snapshot = base.clone();
    populate_snapshot_enrichment(&mut snapshot, owls_dashboard, matchbook_account_state);
    snapshot
}
