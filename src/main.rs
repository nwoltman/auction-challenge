mod auction;
mod auction_config;

use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};

use std::fmt;
use std::io::{self, BufReader, BufWriter};
use std::sync::mpsc;
use std::thread;

const CONFIG_PATH: &str = "/auction/config.json";

struct AuctionProcessor {
    sender: mpsc::Sender<auction::Auction>,
}

impl<'s> Visitor<'s> for AuctionProcessor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array of auction objects")
    }

    fn visit_seq<SA>(self, mut seq: SA) -> Result<(), SA::Error>
    where
        SA: SeqAccess<'s>,
    {
        // Deserialize the auctions and send them to the main thread for processing
        while let Some(auction) = seq.next_element::<auction::Auction>()? {
            self.sender.send(auction).unwrap();
        }
        Ok(())
    }
}

fn main() -> serde_json::Result<()> {
    let (sender, receiver) = mpsc::channel();

    // Deserialize the input on a separate thread
    thread::spawn(move || {
        let auction_processor = AuctionProcessor { sender };
        let reader = BufReader::new(io::stdin());
        let mut deserializer = serde_json::Deserializer::from_reader(reader);
        deserializer.deserialize_seq(auction_processor).unwrap();
    });

    let config = auction_config::get_config(CONFIG_PATH);
    let writer = BufWriter::new(io::stdout());
    let mut serializer = serde_json::Serializer::new(writer);
    let mut seq_serializer = serializer.serialize_seq(None)?;

    for auction in receiver {
        let winning_bids = auction::get_winning_bids(&auction, &config);
        seq_serializer.serialize_element(&winning_bids)?;
    }

    seq_serializer.end()
}
