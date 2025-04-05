use serde::Deserialize;
use time::{Date, macros::format_description};

/// Deserialize a raw input into a [`time::Date`] object.
pub fn raw_to_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = Deserialize::deserialize(deserializer)?;
    let format = format_description!("[year]-[month]-[day]");
    Date::parse(date_str, &format).map_err(serde::de::Error::custom)
}
