// Reporter module - Test report generation

pub mod html;
pub mod json;
pub mod junit;
pub mod markdown;

// Re-export reporters
pub use html::HtmlReporter;
pub use json::JsonReporter;
pub use junit::JunitReporter;
pub use markdown::MarkdownReporter;
