use crate::newtypes::Euro;
use crate::KRW_TO_USD;
use anyhow::Error;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use std::collections::HashMap;
use tabled::Tabled;

pub fn korbit_aggregate() -> Result<Vec<KorbitAggregate>, Error> {
    let korbit: Korbit = ureq::get("https://api.korbit.co.kr/v1/ticker/detailed/all")
        .call()?
        .into_json()?;
    let mut korbit_aggregate: Vec<KorbitAggregate> = korbit
        .into_iter()
        // only keep crypto currency name (everything is krw pair)
        .map(|(k, v)| (k.replace("_krw", ""), v))
        // aggregate data we care about
        .map(|(k, v)| KorbitAggregate {
            crypto_market: k,
            price: Euro((v.bid + v.ask) / 2.),
            volume: Euro(v.volume),
        })
        // krw to eur
        .map(|mut k| {
            k.price = k.price * KRW_TO_USD;
            k
        })
        // convert volume currency from crypto to eur
        .map(|mut k| {
            k.volume = Euro((k.volume * k.price).0.round());
            k
        })
        .filter(|k| k.volume > Euro(3000.))
        .collect();

    // sort by volume
    korbit_aggregate.sort_by(|a, b| b.volume.0.total_cmp(&a.volume.0));

    Ok(korbit_aggregate)
}

type Korbit = HashMap<String, KorbitMarket>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct KorbitMarket {
    timestamp: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    last: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    open: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    bid: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    ask: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    low: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    high: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    volume: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    change: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    change_percent: f32,
}

#[derive(Debug, Tabled, Clone)]
pub struct KorbitAggregate {
    #[tabled(rename = "Market")]
    pub(crate) crypto_market: String,
    #[tabled(rename = "Average Price (Eur)")]
    pub(crate) price: Euro,
    #[tabled(rename = "Volume (Eur)")]
    pub(crate) volume: Euro,
}
