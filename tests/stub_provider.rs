use operator_console::provider::{ExchangeProvider, ProviderRequest};
use operator_console::stub_provider::StubExchangeProvider;

#[test]
fn stub_provider_uses_transport_shaped_snapshot() {
    let mut provider = StubExchangeProvider::default();
    let snapshot = provider
        .handle(ProviderRequest::LoadDashboard)
        .expect("snapshot");

    assert!(!snapshot.venues.is_empty());
    assert_eq!(snapshot.venues[0].label, "Smarkets");
}
