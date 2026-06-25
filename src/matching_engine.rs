use std::{
    collections::{BTreeMap, VecDeque},
    sync::mpsc::{Receiver, Sender},
};
use tokio::sync::{broadcast, mpsc};

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
    pub fn match_order(&mut self, order: Order) -> Vec<Fill> {
        vec![Fill {
            maker_order_id: order.id,
            taker_order_id: order.id,
            price: order.price,
            qty: order.qty,
        }]
    }
}

pub async fn run_matching_engine(
    mut rx: mpsc::Receiver<Order>,
    fill_tx: broadcast::Sender<String>,
) {
    let mut orderbook = Orderbook::new();

    loop {
        let order = rx.recv().await.expect("error while getting order");

        let fills = orderbook.match_order(order);

        for f in fills {
            let msg = format!("{:?}", f);
            let _ = fill_tx.send(msg);
        }
    }
}
