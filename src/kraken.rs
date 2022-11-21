use crate::newtypes::Euro;
use anyhow::{anyhow, Error};
use serde::Deserialize;
use std::collections::HashMap;
use tabled::Tabled;

pub fn kraken_aggregate() -> Result<Vec<KrakenAggregate>, Error> {
    let kraken: KrakenResult = ureq::get("https://api.kraken.com/0/public/Ticker")
        .call()?
        .into_json()?;
    if !kraken.error.is_empty() {
        return Err(anyhow!("{:?}", kraken.error));
    }
    let mut kraken_aggregate: Vec<KrakenAggregate> = kraken
        .result
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        // only eur pairs (todo: add other pairs)
        .filter(|(k, _)| k.contains("eur"))
        .map(|(k, v)| (k.replace("eur", ""), v))
        // need to fix some ticker mappings
        .map(|(k, v)| {
            (
                k.replace("xethz", "eth")
                    .replace("xxbtz", "btc")
                    .replace("xxrpz", "xrp")
                    .replace("xdg", "doge"),
                v,
            )
        })
        .map(|(k, v)| {
            let price =
                Euro((v.a[0].parse::<f32>().unwrap() + v.b[0].parse::<f32>().unwrap()) / 2.);
            KrakenAggregate {
                crypto_market: k,
                price,
                volume: Euro(v.v[0].parse().unwrap()),
            }
        })
        // convert kraken volume currency from crypto to eur
        .map(|mut k| {
            k.volume = Euro((k.volume * k.price).0.round());
            k
        })
        .filter(|k| k.volume > Euro(3000.))
        .collect();

    // sort by volume
    kraken_aggregate.sort_by(|a, b| b.volume.0.total_cmp(&a.volume.0));

    Ok(kraken_aggregate)
}

#[derive(Deserialize, Debug)]
struct KrakenResult {
    error: Vec<String>,
    result: HashMap<String, KrakenTicker>,
}

#[derive(Deserialize, Debug)]
struct KrakenTicker {
    /// ask
    a: [String; 3],
    /// bid
    b: [String; 3],
    /// volume
    v: [String; 2],
}

#[derive(Debug, Tabled, Clone)]
pub struct KrakenAggregate {
    #[tabled(rename = "Market")]
    pub(crate) crypto_market: String,
    #[tabled(rename = "Average Price (Eur)")]
    pub(crate) price: Euro,
    #[tabled(rename = "Volume (Eur)")]
    pub(crate) volume: Euro,
}
