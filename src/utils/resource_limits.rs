use crate::error::{CliTestError, Result};
use std::time::Duration;

/// Resource limits for DOS attack prevention
///
/// Enforces strict limits on resources that can be consumed during analysis:
/// - Memory usage (prevents memory exhaustion)
/// - File descriptors (prevents FD exhaustion)
/// - Process count (prevents fork bombs)
/// - Execution timeout (prevents infinite loops)
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes (default: 500MB)
    pub max_memory_bytes: u64,

    /// Maximum number of file descriptors (default: 1024)
    pub max_file_descriptors: u64,

    /// Maximum number of processes (default: 100)
    pub max_processes: u64,

    /// Maximum execution time (default: 300s)
    pub execution_timeout: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 500 * 1024 * 1024, // 500MB
            max_file_descriptors: 1024,
            max_processes: 100,
            execution_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits with custom values
    pub fn new(
        max_memory_bytes: u64,
        max_file_descriptors: u64,
        max_processes: u64,
        execution_timeout: Duration,
    ) -> Self {
        Self {
            max_memory_bytes,
            max_file_descriptors,
            max_processes,
            execution_timeout,
        }
    }

    /// Apply resource limits to the current process (Unix only)
    ///
    /// This method uses `setrlimit` to enforce hard limits on resources.
    /// On non-Unix platforms, this is a no-op.
    #[cfg(unix)]
    pub fn apply(&self) -> Result<()> {
        use libc::{rlimit, setrlimit, RLIMIT_AS, RLIMIT_NOFILE, RLIMIT_NPROC};

        // Set memory limit (address space)
        let mem_limit = rlimit {
            rlim_cur: self.max_memory_bytes,
            rlim_max: self.max_memory_bytes,
        };

        unsafe {
            if setrlimit(RLIMIT_AS, &mem_limit) != 0 {
                return Err(CliTestError::ExecutionFailed(
                    "Failed to set memory limit".to_string(),
                ));
            }
        }

        // Set file descriptor limit
        let fd_limit = rlimit {
            rlim_cur: self.max_file_descriptors,
            rlim_max: self.max_file_descriptors,
        };

        unsafe {
            if setrlimit(RLIMIT_NOFILE, &fd_limit) != 0 {
                return Err(CliTestError::ExecutionFailed(
                    "Failed to set file descriptor limit".to_string(),
                ));
            }
        }

        // Set process limit
        let proc_limit = rlimit {
            rlim_cur: self.max_processes,
            rlim_max: self.max_processes,
        };

        unsafe {
            if setrlimit(RLIMIT_NPROC, &proc_limit) != 0 {
                return Err(CliTestError::ExecutionFailed(
                    "Failed to set process limit".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Apply resource limits (Windows stub)
    ///
    /// Windows does not support setrlimit. Job Objects could be used
    /// but are significantly more complex. This is marked as future work.
    #[cfg(not(unix))]
    pub fn apply(&self) -> Result<()> {
        log::warn!("Resource limits not supported on this platform");
        Ok(())
    }

    /// Get timeout duration
    pub fn timeout(&self) -> Duration {
        self.execution_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.max_memory_bytes, 500 * 1024 * 1024);
        assert_eq!(limits.max_file_descriptors, 1024);
        assert_eq!(limits.max_processes, 100);
        assert_eq!(limits.execution_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_custom_limits() {
        let limits = ResourceLimits::new(100 * 1024 * 1024, 512, 50, Duration::from_secs(60));

        assert_eq!(limits.max_memory_bytes, 100 * 1024 * 1024);
        assert_eq!(limits.max_file_descriptors, 512);
        assert_eq!(limits.max_processes, 50);
        assert_eq!(limits.execution_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_timeout_accessor() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.timeout(), Duration::from_secs(300));
    }

    #[cfg(unix)]
    #[test]
    fn test_apply_limits_unix() {
        // Note: This test may fail if the process doesn't have permission
        // to set resource limits. It's primarily for compilation checking.
        let limits = ResourceLimits::new(100 * 1024 * 1024, 512, 50, Duration::from_secs(60));

        // This may fail in restricted environments, so we don't assert success
        let _ = limits.apply();
    }
}
