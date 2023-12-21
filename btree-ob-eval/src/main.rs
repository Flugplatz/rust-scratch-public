use std::time::Instant;

use btree_ob::BTreeOrderBook;
use clap::Parser;
use serde::Deserialize;

use crate::ob::OrderBook;
use crate::stable_ob::StableOrderBook;

mod btree_ob;
mod ob;
mod stable_ob;

#[derive(Debug, Deserialize)]
struct CsvRow {
    timestamp: u64,
    side: String,
    price: f64,
    qty: f64,
}

#[derive(Parser)]
struct Args {
    ob_impl: String,
}

mod constants {
    pub const SNAPSHOT_PATH: &str = "/tmp/BTCUSDT_T_DEPTH_2023-10-30_depth_snap.csv";
    pub const DELTAS_PATH: &str = "/tmp/BTCUSDT_T_DEPTH_2023-10-30_depth_update.csv";
    pub const DELTAS_CAPACITY: usize = 141327104;
    pub const SNAPSHOT_MEASURE_N: usize = 5;
}

fn main() {
    let args = Args::parse();

    // TODO: avoid dynamic dispatch
    let mut ob: Box<dyn OrderBook> = if args.ob_impl == "b" {
        Box::new(BTreeOrderBook::new(2))
    } else {
        Box::new(StableOrderBook::new(2))
    };

    let mut reader = csv::Reader::from_path(constants::SNAPSHOT_PATH).unwrap();
    for result in reader.deserialize::<CsvRow>() {
        let row = result.unwrap();

        if row.side == "b" {
            ob.add_bid(row.price, row.qty, row.timestamp);
        } else {
            ob.add_ask(row.price, row.qty, row.timestamp);
        }
    }

    let mut reader = csv::Reader::from_path(constants::DELTAS_PATH).unwrap();
    let mut data: Vec<CsvRow> = Vec::with_capacity(constants::DELTAS_CAPACITY);
    for result in reader.deserialize::<CsvRow>() {
        let row = result.unwrap();
        data.push(row);
    }

    let before: Instant = Instant::now();

    for row in data {
        if row.side == "b" {
            ob.add_bid(row.price, row.qty, row.timestamp);
        } else {
            ob.add_ask(row.price, row.qty, row.timestamp);
        }
    }
    println!("LOAD COMPLETE {:?}", before.elapsed());

    // snapshot measurement

    let mut measurements = Vec::with_capacity(constants::SNAPSHOT_MEASURE_N);
    let mut counter = 0;
    let snapshot = loop {
        let before: Instant = Instant::now();
        let ob = ob.get_snapshot(256);
        let measurement = before.elapsed().as_nanos();

        println!("GET SNAPSHOT ELAPSED: {:?}ns", measurement);

        // first lookup is to warm the cache, not for measurement
        if counter != 0 {
            measurements.push(measurement);
        }

        if counter == constants::SNAPSHOT_MEASURE_N {
            break ob;
        }

        counter += 1;
    };

    let mean_snapshot_tm = measurements.iter().sum::<u128>() / measurements.len() as u128;

    println!(
        "SNAPSHOT COMPLETE N:{:?} MEAN_ELAPSED:{:?}ns DATA:{:?}",
        snapshot.bid_prices.len(),
        mean_snapshot_tm,
        snapshot
    );
}
