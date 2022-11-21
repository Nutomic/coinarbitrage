mod korbit;
mod kraken;
mod newtypes;

use crate::korbit::korbit_aggregate;
use crate::kraken::kraken_aggregate;
use crate::newtypes::{Euro, Percent};
use anyhow::Error;
use tabled::object::Segment;
use tabled::{Alignment, Modify, Table, Tabled};

const KRW_TO_USD: Euro = Euro(0.000746);

fn main() -> Result<(), Error> {
    let korbit_aggregate = korbit_aggregate()?;
    let kraken_aggregate = kraken_aggregate()?;

    let mut both_aggregate: Vec<BothAggregate> = kraken_aggregate
        .iter()
        .filter(|kraken| {
            korbit_aggregate
                .iter()
                .any(|korbit| korbit.crypto_market == kraken.crypto_market)
        })
        .map(|kraken| {
            let korbit = korbit_aggregate
                .iter()
                .find(|k2| k2.crypto_market == kraken.crypto_market)
                .unwrap()
                .clone();
            BothAggregate {
                crypto_market: korbit.crypto_market,
                kraken_price: kraken.price,
                korbit_price: korbit.price,
                price_difference: Percent((korbit.price.0 / kraken.price.0) * 100. - 100.),
                kraken_volume: kraken.volume,
                korbit_volume: korbit.volume,
            }
        })
        .collect();

    // TODO: some pairs are still missing
    let korbit_unmatched: Vec<String> = korbit_aggregate
        .iter()
        .map(|k| k.crypto_market.clone())
        .filter(|korbit| {
            !both_aggregate
                .iter()
                .any(|kraken| &kraken.crypto_market == korbit)
        })
        .collect();
    dbg!(&korbit_unmatched);

    both_aggregate.sort_by(|a, b| b.price_difference.0.total_cmp(&a.price_difference.0));
    let mut table = Table::new(both_aggregate.clone());
    table.with(Modify::new(Segment::all()).with(Alignment::right()));
    print!("{}", table);

    Ok(())
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
