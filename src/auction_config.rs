use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct JsonSiteConfig {
    name: String,
    bidders: Vec<String>,
    floor: f64,
}

#[derive(Deserialize, Debug)]
struct JsonBidderConfig {
    name: String,
    adjustment: f64,
}

#[derive(Deserialize, Debug)]
struct JsonConfig {
    sites: Vec<JsonSiteConfig>,
    bidders: Vec<JsonBidderConfig>,
}

#[derive(Debug)]
pub struct SiteConfig {
    pub bidders: HashSet<String>,
    pub floor: f64,
}

#[derive(Debug)]
pub struct Config {
    pub sites: HashMap<String, SiteConfig>,
    pub bidder_adjustments: HashMap<String, f64>,
}

fn load_json_config<P: AsRef<Path>>(path: P) -> Result<JsonConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

pub fn get_config<P: AsRef<Path>>(path: P) -> Config {
    let json_config = load_json_config(path).unwrap();

    let mut sites = HashMap::new();

    for site in &json_config.sites {
        sites.insert(
            site.name.clone(),
            SiteConfig {
                bidders: HashSet::from_iter(site.bidders.clone()),
                floor: site.floor,
            },
        );
    }

    let mut bidder_adjustments = HashMap::new();

    for bidder in &json_config.bidders {
        bidder_adjustments.insert(bidder.name.clone(), bidder.adjustment);
    }

    Config {
        sites,
        bidder_adjustments,
    }
}
