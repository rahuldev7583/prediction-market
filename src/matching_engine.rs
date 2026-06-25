use serde::Serialize;
use std::collections::{BTreeMap, VecDeque};
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Serialize)]
pub struct OrderbookLevel {
    pub price: u64,
    pub qty: u64,
}

#[derive(Serialize)]
pub struct OrderbookSnapshot {
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
}

pub enum Side {
    Buy,
    Sell,
}

pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,
    pub qty: u64,
}

#[derive(Debug)]
pub struct Fill {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: u64,
    pub qty: u64,
}

pub struct Orderbook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }
    pub fn match_order(&mut self, mut order: Order) -> Vec<Fill> {
        let mut fills = Vec::new();

        fills
    }

    pub fn snapshot(&self) -> OrderbookSnapshot {
        let bids = self
            .bids
            .iter()
            .rev()
            .map(|(price, orders)| OrderbookLevel {
                price: *price,
                qty: orders.iter().map(|o| o.qty).sum(),
            })
            .collect();

        let asks = self
            .asks
            .iter()
            .map(|(price, orders)| OrderbookLevel {
                price: *price,
                qty: orders.iter().map(|o| o.qty).sum(),
            })
            .collect();

        OrderbookSnapshot { bids, asks }
    }
}

pub enum EngineCommand {
    NewOrder(Order),
    GetSnapshot(oneshot::Sender<OrderbookSnapshot>),
}

pub async fn run_matching_engine(
    mut rx: mpsc::Receiver<EngineCommand>,
    fill_tx: broadcast::Sender<String>,
) {
    let mut orderbook = Orderbook::new();

    loop {
        let cmd = match rx.recv().await {
            Some(c) => c,
            None => break,
        };
        match cmd {
            EngineCommand::NewOrder(order) => {
                let fills = orderbook.match_order(order);
                for f in fills {
                    let _ = fill_tx.send(format!("{:?}", f));
                }
            }
            EngineCommand::GetSnapshot(resp_tx) => {
                let snapshot = orderbook.snapshot();
                let _ = resp_tx.send(snapshot);
            }
        }
    }
}
