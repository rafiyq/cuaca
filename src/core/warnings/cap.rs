use crate::core::error::CuacaError;
use quick_xml::de::Deserializer;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "alert")]
pub(super) struct CapAlert {
    pub(super) info: Info,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) struct Info {
    pub(super) headline: String,
    pub(super) effective: String,
    pub(super) expires: String,
    pub(super) area: Area,
    pub(super) web: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) struct Area {
    #[serde(rename = "areaDesc")]
    pub(super) area_desc: String,
    #[serde(rename = "polygon")]
    pub(super) polygons: Vec<String>,
}

pub(super) fn parse_cap(xml: &[u8]) -> Result<CapAlert, CuacaError> {
    let mut deserializer = Deserializer::from_reader(xml);
    CapAlert::deserialize(&mut deserializer).map_err(|e| CuacaError::Parse(e.to_string()))
}
