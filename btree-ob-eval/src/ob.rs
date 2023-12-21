#[derive(Debug)]
pub struct OrderBookSnapshot {
    pub ts: u64,
    pub bid_prices: Vec<f64>,
    pub bid_quantities: Vec<f64>,
    pub ask_prices: Vec<f64>,
    pub ask_quantities: Vec<f64>,
}

pub trait OrderBook {
    fn add_ask(&mut self, price: f64, qty: f64, timestamp: u64);
    fn add_bid(&mut self, price: f64, qty: f64, timestamp: u64);
    fn get_snapshot(&self, n: usize) -> OrderBookSnapshot;
}
