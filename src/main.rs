mod auction;
mod auction_config;

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serializer, SerializeSeq};

use std::fmt;
use std::io::{self, BufReader, BufWriter};

const CONFIG_PATH: &str = "/auction/config.json";

struct AuctionProcessor<'s, S: SerializeSeq> {
    config: auction_config::Config,
    seq_serializer: &'s mut S,
}

impl<'de, S> Visitor<'de> for AuctionProcessor<'de, S>
where
    S: SerializeSeq,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array of auctions")
    }

    fn visit_seq<SA>(self, mut seq: SA) -> Result<(), SA::Error>
    where
        SA: SeqAccess<'de>,
    {
        // Stream the auctions, processing them and serializing the results as they arrive
        while let Some(auction) = seq.next_element::<auction::Auction>()? {
            let winning_bids = auction::get_winning_bids(&auction, &self.config);
            self.seq_serializer
                .serialize_element(&winning_bids)
                .map_err(|_| de::Error::custom("failed to serialize winning bids"))?;
        }

        Ok(())
    }
}

fn main() -> serde_json::Result<()> {
    let reader = BufReader::new(io::stdin());
    let mut deserializer = serde_json::Deserializer::from_reader(reader);

    let writer = BufWriter::new(io::stdout());
    let mut serializer = serde_json::Serializer::new(writer);
    let mut seq_serializer = serializer.serialize_seq(None)?;

    let processor = AuctionProcessor {
        config: auction_config::get_config(CONFIG_PATH),
        seq_serializer: &mut seq_serializer,
    };

    deserializer.deserialize_seq(processor).unwrap();

    seq_serializer.end()
}
