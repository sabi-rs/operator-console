use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use color_eyre::eyre::{eyre, Result};
use serde_json::{json, Value};

use crate::agent_browser::AgentBrowserClient;
use crate::trading_actions::{TradingActionIntent, TradingTimeInForce};

type BrowserRunner = dyn Fn(&[String]) -> Result<Value> + Send + Sync;

const SMARKETS_PRIMARY_MARKET_SELECTOR: &str =
    "div[class*='CompetitorsEventPrimaryMarket_primaryContracts']";
const SMARKETS_CONTRACT_ROW_SELECTOR: &str = "div[class*='ContractRow_row']";
const SMARKETS_SLIP_CONTRACT_SELECTOR: &str = "[aria-label='Selected contract']";
const SMARKETS_SLIP_SIDE_SELECTOR: &str = "button[aria-label='Toggle side']";
const SMARKETS_STAKE_INPUT_SELECTOR: &str = "input[aria-label='Stake input']";
const SMARKETS_PLACE_BET_LABEL: &str = "Place bet";

#[derive(Debug, Clone)]
pub struct NativeTradingResult {
    pub detail: String,
    pub action_status: String,
}

pub fn execute_smarkets_trade(
    intent: &TradingActionIntent,
    session: Option<String>,
    run_dir: Option<&Path>,
    runner: Option<Arc<BrowserRunner>>,
) -> Result<NativeTradingResult> {
    if session.as_deref().unwrap_or_default().trim().is_empty() {
        return Err(eyre!(
            "Trading action execution requires agent_browser_session in recorder config."
        ));
    }
    let client = if let Some(runner) = runner {
        AgentBrowserClient::with_runner(session, runner)
    } else {
        AgentBrowserClient::new(session)
    };

    record_trading_action_marker(
        run_dir,
        intent,
        "request",
        "requested",
        &format!(
            "{} {} {} stake {:.2} at {:.2}",
            intent.mode.label(),
            intent.side.label(),
            intent.selection_name,
            intent.stake,
            intent.expected_price
        ),
    )?;

    let result = (|| -> Result<NativeTradingResult> {
        let target_url = intent
            .deep_link_url
            .as_deref()
            .or(intent.event_url.as_deref())
            .ok_or_else(|| eyre!("Trading action target URL is missing."))?;
        assert_smarkets_target_url(target_url)?;
        client.open_url(target_url)?;
        client.wait(1_200)?;

        if !betslip_is_visible(&client)? {
            if intent.event_url.is_none() {
                return Err(eyre!(
                    "Trading action deep link did not expose a populated Smarkets bet slip."
                ));
            }
            populate_smarkets_bet_slip(&client, intent)?;
        }

        set_smarkets_stake(&client, intent.stake)?;
        let verification = verify_smarkets_bet_slip(&client)?;
        assert_smarkets_bet_slip_matches_intent(intent, &verification)?;

        if intent.mode == crate::trading_actions::TradingActionMode::Review {
            Ok(NativeTradingResult {
                detail: format!(
                    "Smarkets {} {} stake {:.2} loaded in review mode for {}.",
                    intent.side.label().to_lowercase(),
                    intent.selection_name,
                    intent.stake,
                    intent.event_name
                ),
                action_status: String::from("review_ready"),
            })
        } else {
            click_smarkets_place_bet(&client)?;
            client.wait(600)?;
            let action_status =
                if intent.execution_policy.time_in_force == TradingTimeInForce::FillOrKill {
                    "submitted_fill_or_kill"
                } else {
                    "submitted"
                };
            Ok(NativeTradingResult {
                detail: format!(
                    "Smarkets {} {} stake {:.2} submitted for {}.",
                    intent.side.label().to_lowercase(),
                    intent.selection_name,
                    intent.stake,
                    intent.event_name
                ),
                action_status: String::from(action_status),
            })
        }
    })();

    match result {
        Ok(outcome) => {
            record_trading_action_marker(
                run_dir,
                intent,
                "response",
                &outcome.action_status,
                &outcome.detail,
            )?;
            Ok(outcome)
        }
        Err(error) => {
            let _ = record_trading_action_marker(
                run_dir,
                intent,
                "response",
                "error",
                &error.to_string(),
            );
            Err(error)
        }
    }
}

fn assert_smarkets_target_url(url: &str) -> Result<()> {
    if !(url.starts_with("https://smarkets.com") || url.starts_with("https://www.smarkets.com")) {
        return Err(eyre!(
            "Trading action target URL must point to smarkets.com."
        ));
    }
    Ok(())
}

fn betslip_is_visible(client: &AgentBrowserClient) -> Result<bool> {
    Ok(client
        .evaluate(&format!(
            "(() => {{\
            const contract = document.querySelector({:?});\
            const side = document.querySelector({:?});\
            const stake = document.querySelector({:?});\
            return !!contract && !!side && !!stake;\
            }})()",
            SMARKETS_SLIP_CONTRACT_SELECTOR,
            SMARKETS_SLIP_SIDE_SELECTOR,
            SMARKETS_STAKE_INPUT_SELECTOR
        ))?
        .as_bool()
        .unwrap_or(false))
}

fn populate_smarkets_bet_slip(
    client: &AgentBrowserClient,
    intent: &TradingActionIntent,
) -> Result<()> {
    wait_for(
        client,
        &format!(
            "(() => document.querySelectorAll({:?}).length > 0)()",
            SMARKETS_CONTRACT_ROW_SELECTOR
        ),
        6_000,
        "Smarkets contract rows did not load in time.",
    )?;

    client.evaluate(&format!(
        "(() => {{\
        const selector = {:?};\
        const side = {:?};\
        const primarySelector = {:?};\
        const rowSelector = {:?};\
        const primary = document.querySelector(primarySelector);\
        const containers = primary ? [primary] : [document];\
        for (const container of containers) {{\
          const rows = Array.from(container.querySelectorAll(rowSelector));\
          const row = rows.find((candidate) => (candidate.innerText || '').split('\\n').map((value) => value.trim()).includes(selector));\
          if (!row) continue;\
          const button = row.querySelector(`button[class*='BetButton_${{side}}']`);\
          if (!(button instanceof HTMLElement)) throw new Error(`Requested ${{side}} button was not found for ${{selector}}`);\
          button.click();\
          return {{ rowText: row.innerText || '' }};\
        }}\
        throw new Error(`Smarkets contract row was not found for ${{selector}}`);\
        }})()",
        intent.selection_name,
        intent.side.label().to_lowercase(),
        SMARKETS_PRIMARY_MARKET_SELECTOR,
        SMARKETS_CONTRACT_ROW_SELECTOR
    ))?;

    wait_for(
        client,
        &format!(
            "(() => {{\
            const contract = document.querySelector({:?});\
            return contract && (contract.innerText || '').trim().length > 0;\
            }})()",
            SMARKETS_SLIP_CONTRACT_SELECTOR
        ),
        5_000,
        "Smarkets bet slip did not populate after selecting the contract.",
    )?;
    Ok(())
}

fn set_smarkets_stake(client: &AgentBrowserClient, stake: f64) -> Result<()> {
    wait_for(
        client,
        &format!(
            "(() => {{\
            const input = document.querySelector({:?});\
            return input instanceof HTMLInputElement;\
            }})()",
            SMARKETS_STAKE_INPUT_SELECTOR
        ),
        5_000,
        "Smarkets stake input was not found.",
    )?;
    client.set_input_value(SMARKETS_STAKE_INPUT_SELECTOR, &format!("{stake:.2}"))
}

fn verify_smarkets_bet_slip(client: &AgentBrowserClient) -> Result<Value> {
    client.evaluate(&format!(
        "(() => {{\
        const contract = document.querySelector({:?});\
        const side = document.querySelector({:?});\
        const stakeInput = document.querySelector({:?});\
        const bodyText = document.body ? document.body.innerText || '' : '';\
        const priceMatch = bodyText.match(/Current price\\s*:?\\s*(\\d+(?:\\.\\d+)?)/i);\
        return {{\
          contractText: contract ? (contract.innerText || '').trim() : '',\
          sideText: side ? (side.innerText || '').trim() : '',\
          stakeValue: stakeInput instanceof HTMLInputElement ? stakeInput.value.trim() : '',\
          priceValue: priceMatch ? priceMatch[1] : '',\
          bodyText\
        }};\
        }})()",
        SMARKETS_SLIP_CONTRACT_SELECTOR, SMARKETS_SLIP_SIDE_SELECTOR, SMARKETS_STAKE_INPUT_SELECTOR
    ))
}

fn assert_smarkets_bet_slip_matches_intent(
    intent: &TradingActionIntent,
    verification: &Value,
) -> Result<()> {
    let contract_text = verification
        .get("contractText")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let side_text = verification
        .get("sideText")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();
    let stake_value = verification
        .get("stakeValue")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let price_value = verification
        .get("priceValue")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if !contract_text.contains(&intent.selection_name) {
        return Err(eyre!(
            "Smarkets bet slip contract mismatch: expected {:?}, got {:?}.",
            intent.selection_name,
            contract_text
        ));
    }
    if !side_text.contains(&intent.side.label().to_lowercase()) {
        return Err(eyre!(
            "Smarkets bet slip side mismatch: expected {:?}, got {:?}.",
            intent.side.label().to_lowercase(),
            side_text
        ));
    }
    if stake_value != format!("{:.2}", intent.stake) {
        return Err(eyre!(
            "Smarkets bet slip stake mismatch: expected {:.2}, got {:?}.",
            intent.stake,
            stake_value
        ));
    }
    let observed_price = price_value.parse::<f64>().map_err(|_| {
        eyre!(
            "Smarkets bet slip current price was not numeric: {:?}.",
            price_value
        )
    })?;
    if (observed_price - intent.expected_price).abs() > intent.execution_policy.max_price_drift {
        return Err(eyre!(
            "Smarkets bet slip quote drift exceeded the Rust-authored execution policy."
        ));
    }
    Ok(())
}

fn click_smarkets_place_bet(client: &AgentBrowserClient) -> Result<()> {
    client.evaluate(&format!(
        "(() => {{\
        const button = Array.from(document.querySelectorAll('button')).find((candidate) => ((candidate.innerText || '').trim() === {:?}));\
        if (!(button instanceof HTMLElement)) throw new Error('Smarkets Place bet button was not found');\
        button.click();\
        return true;\
        }})()",
        SMARKETS_PLACE_BET_LABEL
    ))?;
    Ok(())
}

fn wait_for(
    client: &AgentBrowserClient,
    predicate_js: &str,
    timeout_ms: u64,
    error_message: &str,
) -> Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    while std::time::Instant::now() <= deadline {
        if client.evaluate(predicate_js)?.as_bool().unwrap_or(false) {
            return Ok(());
        }
        client.wait(125)?;
    }
    Err(eyre!(error_message.to_string()))
}

fn record_trading_action_marker(
    run_dir: Option<&Path>,
    intent: &TradingActionIntent,
    phase: &str,
    status: &str,
    detail: &str,
) -> Result<()> {
    let Some(run_dir) = run_dir else {
        return Ok(());
    };
    fs::create_dir_all(run_dir)?;
    append_jsonl(
        &run_dir.join("events.jsonl"),
        json!({
            "captured_at": now_iso(),
            "source": "operator_console",
            "kind": "operator_interaction",
            "page": "worker_request",
            "action": "place_bet",
            "status": format!("{phase}:{status}"),
            "detail": detail,
            "request_id": intent.request_id,
            "reference_id": if intent.source_ref.is_empty() { Value::Null } else { Value::String(intent.source_ref.clone()) },
            "metadata": {
                "venue": intent.venue.as_str(),
                "mode": format!("{:?}", intent.mode).to_lowercase(),
                "side": format!("{:?}", intent.side).to_lowercase(),
                "stake": intent.stake,
                "expected_price": intent.expected_price,
                "event_name": intent.event_name,
                "market_name": intent.market_name,
                "selection_name": intent.selection_name,
            }
        }),
    )?;
    let transport_path = run_dir.join("transport.jsonl");
    if transport_path.exists() {
        append_jsonl(
            &transport_path,
            json!({
                "captured_at": now_iso(),
                "kind": "interaction_marker",
                "action": "place_bet",
                "phase": phase,
                "detail": detail,
                "request_id": intent.request_id,
                "reference_id": if intent.source_ref.is_empty() { Value::Null } else { Value::String(intent.source_ref.clone()) },
                "metadata": {
                    "venue": intent.venue.as_str(),
                    "mode": format!("{:?}", intent.mode).to_lowercase(),
                    "side": format!("{:?}", intent.side).to_lowercase(),
                    "stake": intent.stake,
                    "selection_name": intent.selection_name,
                    "event_name": intent.event_name,
                    "status": status,
                }
            }),
        )?;
    }
    Ok(())
}

fn append_jsonl(path: &Path, value: Value) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(serde_json::to_string(&value)?.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    chrono_like_iso(now)
}

fn chrono_like_iso(epoch_secs: u64) -> String {
    use std::process::Command;
    let output = Command::new("date")
        .arg("-u")
        .arg("-d")
        .arg(format!("@{epoch_secs}"))
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output();
    match output {
        Ok(value) if value.status.success() => {
            String::from_utf8_lossy(&value.stdout).trim().to_string()
        }
        _ => String::from("1970-01-01T00:00:00Z"),
    }
}
