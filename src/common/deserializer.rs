use serde::{Deserialize, de};
use time::{Date, macros::format_description};

/// Deserialize a raw input into a [`time::Date`] object.
pub fn date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str = Deserialize::deserialize(deserializer)?;
    let format = format_description!("[year]-[month]-[day]");
    Date::parse(date_str, &format).map_err(de::Error::custom)
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

/// Deserialize a raw input into an optional boolean value.
/// Invalid inputs will be converted to `None`.
pub fn bool_with_fallback<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer) {
        Ok(value) => Ok(Some(value)),
        Err(_) => Ok(None),
    }
}

/// Deserialize a raw input into a non-negative integer.
pub fn non_negative_int<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i32::deserialize(deserializer)?;
    if value < 0 {
        return Err(de::Error::custom("Value must be non-negative"));
    }

    Ok(value)
}

/// Deserialize a raw input into a positive integer.
/// Invalid input will result in an error.
pub fn positive_int<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i32::deserialize(deserializer)?;
    if value < 1 {
        return Err(de::Error::custom("Value must be positive"));
    }

    Ok(value)
}

/// Deserialize a raw input into a vector of positive integers.
/// Invalid input will result in an error.
pub fn positive_int_vec<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Vec::<i32>::deserialize(deserializer)?;
    for &value in &values {
        if value < 1 {
            return Err(de::Error::custom("Value must be positive"));
        }
    }

    Ok(values)
}

/// Deserialize a raw input into a priority value.
/// A valid priority value is between 0 and 2.
pub fn priority_value<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i32::deserialize(deserializer)?;
    if value < 0 || value > 2 {
        return Err(de::Error::custom("Priority must be between 0 and 2"));
    }

    Ok(value)
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

    #[derive(Debug, Deserialize, PartialEq)]
    struct NullableTestStruct {
        #[serde(deserialize_with = "bool_with_fallback")]
        nullable: Option<bool>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct NonNegativeIntTestStruct {
        #[serde(deserialize_with = "non_negative_int")]
        value: i32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct PositiveIntTestStruct {
        #[serde(deserialize_with = "positive_int")]
        value: i32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct PositiveIntVecTestStruct {
        #[serde(deserialize_with = "positive_int_vec")]
        values: Vec<i32>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct PriorityTestStruct {
        #[serde(deserialize_with = "priority_value")]
        priority: i32,
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

    #[test]
    fn test_bool_with_fallback_happy() {
        let json_str = r#"{
            "nullable": true
        }"#;
        let test_struct: NullableTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.nullable, Some(true));
    }

    #[test]
    fn test_bool_with_fallback_invalid() {
        let json_str = r#"{
            "nullable": "invalid"
        }"#;
        let result = serde_json::from_str::<NullableTestStruct>(json_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nullable, None);
    }

    #[test]
    fn test_non_negative_int_positive() {
        let json_str = r#"{
            "value": 10
        }"#;
        let test_struct: NonNegativeIntTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.value, 10);
    }

    #[test]
    fn test_non_negative_int_zero() {
        let json_str = r#"{
            "value": 0
        }"#;
        let test_struct: NonNegativeIntTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.value, 0);
    }

    #[test]
    fn test_non_negative_int_negative() {
        let json_str = r#"{
            "value": -5
        }"#;
        let result = serde_json::from_str::<NonNegativeIntTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be non-negative")
        );
    }

    #[test]
    fn test_positive_int_happy() {
        let json_str = r#"{
            "value": 10
        }"#;
        let test_struct: PositiveIntTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.value, 10);
    }

    #[test]
    fn test_positive_int_zero() {
        let json_str = r#"{
            "value": 0
        }"#;
        let result = serde_json::from_str::<PositiveIntTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_positive_int_negative() {
        let json_str = r#"{
            "value": -5
        }"#;
        let result = serde_json::from_str::<PositiveIntTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_positive_int_vec_happy() {
        let json_str = r#"{
            "values": [1, 5, 10, 42]
        }"#;
        let test_struct: PositiveIntVecTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.values, vec![1, 5, 10, 42]);
    }

    #[test]
    fn test_positive_int_vec_empty() {
        let json_str = r#"{
            "values": []
        }"#;
        let test_struct: PositiveIntVecTestStruct = serde_json::from_str(json_str).unwrap();
        assert_eq!(test_struct.values, Vec::<i32>::new());
    }

    #[test]
    fn test_positive_int_vec_with_zero() {
        let json_str = r#"{
            "values": [1, 0, 5]
        }"#;
        let result = serde_json::from_str::<PositiveIntVecTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_positive_int_vec_with_negative() {
        let json_str = r#"{
            "values": [10, -5, 20]
        }"#;
        let result = serde_json::from_str::<PositiveIntVecTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_priority_value_valid() {
        let valid_values = [0, 1, 2];

        for value in valid_values {
            let json_str = format!(
                r#"{{
                "priority": {}
            }}"#,
                value
            );

            let test_struct: PriorityTestStruct = serde_json::from_str(&json_str).unwrap();
            assert_eq!(test_struct.priority, value);
        }
    }

    #[test]
    fn test_priority_value_too_low() {
        let json_str = r#"{
            "priority": -1
        }"#;

        let result = serde_json::from_str::<PriorityTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Priority must be between 0 and 2")
        );
    }

    #[test]
    fn test_priority_value_too_high() {
        let json_str = r#"{
            "priority": 3
        }"#;

        let result = serde_json::from_str::<PriorityTestStruct>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Priority must be between 0 and 2")
        );
    }
}
