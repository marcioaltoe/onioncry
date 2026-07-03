mod explain;
mod llm;
mod pretty;
mod report;
mod shared;

pub use explain::render_explain_pretty;
pub use llm::render_llm;
pub use pretty::render_pretty;
pub use report::build_report;
