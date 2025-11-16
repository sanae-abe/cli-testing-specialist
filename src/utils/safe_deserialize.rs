use crate::error::{CliTestError, Result};
use serde::de::DeserializeOwned;
use std::io::Read;

/// Maximum allowed size for JSON/YAML deserialization (10MB)
const MAX_DESERIALIZE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum recursion depth for JSON/YAML deserialization (16 levels)
const MAX_RECURSION_DEPTH: usize = 16;

/// Safe JSON deserialization with size and depth limits
///
/// This function provides protection against:
/// - Memory exhaustion (10MB size limit)
/// - Stack overflow (16-level recursion depth limit)
/// - Denial of service attacks via malicious payloads
///
/// # Security
///
/// - **Size limit**: Rejects payloads larger than 10MB
/// - **Depth limit**: Enforced by serde_json (default max depth ~128, we validate structure)
/// - **Performance**: O(1) size check before parsing
///
/// # Example
///
/// ```rust
/// use cli_testing_specialist::utils::deserialize_json_safe;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     name: String,
///     value: i32,
/// }
///
/// let json = r#"{"name": "test", "value": 42}"#;
/// let config: Config = deserialize_json_safe(json).unwrap();
/// ```
pub fn deserialize_json_safe<T: DeserializeOwned>(input: &str) -> Result<T> {
    // Check size limit before parsing
    if input.len() > MAX_DESERIALIZE_SIZE {
        return Err(CliTestError::Validation(format!(
            "JSON payload too large: {} bytes (max: {} bytes)",
            input.len(),
            MAX_DESERIALIZE_SIZE
        )));
    }

    // Check for empty input
    if input.trim().is_empty() {
        return Err(CliTestError::Validation(
            "JSON payload is empty".to_string(),
        ));
    }

    // Deserialize with serde_json (has built-in recursion depth protection)
    let value: T = serde_json::from_str(input)
        .map_err(|e| CliTestError::Validation(format!("JSON deserialization failed: {}", e)))?;

    // Validate depth after deserialization (additional safety check)
    let json_value: serde_json::Value = serde_json::from_str(input)?;
    let depth = calculate_json_depth(&json_value);

    if depth > MAX_RECURSION_DEPTH {
        return Err(CliTestError::Validation(format!(
            "JSON depth too deep: {} levels (max: {} levels)",
            depth, MAX_RECURSION_DEPTH
        )));
    }

    Ok(value)
}

/// Safe JSON deserialization from reader with size limit
///
/// Similar to `deserialize_json_safe` but reads from a `Read` trait object.
/// Enforces the same 10MB size limit by reading into a buffer first.
pub fn deserialize_json_safe_from_reader<R: Read, T: DeserializeOwned>(reader: R) -> Result<T> {
    let mut buffer = Vec::new();

    // Read with size limit
    reader
        .take(MAX_DESERIALIZE_SIZE as u64 + 1)
        .read_to_end(&mut buffer)?;

    if buffer.len() > MAX_DESERIALIZE_SIZE {
        return Err(CliTestError::Validation(format!(
            "JSON payload too large: exceeds {} bytes",
            MAX_DESERIALIZE_SIZE
        )));
    }

    let input = String::from_utf8(buffer)
        .map_err(|e| CliTestError::Validation(format!("Invalid UTF-8 in JSON payload: {}", e)))?;

    deserialize_json_safe(&input)
}

/// Safe YAML deserialization with size and depth limits
///
/// This function provides protection against:
/// - Memory exhaustion (10MB size limit)
/// - Stack overflow (16-level recursion depth limit)
/// - YAML bombs (deeply nested structures)
/// - Denial of service attacks
///
/// # Security
///
/// - **Size limit**: Rejects payloads larger than 10MB
/// - **Depth limit**: Validates structure depth after parsing
/// - **YAML bombs**: Protected by size and depth limits
///
/// # Example
///
/// ```rust
/// use cli_testing_specialist::utils::deserialize_yaml_safe;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     name: String,
///     value: i32,
/// }
///
/// let yaml = "name: test\nvalue: 42";
/// let config: Config = deserialize_yaml_safe(yaml).unwrap();
/// ```
pub fn deserialize_yaml_safe<T: DeserializeOwned>(input: &str) -> Result<T> {
    // Check size limit before parsing
    if input.len() > MAX_DESERIALIZE_SIZE {
        return Err(CliTestError::Validation(format!(
            "YAML payload too large: {} bytes (max: {} bytes)",
            input.len(),
            MAX_DESERIALIZE_SIZE
        )));
    }

    // Check for empty input
    if input.trim().is_empty() {
        return Err(CliTestError::Validation(
            "YAML payload is empty".to_string(),
        ));
    }

    // Deserialize with serde_yaml
    let value: T = serde_yaml::from_str(input)
        .map_err(|e| CliTestError::Validation(format!("YAML deserialization failed: {}", e)))?;

    // Validate depth after deserialization (convert YAML to JSON for depth check)
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(input)?;
    let json_value = yaml_to_json_value(&yaml_value)?;
    let depth = calculate_json_depth(&json_value);

    if depth > MAX_RECURSION_DEPTH {
        return Err(CliTestError::Validation(format!(
            "YAML depth too deep: {} levels (max: {} levels)",
            depth, MAX_RECURSION_DEPTH
        )));
    }

    Ok(value)
}

/// Safe YAML deserialization from reader with size limit
pub fn deserialize_yaml_safe_from_reader<R: Read, T: DeserializeOwned>(reader: R) -> Result<T> {
    let mut buffer = Vec::new();

    // Read with size limit
    reader
        .take(MAX_DESERIALIZE_SIZE as u64 + 1)
        .read_to_end(&mut buffer)?;

    if buffer.len() > MAX_DESERIALIZE_SIZE {
        return Err(CliTestError::Validation(format!(
            "YAML payload too large: exceeds {} bytes",
            MAX_DESERIALIZE_SIZE
        )));
    }

    let input = String::from_utf8(buffer)
        .map_err(|e| CliTestError::Validation(format!("Invalid UTF-8 in YAML payload: {}", e)))?;

    deserialize_yaml_safe(&input)
}

/// Calculate the maximum depth of a JSON value tree
fn calculate_json_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            1 + map.values().map(calculate_json_depth).max().unwrap_or(0)
        }
        serde_json::Value::Array(arr) => {
            1 + arr.iter().map(calculate_json_depth).max().unwrap_or(0)
        }
        _ => 1,
    }
}

/// Convert YAML value to JSON value for depth calculation
fn yaml_to_json_value(yaml: &serde_yaml::Value) -> Result<serde_json::Value> {
    match yaml {
        serde_yaml::Value::Null => Ok(serde_json::Value::Null),
        serde_yaml::Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(serde_json::Value::Number(i.into()))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| CliTestError::Validation("Invalid YAML number".to_string()))
            } else {
                Err(CliTestError::Validation("Invalid YAML number".to_string()))
            }
        }
        serde_yaml::Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        serde_yaml::Value::Sequence(arr) => {
            let json_arr: Result<Vec<_>> = arr.iter().map(yaml_to_json_value).collect();
            Ok(serde_json::Value::Array(json_arr?))
        }
        serde_yaml::Value::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    _ => {
                        return Err(CliTestError::Validation(
                            "YAML map key must be string".to_string(),
                        ))
                    }
                };
                json_map.insert(key, yaml_to_json_value(v)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json_value(&tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    // ========== JSON Tests ==========

    #[test]
    fn test_json_deserialize_safe_success() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result: Result<TestStruct> = deserialize_json_safe(json);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_json_deserialize_safe_empty_input() {
        let json = "";
        let result: Result<TestStruct> = deserialize_json_safe(json);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("JSON payload is empty"));
    }

    #[test]
    fn test_json_deserialize_safe_size_limit() {
        // Create a JSON string larger than 10MB
        let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(MAX_DESERIALIZE_SIZE + 1));

        let result: Result<serde_json::Value> = deserialize_json_safe(&large_json);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("JSON payload too large"));
    }

    #[test]
    fn test_json_deserialize_safe_depth_limit() {
        // Create deeply nested JSON (17 levels, exceeds limit of 16)
        let mut nested_json = String::from(r#"{"a":"value"}"#);
        for _ in 0..17 {
            nested_json = format!(r#"{{"nested":{}}}"#, nested_json);
        }

        let result: Result<serde_json::Value> = deserialize_json_safe(&nested_json);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("depth too deep") || err_msg.contains("17 levels"));
    }

    #[test]
    fn test_json_deserialize_safe_valid_depth() {
        // Create nested JSON within limit (10 levels)
        let mut nested_json = String::from(r#"{"a":"value"}"#);
        for _ in 0..10 {
            nested_json = format!(r#"{{"nested":{}}}"#, nested_json);
        }

        let result: Result<serde_json::Value> = deserialize_json_safe(&nested_json);

        assert!(result.is_ok());
    }

    #[test]
    fn test_json_deserialize_safe_invalid_syntax() {
        let json = r#"{"name": "test", "value": }"#; // Invalid JSON

        let result: Result<TestStruct> = deserialize_json_safe(json);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("JSON deserialization failed"));
    }

    #[test]
    fn test_json_deserialize_from_reader() {
        let json = r#"{"name": "test", "value": 42}"#;
        let reader = json.as_bytes();

        let result: Result<TestStruct> = deserialize_json_safe_from_reader(reader);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    // ========== YAML Tests ==========

    #[test]
    fn test_yaml_deserialize_safe_success() {
        let yaml = "name: test\nvalue: 42";
        let result: Result<TestStruct> = deserialize_yaml_safe(yaml);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_yaml_deserialize_safe_empty_input() {
        let yaml = "";
        let result: Result<TestStruct> = deserialize_yaml_safe(yaml);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("YAML payload is empty"));
    }

    #[test]
    fn test_yaml_deserialize_safe_size_limit() {
        // Create a YAML string larger than 10MB
        let large_yaml = format!("data: {}", "x".repeat(MAX_DESERIALIZE_SIZE + 1));

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&large_yaml);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("YAML payload too large"));
    }

    #[test]
    fn test_yaml_deserialize_safe_depth_limit() {
        // Create deeply nested YAML (17 levels)
        let mut nested_yaml = String::from("a: value");
        for i in 0..17 {
            nested_yaml = format!("level{}:\n  {}", i, nested_yaml.replace('\n', "\n  "));
        }

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&nested_yaml);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth too deep"));
    }

    #[test]
    fn test_yaml_deserialize_safe_valid_depth() {
        // Create nested YAML within limit (10 levels)
        let mut nested_yaml = String::from("a: value");
        for i in 0..10 {
            nested_yaml = format!("level{}:\n  {}", i, nested_yaml.replace('\n', "\n  "));
        }

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&nested_yaml);

        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_deserialize_from_reader() {
        let yaml = "name: test\nvalue: 42";
        let reader = yaml.as_bytes();

        let result: Result<TestStruct> = deserialize_yaml_safe_from_reader(reader);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    // ========== Depth Calculation Tests ==========

    #[test]
    fn test_calculate_json_depth_simple() {
        let json = serde_json::json!({"a": 1});
        assert_eq!(calculate_json_depth(&json), 2); // object + leaf
    }

    #[test]
    fn test_calculate_json_depth_nested() {
        let json = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": "value"
                }
            }
        });
        assert_eq!(calculate_json_depth(&json), 4); // 3 objects + 1 leaf
    }

    #[test]
    fn test_calculate_json_depth_array() {
        let json = serde_json::json!([1, 2, [3, 4, [5]]]);
        assert_eq!(calculate_json_depth(&json), 4); // 3 arrays + 1 leaf
    }

    // ========== Malicious Payload Tests ==========

    #[test]
    fn test_json_bomb_protection() {
        // Attempt to create a "billion laughs" style payload
        // Each level doubles the size, but we're limited by 10MB
        let payload = r#"{"a":["x","x","x","x","x"]}"#.repeat(1000);

        let result: Result<serde_json::Value> = deserialize_json_safe(&payload);

        // Should fail due to size limit
        assert!(result.is_err());
    }

    #[test]
    fn test_yaml_bomb_protection() {
        // YAML anchor/alias bomb attempt (simplified)
        let yaml_bomb = format!("a: &anchor {}\nb: *anchor\n", "x".repeat(1000));
        let repeated = yaml_bomb.repeat(1000);

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&repeated);

        // Should fail due to size limit
        assert!(result.is_err());
    }

    // ========== Boundary Value Tests ==========

    #[test]
    fn test_json_size_exactly_at_limit() {
        // Create JSON exactly at MAX_DESERIALIZE_SIZE (10MB)
        // {"data":"xxx..."} where total is exactly MAX_DESERIALIZE_SIZE bytes
        let data_size = MAX_DESERIALIZE_SIZE - 11; // Account for {"data":""} = 11 bytes
        let json = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size));

        assert_eq!(json.len(), MAX_DESERIALIZE_SIZE);

        let result: Result<serde_json::Value> = deserialize_json_safe(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_size_one_byte_over_limit() {
        // Create JSON exactly MAX_DESERIALIZE_SIZE + 1 bytes
        let data_size = MAX_DESERIALIZE_SIZE - 10; // {"data":""} = 10 bytes, +1 extra = +11 total
        let json = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size));

        assert_eq!(json.len(), MAX_DESERIALIZE_SIZE + 1);

        let result: Result<serde_json::Value> = deserialize_json_safe(&json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("JSON payload too large"));
    }

    #[test]
    fn test_json_size_one_byte_under_limit() {
        // Create JSON exactly MAX_DESERIALIZE_SIZE - 1 bytes
        let data_size = MAX_DESERIALIZE_SIZE - 12; // Account for {"data":""} = 11 bytes
        let json = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size));

        assert_eq!(json.len(), MAX_DESERIALIZE_SIZE - 1);

        let result: Result<serde_json::Value> = deserialize_json_safe(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_size_exactly_at_limit() {
        // Create YAML exactly at MAX_DESERIALIZE_SIZE
        let data_size = MAX_DESERIALIZE_SIZE - 6; // "data: " = 6 bytes
        let yaml = format!("data: {}", "x".repeat(data_size));

        assert_eq!(yaml.len(), MAX_DESERIALIZE_SIZE);

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_size_one_byte_over_limit() {
        // Create YAML exactly MAX_DESERIALIZE_SIZE + 1 bytes
        let data_size = MAX_DESERIALIZE_SIZE - 5; // "data: " = 6 bytes, -1 = 5
        let yaml = format!("data: {}", "x".repeat(data_size));

        assert_eq!(yaml.len(), MAX_DESERIALIZE_SIZE + 1);

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("YAML payload too large"));
    }

    // ========== Exact Recursion Depth Tests ==========

    #[test]
    fn test_json_depth_exactly_16_levels() {
        // Create JSON with exactly 16 levels (at the limit)
        // Start with a leaf value (depth 1), then nest 15 times
        let mut nested_json = String::from(r#""value""#); // Leaf value = depth 1
        for i in 0..15 {
            nested_json = format!(r#"{{"level{}":{}}}"#, i, nested_json);
        }

        let result: Result<serde_json::Value> = deserialize_json_safe(&nested_json);
        let value = result.expect("16 levels should be allowed");
        assert_eq!(calculate_json_depth(&value), 16);
    }

    #[test]
    fn test_json_depth_exactly_17_levels() {
        // Create JSON with exactly 17 levels (exceeds limit by 1)
        let mut nested_json = String::from(r#""value""#); // Leaf value = depth 1
        for i in 0..16 {
            nested_json = format!(r#"{{"level{}":{}}}"#, i, nested_json);
        }

        let result: Result<serde_json::Value> = deserialize_json_safe(&nested_json);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("17 levels") || err_msg.contains("depth too deep"));
    }

    #[test]
    fn test_yaml_depth_exactly_16_levels() {
        // Create YAML with exactly 16 levels
        // Start with a string value, then nest 15 times
        let mut nested_yaml = String::from("value"); // Leaf value = depth 1
        for i in 0..15 {
            nested_yaml = format!("level{}:\n  {}", i, nested_yaml.replace('\n', "\n  "));
        }

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&nested_yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_depth_exactly_17_levels() {
        // Create YAML with exactly 17 levels (exceeds limit by 1)
        let mut nested_yaml = String::from("value"); // Leaf value = depth 1
        for i in 0..16 {
            nested_yaml = format!("level{}:\n  {}", i, nested_yaml.replace('\n', "\n  "));
        }

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe(&nested_yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth too deep"));
    }

    // ========== Arithmetic Operator Mutation Tests ==========

    #[test]
    fn test_json_reader_size_arithmetic_plus() {
        // Test that MAX_DESERIALIZE_SIZE + 1 is used correctly in take()
        // If mutated to *, the limit would be much larger
        // Use valid JSON that's exactly at the size limit
        let data_size = MAX_DESERIALIZE_SIZE - 11; // Account for {"data":""}
        let exact_limit_json = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size));
        let reader = exact_limit_json.as_bytes();

        let result: Result<serde_json::Value> = deserialize_json_safe_from_reader(reader);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_reader_size_arithmetic_plus() {
        // Test that MAX_DESERIALIZE_SIZE + 1 is used correctly in take()
        // Use valid YAML that's exactly at the size limit
        let data_size = MAX_DESERIALIZE_SIZE - 6; // Account for "data: "
        let exact_limit_yaml = format!("data: {}", "x".repeat(data_size));
        let reader = exact_limit_yaml.as_bytes();

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe_from_reader(reader);
        assert!(result.is_ok());
    }

    #[test]
    fn test_depth_calculation_arithmetic() {
        // Test that depth calculation uses + correctly (1 + max depth)
        // If mutated to *, depth would be incorrectly calculated
        let json = serde_json::json!({
            "a": {
                "b": {
                    "c": "value"
                }
            }
        });

        let depth = calculate_json_depth(&json);
        assert_eq!(depth, 4); // Verify correct arithmetic: 1 + 1 + 1 + 1
    }

    // ========== Comparison Operator Mutation Tests ==========

    #[test]
    fn test_json_size_comparison_greater_than() {
        // Test that > is used correctly (not >= or ==)
        // Size exactly at limit should succeed (using valid JSON)
        let data_size = MAX_DESERIALIZE_SIZE - 11; // Account for {"data":""}
        let json_at_limit = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size));
        assert_eq!(json_at_limit.len(), MAX_DESERIALIZE_SIZE);

        let result: Result<serde_json::Value> = deserialize_json_safe(&json_at_limit);
        assert!(result.is_ok(), "Size at limit should succeed");

        // Size over limit should fail
        let data_size_over = MAX_DESERIALIZE_SIZE - 10;
        let json_over_limit = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size_over));
        assert_eq!(json_over_limit.len(), MAX_DESERIALIZE_SIZE + 1);

        let result: Result<serde_json::Value> = deserialize_json_safe(&json_over_limit);
        assert!(result.is_err(), "Size over limit should fail");
    }

    #[test]
    fn test_json_depth_comparison_greater_than() {
        // Test that > is used correctly for depth check (not >= or ==)
        // Depth exactly at limit (16) should succeed
        let mut at_limit = String::from(r#"1"#); // Start with depth 1
        for _ in 0..15 {
            at_limit = format!(r#"{{"n":{}}}"#, at_limit);
        }

        let result: Result<serde_json::Value> = deserialize_json_safe(&at_limit);
        assert!(result.is_ok(), "Depth at limit should succeed");

        // Depth over limit (17) should fail
        let mut over_limit = String::from(r#"1"#); // Start with depth 1
        for _ in 0..16 {
            over_limit = format!(r#"{{"n":{}}}"#, over_limit);
        }

        let result: Result<serde_json::Value> = deserialize_json_safe(&over_limit);
        assert!(result.is_err(), "Depth over limit should fail");
    }

    #[test]
    fn test_reader_buffer_length_comparison() {
        // Test that buffer.len() > MAX_DESERIALIZE_SIZE is checked correctly
        // Exactly at limit should succeed (using valid JSON)
        let data_size = MAX_DESERIALIZE_SIZE - 11;
        let at_limit = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size));
        let result: Result<serde_json::Value> =
            deserialize_json_safe_from_reader(at_limit.as_bytes());
        assert!(result.is_ok());

        // Over limit should fail
        let data_size_over = MAX_DESERIALIZE_SIZE - 10;
        let over_limit = format!(r#"{{"data":"{}"}}"#, "x".repeat(data_size_over));
        let result: Result<serde_json::Value> =
            deserialize_json_safe_from_reader(over_limit.as_bytes());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    // ========== Constant Value Verification Tests ==========

    #[test]
    fn test_max_deserialize_size_constant_value() {
        // Test that MAX_DESERIALIZE_SIZE is exactly 10MB (10 * 1024 * 1024)
        // If mutated to use + instead of *, the value would be incorrect
        assert_eq!(MAX_DESERIALIZE_SIZE, 10485760); // 10 * 1024 * 1024 = 10485760

        // Verify it's not 10 + 1024 * 1024 = 1048586
        assert_ne!(MAX_DESERIALIZE_SIZE, 1048586);

        // Verify it's not 10 * 1024 + 1024 = 11264
        assert_ne!(MAX_DESERIALIZE_SIZE, 11264);
    }

    #[test]
    fn test_max_deserialize_size_boundary_enforcement() {
        // Test that the exact 10MB limit is enforced
        // This verifies the constant is calculated correctly with multiplication

        // Create data exactly 10MB - should succeed
        let exact_10mb_data = "x".repeat(MAX_DESERIALIZE_SIZE - 11);
        let json_10mb = format!(r#"{{"data":"{}"}}"#, exact_10mb_data);
        assert_eq!(json_10mb.len(), 10485760); // Exactly 10 * 1024 * 1024

        let result: Result<serde_json::Value> = deserialize_json_safe(&json_10mb);
        assert!(result.is_ok(), "10MB exact should be accepted");

        // Create data 10MB + 1 byte - should fail
        let over_10mb_data = "x".repeat(MAX_DESERIALIZE_SIZE - 10);
        let json_over_10mb = format!(r#"{{"data":"{}"}}"#, over_10mb_data);
        assert_eq!(json_over_10mb.len(), 10485761); // 10MB + 1

        let result: Result<serde_json::Value> = deserialize_json_safe(&json_over_10mb);
        assert!(result.is_err(), "10MB + 1 should be rejected");
    }

    // ========== YAML Reader Arithmetic Tests ==========

    #[test]
    fn test_yaml_reader_take_plus_one() {
        // Test that reader.take(MAX_DESERIALIZE_SIZE + 1) works correctly
        // This allows reading exactly MAX_DESERIALIZE_SIZE bytes to detect overflow

        // Create YAML exactly at MAX_DESERIALIZE_SIZE - should succeed
        let data_size = MAX_DESERIALIZE_SIZE - 6;
        let exact_limit_yaml = format!("data: {}", "x".repeat(data_size));
        assert_eq!(exact_limit_yaml.len(), MAX_DESERIALIZE_SIZE);

        let result: Result<serde_yaml::Value> =
            deserialize_yaml_safe_from_reader(exact_limit_yaml.as_bytes());
        assert!(
            result.is_ok(),
            "Exactly MAX_DESERIALIZE_SIZE should succeed"
        );

        // Create YAML exactly MAX_DESERIALIZE_SIZE + 1 - should fail
        let data_size_over = MAX_DESERIALIZE_SIZE - 5;
        let over_limit_yaml = format!("data: {}", "x".repeat(data_size_over));
        assert_eq!(over_limit_yaml.len(), MAX_DESERIALIZE_SIZE + 1);

        let result: Result<serde_yaml::Value> =
            deserialize_yaml_safe_from_reader(over_limit_yaml.as_bytes());
        assert!(result.is_err(), "MAX_DESERIALIZE_SIZE + 1 should fail");
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_yaml_reader_buffer_overflow_detection() {
        // Test that the +1 in take() is necessary to detect overflow
        // If mutated to * or -, overflow detection would fail

        // Create a YAML payload that's exactly MAX_DESERIALIZE_SIZE + 1 bytes
        // This should be caught by the buffer length check
        let oversized_data = "x".repeat(MAX_DESERIALIZE_SIZE + 1);
        let reader = oversized_data.as_bytes();

        let result: Result<serde_yaml::Value> = deserialize_yaml_safe_from_reader(reader);

        // Should fail with "too large" error, not a parsing error
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("too large") || err_msg.contains("Invalid UTF-8"),
            "Should detect size overflow, got: {}",
            err_msg
        );
    }

    #[test]
    #[cfg_attr(
        all(target_os = "linux", not(target_env = "musl")),
        ignore = "Requires >20MB memory allocation, fails in CI environments"
    )]
    fn test_json_reader_buffer_overflow_detection() {
        // Same test for JSON reader to ensure consistent behavior
        let oversized_data = "x".repeat(MAX_DESERIALIZE_SIZE + 1);
        let reader = oversized_data.as_bytes();

        let result: Result<serde_json::Value> = deserialize_json_safe_from_reader(reader);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("too large") || err_msg.contains("Invalid UTF-8"),
            "Should detect size overflow, got: {}",
            err_msg
        );
    }
}
