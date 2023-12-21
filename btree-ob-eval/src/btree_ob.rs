use std::collections::BTreeMap;

use crate::ob::{OrderBook, OrderBookSnapshot};

type OrderBookPrice = u64;
type OrderBookSide = BTreeMap<OrderBookPrice, f64>;

pub struct BTreeOrderBook {
    price_precision: u64,
    bids: OrderBookSide,
    asks: OrderBookSide,
    last_timestamp: u64,
}

impl BTreeOrderBook {
    pub fn new(price_precision: u64) -> Self {
        BTreeOrderBook {
            price_precision,
            bids: OrderBookSide::new(),
            asks: OrderBookSide::new(),
            last_timestamp: 0,
        }
    }

    pub fn upscale(n: f64, precision: u64) -> u64 {
        let result = n * 10_f64.powf(precision as f64);
        result as u64
    }

    pub fn downscale(n: u64, precision: u64) -> f64 {
        (n as f64) / 10_f64.powi(precision as i32)
    }

    pub fn update(side: &mut OrderBookSide, price: u64, qty: f64) {
        if qty == 0.0 {
            side.remove(&price);
        } else {
            side.insert(price, qty);
        }
    }
}

impl OrderBook for BTreeOrderBook {
    fn add_ask(&mut self, price: f64, qty: f64, timestamp: u64) {
        let price_upscale = BTreeOrderBook::upscale(price, self.price_precision);
        BTreeOrderBook::update(&mut self.asks, price_upscale, qty);
        self.last_timestamp = timestamp;
    }

    fn add_bid(&mut self, price: f64, qty: f64, timestamp: u64) {
        let price_upscale = BTreeOrderBook::upscale(price, self.price_precision);
        BTreeOrderBook::update(&mut self.bids, price_upscale, qty);
        self.last_timestamp = timestamp;
    }

    fn get_snapshot(&self, n: usize) -> crate::ob::OrderBookSnapshot {
        OrderBookSnapshot {
            ts: self.last_timestamp,
            bid_prices: self
                .bids
                .keys()
                .rev()
                .take(n)
                .rev()
                .map(|p| BTreeOrderBook::downscale(*p, self.price_precision))
                .collect(),
            bid_quantities: self.bids.values().rev().take(n).rev().copied().collect(),
            ask_prices: self
                .asks
                .keys()
                .take(n)
                .rev()
                .map(|p| BTreeOrderBook::downscale(*p, self.price_precision))
                .collect(),
            ask_quantities: self.asks.values().take(n).copied().collect(),
        }
    }
}
