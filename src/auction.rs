use crate::auction_config::{Config, SiteConfig};

use serde::{Deserialize, Serialize, Serializer};

use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Bid {
    pub bidder: String,
    pub unit: String,
    #[serde(serialize_with = "serialize_float")]
    pub bid: f64,
}

#[derive(Deserialize, Debug)]
pub struct Auction {
    pub site: String,
    pub units: Vec<String>,
    pub bids: Vec<Bid>,
}

struct WinningBid<'a> {
    bid: &'a Bid,
    adjusted_bid_value: f64,
}

fn is_valid_bid(bid: &Bid, auction: &Auction, site_config: &SiteConfig) -> bool {
    auction.units.contains(&bid.unit) &&      // Bid is for a valid ad unit
    site_config.bidders.contains(&bid.bidder) // Bidder is permitted to bid on the site
}

pub fn get_winning_bids<'a>(auction: &'a Auction, config: &Config) -> Vec<&'a Bid> {
    let site_config = match config.sites.get(&auction.site) {
        Some(site_config) => site_config,
        None => return Vec::new(), // The site is unrecognized
    };
    let site_floor = site_config.floor;

    let mut unit_winning_bids: BTreeMap<&String, WinningBid> = BTreeMap::new();

    for bid in &auction.bids {
        if !is_valid_bid(&bid, &auction, &site_config) {
            continue;
        }

        let bidder_adjustment = match config.bidder_adjustments.get(&bid.bidder) {
            Some(adjustment) => adjustment,
            None => continue, // Bidder is unknown
        };
        let adjusted_bid_value = bid.bid + bidder_adjustment;

        if adjusted_bid_value < site_floor {
            continue; // Bid is invalid since it's below the site's floor
        }

        let cur_winning_bid = unit_winning_bids.get(&bid.unit);

        if cur_winning_bid.is_none() // No other bids yet
            || adjusted_bid_value > cur_winning_bid.unwrap().adjusted_bid_value
        {
            unit_winning_bids.insert(
                &bid.unit,
                WinningBid {
                    bid,
                    adjusted_bid_value,
                },
            );
        }
    }

    // Return the winners' original bid objects
    unit_winning_bids
        .values()
        .map(|winning_bid| winning_bid.bid)
        .collect()
}

// Fix for serializing f64 values with no fractional part
fn serialize_float<S>(f: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if f.fract() == 0.0 {
        serializer.serialize_i64(*f as i64)
    } else {
        serializer.serialize_f64(*f)
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    use crate::auction_config;

    fn get_test_config() -> Config {
        auction_config::get_config("config.json")
    }

    fn auction_from_json(json: &str) -> Auction {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_example_auction() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "banner",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![
                &Bid {
                    bidder: "AUCT".to_string(),
                    unit: "banner".to_string(),
                    bid: 35.0,
                },
                &Bid {
                    bidder: "BIDD".to_string(),
                    unit: "sidebar".to_string(),
                    bid: 60.0,
                },
            ]
        );
    }

    #[test]
    fn test_unknwn_site() {
        let auction = auction_from_json(
            r#"{
              "site": "unknown.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "banner",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        let expected: Vec<&Bid> = vec![];

        assert_eq!(get_winning_bids(&auction, &get_test_config()), expected);
    }

    #[test]
    fn test_all_bids_below_floor() {
        let auction = auction_from_json(
            r#"{
              "site": "expensive.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "banner",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        let expected: Vec<&Bid> = vec![];

        assert_eq!(get_winning_bids(&auction, &get_test_config()), expected);
    }

    #[test]
    fn test_unknown_bidders() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "WHO?",
                  "unit": "banner",
                  "bid": 35
                },
                {
                  "bidder": "UNKNOWN",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "AUCT".to_string(),
                unit: "sidebar".to_string(),
                bid: 55.0,
            },]
        );
    }

    #[test]
    fn test_bidder_not_allowed_by_site() {
        let auction = auction_from_json(
            r#"{
              "site": "auct-only.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "BIDD",
                  "unit": "banner",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 600
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "AUCT".to_string(),
                unit: "sidebar".to_string(),
                bid: 55.0,
            },]
        );
    }

    #[test]
    fn test_unknwn_auction_unit() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "banner",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "AUCT".to_string(),
                unit: "banner".to_string(),
                bid: 35.0,
            },]
        );
    }

    #[test]
    fn test_unknwn_bidder_unit() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "unknown",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "BIDD".to_string(),
                unit: "sidebar".to_string(),
                bid: 60.0,
            },]
        );
    }

    #[test]
    fn test_adjustment_below_site_floor() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "banner",
                  "bid": 32
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 55
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "BIDD".to_string(),
                unit: "sidebar".to_string(),
                bid: 60.0,
            },]
        );
    }

    #[test]
    fn test_subsequent_highest_bidder_wins() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                },
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 61
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "AUCT".to_string(),
                unit: "sidebar".to_string(),
                bid: 61.0,
            },]
        );
    }

    #[test]
    fn test_first_highest_bidder_wins() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 60.0625
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "AUCT".to_string(),
                unit: "sidebar".to_string(),
                bid: 60.0625,
            },]
        );
    }

    #[test]
    fn test_bidder_loses_due_to_adjustment() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": [
                {
                  "bidder": "AUCT",
                  "unit": "sidebar",
                  "bid": 60.062
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "BIDD".to_string(),
                unit: "sidebar".to_string(),
                bid: 60.0,
            },]
        );
    }

    #[test]
    fn test_bidder_multiple_bids() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["sidebar"],
              "bids": [
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 35
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60.0625
                },
                {
                  "bidder": "BIDD",
                  "unit": "sidebar",
                  "bid": 60
                }
              ]
            }"#,
        );

        assert_eq!(
            get_winning_bids(&auction, &get_test_config()),
            vec![&Bid {
                bidder: "BIDD".to_string(),
                unit: "sidebar".to_string(),
                bid: 60.0625,
            },]
        );
    }

    #[test]
    fn test_no_bids() {
        let auction = auction_from_json(
            r#"{
              "site": "houseofcheese.com",
              "units": ["banner", "sidebar"],
              "bids": []
            }"#,
        );

        let expected: Vec<&Bid> = vec![];

        assert_eq!(get_winning_bids(&auction, &get_test_config()), expected);
    }
}
