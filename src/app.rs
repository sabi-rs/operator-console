use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::widgets::ListState;
use ratatui::{Frame, Terminal};

use crate::domain::{ExchangePanelSnapshot, VenueId};
use crate::provider::{ExchangeProvider, ProviderRequest};
use crate::stub_provider::StubExchangeProvider;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Dashboard,
    Exchanges,
}

pub struct App<P> {
    provider: P,
    snapshot: ExchangePanelSnapshot,
    active_panel: Panel,
    exchange_list_state: ListState,
    running: bool,
    status_message: String,
}

impl Default for App<StubExchangeProvider> {
    fn default() -> Self {
        Self::from_provider(StubExchangeProvider::default())
            .expect("default stub provider should load dashboard")
    }
}

impl<P: ExchangeProvider> App<P> {
    pub fn from_provider(mut provider: P) -> Result<Self> {
        let snapshot = provider.handle(ProviderRequest::LoadDashboard)?;
        Ok(Self {
            provider,
            status_message: snapshot.status_line.clone(),
            snapshot,
            active_panel: Panel::Dashboard,
            exchange_list_state: ListState::default(),
            running: true,
        })
    }

    pub fn snapshot(&self) -> &ExchangePanelSnapshot {
        &self.snapshot
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn active_panel(&self) -> Panel {
        self.active_panel
    }

    pub fn set_active_panel(&mut self, panel: Panel) {
        self.active_panel = panel;
    }

    pub fn help_text(&self) -> &'static str {
        "q quit | tab switch panel | j/k move | r refresh"
    }

    pub fn selected_exchange_row(&self) -> Option<usize> {
        self.exchange_list_state.selected()
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.snapshot = self.provider.handle(ProviderRequest::Refresh)?;
        self.status_message = self.snapshot.status_line.clone();
        self.clamp_selected_exchange_row();
        Ok(())
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Dashboard => Panel::Exchanges,
            Panel::Exchanges => Panel::Dashboard,
        };
    }

    pub fn previous_panel(&mut self) {
        self.next_panel();
    }

    pub fn select_next_exchange_row(&mut self) {
        if self.snapshot.venues.is_empty() {
            self.exchange_list_state.select(None);
            return;
        }

        let next_index = match self.exchange_list_state.selected() {
            Some(index) if index + 1 < self.snapshot.venues.len() => index + 1,
            Some(index) => index,
            None => 0,
        };

        self.exchange_list_state.select(Some(next_index));
        self.sync_selected_venue();
    }

    pub fn select_previous_exchange_row(&mut self) {
        if self.snapshot.venues.is_empty() {
            self.exchange_list_state.select(None);
            return;
        }

        let previous_index = match self.exchange_list_state.selected() {
            Some(index) if index > 0 => index - 1,
            Some(index) => index,
            None => 0,
        };

        self.exchange_list_state.select(Some(previous_index));
        self.sync_selected_venue();
    }

    pub fn exchange_list_state(&mut self) -> &mut ListState {
        &mut self.exchange_list_state
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key_code(key.code)
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>) {
        ui::render(frame, self);
    }

    fn handle_key_code(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.next_panel(),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => self.previous_panel(),
            KeyCode::Char('r') => {
                if let Err(error) = self.refresh() {
                    self.status_message = format!("Refresh failed: {error}");
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.active_panel == Panel::Exchanges {
                    self.select_next_exchange_row();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.active_panel == Panel::Exchanges {
                    self.select_previous_exchange_row();
                }
            }
            _ => {}
        }
    }

    fn clamp_selected_exchange_row(&mut self) {
        match self.exchange_list_state.selected() {
            Some(index) if index >= self.snapshot.venues.len() => {
                self.exchange_list_state.select(None);
            }
            _ => {}
        }
    }

    fn sync_selected_venue(&mut self) {
        let Some(selected_index) = self.exchange_list_state.selected() else {
            return;
        };
        let Some(venue) = self.snapshot.venues.get(selected_index) else {
            return;
        };

        self.snapshot.selected_venue = Some(venue.id);

        match self.provider.handle(ProviderRequest::SelectVenue(venue.id)) {
            Ok(snapshot) => {
                self.snapshot = snapshot;
                self.status_message = self.snapshot.status_line.clone();
            }
            Err(error) => {
                self.status_message = format!("Venue sync failed: {error}");
            }
        }
    }

    pub fn selected_venue(&self) -> Option<VenueId> {
        self.exchange_list_state
            .selected()
            .and_then(|index| self.snapshot.venues.get(index).map(|venue| venue.id))
            .or(self.snapshot.selected_venue)
    }
}
