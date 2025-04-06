use serde::Deserialize;
use time::{Date, macros::format_description};

/// Deserialize a raw input into a [`time::Date`] object.
pub fn date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = Deserialize::deserialize(deserializer)?;
    let format = format_description!("[year]-[month]-[day]");
    Date::parse(date_str, &format).map_err(serde::de::Error::custom)
}

/// Deserialize a raw optional input into a [`time::Date`] object.
/// Invalid inputs will be converted to `None`.
pub fn optional_date<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = Deserialize::deserialize(deserializer)?;
    let format = format_description!("[year]-[month]-[day]");
    match date_str {
        Some(date_str) => match Date::parse(date_str, &format).map(Some) {
            Ok(date) => Ok(date),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

/// Deserialize a raw input into an optional pagination value.
/// Invalid inputs will be converted to `None`.
pub fn pagination_value_with_fallback<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer) {
        Ok(value) => Ok(Some(value)),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{self, Deserialize};
    use time::Date;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(deserialize_with = "date")]
        date: Date,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct OptionalTestStruct {
        #[serde(deserialize_with = "optional_date")]
        date: Option<Date>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct PaginationTestStruct {
        #[serde(deserialize_with = "pagination_value_with_fallback")]
        limit: Option<u32>,
    }

    #[test]
    fn test_date_happy() {
        let json_str = r#"{
            "date": "2025-04-01"
        }"#;
        let test_struct: TestStruct = serde_json::from_str(json_str).unwrap();
        let expected_date = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();
        assert_eq!(test_struct.date, expected_date);
    }

    #[test]
    fn test_date_invalid_date() {
        let json_str = r#"{
            "date": "2025-04-31"
        }"#;
        let result = serde_json::from_str::<TestStruct>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_date_null_date() {
        let json_str = r#"{
            "date": null
        }"#;
        let result = serde_json::from_str::<TestStruct>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_optional_date_happy() {
        let json_str = r#"{
            "date": "2025-04-01"
        }"#;
        let test_struct: OptionalTestStruct = serde_json::from_str(json_str).unwrap();
        let expected_date = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();
        assert_eq!(test_struct.date, Some(expected_date));
    }

    #[test]
    fn test_optional_date_invalid_date() {
        let json_str = r#"{
            "date": "2025-04-31"
        }"#;
        let result = serde_json::from_str::<OptionalTestStruct>(json_str);
        assert!(result.is_ok());
        assert!(result.unwrap().date.is_none());
    }

    #[test]
    fn test_optional_date_null_date() {
        let json_str = r#"{
            "date": null
        }"#;
        let result = serde_json::from_str::<OptionalTestStruct>(json_str);
        assert!(result.is_ok());
        assert!(result.unwrap().date.is_none());
    }

    #[test]
    fn test_pagination_value_with_fallback_happy() {
        let json_str = r#"{
            "limit": 10
        }"#;
        let test_struct: PaginationTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.limit, Some(10));
    }

    #[test]
    fn test_pagination_value_with_fallback_invalid() {
        let json_str = r#"{
            "limit": "invalid"
        }"#;
        let result = serde_json::from_str::<PaginationTestStruct>(json_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().limit, None);
    }
}
