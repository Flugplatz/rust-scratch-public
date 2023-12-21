#[cfg(feature = "inline_stable")]
use stable_vec::InlineStableVec;
#[cfg(not(feature = "inline_stable"))]
use stable_vec::StableVec;

use crate::ob::{OrderBook, OrderBookSnapshot};

#[cfg(feature = "inline_stable")]
type OrderBookSide<T> = InlineStableVec<T>;
#[cfg(not(feature = "inline_stable"))]
type OrderBookSide<T> = StableVec<T>;

mod constants {
    pub const CAPACITY: usize = 1_000_000_000;
}

pub struct StableOrderBook {
    price_precision: u64,
    bids: OrderBookSide<f64>,
    asks: OrderBookSide<f64>,
    last_timestamp: u64,
    best_ask: Option<usize>,
    best_bid: Option<usize>,
}

impl StableOrderBook {
    pub fn new(price_precision: u64) -> Self {
        StableOrderBook {
            price_precision,
            bids: OrderBookSide::with_capacity(constants::CAPACITY),
            asks: OrderBookSide::with_capacity(constants::CAPACITY),
            last_timestamp: 0,
            best_ask: None,
            best_bid: None,
        }
    }

    pub fn upscale(n: f64, precision: u64) -> usize {
        let result = n * 10_f64.powf(precision as f64);
        result as usize
    }

    pub fn downscale(n: usize, precision: u64) -> f64 {
        (n as f64) / 10_f64.powi(precision as i32)
    }
}

impl OrderBook for StableOrderBook {
    fn add_ask(&mut self, price: f64, qty: f64, timestamp: u64) {
        let upscale = StableOrderBook::upscale(price, self.price_precision) as usize;

        if qty == 0.0 {
            self.asks.remove(upscale);

            if self.best_ask.unwrap_or(0) == upscale {
                self.best_ask = self.asks.first_filled_slot_from(upscale);
            }
        } else {
            self.asks.insert(upscale, qty);

            self.best_ask = self.best_ask.map_or(Some(constants::CAPACITY), |v| {
                Some(std::cmp::min(v, upscale))
            });
        }

        self.last_timestamp = timestamp;
    }

    fn add_bid(&mut self, price: f64, qty: f64, timestamp: u64) {
        let upscale = StableOrderBook::upscale(price, self.price_precision) as usize;

        if qty == 0.0 {
            self.bids.remove(upscale);

            if self.best_bid.unwrap_or(0) == upscale {
                self.best_bid = self.bids.first_filled_slot_below(upscale);
            }
        } else {
            self.bids.insert(upscale, qty);

            self.best_bid = self
                .best_bid
                .map_or(Some(0), |v| Some(std::cmp::max(v, upscale)));
        }

        self.last_timestamp = timestamp;
    }

    fn get_snapshot(&self, n: usize) -> crate::ob::OrderBookSnapshot {
        let mut bid_prices: Vec<f64> = Vec::with_capacity(n);
        let mut bid_quantities: Vec<f64> = Vec::with_capacity(n);

        let start_idx = self.best_bid.unwrap();
        let mut search_idx = start_idx;
        while bid_prices.len() < n {
            let data_idx = self.bids.first_filled_slot_below(search_idx).unwrap();
            let price = StableOrderBook::downscale(data_idx, self.price_precision);
            let qty = self.bids.get(data_idx).unwrap();

            bid_prices.push(price);
            bid_quantities.push(*qty);

            search_idx = data_idx - 1;
        }

        bid_prices.reverse();
        bid_quantities.reverse();

        let mut ask_prices: Vec<f64> = Vec::with_capacity(n);
        let mut ask_quantities: Vec<f64> = Vec::with_capacity(n);

        let start_idx = self.best_ask.unwrap();
        let mut search_idx = start_idx;
        while ask_prices.len() < n {
            let data_idx = self.asks.first_filled_slot_from(search_idx).unwrap();
            let price = StableOrderBook::downscale(data_idx, self.price_precision);
            let qty = self.asks.get(data_idx).unwrap();

            ask_prices.push(price);
            ask_quantities.push(*qty);

            search_idx = data_idx + 1;
        }

        OrderBookSnapshot {
            ts: self.last_timestamp,
            bid_prices,
            bid_quantities,
            ask_prices,
            ask_quantities,
        }
    }
}
