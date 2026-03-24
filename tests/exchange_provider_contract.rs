use operator_console::domain::{ExchangePanelSnapshot, VenueId};
use operator_console::provider::{ExchangeProvider, ProviderRequest};

struct NullProvider;

impl ExchangeProvider for NullProvider {
    fn handle(&mut self, request: ProviderRequest) -> color_eyre::Result<ExchangePanelSnapshot> {
        match request {
            ProviderRequest::LoadDashboard => Ok(ExchangePanelSnapshot::empty()),
            _ => unreachable!("test provider only handles initial load"),
        }
    }
}

#[test]
fn provider_contract_returns_snapshot() {
    let mut provider = NullProvider;
    let snapshot = provider
        .handle(ProviderRequest::LoadDashboard)
        .expect("snapshot");

    assert!(snapshot.venues.is_empty());
    assert_eq!(snapshot.selected_venue, None);
}

#[test]
fn venue_id_display_labels_are_stable() {
    assert_eq!(VenueId::Smarkets.as_str(), "smarkets");
    assert_eq!(VenueId::Bet365.as_str(), "bet365");
    assert_eq!(VenueId::Betfair.as_str(), "betfair");
    assert_eq!(VenueId::Betano.as_str(), "betano");
    assert_eq!(VenueId::Bet10.as_str(), "bet10");
    assert_eq!(VenueId::Betmgm.as_str(), "betmgm");
    assert_eq!(VenueId::Betfred.as_str(), "betfred");
    assert_eq!(VenueId::Betvictor.as_str(), "betvictor");
    assert_eq!(VenueId::Boylesports.as_str(), "boylesports");
    assert_eq!(VenueId::Coral.as_str(), "coral");
    assert_eq!(VenueId::Fanteam.as_str(), "fanteam");
    assert_eq!(VenueId::Ladbrokes.as_str(), "ladbrokes");
    assert_eq!(VenueId::Kwik.as_str(), "kwik");
    assert_eq!(VenueId::Bet600.as_str(), "bet600");
    assert_eq!(VenueId::Leovegas.as_str(), "leovegas");
    assert_eq!(VenueId::Matchbook.as_str(), "matchbook");
    assert_eq!(VenueId::Betdaq.as_str(), "betdaq");
    assert_eq!(VenueId::Betway.as_str(), "betway");
    assert_eq!(VenueId::Betuk.as_str(), "betuk");
    assert_eq!(VenueId::Midnite.as_str(), "midnite");
    assert_eq!(VenueId::Paddypower.as_str(), "paddypower");
    assert_eq!(VenueId::Skybet.as_str(), "skybet");
    assert_eq!(VenueId::Sportingindex.as_str(), "sportingindex");
    assert_eq!(VenueId::Talksportbet.as_str(), "talksportbet");
    assert_eq!(VenueId::Williamhill.as_str(), "williamhill");
}
