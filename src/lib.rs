pub mod app;
mod app_state;
pub mod calculator;
pub mod domain;
pub mod horse_matcher;
pub mod oddsmatcher;
pub mod panels;
pub mod provider;
pub mod recorder;
pub mod stub_provider;
pub mod trading_actions;
pub mod transport;
pub mod ui;
pub mod worker_client;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
