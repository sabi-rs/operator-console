#[derive(Debug, Clone, PartialEq, Eq)]
struct EventParticipants {
    home: String,
    away: String,
}

pub fn normalize_key(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace("versus", "v")
        .replace("vs.", "v")
        .replace("vs", "v")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn text_matches(left: &str, right: &str) -> bool {
    let left = normalize_key(left);
    let right = normalize_key(right);
    !left.is_empty()
        && !right.is_empty()
        && (left == right
            || (left.len() > 1 && left.contains(&right))
            || (right.len() > 1 && right.contains(&left)))
}

pub fn normalize_market(value: &str) -> String {
    let normalized = normalize_key(value);
    if matches!(
        normalized.as_str(),
        "full time result"
            | "fulltime result"
            | "match result"
            | "match odds"
            | "match betting"
            | "1 x 2"
            | "1x2"
            | "three way"
            | "to win"
            | "winner"
            | "moneyline"
            | "h2h"
    ) {
        return String::from("match odds");
    }
    normalized
}

pub fn market_matches(left: &str, right: &str) -> bool {
    let left = normalize_market(left);
    let right = normalize_market(right);
    !left.is_empty() && !right.is_empty() && left == right
}

pub fn event_matches(left: &str, right: &str) -> bool {
    if left.trim().is_empty() || right.trim().is_empty() {
        return true;
    }
    if text_matches(left, right) {
        return true;
    }
    match (event_participants(left), event_participants(right)) {
        (Some(left), Some(right)) => {
            [left.home.as_str(), left.away.as_str()] == [right.home.as_str(), right.away.as_str()]
                || [left.home.as_str(), left.away.as_str()]
                    == [right.away.as_str(), right.home.as_str()]
        }
        _ => false,
    }
}

pub fn selection_matches(left: &str, right: &str) -> bool {
    let left = normalize_key(left);
    let right = normalize_key(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    left == right
        || (left.len() > 1 && left.contains(&right))
        || (right.len() > 1 && right.contains(&left))
        || (is_draw_alias(&left) && is_draw_alias(&right))
        || (is_home_alias(&left) && is_home_alias(&right))
        || (is_away_alias(&left) && is_away_alias(&right))
}

pub fn selection_matches_with_context(
    left_selection: &str,
    left_event: &str,
    left_market: &str,
    right_selection: &str,
    right_event: &str,
    right_market: &str,
) -> bool {
    if selection_matches(left_selection, right_selection) {
        return true;
    }
    let left = canonical_selection(left_selection, left_event, left_market);
    let right = canonical_selection(right_selection, right_event, right_market);
    matches!((left, right), (Some(left), Some(right)) if left == right)
}

fn canonical_selection(selection: &str, event: &str, market: &str) -> Option<String> {
    let normalized = normalize_key(selection);
    if normalized.is_empty() {
        return None;
    }
    if is_draw_alias(&normalized) {
        return Some(String::from("draw"));
    }
    let participants = event_participants(event);
    if market_matches(market, "match odds") {
        if is_home_alias(&normalized) {
            return participants
                .as_ref()
                .map(|participants| participants.home.clone());
        }
        if is_away_alias(&normalized) {
            return participants
                .as_ref()
                .map(|participants| participants.away.clone());
        }
    }
    if let Some(participants) = participants {
        if text_matches(&participants.home, &normalized) {
            return Some(participants.home);
        }
        if text_matches(&participants.away, &normalized) {
            return Some(participants.away);
        }
    }
    Some(normalized)
}

fn event_participants(value: &str) -> Option<EventParticipants> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    for separator in [" versus ", " vs. ", " vs ", " v ", " @ ", " at "] {
        if let Some((left, right)) = split_once_case_insensitive(trimmed, separator) {
            let left = normalize_key(left);
            let right = normalize_key(right);
            if left.is_empty() || right.is_empty() {
                continue;
            }
            if matches!(separator, " @ " | " at ") {
                return Some(EventParticipants {
                    home: right,
                    away: left,
                });
            }
            return Some(EventParticipants {
                home: left,
                away: right,
            });
        }
    }
    None
}

fn split_once_case_insensitive<'a>(value: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find(separator)?;
    let end = start + separator.len();
    Some((&value[..start], &value[end..]))
}

fn is_draw_alias(value: &str) -> bool {
    matches!(value, "x" | "draw" | "tie")
}

fn is_home_alias(value: &str) -> bool {
    matches!(value, "1" | "home" | "home team")
}

fn is_away_alias(value: &str) -> bool {
    matches!(value, "2" | "away" | "away team")
}

#[cfg(test)]
mod tests {
    use super::{event_matches, market_matches, normalize_market, selection_matches_with_context};

    #[test]
    fn event_matching_handles_reversed_home_away_formats() {
        assert!(event_matches("Malta vs Luxembourg", "Luxembourg @ Malta",));
    }

    #[test]
    fn market_matching_handles_owls_h2h_alias() {
        assert!(market_matches("Full-time result", "h2h"));
        assert_eq!(normalize_market("1x2"), "match odds");
    }

    #[test]
    fn selection_matching_handles_match_odds_aliases() {
        assert!(selection_matches_with_context(
            "X",
            "Malta vs Luxembourg",
            "Full-time result",
            "Draw",
            "Luxembourg @ Malta",
            "h2h",
        ));
        assert!(selection_matches_with_context(
            "1",
            "Malta vs Luxembourg",
            "Full-time result",
            "Malta",
            "Luxembourg @ Malta",
            "match betting",
        ));
        assert!(selection_matches_with_context(
            "2",
            "Malta vs Luxembourg",
            "Full-time result",
            "Luxembourg",
            "Luxembourg @ Malta",
            "match betting",
        ));
    }
}
