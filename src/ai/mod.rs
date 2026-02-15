pub mod provider;
pub mod prompts;

pub use provider::{chat_completion, ChatMessage};
pub use prompts::build_company_context;
