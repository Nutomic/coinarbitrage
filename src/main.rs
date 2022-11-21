use anyhow::Error;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Mul;
use tabled::{Table, Tabled};
use thousands::Separable;

const KRW_TO_USD: Euro = Euro(0.000746);

fn main() -> Result<(), Error> {
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
        .filter(|k| k.volume > Euro(1000.))
        .collect();
    // sort by volume
    korbit_aggregate.sort_by(|a, b| b.volume.0.total_cmp(&a.volume.0));
    let table = Table::new(korbit_aggregate.clone()).to_string();
    print!("{}", table);

    let kraken: KrakenResult = ureq::get("https://api.kraken.com/0/public/Ticker")
        .call()?
        .into_json()?;
    if kraken.error.is_empty() {
        dbg!(&kraken.error);
    }
    let mut both_aggregate: Vec<BothAggregate> = kraken
        .result
        .iter()
        .map(|(k, v)| (k.replace("TBTCEUR", "btceur"), v))
        .map(|(k, v)| (k.to_lowercase(), v))
        // only eur pairs (todo: add other pairs)
        .filter(|(k, _)| k.contains("eur"))
        .map(|(k, v)| (k.replace("eur", ""), v))
        // need to fix some ticker mappings
        .map(|(k, v)| (k.replace("xethz", "eth").replace("tbtc", "btc"), v))
        .filter(|(k, _)| korbit_aggregate.iter().any(|k2| &k2.crypto_market == k))
        .map(|(k, v)| {
            let korbit = korbit_aggregate
                .iter()
                .find(|k2| k2.crypto_market == k)
                .unwrap()
                .clone();
            let kraken_price =
                Euro((v.a[0].parse::<f32>().unwrap() + v.b[0].parse::<f32>().unwrap()) / 2.);
            BothAggregate {
                crypto_market: k,
                kraken_price,
                korbit_price: korbit.price,
                price_difference: Percent((korbit.price.0 / kraken_price.0) * 100. - 100.),
                kraken_volume: Euro(v.v[0].parse().unwrap()),
                korbit_volume: korbit.volume,
            }
        })
        // convert kraken volume currency from crypto to eur
        .map(|mut k| {
            k.kraken_volume = Euro((k.kraken_volume * k.kraken_price).0.round());
            k
        })
        .filter(|k| k.kraken_volume > Euro(1000.))
        .collect();

    // TODO: why are btc and other pairs missing?
    let kraken_unmatched: Vec<String> = kraken
        .result
        .iter()
        .map(|k| k.0.clone())
        .map(|k| k.to_lowercase())
        .filter(|k| k.contains("eur"))
        .map(|k| k.replace("eur", ""))
        .filter(|m| !korbit_aggregate.iter().any(|b| &b.crypto_market == m))
        .collect();
    let korbit_unmatched: Vec<String> = korbit_aggregate
        .iter()
        .map(|k| k.crypto_market.clone())
        .filter(|m| !both_aggregate.iter().any(|b| &b.crypto_market == m))
        .collect();
    dbg!(&korbit_unmatched, &kraken_unmatched);

    both_aggregate.sort_by(|a, b| b.price_difference.0.total_cmp(&a.price_difference.0));
    let table = Table::new(both_aggregate.clone()).to_string();
    print!("{}", table);

    Ok(())
}

pub type Korbit = HashMap<String, KorbitMarket>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct KorbitMarket {
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
    crypto_market: String,
    #[tabled(rename = "Average Price (Eur)")]
    price: Euro,
    #[tabled(rename = "Volume (Eur)")]
    volume: Euro,
}

#[derive(Debug, Tabled, Clone)]
pub struct BothAggregate {
    #[tabled(rename = "Market")]
    crypto_market: String,
    #[tabled(rename = "Kraken Price")]
    kraken_price: Euro,
    #[tabled(rename = "Korbit Price")]
    korbit_price: Euro,
    #[tabled(rename = "Korbit Premium")]
    price_difference: Percent,
    #[tabled(rename = "Kraken Volume")]
    kraken_volume: Euro,
    #[tabled(rename = "Korbit Volume")]
    korbit_volume: Euro,
}

#[derive(Debug, Tabled, PartialOrd, PartialEq, Copy, Clone)]
pub struct Euro(f32);

impl Display for Euro {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} €", self.0.separate_with_spaces())
    }
}

impl Mul for Euro {
    type Output = Euro;

    fn mul(self, rhs: Self) -> Self::Output {
        Euro(self.0.mul(rhs.0))
    }
}

#[derive(Debug, Tabled, PartialOrd, PartialEq, Copy, Clone)]
pub struct Percent(f32);

impl Display for Percent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} %", self.0)
    }
}

#[derive(Deserialize, Debug)]
struct KrakenResult {
    error: Vec<String>,
    result: HashMap<String, KrakenTicker>,
}

#[derive(Deserialize, Debug)]
struct KrakenTicker {
    a: [String; 3],
    b: [String; 3],
    v: [String; 2],
}
