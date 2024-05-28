use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use fixnum::ops::Zero;
use log::info;

use crate::model::internal::{L2Increment, L2Snapshot, Side};
use crate::utils::basic_types::{Amount, Price};

const ZERO: Price = Price::ZERO;

pub struct OrderBook {
    pub bids: BTreeMap<Price, Amount>,
    pub asks: BTreeMap<Price, Amount>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn update_on_order_book(&mut self, ob: OrderBook) {
        self.bids = ob.bids;
        self.asks = ob.asks;
    }

    pub fn process_snapshot(&mut self, snapshot: L2Snapshot) {
        self.bids = snapshot.bids
            .into_iter()
            .filter(|s| s.amount != Amount::ZERO)
            .map(|s| (s.price, s.amount))
            .collect();
        self.asks = snapshot.asks
            .into_iter()
            .filter(|s| s.amount != Amount::ZERO)
            .map(|s| (s.price, s.amount))
            .collect();
    }

    pub fn process_update(&mut self, update: L2Increment) {
        let book = match update.side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        if update.amount != Amount::ZERO {
            book.insert(update.price, update.amount);
        } else {
            book.remove(&update.price);
        }

        if update.is_eot {
            info!("{self}");
        }
    }
}

impl Display for OrderBook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OrderBook {{ bid: {:?}, ask: {:?} }}",
               self.bids.last_key_value().map(|l| l.0).unwrap_or(&ZERO),
               self.asks.first_key_value().map(|l| l.0).unwrap_or(&ZERO),
        )
    }
}
