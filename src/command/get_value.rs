use anyhow::{Context, Ok, anyhow};
use log::info;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use rust_decimal::{Decimal, dec};
use std::{collections::HashMap, env, ops::AddAssign, result::Result};

use crate::{
    data::{db::Db, trade_store},
    domain::{
        asset::{Asset, Crypto, Equity, EquityDetails, FigiData},
        trade::{EquityTrade, Provider as DomainProvider, Trade},
    },
};

const FIGI_API_KEY: &str = "INVPORIS_OPENFIGI_API_KEY";

pub async fn run(db: Db) -> Result<(), anyhow::Error> {
    let figi_api_key = env::var(FIGI_API_KEY).ok();

    let trades = trade_store::list_trades(&db)?;

    let figi_mappings: Vec<FigiMapping> = trades
        .iter()
        .filter_map(|t| match t {
            Trade::Equity(equity_trade) => Some(EquityTrade::to_figi_mapping(equity_trade)),
            Trade::Crypto(_) => None,
        })
        .collect();

    let instruments = fetch_instrument_metadata(figi_mappings, figi_api_key).await?;

    if !instruments.errors_by_index.is_empty() {
        if instruments.instruments_by_index.is_empty() {
            return Err(anyhow!("could not fetch instruments for any securities"));
        }

        return Err(anyhow!(
            "could not fetch instruments for {} securities",
            instruments.instruments_by_index.len()
        ));
    }

    let mut positions: HashMap<Asset, Decimal> = HashMap::new();

    for trade in trades {
        let quantity = match &trade {
            Trade::Equity(e) => e.quantity,
            Trade::Crypto(c) => c.quantity,
        };

        let asset = match trade {
            Trade::Equity(equity_trade) => {
                let equity = equity_from_trade(&equity_trade, &instruments.instruments_by_index)?;

                Asset::Equity(equity)
            }
            Trade::Crypto(crypto_trade) => Asset::Crypto(Crypto {
                symbol: crypto_trade.symbol,
                currency: crypto_trade.price.currency,
            }),
        };

        positions
            .entry(asset)
            .or_insert_with(|| dec!(0))
            .add_assign(quantity);
    }

    // TODO: For a stock where we have the same ISIN/Currency, but not the same a MIC in both,
    // we need to combine them, because they are the same asset. Currently we have two different
    // positions.
    let _total_value = get_total_value(
        positions
            .iter()
            .filter(|(_, quantity)| **quantity > Decimal::ZERO),
    );

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
struct MappingJob {
    #[serde(rename = "idType")]
    id_type: String,

    #[serde(rename = "idValue")]
    id_value: String,

    #[serde(rename = "currency")]
    currency: String,

    #[serde(rename = "micCode", skip_serializing_if = "Option::is_none")]
    mic_code: Option<String>,
}

#[derive(serde::Deserialize)]
enum MappingResult {
    #[serde(rename = "data")]
    Data(Vec<InstrumentMetadata>),

    #[serde(rename = "error")]
    Error(String),

    #[serde(rename = "warning")]
    Warning(String),
}

// TODO: Remove allow dead_code when we use the fields
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct InstrumentMetadata {
    #[serde(rename = "figi")]
    figi: String,

    #[serde(rename = "compositeFIGI")]
    composite_figi: String,

    #[serde(rename = "securityType")]
    security_type: String,

    #[serde(rename = "exchCode")]
    exchange_code: String,

    #[serde(rename = "ticker")]
    ticker: String,
}

struct InstrumentFetchResult {
    instruments_by_index: HashMap<FigiMapping, Vec<InstrumentMetadata>>,
    errors_by_index: HashMap<FigiMapping, anyhow::Error>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct FigiMapping {
    isin: String,
    currency: String,
    market_identifier_code: Option<String>,
}

impl std::fmt::Display for FigiMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ISIN: {}, Currency: {}, MIC: {}",
            self.isin,
            self.currency,
            self.market_identifier_code.as_deref().unwrap_or("N/A")
        )
    }
}

impl From<MappingJob> for FigiMapping {
    fn from(mapping_job: MappingJob) -> Self {
        Self {
            isin: mapping_job.id_value,
            currency: mapping_job.currency,
            market_identifier_code: mapping_job.mic_code,
        }
    }
}

impl EquityTrade {
    fn to_figi_mapping(&self) -> FigiMapping {
        let isin = &self.isin;

        let mic = if self.provider == Some(DomainProvider::Saxo) {
            // Expected format like: "BRKb:xnys"
            self.symbol
                .as_ref()
                .and_then(|s| s.split_once(':'))
                .map(|(_, mic)| mic.to_owned().to_uppercase())
        } else {
            None
        };

        FigiMapping {
            isin: isin.clone(),
            currency: self.price.currency.clone(),
            market_identifier_code: mic,
        }
    }
}

async fn fetch_instrument_metadata(
    mut figi_mappings: Vec<FigiMapping>,
    figi_api_key: Option<String>,
) -> Result<InstrumentFetchResult, anyhow::Error> {
    const ID_ISIN: &str = "ID_ISIN";
    const FIGI_MAX_NO_API_KEY: usize = 10;
    const FIGI_MAX_WITH_API_KEY: usize = 100;

    let chunk_size = if figi_api_key.is_some() {
        FIGI_MAX_WITH_API_KEY
    } else {
        info!(
            "no {FIGI_API_KEY} provided. Using unauthenticated rate limit of {FIGI_MAX_NO_API_KEY}"
        );
        FIGI_MAX_NO_API_KEY
    };

    // Deduplicate to prevent redundant requests.
    figi_mappings.sort_unstable();
    figi_mappings.dedup_by(|a, b| {
        a.isin == b.isin
            && a.currency == b.currency
            && a.market_identifier_code == b.market_identifier_code
    });

    // Most jobs are expected to succeed, so preallocate for the expected number
    // of instrument entries.
    let mut instruments = HashMap::with_capacity(figi_mappings.len());

    let mut errors = HashMap::new();
    let mut identifiers_not_found = Vec::new();

    for trades_chunk in figi_mappings.chunks(chunk_size) {
        let mapping_jobs: Vec<MappingJob> = trades_chunk
            .iter()
            .map(|trade| MappingJob {
                id_type: String::from(ID_ISIN),
                id_value: trade.isin.clone(),
                currency: trade.currency.clone(),
                mic_code: trade.market_identifier_code.clone(),
            })
            .collect();

        let res = process_mapping_batch(&mapping_jobs, figi_api_key.as_ref())
            .await
            .context("failed to process mapping batch")?;

        instruments.extend(res.instruments);
        errors.extend(res.errors);
        identifiers_not_found.extend(res.identifiers_not_found);
    }

    if !identifiers_not_found.is_empty() {
        // Retry without the MIC in case that is what prevented OpenFIGI from finding a match
        for mapping_job in &mut identifiers_not_found {
            mapping_job.mic_code = None;
        }

        for mapping_jobs in identifiers_not_found.chunks(chunk_size) {
            let res = process_mapping_batch(mapping_jobs, figi_api_key.as_ref())
                .await
                .context("failed to process mapping batch")?;

            instruments.extend(res.instruments);
            errors.extend(res.errors);
            errors.extend(
                res.identifiers_not_found
                    .into_iter()
                    .map(|m| (FigiMapping::from(m), anyhow!("failed to get FIGI"))),
            );
        }
    }

    Ok(InstrumentFetchResult {
        instruments_by_index: instruments,
        errors_by_index: errors,
    })
}

struct MappingJobResult {
    instruments: HashMap<FigiMapping, Vec<InstrumentMetadata>>,
    errors: HashMap<FigiMapping, anyhow::Error>,
    identifiers_not_found: Vec<MappingJob>,
}

async fn process_mapping_batch(
    mapping_jobs: &[MappingJob],
    figi_api_key: Option<&String>,
) -> Result<MappingJobResult, anyhow::Error> {
    let response = post_mapping_jobs(mapping_jobs, figi_api_key).await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "failed to get mappings from OpenFigi. Status code: {}",
            response.status()
        ));
    }

    let mapping_results: Vec<MappingResult> = response
        .json()
        .await
        .context("failed to deserialize the response body as JSON")?;

    if mapping_results.is_empty() {
        return Err(anyhow!("no mapping results returned for batch"));
    }

    // Most jobs are expected to succeed, so preallocate for the expected number
    // of instrument entries.
    let mut instruments = HashMap::with_capacity(mapping_jobs.len());

    let mut errors = HashMap::new();
    let mut identifiers_not_found: Vec<MappingJob> = Vec::new();

    // The API preserves request order: the result at `index` corresponds to
    // `mapping_jobs[index]`.
    for (index, mapping) in mapping_results.into_iter().enumerate() {
        const NO_IDENTIFIER_FOUND: &str = "No identifier found.";

        let mapping_job = mapping_jobs[index].clone();

        let metadata = match mapping {
            MappingResult::Data(data) => data,
            MappingResult::Error(error) => {
                errors.insert(
                    mapping_job.into(),
                    anyhow!(error).context("failed to get FIGI"),
                );
                continue;
            }
            MappingResult::Warning(warning) => {
                if mapping_job.mic_code.is_some() && warning == NO_IDENTIFIER_FOUND {
                    identifiers_not_found.push(mapping_job);
                } else {
                    errors.insert(
                        mapping_job.into(),
                        anyhow!(warning).context("failed to get FIGI"),
                    );
                }

                continue;
            }
        };

        instruments.insert(mapping_job.into(), metadata);
    }

    Ok(MappingJobResult {
        instruments,
        errors,
        identifiers_not_found,
    })
}

async fn post_mapping_jobs(
    mapping_jobs: &[MappingJob],
    figi_api_key: Option<&String>,
) -> Result<reqwest::Response, anyhow::Error> {
    const OPENFIGI_APIKEY: &str = "X-OPENFIGI-APIKEY";
    const APPLICATION_JSON: &str = "application/json";

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let mut request = client
        .post("https://api.openfigi.com/v3/mapping")
        .header(ACCEPT, APPLICATION_JSON)
        .header(CONTENT_TYPE, APPLICATION_JSON);

    if let Some(api_key) = figi_api_key {
        request = request.header(OPENFIGI_APIKEY, api_key);
    }

    let response = request
        .body(serde_json::to_vec(mapping_jobs)?)
        .send()
        .await?;

    Ok(response)
}

fn equity_from_trade(
    trade: &EquityTrade,
    instruments: &HashMap<FigiMapping, Vec<InstrumentMetadata>>,
) -> Result<Equity, anyhow::Error> {
    let mut mapping = EquityTrade::to_figi_mapping(trade);
    let metadata = instruments.get(&mapping);

    if let Some(metadata) = metadata {
        return equity_from_metadata(metadata, mapping);
    }

    // No MIC is available, so there is no further mapping we can try.
    if mapping.market_identifier_code.is_none() {
        return Err(anyhow!("could not find metadata for {mapping}"));
    }

    // The trade has a MIC. But no FIGI mapping was found with the ISIN/Currency/MIC combination.
    // Look up the mapping without the MIC. A mapping should exist.
    mapping.market_identifier_code = None;
    let metadata = instruments.get(&mapping);

    if let Some(metadata) = metadata {
        return equity_from_metadata(metadata, mapping);
    }

    Err(anyhow!("could not find metadata"))
}

fn equity_from_metadata(
    metadata: &[InstrumentMetadata],
    mapping: FigiMapping,
) -> Result<Equity, anyhow::Error> {
    // TODO:
    // Priority when getting FIGI mapping (when there are more than one)
    // 1. If only one FIGI, use that one
    // 2. If multiple, find where FIGI == CompositeFIGI
    // 3. If only one match, use that one
    // 4. If multiple matches, then use the dominant one; the CompositeFIGI in the most mappings
    // 5. If there is a tie, use a preferred list of exchanges in order, so it's consistent. When creating the ordered list, prefer liquid exchanges
    // 6. If we have no FIGI == CompositeFIGI, then fail and log it as an error.

    let first = metadata
        .first()
        .ok_or_else(|| anyhow!("could not find metadata for {mapping}"))?;

    Ok(Equity {
        figi_data: FigiData {
            ticker: first.ticker.clone(),
            exchange_code: first.exchange_code.clone(),
            figi: first.figi.clone(),
        },
        details: EquityDetails {
            isin: mapping.isin,
            currency: mapping.currency,
            market_identifier_code: mapping.market_identifier_code,
        },
    })
}

fn get_total_value<'a>(positions: impl Iterator<Item = (&'a Asset, &'a Decimal)>) -> Decimal {
    for (position, quantity) in positions {
        println!("{position:?}: {quantity}");
    }

    // TODO: Get current asset prices and calculate total value for portfolio.
    dec!(0)
}
