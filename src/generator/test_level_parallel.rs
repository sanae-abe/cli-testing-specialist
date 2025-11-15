//! Test-level parallelism for fine-grained parallel test generation
//!
//! This module provides helper functions to parallelize test case generation
//! within each category, enabling better CPU utilization for large test suites.

#![allow(dead_code)] // Helper functions reserved for future use

use crate::error::Result;
use crate::types::TestCase;
use rayon::prelude::*;

/// Generate tests in parallel from a collection of test builders
///
/// This function takes a collection of closures that each produce a `Result<TestCase>`,
/// executes them in parallel, and collects the results.
///
/// # Examples
///
/// ```ignore
/// let test_builders = vec![
///     || generate_test_1(),
///     || generate_test_2(),
///     || generate_test_3(),
/// ];
///
/// let tests = parallel_generate(test_builders)?;
/// ```
///
/// # Performance
///
/// - Small workloads (<10 tests): Sequential execution (avoid overhead)
/// - Medium workloads (10-50 tests): Parallel execution (optimal)
/// - Large workloads (50+ tests): Parallel execution with chunking
pub fn parallel_generate<F>(test_builders: Vec<F>) -> Result<Vec<TestCase>>
where
    F: Fn() -> Result<TestCase> + Send + Sync,
{
    let test_count = test_builders.len();

    // Strategy: Use sequential for small workloads to avoid thread overhead
    if test_count < 10 {
        test_builders.into_iter().map(|f| f()).collect()
    } else {
        // Parallel execution for medium/large workloads
        test_builders.par_iter().map(|f| f()).collect()
    }
}

/// Generate optional tests in parallel, filtering out None values
///
/// This is useful when some tests are conditionally generated based on
/// CLI analysis (e.g., version flag only if version detected).
///
/// # Examples
///
/// ```ignore
/// let test_builders = vec![
///     || Some(generate_required_test()?),
///     || if condition { Some(generate_optional_test()?) } else { None },
/// ];
///
/// let tests = parallel_generate_optional(test_builders)?;
/// ```
pub fn parallel_generate_optional<F>(test_builders: Vec<F>) -> Result<Vec<TestCase>>
where
    F: Fn() -> Result<Option<TestCase>> + Send + Sync,
{
    let test_count = test_builders.len();

    if test_count < 10 {
        // Sequential execution
        Ok(test_builders
            .into_iter()
            .map(|f| f())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    } else {
        // Parallel execution
        Ok(test_builders
            .par_iter()
            .map(|f| f())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

/// Generate tests in parallel with explicit chunk size
///
/// This function allows manual control over the chunk size for workload balancing.
/// Useful when test generation times vary significantly.
///
/// # Arguments
///
/// * `test_builders` - Collection of test generation functions
/// * `chunk_size` - Number of tests per parallel chunk (recommended: 5-10)
///
/// # Examples
///
/// ```ignore
/// let tests = parallel_generate_chunked(test_builders, 5)?;
/// ```
pub fn parallel_generate_chunked<F>(
    test_builders: Vec<F>,
    chunk_size: usize,
) -> Result<Vec<TestCase>>
where
    F: Fn() -> Result<TestCase> + Send + Sync,
{
    test_builders
        .par_chunks(chunk_size)
        .flat_map(|chunk| chunk.iter().map(|f| f()).collect::<Vec<_>>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TestCategory;

    fn create_test_builder(id: usize) -> impl Fn() -> Result<TestCase> {
        move || {
            Ok(TestCase::new(
                format!("test-{}", id),
                format!("Test {}", id),
                TestCategory::Basic,
                "echo test".to_string(),
            ))
        }
    }

    #[test]
    fn test_parallel_generate_small_workload() {
        // Small workload (5 tests) - should use sequential
        let builders: Vec<_> = (0..5).map(create_test_builder).collect();
        let tests = parallel_generate(builders).unwrap();
        assert_eq!(tests.len(), 5);
    }

    #[test]
    fn test_parallel_generate_medium_workload() {
        // Medium workload (20 tests) - should use parallel
        let builders: Vec<_> = (0..20).map(create_test_builder).collect();
        let tests = parallel_generate(builders).unwrap();
        assert_eq!(tests.len(), 20);
    }

    #[test]
    fn test_parallel_generate_large_workload() {
        // Large workload (100 tests) - should use parallel
        let builders: Vec<_> = (0..100).map(create_test_builder).collect();
        let tests = parallel_generate(builders).unwrap();
        assert_eq!(tests.len(), 100);
    }

    #[test]
    fn test_parallel_generate_optional() {
        let builders = vec![
            || Ok(Some(create_test_builder(1)()?)),
            || Ok(None), // Filtered out
            || Ok(Some(create_test_builder(3)()?)),
        ];

        let tests = parallel_generate_optional(builders).unwrap();
        assert_eq!(tests.len(), 2);
    }

    #[test]
    fn test_parallel_generate_chunked() {
        let builders: Vec<_> = (0..15).map(create_test_builder).collect();
        let tests = parallel_generate_chunked(builders, 5).unwrap();
        assert_eq!(tests.len(), 15);
    }
}
