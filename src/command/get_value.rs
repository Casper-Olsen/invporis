use anyhow::{Context, Ok, anyhow};
use log::info;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::{collections::HashMap, env};

use crate::{
    data::{db::Db, trade_store},
    domain::trade::Provider as DomainProvider,
    domain::trade::Trade,
};

const FIGI_API_KEY: &str = "INVPORIS_OPENFIGI_API_KEY";

pub async fn run(db: Db) -> Result<(), anyhow::Error> {
    let figi_api_key = env::var(FIGI_API_KEY).ok();

    let trades = trade_store::list_trades(&db)?;

    let figi_mappings: Vec<FigiMapping> =
        trades.iter().filter_map(Trade::to_figi_mapping).collect();

    let instruments = fetch_instrument_metadata(figi_mappings, figi_api_key).await?;

    if !instruments.errors_by_index.is_empty() {
        if instruments.instruments_by_index.is_empty() {
            return Err(anyhow!("could not fetch instruments for any securities"));
        }

        return Err(anyhow!(
            "Could not fetch instruments for {} securities",
            instruments.instruments_by_index.len()
        ));
    }

    for instrument in instruments.instruments_by_index {
        println!("Figi: {:?}", instrument.0);

        for metadata in instrument.1 {
            println!("{metadata:?}");
        }
    }

    // TODO: Filter out trades we dont want before calculating total vavlue.
    // - If we don't own any (Buy - Sell = 0)

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

impl From<MappingJob> for FigiMapping {
    fn from(mapping_job: MappingJob) -> Self {
        Self {
            isin: mapping_job.id_value,
            currency: mapping_job.currency,
            market_identifier_code: mapping_job.mic_code,
        }
    }
}

impl Trade {
    fn to_figi_mapping(&self) -> Option<FigiMapping> {
        let isin = self.isin.as_ref()?;

        let mic = if self.provider == Some(DomainProvider::Saxo) {
            self.symbol
                .as_ref()
                .and_then(|s| s.split_once(':'))
                .map(|(_, mic)| mic.to_owned().to_uppercase())
        } else {
            None
        };

        let fm = FigiMapping {
            isin: isin.to_owned(),
            currency: self.price.currency.clone(),
            market_identifier_code: mic,
        };

        Some(fm)
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
