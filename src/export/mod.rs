//! Export module — saves investigation data to disk.
//!
//! Layout: `~/Corpex Investigations/{project_name}/{company_number} - {name}/`
//!   - analysis.md      — AI analysis text
//!   - chat_log.md      — full AI chat history
//!   - profile.json     — raw company profile
//!   - filings/         — extracted text files
//!
//! On WASM, export is not available (no filesystem access).

// ═══════════════════════════════════════════════════════════════════════
// Native implementation
// ═══════════════════════════════════════════════════════════════════════
#[cfg(not(target_arch = "wasm32"))]
mod native_export {
    use anyhow::{Context, Result};
    use std::path::PathBuf;

    /// Return the default export root directory.
    pub fn export_root() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Corpex Investigations")
    }

    /// List existing project names (subdirectories of the export root).
    pub fn list_projects() -> Vec<String> {
        let root = export_root();
        if !root.exists() {
            return Vec::new();
        }
        let mut projects = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                if entry.file_type().map_or(false, |t| t.is_dir()) {
                    if let Some(name) = entry.file_name().to_str() {
                        projects.push(name.to_string());
                    }
                }
            }
        }
        projects.sort();
        projects
    }

    /// Export all available data for a single company into a project folder.
    ///
    /// Returns the path to the company folder.
    pub fn export_company(
        project_name: &str,
        company_number: &str,
        company_name: &str,
        profile_json: Option<&str>,
        ai_analysis: Option<&str>,
        chat_history: Option<&[(String, String)]>, // (role, content)
        extracted_texts: &std::collections::HashMap<String, String>,
        filing_descriptions: &std::collections::HashMap<String, String>, // filing_id -> description
    ) -> Result<PathBuf> {
        // Build path: ~/Corpex Investigations/{project}/{number} - {name}/
        let safe_name = sanitise_filename(company_name);
        let folder_name = format!("{} - {}", company_number, safe_name);
        let company_dir = export_root().join(project_name).join(&folder_name);
        std::fs::create_dir_all(&company_dir).context("Failed to create export directory")?;

        // Profile JSON
        if let Some(json) = profile_json {
            std::fs::write(company_dir.join("profile.txt"), json)?;
        }

        // AI Analysis
        if let Some(analysis) = ai_analysis {
            let content = format!(
                "# AI Analysis: {} ({})\n\n{}\n",
                company_name, company_number, analysis
            );
            std::fs::write(company_dir.join("analysis.md"), content)?;
        }

        // Chat log
        if let Some(messages) = chat_history {
            if !messages.is_empty() {
                let mut content =
                    format!("# AI Chat Log: {} ({})\n\n", company_name, company_number);
                for (role, text) in messages {
                    let label = if role == "user" { "USER" } else { "AI" };
                    content.push_str(&format!("## {}\n{}\n\n", label, text));
                }
                std::fs::write(company_dir.join("chat_log.md"), content)?;
            }
        }

        // Extracted filing texts
        if !extracted_texts.is_empty() {
            let filings_dir = company_dir.join("filings");
            std::fs::create_dir_all(&filings_dir)?;
            for (filing_id, text) in extracted_texts {
                let desc = filing_descriptions
                    .get(filing_id)
                    .map(|d| sanitise_filename(d))
                    .unwrap_or_default();
                let filename = if desc.is_empty() {
                    format!("{}.txt", filing_id)
                } else {
                    format!("{} - {}.txt", filing_id, desc)
                };
                std::fs::write(filings_dir.join(filename), text)?;
            }
        }

        Ok(company_dir)
    }

    /// Make a string safe for use as a file/folder name.
    fn sanitise_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native_export::*;

// ═══════════════════════════════════════════════════════════════════════
// WASM stubs — no filesystem access
// ═══════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "wasm32")]
pub fn export_root() -> std::path::PathBuf {
    std::path::PathBuf::from("/unavailable")
}

#[cfg(target_arch = "wasm32")]
pub fn list_projects() -> Vec<String> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
pub fn export_company(
    _project_name: &str,
    _company_number: &str,
    _company_name: &str,
    _profile_json: Option<&str>,
    _ai_analysis: Option<&str>,
    _chat_history: Option<&[(String, String)]>,
    _extracted_texts: &std::collections::HashMap<String, String>,
    _filing_descriptions: &std::collections::HashMap<String, String>,
) -> anyhow::Result<std::path::PathBuf> {
    anyhow::bail!("Export is not available in the web version")
}
