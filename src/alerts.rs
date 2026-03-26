use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertConfig {
    pub enabled: bool,
    pub desktop_notifications: bool,
    pub sound_effects: bool,
    pub bet_placed: bool,
    pub bet_settled: bool,
    pub recorder_failures: bool,
    pub provider_errors: bool,
    pub matchbook_failures: bool,
    pub snapshot_stale: bool,
    pub exit_recommendations: bool,
    pub decision_queue: bool,
    pub tracked_bets: bool,
    pub opportunity_detected: bool,
    pub opportunity_threshold_pct: f64,
    pub watched_movement: bool,
    pub watched_movement_threshold_pct: f64,
    pub owls_errors: bool,
    pub cooldown_seconds: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            desktop_notifications: true,
            sound_effects: true,
            bet_placed: true,
            bet_settled: true,
            recorder_failures: true,
            provider_errors: true,
            matchbook_failures: true,
            snapshot_stale: true,
            exit_recommendations: true,
            decision_queue: true,
            tracked_bets: true,
            opportunity_detected: true,
            opportunity_threshold_pct: 3.0,
            watched_movement: true,
            watched_movement_threshold_pct: 5.0,
            owls_errors: true,
            cooldown_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertField {
    Enabled,
    DesktopNotifications,
    SoundEffects,
    BetPlaced,
    BetSettled,
    RecorderFailures,
    ProviderErrors,
    MatchbookFailures,
    SnapshotStale,
    ExitRecommendations,
    DecisionQueue,
    TrackedBets,
    OpportunityDetected,
    OpportunityThresholdPct,
    WatchedMovement,
    WatchedMovementThresholdPct,
    OwlsErrors,
    CooldownSeconds,
}

impl AlertField {
    pub const ALL: [Self; 18] = [
        Self::Enabled,
        Self::DesktopNotifications,
        Self::SoundEffects,
        Self::BetPlaced,
        Self::BetSettled,
        Self::RecorderFailures,
        Self::ProviderErrors,
        Self::MatchbookFailures,
        Self::SnapshotStale,
        Self::ExitRecommendations,
        Self::DecisionQueue,
        Self::TrackedBets,
        Self::OpportunityDetected,
        Self::OpportunityThresholdPct,
        Self::WatchedMovement,
        Self::WatchedMovementThresholdPct,
        Self::OwlsErrors,
        Self::CooldownSeconds,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::DesktopNotifications => "Desktop Notify",
            Self::SoundEffects => "Sounds",
            Self::BetPlaced => "Bet Placed",
            Self::BetSettled => "Bet Settled",
            Self::RecorderFailures => "Recorder Failures",
            Self::ProviderErrors => "Provider Errors",
            Self::MatchbookFailures => "Matchbook Failures",
            Self::SnapshotStale => "Snapshot Stale",
            Self::ExitRecommendations => "Exit Recs",
            Self::DecisionQueue => "Decision Queue",
            Self::TrackedBets => "Tracked Bets",
            Self::OpportunityDetected => "Opportunity",
            Self::OpportunityThresholdPct => "Opportunity %",
            Self::WatchedMovement => "Watch Move",
            Self::WatchedMovementThresholdPct => "Watch Move %",
            Self::OwlsErrors => "Owls Errors",
            Self::CooldownSeconds => "Cooldown Seconds",
        }
    }

    pub fn display_value(self, config: &AlertConfig) -> String {
        match self {
            Self::Enabled => format_bool(config.enabled),
            Self::DesktopNotifications => format_bool(config.desktop_notifications),
            Self::SoundEffects => format_bool(config.sound_effects),
            Self::BetPlaced => format_bool(config.bet_placed),
            Self::BetSettled => format_bool(config.bet_settled),
            Self::RecorderFailures => format_bool(config.recorder_failures),
            Self::ProviderErrors => format_bool(config.provider_errors),
            Self::MatchbookFailures => format_bool(config.matchbook_failures),
            Self::SnapshotStale => format_bool(config.snapshot_stale),
            Self::ExitRecommendations => format_bool(config.exit_recommendations),
            Self::DecisionQueue => format_bool(config.decision_queue),
            Self::TrackedBets => format_bool(config.tracked_bets),
            Self::OpportunityDetected => format_bool(config.opportunity_detected),
            Self::OpportunityThresholdPct => format!("{:.2}", config.opportunity_threshold_pct),
            Self::WatchedMovement => format_bool(config.watched_movement),
            Self::WatchedMovementThresholdPct => {
                format!("{:.2}", config.watched_movement_threshold_pct)
            }
            Self::OwlsErrors => format_bool(config.owls_errors),
            Self::CooldownSeconds => config.cooldown_seconds.to_string(),
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::Enabled => "Master switch for alert evaluation and delivery.",
            Self::DesktopNotifications => "Send notifications to the desktop notification daemon.",
            Self::SoundEffects => "Play different sounds for different alert types.",
            Self::BetPlaced => "Alert when a confirmed bet placement succeeds.",
            Self::BetSettled => "Alert when a tracked bet moves into a settled state.",
            Self::RecorderFailures => "Alert when the recorder stops or errors.",
            Self::ProviderErrors => "Alert when snapshot/provider requests fail.",
            Self::MatchbookFailures => "Alert when Matchbook sync fails.",
            Self::SnapshotStale => "Alert when the active snapshot turns stale.",
            Self::ExitRecommendations => "Alert when new actionable exit recommendations appear.",
            Self::DecisionQueue => "Alert when actionable decisions enter the queue.",
            Self::TrackedBets => "Alert when tracked bets increase.",
            Self::OpportunityDetected => {
                "Alert when a live EV/opportunity metric crosses the configured threshold."
            }
            Self::OpportunityThresholdPct => "Minimum EV/opportunity percentage required to alert.",
            Self::WatchedMovement => "Alert when a watched result moves sharply between snapshots.",
            Self::WatchedMovementThresholdPct => {
                "Minimum watched-odds percentage move required to alert."
            }
            Self::OwlsErrors => "Alert when Owls endpoint errors appear or increase.",
            Self::CooldownSeconds => {
                "Suppress duplicate alerts for the same rule inside this window."
            }
        }
    }

    pub fn apply_value(self, config: &mut AlertConfig, value: &str) -> Result<()> {
        match self {
            Self::Enabled => config.enabled = parse_bool(value)?,
            Self::DesktopNotifications => config.desktop_notifications = parse_bool(value)?,
            Self::SoundEffects => config.sound_effects = parse_bool(value)?,
            Self::BetPlaced => config.bet_placed = parse_bool(value)?,
            Self::BetSettled => config.bet_settled = parse_bool(value)?,
            Self::RecorderFailures => config.recorder_failures = parse_bool(value)?,
            Self::ProviderErrors => config.provider_errors = parse_bool(value)?,
            Self::MatchbookFailures => config.matchbook_failures = parse_bool(value)?,
            Self::SnapshotStale => config.snapshot_stale = parse_bool(value)?,
            Self::ExitRecommendations => config.exit_recommendations = parse_bool(value)?,
            Self::DecisionQueue => config.decision_queue = parse_bool(value)?,
            Self::TrackedBets => config.tracked_bets = parse_bool(value)?,
            Self::OpportunityDetected => config.opportunity_detected = parse_bool(value)?,
            Self::OpportunityThresholdPct => {
                config.opportunity_threshold_pct = parse_f64(value, "Opportunity threshold")?;
            }
            Self::WatchedMovement => config.watched_movement = parse_bool(value)?,
            Self::WatchedMovementThresholdPct => {
                config.watched_movement_threshold_pct = parse_f64(value, "Watch move threshold")?;
            }
            Self::OwlsErrors => config.owls_errors = parse_bool(value)?,
            Self::CooldownSeconds => {
                config.cooldown_seconds = value
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| eyre!("Cooldown must be a whole number of seconds."))?;
            }
        }
        Ok(())
    }

    pub fn suggestions(self) -> &'static [&'static str] {
        match self {
            Self::OpportunityThresholdPct => &["1.0", "2.5", "3.0", "5.0", "8.0"],
            Self::WatchedMovementThresholdPct => &["2.0", "5.0", "8.0", "10.0", "15.0"],
            Self::CooldownSeconds => &["0", "15", "30", "60", "300"],
            _ => &["true", "false"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlertEditorState {
    selected_field: AlertField,
    pub editing: bool,
    pub buffer: String,
    pub replace_on_input: bool,
}

impl Default for AlertEditorState {
    fn default() -> Self {
        Self {
            selected_field: AlertField::Enabled,
            editing: false,
            buffer: String::new(),
            replace_on_input: false,
        }
    }
}

impl AlertEditorState {
    pub fn selected_field(&self) -> AlertField {
        self.selected_field
    }

    pub fn select_next_field(&mut self) {
        self.selected_field = next_from(self.selected_field, &AlertField::ALL);
    }

    pub fn select_previous_field(&mut self) {
        self.selected_field = previous_from(self.selected_field, &AlertField::ALL);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Critical,
}

impl NotificationLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warn",
            Self::Critical => "crit",
        }
    }

    pub fn notify_send_urgency(self) -> &'static str {
        match self {
            Self::Info => "normal",
            Self::Warning => "normal",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationEntry {
    pub created_at: String,
    pub rule_key: String,
    pub level: NotificationLevel,
    pub title: String,
    pub detail: String,
    pub unread: bool,
}

pub fn default_config_path() -> PathBuf {
    if let Some(path) = env::var_os("SABI_ALERTS_CONFIG_PATH") {
        return PathBuf::from(path);
    }
    config_root().join("sabi").join("alerts.json")
}

pub fn load_alert_config_or_default(path: &Path) -> Result<(AlertConfig, String)> {
    if !path.exists() {
        return Ok((
            AlertConfig::default(),
            String::from("Using default alerts config."),
        ));
    }

    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str::<AlertConfig>(&content)?;
    Ok((
        config,
        format!("Loaded alerts config from {}.", path.display()),
    ))
}

pub fn save_alert_config(path: &Path, config: &AlertConfig) -> Result<String> {
    write_private_file(path, &(serde_json::to_string_pretty(config)? + "\n"))?;
    Ok(format!("Saved alerts config to {}.", path.display()))
}

fn config_root() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn ensure_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn write_private_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(contents.as_bytes())?;
    file.flush()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(eyre!("Expected a boolean value.")),
    }
}

fn parse_f64(value: &str, label: &str) -> Result<f64> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| eyre!("{label} must be numeric."))
}

fn format_bool(value: bool) -> String {
    if value {
        String::from("true")
    } else {
        String::from("false")
    }
}

fn next_from<T: Copy + PartialEq>(value: T, all: &[T]) -> T {
    let index = all
        .iter()
        .position(|candidate| candidate == &value)
        .unwrap_or(0);
    all[(index + 1) % all.len()]
}

fn previous_from<T: Copy + PartialEq>(value: T, all: &[T]) -> T {
    let index = all
        .iter()
        .position(|candidate| candidate == &value)
        .unwrap_or(0);
    if index == 0 {
        all[all.len() - 1]
    } else {
        all[index - 1]
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{load_alert_config_or_default, save_alert_config, AlertConfig, AlertField};

    #[test]
    fn save_and_load_alert_config_round_trip() {
        let temp_dir = tempdir().expect("temp dir");
        let path = temp_dir.path().join("alerts.json");
        let config = AlertConfig {
            desktop_notifications: true,
            sound_effects: true,
            cooldown_seconds: 15,
            ..AlertConfig::default()
        };

        save_alert_config(&path, &config).expect("save config");
        let (loaded, note) = load_alert_config_or_default(&path).expect("load config");

        assert_eq!(loaded, config);
        assert!(note.contains("Loaded alerts config"));
    }

    #[test]
    fn alert_field_applies_boolean_and_numeric_values() {
        let mut config = AlertConfig::default();

        AlertField::SoundEffects
            .apply_value(&mut config, "true")
            .expect("apply bool");
        AlertField::CooldownSeconds
            .apply_value(&mut config, "30")
            .expect("apply cooldown");

        assert!(config.desktop_notifications);
        assert_eq!(config.cooldown_seconds, 30);
    }
}
