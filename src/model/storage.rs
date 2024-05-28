use std::collections::hash_map::Entry;
use std::collections::HashMap;

use compact_str::CompactString;

use crate::model::internal::MdMessage;
use crate::model::order_book::OrderBook;

pub struct Storage {
    order_books: HashMap<CompactString, OrderBook>
}

impl Storage {
    pub fn new() -> Self {
        Self {
            order_books: HashMap::new(),
        }
    }

    pub fn on_ws_update(&mut self, message: MdMessage) {
        match message {
            MdMessage::L2Snapshot(snapshot) => {
                match self.order_books.entry(snapshot.symbol.clone()) {
                    Entry::Occupied(mut o) => {
                        o.get_mut().process_snapshot(snapshot);
                    }
                    Entry::Vacant(v) => {
                        let mut ob = OrderBook::new();
                        ob.process_snapshot(snapshot);
                        v.insert(ob);
                    }
                }
            }
            MdMessage::L2Increment(increment) => {
                match self.order_books.entry(increment.symbol.clone()) {
                    Entry::Occupied(mut o) => {
                        o.get_mut().process_update(increment);
                    }
                    Entry::Vacant(v) => {
                        let mut ob = OrderBook::new();
                        ob.process_update(increment);
                        v.insert(ob);
                    }
                }
            }
        }
    }

    pub fn on_order_book(&mut self, symbol: CompactString, order_book: OrderBook) {
        match self.order_books.entry(symbol) {
            Entry::Occupied(mut o) => {
                o.get_mut().update_on_order_book(order_book);
            }
            Entry::Vacant(v) => {
                let mut ob = OrderBook::new();
                ob.update_on_order_book(order_book);
                v.insert(ob);
            }
        }
    }
}