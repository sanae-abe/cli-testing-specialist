//! Parallel processing strategy selection module
//!
//! This module provides intelligent parallel processing strategy selection
//! based on workload size and characteristics.

use crate::types::TestCategory;

/// Parallel processing strategy
///
/// Determines the level of parallelism for test generation:
/// - Sequential: Single-threaded execution (small workloads)
/// - CategoryLevel: Parallel execution per test category (medium workloads)
/// - TestLevel: Maximum parallelism within categories (large workloads)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelStrategy {
    /// Single-threaded execution
    ///
    /// Best for:
    /// - Small workloads (1-2 categories, <20 total tests)
    /// - Single category with few options
    /// - Avoiding thread overhead
    Sequential,

    /// Parallel execution per test category
    ///
    /// Best for:
    /// - Medium workloads (3-5 categories, 20-100 tests)
    /// - Multiple independent categories
    /// - Balanced CPU usage
    CategoryLevel,

    /// Maximum parallelism (both category and test level)
    ///
    /// Best for:
    /// - Large workloads (6+ categories, 100+ tests)
    /// - Complex CLI tools (kubectl, docker, git)
    /// - High-end systems with many CPU cores
    TestLevel,
}

/// Workload characteristics for strategy selection
#[derive(Debug, Clone)]
pub struct Workload {
    /// Number of test categories to generate
    pub num_categories: usize,

    /// Estimated number of tests per category (average)
    pub estimated_tests_per_category: usize,

    /// Number of available CPU cores
    pub num_cpus: usize,
}

impl Workload {
    /// Create a new workload descriptor
    pub fn new(
        categories: &[TestCategory],
        num_global_options: usize,
        num_subcommands: usize,
    ) -> Self {
        let num_categories = categories.len();

        // Estimate tests per category based on CLI complexity
        // Basic formula: (global_options + subcommands) * category_multiplier
        let estimated_tests_per_category =
            estimate_tests_per_category(num_global_options, num_subcommands, num_categories);

        let num_cpus = num_cpus::get();

        Self {
            num_categories,
            estimated_tests_per_category,
            num_cpus,
        }
    }

    /// Calculate total estimated tests
    pub fn total_estimated_tests(&self) -> usize {
        self.num_categories * self.estimated_tests_per_category
    }
}

/// Estimate tests per category based on CLI complexity
fn estimate_tests_per_category(
    num_global_options: usize,
    num_subcommands: usize,
    num_categories: usize,
) -> usize {
    if num_categories == 0 {
        return 0;
    }

    // Different categories have different test multipliers
    // Basic: ~1-2 tests per option
    // Security: ~2-3 tests per option
    // Path: ~1 test per path option
    // Average: ~2 tests per option/subcommand

    let complexity_score = num_global_options + num_subcommands;
    let avg_tests_per_category = complexity_score.max(1) * 2 / num_categories.max(1);

    // Clamp to reasonable range
    avg_tests_per_category.clamp(5, 50)
}

/// Choose optimal parallel processing strategy
///
/// Decision algorithm:
/// 1. Sequential: total_tests < 20 OR num_categories <= 1
/// 2. CategoryLevel: total_tests < 100 OR num_cpus < 4
/// 3. TestLevel: total_tests >= 100 AND num_cpus >= 4
///
/// # Examples
///
/// ```
/// use cli_testing_specialist::utils::parallel::{choose_strategy, Workload, ParallelStrategy};
/// use cli_testing_specialist::types::TestCategory;
///
/// let categories = vec![TestCategory::Basic, TestCategory::Security];
/// let workload = Workload::new(&categories, 10, 5);
/// let strategy = choose_strategy(&workload);
///
/// // Small workload -> Sequential or CategoryLevel
/// assert!(matches!(strategy, ParallelStrategy::Sequential | ParallelStrategy::CategoryLevel));
/// ```
pub fn choose_strategy(workload: &Workload) -> ParallelStrategy {
    let total_tests = workload.total_estimated_tests();

    // Strategy 1: Sequential (small workloads)
    if total_tests < 20 || workload.num_categories <= 1 {
        log::debug!(
            "Choosing Sequential strategy (total_tests={}, num_categories={})",
            total_tests,
            workload.num_categories
        );
        return ParallelStrategy::Sequential;
    }

    // Strategy 2: CategoryLevel (medium workloads or low CPU)
    if total_tests < 100 || workload.num_cpus < 4 {
        log::debug!(
            "Choosing CategoryLevel strategy (total_tests={}, num_cpus={})",
            total_tests,
            workload.num_cpus
        );
        return ParallelStrategy::CategoryLevel;
    }

    // Strategy 3: TestLevel (large workloads and high CPU)
    log::debug!(
        "Choosing TestLevel strategy (total_tests={}, num_cpus={})",
        total_tests,
        workload.num_cpus
    );
    ParallelStrategy::TestLevel
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workload(
        num_categories: usize,
        num_global_options: usize,
        num_subcommands: usize,
    ) -> Workload {
        let categories: Vec<TestCategory> = (0..num_categories)
            .map(|i| match i % 3 {
                0 => TestCategory::Basic,
                1 => TestCategory::Security,
                _ => TestCategory::Help,
            })
            .collect();

        Workload::new(&categories, num_global_options, num_subcommands)
    }

    #[test]
    fn test_choose_strategy_sequential_small_workload() {
        // 1 category, 5 options -> ~10 tests
        let workload = create_test_workload(1, 5, 0);
        assert_eq!(choose_strategy(&workload), ParallelStrategy::Sequential);
    }

    #[test]
    fn test_choose_strategy_sequential_single_category() {
        // 1 category, even with many options -> Sequential
        let workload = create_test_workload(1, 50, 10);
        assert_eq!(choose_strategy(&workload), ParallelStrategy::Sequential);
    }

    #[test]
    fn test_choose_strategy_category_level_medium_workload() {
        // 3 categories, 10 options -> ~60 tests
        let workload = create_test_workload(3, 10, 5);
        assert_eq!(choose_strategy(&workload), ParallelStrategy::CategoryLevel);
    }

    #[test]
    fn test_choose_strategy_test_level_large_workload() {
        // 6 categories, 30 options, 50 subcommands -> ~160 tests
        let workload = create_test_workload(6, 30, 50);
        let total_tests = workload.total_estimated_tests();

        // Verify workload is large enough
        assert!(
            total_tests >= 100,
            "Expected large workload (>=100 tests), got {}",
            total_tests
        );

        let strategy = choose_strategy(&workload);

        // Result depends on CPU count
        if workload.num_cpus >= 4 {
            assert_eq!(
                strategy,
                ParallelStrategy::TestLevel,
                "Expected TestLevel with {} CPUs and {} tests",
                workload.num_cpus,
                total_tests
            );
        } else {
            assert_eq!(
                strategy,
                ParallelStrategy::CategoryLevel,
                "Expected CategoryLevel with {} CPUs and {} tests",
                workload.num_cpus,
                total_tests
            );
        }
    }

    #[test]
    fn test_estimate_tests_per_category() {
        // 10 options, 5 subcommands, 3 categories
        // complexity_score = 15
        // avg = 15 * 2 / 3 = 10
        let result = estimate_tests_per_category(10, 5, 3);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_estimate_tests_per_category_clamping() {
        // Very small complexity -> clamp to 5
        let result = estimate_tests_per_category(1, 0, 5);
        assert_eq!(result, 5);

        // Very large complexity -> clamp to 50
        let result = estimate_tests_per_category(100, 50, 1);
        assert_eq!(result, 50);
    }

    #[test]
    fn test_workload_total_estimated_tests() {
        let workload = create_test_workload(4, 10, 5);
        let total = workload.total_estimated_tests();

        // 4 categories * estimated_tests_per_category
        assert!(total > 0);
        assert!(total <= 200); // Sanity check
    }
}
