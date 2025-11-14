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

    /// Apply resource limits using Windows Job Objects
    ///
    /// Windows uses Job Objects to enforce resource limits, which is more complex
    /// than Unix setrlimit but provides similar functionality.
    #[cfg(windows)]
    pub fn apply(&self) -> Result<()> {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_BASIC_LIMIT_INFORMATION,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
            JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
        };
        use windows::Win32::System::Threading::GetCurrentProcess;

        unsafe {
            // Create a job object
            let job = CreateJobObjectW(None, None).map_err(|e| {
                CliTestError::ExecutionFailed(format!("Failed to create job object: {}", e))
            })?;

            // Set job limits
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
                BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                    LimitFlags: JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                        | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                        | JOB_OBJECT_LIMIT_JOB_MEMORY,
                    ActiveProcessLimit: self.max_processes as u32,
                    ..Default::default()
                },
                ProcessMemoryLimit: self.max_memory_bytes as usize,
                JobMemoryLimit: self.max_memory_bytes as usize,
                ..Default::default()
            };

            // Apply limits to job object
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &mut limits as *mut _ as *mut _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|e| {
                CloseHandle(job);
                CliTestError::ExecutionFailed(format!("Failed to set job limits: {}", e))
            })?;

            // Assign current process to job
            let current_process = GetCurrentProcess();
            AssignProcessToJobObject(job, current_process).map_err(|e| {
                CloseHandle(job);
                CliTestError::ExecutionFailed(format!("Failed to assign process to job: {}", e))
            })?;

            // Note: We intentionally don't close the job handle here
            // because it needs to remain valid for the lifetime of the process
            // The OS will clean it up when the process exits
            log::debug!("Resource limits applied via Job Object");
        }

        Ok(())
    }

    /// Apply resource limits (non-Unix, non-Windows platforms)
    #[cfg(not(any(unix, windows)))]
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
