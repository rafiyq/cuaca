use crate::core::error::CuacaError;
use quick_xml::de::Deserializer;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct RssFeed {
    pub(super) channel: Channel,
}

#[derive(Debug, Deserialize)]
pub(super) struct Channel {
    pub(super) item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(super) struct Item {
    pub(super) title: Option<String>,
    pub(super) link: Option<String>,
    pub(super) description: Option<String>,
    #[serde(rename = "pubDate")]
    pub(super) pub_date: Option<String>,
}

pub(super) fn parse_rss(xml: &[u8]) -> Result<RssFeed, CuacaError> {
    let mut deserializer = Deserializer::from_reader(xml);
    RssFeed::deserialize(&mut deserializer).map_err(|e| CuacaError::Parse(e.to_string()))
}
