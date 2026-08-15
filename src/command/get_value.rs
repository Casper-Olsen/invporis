use anyhow::anyhow;
use reqwest::header::ACCEPT;
use std::env;

use crate::data::db::Db;

#[derive(serde::Serialize)]
struct MappingJob {
    #[serde(rename = "idType")]
    id_type: String,

    #[serde(rename = "idValue")]
    id_value: String,

    #[serde(rename = "currency")]
    currency: String,
}
pub async fn run(_: Db) -> Result<(), anyhow::Error> {
    let figi_api_key = env::var("INVPORIS_OPENFIGI_API_KEY")?;

    let input = MappingJob {
        id_type: String::from("ID_ISIN"),
        id_value: String::from("US4581401001"),
        currency: String::from("USD"),
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openfigi.com/v3/mapping")
        .header(ACCEPT, "application/json")
        .header("X-OPENFIGI-APIKEY", figi_api_key)
        .json(&[input])
        .send()
        .await?;

    let body = response.text().await?;

    println!("{body:?}");

    Err(anyhow!("Not implemented yet"))
}
