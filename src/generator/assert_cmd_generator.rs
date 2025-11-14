use crate::error::Result;
use crate::generator::test_generator_trait::TestGenerator as TestGeneratorTrait;
use crate::types::analysis::CliAnalysis;
use crate::types::test_case::TestCategory;
use handlebars::Handlebars;
use serde_json::json;

/// Generator for assert_cmd-based Rust tests
///
/// Generates idiomatic Rust test code using the assert_cmd and predicates crates.
/// Tests are self-contained and can be run with `cargo test`.
///
/// # Example Output
///
/// ```rust,ignore
/// use assert_cmd::Command;
/// use predicates::prelude::*;
///
/// #[test]
/// fn test_help() {
///     let mut cmd = Command::cargo_bin("my-cli").unwrap();
///     cmd.arg("--help")
///         .assert()
///         .success()
///         .stdout(predicate::str::contains("Usage:"));
/// }
/// ```
pub struct AssertCmdGenerator {
    handlebars: Handlebars<'static>,
    cli_name: String,
}

impl AssertCmdGenerator {
    /// Create a new AssertCmdGenerator
    ///
    /// # Arguments
    ///
    /// * `analysis` - CLI analysis results
    ///
    /// # Returns
    ///
    /// New AssertCmdGenerator instance
    pub fn new(analysis: &CliAnalysis) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register templates
        Self::register_templates(&mut handlebars)?;

        // Configure Handlebars
        handlebars.set_strict_mode(true);

        Ok(Self {
            handlebars,
            cli_name: analysis.binary_name.clone(),
        })
    }

    /// Register all test templates
    fn register_templates(handlebars: &mut Handlebars) -> Result<()> {
        // Basic tests template
        handlebars
            .register_template_string("basic", include_str!("../templates/assert_cmd/basic.hbs"))?;

        // Security tests template
        handlebars.register_template_string(
            "security",
            include_str!("../templates/assert_cmd/security.hbs"),
        )?;

        // Help tests template
        handlebars
            .register_template_string("help", include_str!("../templates/assert_cmd/help.hbs"))?;

        // Path tests template
        handlebars
            .register_template_string("path", include_str!("../templates/assert_cmd/path.hbs"))?;

        // InputValidation tests template
        handlebars.register_template_string(
            "input_validation",
            include_str!("../templates/assert_cmd/input_validation.hbs"),
        )?;

        // DestructiveOps tests template
        handlebars.register_template_string(
            "destructive_ops",
            include_str!("../templates/assert_cmd/destructive_ops.hbs"),
        )?;

        // Performance tests template
        handlebars.register_template_string(
            "performance",
            include_str!("../templates/assert_cmd/performance.hbs"),
        )?;

        // MultiShell tests template
        handlebars.register_template_string(
            "multi_shell",
            include_str!("../templates/assert_cmd/multi_shell.hbs"),
        )?;

        Ok(())
    }

    /// Sanitize string for Rust code generation
    ///
    /// Escapes special characters to prevent code injection and ensure valid Rust syntax.
    ///
    /// # Security
    ///
    /// This function implements the security recommendation from the v1.1.0 design review:
    /// - Explicitly escape backslashes, quotes, and newlines
    /// - Does NOT rely on Handlebars default escaping
    ///
    /// # Arguments
    ///
    /// * `input` - Raw string to sanitize
    ///
    /// # Returns
    ///
    /// Sanitized string safe for Rust string literals
    pub fn sanitize_for_rust_string(input: &str) -> String {
        input
            .replace('\\', "\\\\") // Backslash must be first
            .replace('"', "\\\"") // Double quote
            .replace('\n', "\\n") // Newline
            .replace('\r', "\\r") // Carriage return
            .replace('\t', "\\t") // Tab
    }
}

impl TestGeneratorTrait for AssertCmdGenerator {
    fn generate(&self, analysis: &CliAnalysis, category: TestCategory) -> Result<String> {
        let template_name = match category {
            TestCategory::Basic => "basic",
            TestCategory::Security => "security",
            TestCategory::Help => "help",
            TestCategory::Path => "path",
            TestCategory::InputValidation => "input_validation",
            TestCategory::DestructiveOps => "destructive_ops",
            TestCategory::DirectoryTraversal => "security", // Reuse security template
            TestCategory::Performance => "performance",
            TestCategory::MultiShell => "multi_shell",
        };

        // Prepare template data
        let data = json!({
            "cli_name": Self::sanitize_for_rust_string(&self.cli_name),
            "version": analysis.version.as_ref().map(|v| Self::sanitize_for_rust_string(v)),
            "subcommands": analysis.subcommands.iter().map(|sc| {
                json!({
                    "name": Self::sanitize_for_rust_string(&sc.name),
                    "description": sc.description.as_ref().map(|d| Self::sanitize_for_rust_string(d)),
                })
            }).collect::<Vec<_>>(),
        });

        // Render template
        let test_code = self.handlebars.render(template_name, &data)?;

        Ok(test_code)
    }

    fn file_extension(&self) -> &str {
        "rs"
    }

    fn name(&self) -> &str {
        "assert_cmd"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_for_rust_string() {
        assert_eq!(
            AssertCmdGenerator::sanitize_for_rust_string("hello"),
            "hello"
        );
        assert_eq!(
            AssertCmdGenerator::sanitize_for_rust_string("hello\\world"),
            "hello\\\\world"
        );
        assert_eq!(
            AssertCmdGenerator::sanitize_for_rust_string("hello\"world"),
            "hello\\\"world"
        );
        assert_eq!(
            AssertCmdGenerator::sanitize_for_rust_string("hello\nworld"),
            "hello\\nworld"
        );
        assert_eq!(
            AssertCmdGenerator::sanitize_for_rust_string("test; rm -rf /"),
            "test; rm -rf /"
        );
    }

    #[test]
    fn test_sanitize_complex_string() {
        let input = "test\\path\"with\nnewline\tand\rtab";
        let expected = "test\\\\path\\\"with\\nnewline\\tand\\rtab";
        assert_eq!(
            AssertCmdGenerator::sanitize_for_rust_string(input),
            expected
        );
    }
}
