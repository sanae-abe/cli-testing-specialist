use crate::error::{Error, Result};
use std::collections::HashMap;

/// Embedded templates using include_str! macro
const BATS_TEST_TEMPLATE: &str = include_str!("../../templates/bats-test.template");
const CONCURRENCY_TEST: &str = include_str!("../../templates/concurrency-test.fragment");
const DESTRUCTIVE_OPS: &str = include_str!("../../templates/destructive-ops.fragment");
const DIRECTORY_TRAVERSAL_LIMITS: &str =
    include_str!("../../templates/directory-traversal-limits.fragment");
const INPUT_VALIDATION: &str = include_str!("../../templates/input-validation.fragment");
const PERFORMANCE_TEST: &str = include_str!("../../templates/performance-test.fragment");
const SUBCOMMAND_HELP: &str = include_str!("../../templates/subcommand-help.fragment");

/// Template engine for loading and processing BATS test templates
pub struct TemplateEngine {
    /// Cached templates (template_name -> template_content)
    templates: HashMap<String, String>,
}

impl TemplateEngine {
    /// Create a new template engine with embedded templates
    pub fn new() -> Result<Self> {
        Ok(Self {
            templates: HashMap::new(),
        })
    }

    /// Load all embedded templates
    pub fn load_templates(&mut self) -> Result<()> {
        log::info!("Loading embedded templates");

        // Load all embedded templates
        self.templates
            .insert("bats-test".to_string(), BATS_TEST_TEMPLATE.to_string());
        self.templates
            .insert("concurrency-test".to_string(), CONCURRENCY_TEST.to_string());
        self.templates
            .insert("destructive-ops".to_string(), DESTRUCTIVE_OPS.to_string());
        self.templates.insert(
            "directory-traversal-limits".to_string(),
            DIRECTORY_TRAVERSAL_LIMITS.to_string(),
        );
        self.templates
            .insert("input-validation".to_string(), INPUT_VALIDATION.to_string());
        self.templates
            .insert("performance-test".to_string(), PERFORMANCE_TEST.to_string());
        self.templates
            .insert("subcommand-help".to_string(), SUBCOMMAND_HELP.to_string());

        log::info!("Loaded {} templates", self.templates.len());

        Ok(())
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Result<&str> {
        self.templates
            .get(name)
            .map(|s| s.as_str())
            .ok_or_else(|| Error::Config(format!("Template not found: {}", name)))
    }

    /// Substitute variables in a template
    /// Variables are in the format ${VARIABLE_NAME}
    pub fn substitute(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Get template by name and substitute variables
    pub fn render(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let template = self.get_template(template_name)?;
        Ok(self.substitute(template, variables))
    }

    /// List all available template names
    pub fn available_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Validate that required variables are present in the template
    pub fn validate_template(&self, template_name: &str, required_vars: &[&str]) -> Result<()> {
        let template = self.get_template(template_name)?;

        for var in required_vars {
            let placeholder = format!("${{{}}}", var);
            if !template.contains(&placeholder) {
                log::warn!(
                    "Template '{}' does not contain expected variable: {}",
                    template_name,
                    var
                );
            }
        }

        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateEngine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_load_templates() {
        let mut engine = TemplateEngine::new().unwrap();
        let result = engine.load_templates();

        assert!(result.is_ok());
        assert_eq!(engine.templates.len(), 7); // 7 embedded templates
        assert!(engine.templates.contains_key("bats-test"));
        assert!(engine.templates.contains_key("concurrency-test"));
    }

    #[test]
    fn test_get_template() {
        let mut engine = TemplateEngine::new().unwrap();
        engine.load_templates().unwrap();

        let template = engine.get_template("bats-test");
        assert!(template.is_ok());
        assert!(!template.unwrap().is_empty());
    }

    #[test]
    fn test_get_nonexistent_template() {
        let mut engine = TemplateEngine::new().unwrap();
        engine.load_templates().unwrap();

        let result = engine.get_template("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_substitute() {
        let engine = TemplateEngine::new().unwrap();

        let template = "Hello ${NAME}, you are ${AGE} years old.";
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "Alice".to_string());
        vars.insert("AGE".to_string(), "30".to_string());

        let result = engine.substitute(template, &vars);
        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }

    #[test]
    fn test_substitute_missing_variable() {
        let engine = TemplateEngine::new().unwrap();

        let template = "Hello ${NAME}, you are ${AGE} years old.";
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "Alice".to_string());

        let result = engine.substitute(template, &vars);
        // Missing variables are left as-is
        assert_eq!(result, "Hello Alice, you are ${AGE} years old.");
    }

    #[test]
    fn test_available_templates() {
        let mut engine = TemplateEngine::new().unwrap();
        engine.load_templates().unwrap();

        let templates = engine.available_templates();
        assert_eq!(templates.len(), 7);
        assert!(templates.contains(&"bats-test".to_string()));
        assert!(templates.contains(&"performance-test".to_string()));
    }
}
