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

/// Execute binary with timeout and resource limits
///
/// This function provides safe execution with the following guarantees:
/// - Timeout enforcement (prevents infinite loops)
/// - Output capture (stdout and stderr)
/// - Graceful cleanup on timeout
pub fn execute_with_timeout(binary: &Path, args: &[&str], timeout: Duration) -> Result<String> {
    use std::io::Read;

    log::debug!(
        "Executing: {} {} (timeout: {:?})",
        binary.display(),
        args.join(" "),
        timeout
    );

    // Spawn child process
    let mut child = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

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
