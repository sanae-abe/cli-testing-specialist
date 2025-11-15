use crate::error::{CliTestError, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Validate binary path with comprehensive security checks
///
/// Performs the following validations:
/// 1. File existence check
/// 2. Executable permissions check (Unix)
/// 3. Canonicalization (prevents path traversal)
///
/// # Examples
///
/// ```no_run
/// use cli_testing_specialist::utils::validate_binary_path;
/// use std::path::Path;
///
/// // Validate a system binary
/// let binary = validate_binary_path(Path::new("/usr/bin/ls"))?;
/// assert!(binary.is_absolute());
/// # Ok::<(), cli_testing_specialist::error::CliTestError>(())
/// ```
pub fn validate_binary_path(path: &Path) -> Result<PathBuf> {
    // Check existence
    if !path.exists() {
        return Err(CliTestError::BinaryNotFound(path.to_path_buf()));
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(CliTestError::BinaryNotFound(path.to_path_buf()));
    }

    // Check executable permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = path.metadata()?;
        let permissions = metadata.permissions();

        // Check if any execute bit is set (user, group, or other)
        if permissions.mode() & 0o111 == 0 {
            return Err(CliTestError::BinaryNotExecutable(path.to_path_buf()));
        }
    }

    // Resolve to canonical path (prevents path traversal attacks)
    let canonical = path.canonicalize()?;

    Ok(canonical)
}

/// Execute binary with timeout and default resource limits
///
/// This function provides safe execution with the following guarantees:
/// - Timeout enforcement (prevents infinite loops)
/// - Output capture (stdout and stderr)
/// - Graceful cleanup on timeout
/// - Resource limits applied (Unix only)
///
/// # Examples
///
/// ```no_run
/// use cli_testing_specialist::utils::execute_with_timeout;
/// use std::path::Path;
/// use std::time::Duration;
///
/// // Execute echo with 5 second timeout
/// let output = execute_with_timeout(
///     Path::new("/bin/echo"),
///     &["hello", "world"],
///     Duration::from_secs(5)
/// )?;
/// assert!(output.contains("hello"));
/// # Ok::<(), cli_testing_specialist::error::CliTestError>(())
/// ```
pub fn execute_with_timeout(binary: &Path, args: &[&str], timeout: Duration) -> Result<String> {
    execute_with_timeout_and_limits(
        binary,
        args,
        timeout,
        Some(&crate::utils::ResourceLimits::default()),
    )
}

/// Execute binary with custom resource limits
///
/// This function allows specifying custom resource limits for the child process.
/// If limits are None, no resource limits are applied (unsafe for untrusted binaries).
pub fn execute_with_timeout_and_limits(
    binary: &Path,
    args: &[&str],
    timeout: Duration,
    limits: Option<&crate::utils::ResourceLimits>,
) -> Result<String> {
    use std::io::Read;

    log::debug!(
        "Executing: {} {} (timeout: {:?})",
        binary.display(),
        args.join(" "),
        timeout
    );

    // Build command
    let mut command = Command::new(binary);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Apply resource limits in child process (Unix only)
    #[cfg(unix)]
    if let Some(resource_limits) = limits {
        use std::os::unix::process::CommandExt;

        // Clone limits for use in pre_exec closure
        let max_memory = resource_limits.max_memory_bytes;
        let max_fds = resource_limits.max_file_descriptors;
        let max_procs = resource_limits.max_processes;

        unsafe {
            command.pre_exec(move || {
                use libc::{getrlimit, rlimit, setrlimit, RLIMIT_AS, RLIMIT_NOFILE, RLIMIT_NPROC};

                // Set memory limit (only if lower than current)
                let mut current_limit = rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                };

                // Memory limit
                if getrlimit(RLIMIT_AS, &mut current_limit) == 0 {
                    // Only set if we're lowering the limit (or if unlimited)
                    if current_limit.rlim_max == libc::RLIM_INFINITY
                        || current_limit.rlim_max > max_memory
                    {
                        let mem_limit = rlimit {
                            rlim_cur: max_memory,
                            rlim_max: max_memory,
                        };
                        // Ignore error - some systems may not allow lowering limits
                        let _ = setrlimit(RLIMIT_AS, &mem_limit);
                    }
                }

                // File descriptor limit
                if getrlimit(RLIMIT_NOFILE, &mut current_limit) == 0
                    && (current_limit.rlim_max == libc::RLIM_INFINITY
                        || current_limit.rlim_max > max_fds)
                {
                    let fd_limit = rlimit {
                        rlim_cur: max_fds,
                        rlim_max: max_fds,
                    };
                    let _ = setrlimit(RLIMIT_NOFILE, &fd_limit);
                }

                // Process limit
                if getrlimit(RLIMIT_NPROC, &mut current_limit) == 0
                    && (current_limit.rlim_max == libc::RLIM_INFINITY
                        || current_limit.rlim_max > max_procs)
                {
                    let proc_limit = rlimit {
                        rlim_cur: max_procs,
                        rlim_max: max_procs,
                    };
                    let _ = setrlimit(RLIMIT_NPROC, &proc_limit);
                }

                Ok(())
            });
        }
    }

    // Spawn child process
    let mut child = command.spawn()?;

    // Apply resource limits on Windows (must be done after spawn)
    #[cfg(windows)]
    if let Some(resource_limits) = limits {
        apply_windows_job_limits(&child, resource_limits)?;
    }

    // Wait with timeout
    let start = std::time::Instant::now();

    loop {
        // Check if process has finished
        match child.try_wait()? {
            Some(_status) => {
                // Process finished - collect output
                let mut stdout = String::new();
                if let Some(mut pipe) = child.stdout.take() {
                    pipe.read_to_string(&mut stdout)?;
                }

                // Also capture stderr (some CLIs output help to stderr)
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    pipe.read_to_string(&mut stderr)?;
                }

                // Prefer stdout, fallback to stderr
                let output = if !stdout.is_empty() { stdout } else { stderr };

                log::debug!("Execution completed in {:?}", start.elapsed());
                return Ok(output);
            }
            None => {
                // Process still running - check timeout
                if start.elapsed() >= timeout {
                    // Timeout exceeded - kill process
                    log::warn!("Execution timeout exceeded, killing process");
                    child.kill()?;
                    child.wait()?;

                    return Err(CliTestError::ExecutionFailed(format!(
                        "Timeout after {:?}",
                        timeout
                    )));
                }

                // Sleep briefly before checking again
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// Apply resource limits to a Windows child process using Job Objects
#[cfg(windows)]
fn apply_windows_job_limits(
    child: &std::process::Child,
    limits: &crate::utils::ResourceLimits,
) -> Result<()> {
    use std::os::windows::process::CommandExt;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_BASIC_LIMIT_INFORMATION,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };

    unsafe {
        // Create a job object
        let job = CreateJobObjectW(None, None).map_err(|e| {
            CliTestError::ExecutionFailed(format!("Failed to create job object: {}", e))
        })?;

        // Set job limits
        let mut job_limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                LimitFlags: JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                    | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                    | JOB_OBJECT_LIMIT_JOB_MEMORY,
                ActiveProcessLimit: limits.max_processes as u32,
                ..Default::default()
            },
            ProcessMemoryLimit: limits.max_memory_bytes as usize,
            JobMemoryLimit: limits.max_memory_bytes as usize,
            ..Default::default()
        };

        // Apply limits to job object
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &mut job_limits as *mut _ as *mut _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|e| {
            CloseHandle(job);
            CliTestError::ExecutionFailed(format!("Failed to set job limits: {}", e))
        })?;

        // Get child process handle and assign to job
        let child_handle = HANDLE(child.id() as isize);
        AssignProcessToJobObject(job, child_handle).map_err(|e| {
            CloseHandle(job);
            CliTestError::ExecutionFailed(format!("Failed to assign process to job: {}", e))
        })?;

        // Note: We intentionally don't close the job handle here
        // The job will terminate when the child process exits
        log::debug!("Resource limits applied to child process via Job Object");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_validate_nonexistent_binary() {
        let path = Path::new("/nonexistent/binary");
        let result = validate_binary_path(path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CliTestError::BinaryNotFound(_)
        ));
    }

    #[test]
    fn test_validate_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_binary_path(temp_dir.path());

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CliTestError::BinaryNotFound(_)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_non_executable_file() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("non_executable");

        // Create file without execute permissions
        File::create(&file_path).unwrap();
        let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644); // rw-r--r--
        std::fs::set_permissions(&file_path, perms).unwrap();

        let result = validate_binary_path(&file_path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CliTestError::BinaryNotExecutable(_)
        ));
    }

    #[test]
    fn test_execute_with_timeout_echo() {
        // Test with echo command (should be available on all Unix systems)
        #[cfg(unix)]
        {
            let echo_path = Path::new("/bin/echo");
            if echo_path.exists() {
                let result =
                    execute_with_timeout(echo_path, &["hello", "world"], Duration::from_secs(5));

                assert!(result.is_ok());
                let output = result.unwrap();
                assert!(output.contains("hello"));
            }
        }
    }

    #[test]
    fn test_execute_with_timeout_sleep() {
        // Test timeout enforcement
        #[cfg(unix)]
        {
            let sleep_path = Path::new("/bin/sleep");
            if sleep_path.exists() {
                let result = execute_with_timeout(
                    sleep_path,
                    &["10"],                    // Sleep for 10 seconds
                    Duration::from_millis(500), // But timeout after 500ms
                );

                assert!(result.is_err());
                if let Err(CliTestError::ExecutionFailed(msg)) = result {
                    assert!(msg.contains("Timeout"));
                }
            }
        }
    }

    #[test]
    fn test_canonicalization() {
        // Test that canonicalization works with valid binary
        #[cfg(unix)]
        {
            let ls_path = Path::new("/bin/ls");
            if ls_path.exists() {
                let result = validate_binary_path(ls_path);
                assert!(result.is_ok());

                let canonical = result.unwrap();
                assert!(canonical.is_absolute());
            }
        }
    }
}
