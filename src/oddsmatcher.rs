use color_eyre::eyre::{eyre, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub const ODDSMATCHER_GRAPHQL_URL: &str =
    "https://api.oddsplatform.profitaccumulator.com/graphql";

pub const GET_BEST_MATCHES_QUERY: &str = r#"
query GetBestMatches(
  $bookmaker: [String!]
  $exchange: [String!]
  $ratingType: String
  $minRating: String
  $maxRating: String
  $minOdds: String
  $maxOdds: String
  $minLiquidity: String
  $limit: Int
  $skip: Int
  $updatedWithinSeconds: Int
  $excludeDraw: Boolean
  $permittedSports: [String!]
  $permittedMarketGroups: [String!]
  $permittedEventGroups: [String!]
  $permittedCountries: [String!]
  $permittedEventIds: [String!]
  $commissionRates: CommissionRatesInput
) {
  getBestMatches(
    bookmaker: $bookmaker
    exchange: $exchange
    ratingType: $ratingType
    minRating: $minRating
    maxRating: $maxRating
    minOdds: $minOdds
    maxOdds: $maxOdds
    minLiquidity: $minLiquidity
    limit: $limit
    skip: $skip
    updatedWithinSeconds: $updatedWithinSeconds
    excludeDraw: $excludeDraw
    permittedSports: $permittedSports
    permittedMarketGroups: $permittedMarketGroups
    permittedEventGroups: $permittedEventGroups
    permittedCountries: $permittedCountries
    permittedEventIds: $permittedEventIds
    commissionRates: $commissionRates
  ) {
    eventName
    id
    startAt
    selectionId
    marketId
    eventId
    back {
      updatedAt
      odds
      fetchedAt
      deepLink
      bookmaker {
        active
        code
        displayName
        id
        logo
      }
    }
    lay {
      bookmaker {
        active
        code
        displayName
        id
        logo
      }
      deepLink
      fetchedAt
      updatedAt
      odds
      liquidity
      betSlip {
        marketId
        selectionId
      }
    }
    eventGroup {
      displayName
      id
      sourceName
      sport
    }
    marketGroup {
      displayName
      id
      sport
    }
    marketName
    rating
    selectionName
    snr
    sport {
      displayName
      id
    }
    betRequestId
  }
}
"#;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBestMatchesVariables {
    pub bookmaker: Vec<String>,
    pub exchange: Vec<String>,
    pub rating_type: String,
    pub min_rating: Option<String>,
    pub max_rating: Option<String>,
    pub min_odds: Option<String>,
    pub max_odds: Option<String>,
    pub min_liquidity: Option<String>,
    pub limit: usize,
    pub skip: usize,
    pub updated_within_seconds: u64,
    pub exclude_draw: bool,
    pub permitted_sports: Vec<String>,
    pub permitted_market_groups: Vec<String>,
    pub permitted_event_groups: Vec<String>,
    pub permitted_countries: Vec<String>,
    pub permitted_event_ids: Vec<String>,
    pub commission_rates: CommissionRates,
}

impl Default for GetBestMatchesVariables {
    fn default() -> Self {
        Self {
            bookmaker: vec![String::from("betvictor")],
            exchange: vec![String::from("smarketsexchange")],
            rating_type: String::from("rating"),
            min_rating: None,
            max_rating: Some(String::from("99")),
            min_odds: Some(String::from("2.1")),
            max_odds: Some(String::from("5")),
            min_liquidity: Some(String::from("30")),
            limit: 10,
            skip: 0,
            updated_within_seconds: 21_600,
            exclude_draw: false,
            permitted_sports: vec![String::from("soccer")],
            permitted_market_groups: vec![String::from("match-odds")],
            permitted_event_groups: Vec::new(),
            permitted_countries: Vec::new(),
            permitted_event_ids: Vec::new(),
            commission_rates: CommissionRates::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommissionRates {
    pub betdaq: i64,
    pub betfair: i64,
    pub matchbook: i64,
    pub smarkets: i64,
    pub betconnect: i64,
}

impl Default for CommissionRates {
    fn default() -> Self {
        Self {
            betdaq: 0,
            betfair: 5,
            matchbook: 0,
            smarkets: 0,
            betconnect: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GraphQlRequest<T> {
    #[serde(rename = "operationName")]
    pub operation_name: String,
    pub query: String,
    pub variables: T,
}

impl GraphQlRequest<GetBestMatchesVariables> {
    pub fn get_best_matches(variables: GetBestMatchesVariables) -> Self {
        Self {
            operation_name: String::from("GetBestMatches"),
            query: String::from(GET_BEST_MATCHES_QUERY),
            variables,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphQlError>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GraphQlError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetBestMatchesData {
    #[serde(rename = "getBestMatches")]
    pub get_best_matches: Vec<OddsMatcherRow>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OddsMatcherRow {
    #[serde(rename = "eventName")]
    pub event_name: String,
    pub id: String,
    #[serde(rename = "startAt")]
    pub start_at: String,
    #[serde(rename = "selectionId")]
    pub selection_id: String,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "eventId")]
    pub event_id: String,
    pub back: PriceLeg,
    pub lay: LayLeg,
    #[serde(rename = "eventGroup")]
    pub event_group: GroupSummary,
    #[serde(rename = "marketGroup")]
    pub market_group: GroupSummary,
    #[serde(rename = "marketName")]
    pub market_name: String,
    pub rating: f64,
    #[serde(rename = "selectionName")]
    pub selection_name: String,
    pub snr: bool,
    pub sport: SportSummary,
    #[serde(rename = "betRequestId")]
    pub bet_request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PriceLeg {
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    pub odds: f64,
    #[serde(rename = "fetchedAt")]
    pub fetched_at: Option<String>,
    #[serde(rename = "deepLink")]
    pub deep_link: Option<String>,
    pub bookmaker: BookmakerSummary,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LayLeg {
    pub bookmaker: BookmakerSummary,
    #[serde(rename = "deepLink")]
    pub deep_link: Option<String>,
    #[serde(rename = "fetchedAt")]
    pub fetched_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    pub odds: f64,
    pub liquidity: Option<f64>,
    #[serde(rename = "betSlip")]
    pub bet_slip: Option<BetSlipRef>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BookmakerSummary {
    pub active: bool,
    pub code: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BetSlipRef {
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "selectionId")]
    pub selection_id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GroupSummary {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    #[serde(rename = "sourceName")]
    pub source_name: Option<String>,
    pub sport: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SportSummary {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
}

pub fn fetch_best_matches(
    client: &Client,
    variables: &GetBestMatchesVariables,
) -> Result<Vec<OddsMatcherRow>> {
    let payload = GraphQlRequest::get_best_matches(variables.clone());
    let response = client
        .post(ODDSMATCHER_GRAPHQL_URL)
        .json(&payload)
        .send()
        .wrap_err("failed to send OddsMatcher GraphQL request")?;

    let status = response.status();
    let graphql: GraphQlResponse<GetBestMatchesData> = response
        .json()
        .wrap_err("failed to decode OddsMatcher GraphQL response")?;

    if !status.is_success() {
        let detail = graphql
            .errors
            .first()
            .map(|error| error.message.clone())
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(eyre!("OddsMatcher GraphQL request failed: {detail}"));
    }

    if !graphql.errors.is_empty() {
        let detail = graphql
            .errors
            .iter()
            .map(|error| error.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(eyre!("OddsMatcher GraphQL returned errors: {detail}"));
    }

    let data = graphql
        .data
        .ok_or_else(|| eyre!("OddsMatcher GraphQL response did not include data"))?;
    Ok(data.get_best_matches)
}

#[cfg(test)]
mod tests {
    use super::{
        fetch_best_matches, CommissionRates, GetBestMatchesData, GetBestMatchesVariables,
        GraphQlRequest, GraphQlResponse, OddsMatcherRow,
    };

    #[test]
    fn request_payload_uses_captured_defaults() {
        let payload = GraphQlRequest::get_best_matches(GetBestMatchesVariables::default());
        let json = serde_json::to_value(payload).expect("serialize request");

        assert_eq!(json["operationName"], "GetBestMatches");
        assert_eq!(json["variables"]["bookmaker"][0], "betvictor");
        assert_eq!(json["variables"]["exchange"][0], "smarketsexchange");
        assert_eq!(json["variables"]["commissionRates"]["betfair"], 5);
    }

    #[test]
    fn request_variables_serialize_to_graphql_field_names() {
        let variables = serde_json::to_value(GetBestMatchesVariables {
            commission_rates: CommissionRates::default(),
            ..GetBestMatchesVariables::default()
        })
        .expect("serialize variables");

        assert_eq!(variables["ratingType"], "rating");
        assert_eq!(variables["updatedWithinSeconds"], 21600);
    }

    #[test]
    fn response_deserializes_captured_oddsmatcher_shape() {
        let response: GraphQlResponse<GetBestMatchesData> = serde_json::from_str(
            r#"{
              "data": {
                "getBestMatches": [
                  {
                    "eventName": "Arsenal v Everton",
                    "id": "match-1",
                    "startAt": "2026-03-14T17:30:00Z",
                    "selectionId": "sel-1",
                    "marketId": "mkt-1",
                    "eventId": "evt-1",
                    "back": {
                      "updatedAt": "2026-03-18T12:00:00Z",
                      "odds": 2.55,
                      "fetchedAt": "2026-03-18T12:00:00Z",
                      "deepLink": "https://bookie.example/bet",
                      "bookmaker": {
                        "active": true,
                        "code": "betvictor",
                        "displayName": "BetVictor",
                        "id": "101",
                        "logo": "/logos/80x30/betvictor.png"
                      }
                    },
                    "lay": {
                      "bookmaker": {
                        "active": true,
                        "code": "smarketsexchange",
                        "displayName": "Smarkets Exchange",
                        "id": "201",
                        "logo": "/logos/80x30/smarkets.png"
                      },
                      "deepLink": "https://smarkets.example/betslip",
                      "fetchedAt": "2026-03-18T12:00:00Z",
                      "updatedAt": "2026-03-18T12:00:00Z",
                      "odds": 2.66,
                      "liquidity": 30.0,
                      "betSlip": {
                        "marketId": "mkt-1",
                        "selectionId": "sel-1"
                      }
                    },
                    "eventGroup": {
                      "displayName": "Premier League",
                      "id": "grp-1",
                      "sourceName": "premier-league",
                      "sport": "soccer"
                    },
                    "marketGroup": {
                      "displayName": "Match Odds",
                      "id": "mg-1",
                      "sourceName": null,
                      "sport": "soccer"
                    },
                    "marketName": "Match Odds",
                    "rating": 95.81,
                    "selectionName": "Arsenal",
                    "snr": false,
                    "sport": {
                      "displayName": "Soccer",
                      "id": "soccer"
                    },
                    "betRequestId": "req-1"
                  }
                ]
              },
              "errors": []
            }"#,
        )
        .expect("deserialize response");

        let data = response.data.expect("data");
        let row: &OddsMatcherRow = data.get_best_matches.first().expect("row");
        assert_eq!(row.event_name, "Arsenal v Everton");
        assert_eq!(row.back.bookmaker.display_name, "BetVictor");
        assert_eq!(row.lay.liquidity, Some(30.0));
        assert_eq!(row.market_group.display_name, "Match Odds");
    }

    #[test]
    fn fetch_best_matches_surface_is_linkable_for_live_use() {
        let client = reqwest::blocking::Client::new();
        let function_ptr: fn(&reqwest::blocking::Client, &GetBestMatchesVariables)
            -> color_eyre::Result<Vec<OddsMatcherRow>> = fetch_best_matches;
        let _ = (client, function_ptr);
    }
}
