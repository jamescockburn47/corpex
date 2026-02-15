//! Document cache — persists extracted filing text to disk so it survives
//! restarts and doesn't need re-downloading.
//!
//! Layout: `~/.corpex/cache/docs/{company_number}/{filing_id}.txt`

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Return the cache root directory, creating it if needed.
fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    let dir = home.join(".corpex").join("cache").join("docs");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Save extracted text for a filing to disk.
pub fn save_text(company_number: &str, filing_id: &str, text: &str) -> Result<()> {
    let dir = cache_dir()?.join(company_number);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.txt", filing_id));
    std::fs::write(&path, text)?;
    tracing::debug!("Cached filing text: {}", path.display());
    Ok(())
}

/// Load a single cached filing text, if it exists.
pub fn load_text(company_number: &str, filing_id: &str) -> Option<String> {
    let dir = cache_dir().ok()?.join(company_number);
    let path = dir.join(format!("{}.txt", filing_id));
    std::fs::read_to_string(path).ok()
}

/// Check if a filing is already cached.
pub fn is_cached(company_number: &str, filing_id: &str) -> bool {
    cache_dir()
        .ok()
        .map(|d| d.join(company_number).join(format!("{}.txt", filing_id)).exists())
        .unwrap_or(false)
}

/// Load all cached texts for a specific company.
pub fn load_company_texts(company_number: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(dir) = cache_dir() {
        let company_dir = dir.join(company_number);
        if company_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&company_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".txt") {
                            let filing_id = name.trim_end_matches(".txt").to_string();
                            if let Ok(text) = std::fs::read_to_string(entry.path()) {
                                map.insert(filing_id, text);
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

/// Load ALL cached texts across all companies.
pub fn load_all_texts() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(dir) = cache_dir() {
        if let Ok(companies) = std::fs::read_dir(&dir) {
            for company_entry in companies.flatten() {
                if company_entry.file_type().map_or(false, |t| t.is_dir()) {
                    if let Ok(filings) = std::fs::read_dir(company_entry.path()) {
                        for filing_entry in filings.flatten() {
                            if let Some(name) = filing_entry.file_name().to_str() {
                                if name.ends_with(".txt") {
                                    let filing_id = name.trim_end_matches(".txt").to_string();
                                    if let Ok(text) = std::fs::read_to_string(filing_entry.path()) {
                                        map.insert(filing_id, text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}
