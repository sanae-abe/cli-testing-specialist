// Utilities module - Helper functions and validators

pub mod io_optimized;
pub mod parallel;
pub mod resource_limits;
pub mod safe_deserialize;
pub mod validator;

pub use io_optimized::{
    read_json_optimized, read_json_string_optimized, write_json_compact_optimized,
    write_json_optimized,
};
pub use parallel::{choose_strategy, ParallelStrategy, Workload};
pub use resource_limits::ResourceLimits;
pub use safe_deserialize::{
    deserialize_json_safe, deserialize_json_safe_from_reader, deserialize_yaml_safe,
    deserialize_yaml_safe_from_reader,
};
pub use validator::{execute_with_timeout, execute_with_timeout_and_limits, validate_binary_path};
