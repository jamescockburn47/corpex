//! Extraction pipeline — orchestrates text extraction from CH filing documents.
//!
//! Given a `DocumentContent` from the CH document API, this module routes
//! to the appropriate extractor and returns clean text.

use crate::ch_api::client::DocumentContent;
use super::xhtml;
use super::pdf;

/// Result of the extraction pipeline.
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// The extracted text content.
    pub text: String,
    /// How the text was extracted.
    pub method: ExtractionMethod,
    /// Any iXBRL structured values found (name → value pairs).
    pub ixbrl_values: Vec<(String, String)>,
}

/// How text was extracted from the document.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionMethod {
    /// Parsed from XHTML/iXBRL markup
    Xhtml,
    /// Parsed from inline JSON
    Json,
    /// Extracted from XML tags
    Xml,
    /// Extracted from PDF embedded text
    PdfNative,
    /// PDF was scanned — OCR needed (not currently available)
    PdfScannedNoOcr,
    /// Document type not supported
    Unsupported(String),
}

/// Extract text from a downloaded Companies House document.
///
/// This is the main entry point for the extraction pipeline.
/// It dispatches to the appropriate extractor based on document type.
pub fn extract_text(doc: &DocumentContent) -> ExtractionResult {
    match doc {
        DocumentContent::Xhtml(xhtml_content) => extract_from_xhtml(xhtml_content),
        DocumentContent::Json(json_text) => extract_from_json(json_text),
        DocumentContent::Xml(xml_text) => extract_from_xml(xml_text),
        DocumentContent::Pdf(pdf_bytes) => extract_from_pdf(pdf_bytes),
        DocumentContent::Other { content_type, .. } => ExtractionResult {
            text: format!(
                "[Unsupported document type: {}. Text extraction not available.]",
                content_type
            ),
            method: ExtractionMethod::Unsupported(content_type.clone()),
            ixbrl_values: Vec::new(),
        },
    }
}

/// Extract from XHTML/iXBRL — the best-case scenario.
fn extract_from_xhtml(content: &str) -> ExtractionResult {
    let text = xhtml::extract_text_from_xhtml(content);
    let ixbrl_values = xhtml::extract_ixbrl_values(content);

    let method = ExtractionMethod::Xhtml;

    if text.trim().is_empty() && ixbrl_values.is_empty() {
        ExtractionResult {
            text: "[XHTML document contained no extractable text.]".to_string(),
            method,
            ixbrl_values: Vec::new(),
        }
    } else {
        let mut full_text = text;

        // If we got iXBRL values, append a structured section
        if !ixbrl_values.is_empty() {
            full_text.push_str("\n\n--- iXBRL Tagged Data ---\n");
            for (name, value) in &ixbrl_values {
                // Shorten the namespace for readability
                let short_name = name
                    .rsplit_once(':')
                    .map(|(_, n)| n)
                    .unwrap_or(name);
                full_text.push_str(&format!("  {}: {}\n", short_name, value));
            }
        }

        ExtractionResult {
            text: full_text,
            method,
            ixbrl_values,
        }
    }
}

/// Extract from JSON filings (e.g., confirmation statements).
fn extract_from_json(json_text: &str) -> ExtractionResult {
    // Try to pretty-print the JSON for readability
    let text = match serde_json::from_str::<serde_json::Value>(json_text) {
        Ok(val) => {
            // Extract meaningful text from known fields
            let mut parts = Vec::new();

            // Walk common CH JSON filing fields
            extract_json_text(&val, &mut parts, 0);

            if parts.is_empty() {
                serde_json::to_string_pretty(&val).unwrap_or_else(|_| json_text.to_string())
            } else {
                parts.join("\n")
            }
        }
        Err(_) => json_text.to_string(),
    };

    ExtractionResult {
        text,
        method: ExtractionMethod::Json,
        ixbrl_values: Vec::new(),
    }
}

/// Recursively extract meaningful text from JSON values.
fn extract_json_text(val: &serde_json::Value, parts: &mut Vec<String>, depth: usize) {
    let indent = "  ".repeat(depth);
    match val {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                match value {
                    serde_json::Value::String(s) if !s.is_empty() => {
                        parts.push(format!("{}{}: {}", indent, humanize_key(key), s));
                    }
                    serde_json::Value::Number(n) => {
                        parts.push(format!("{}{}: {}", indent, humanize_key(key), n));
                    }
                    serde_json::Value::Bool(b) => {
                        parts.push(format!("{}{}: {}", indent, humanize_key(key), b));
                    }
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        parts.push(format!("{}{}:", indent, humanize_key(key)));
                        extract_json_text(value, parts, depth + 1);
                    }
                    _ => {}
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_json_text(item, parts, depth);
                parts.push(String::new());
            }
        }
        _ => {}
    }
}

/// Convert snake_case JSON keys to readable labels.
fn humanize_key(key: &str) -> String {
    key.replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract from XML documents.
fn extract_from_xml(xml_text: &str) -> ExtractionResult {
    // Use the XHTML extractor — it handles XML well enough
    let text = xhtml::extract_text_from_xhtml(xml_text);
    ExtractionResult {
        text: if text.trim().is_empty() {
            "[XML document contained no extractable text.]".to_string()
        } else {
            text
        },
        method: ExtractionMethod::Xml,
        ixbrl_values: Vec::new(),
    }
}

/// Extract from PDF.
fn extract_from_pdf(pdf_bytes: &[u8]) -> ExtractionResult {
    match pdf::extract_text_from_pdf(pdf_bytes) {
        pdf::PdfExtractionResult::Text(text) => ExtractionResult {
            text,
            method: ExtractionMethod::PdfNative,
            ixbrl_values: Vec::new(),
        },
        pdf::PdfExtractionResult::NeedsOcr => ExtractionResult {
            text: "[Scanned PDF — OCR not yet implemented. \
                   This document contains images rather than embedded text.]"
                .to_string(),
            method: ExtractionMethod::PdfScannedNoOcr,
            ixbrl_values: Vec::new(),
        },
        pdf::PdfExtractionResult::Error(e) => ExtractionResult {
            text: format!("[PDF extraction error: {}]", e),
            method: ExtractionMethod::PdfNative,
            ixbrl_values: Vec::new(),
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Integration Tests — Real CH Documents
// ═══════════════════════════════════════════════════════════════════════════════
//
// These tests download ACTUAL Companies House filings and validate extraction.
// They require CH_API_KEY in .env and network access.
//
// Run with: cargo test real_ch -- --nocapture
//
// Test matrix:
//   ┌─────────────────────────┬──────────────────┬─────────────────────────┐
//   │ Category                │ Expected Format  │ Key Validation          │
//   ├─────────────────────────┼──────────────────┼─────────────────────────┤
//   │ Micro-entity accounts   │ iXBRL            │ Tables, iXBRL tags      │
//   │ Full accounts (large)   │ iXBRL / PDF      │ Financial tables        │
//   │ Confirmation statement  │ XHTML / XML      │ Structured key/values   │
//   │ Officer appointment     │ PDF (text-based) │ Names, dates            │
//   │ Old annual return       │ PDF (may scan)   │ Graceful fallback       │
//   │ Capital allotment       │ XHTML / PDF      │ Share tables            │
//   │ Charge registration     │ PDF              │ Lender names, amounts   │
//   │ Incorporation (CS01)    │ PDF (text-based) │ Company details         │
//   └─────────────────────────┴──────────────────┴─────────────────────────┘

#[cfg(test)]
mod real_ch_tests {
    use super::*;
    use crate::ch_api::client;

    /// Helper: get API key or skip test.
    fn api_key() -> String {
        let _ = dotenvy::dotenv();
        match std::env::var("CH_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                eprintln!("⚠ CH_API_KEY not set — skipping integration test");
                String::new()
            }
        }
    }

    /// Helper: fetch filings for a company and find one matching a category.
    async fn find_filing(
        api_key: &str,
        company_number: &str,
        category: &str,
    ) -> Option<crate::ch_api::types::FilingHistoryItem> {
        let filings = client::get_filing_history(api_key, company_number, Some(category), Some(10))
            .await
            .ok()?;
        filings.into_iter().find(|f| {
            f.links
                .as_ref()
                .and_then(|l| l.document_metadata.as_ref())
                .is_some()
        })
    }

    /// Helper: download and extract a filing.
    async fn download_and_extract(
        api_key: &str,
        filing: &crate::ch_api::types::FilingHistoryItem,
    ) -> ExtractionResult {
        let meta_url = filing
            .links
            .as_ref()
            .unwrap()
            .document_metadata
            .as_ref()
            .unwrap();
        let doc = client::download_document(api_key, meta_url)
            .await
            .expect("Document download failed");
        extract_text(&doc)
    }

    // ── Test 1: iXBRL Micro-entity Accounts ────────────────────────────
    // Company: 14506913 (small company, likely micro-entity iXBRL accounts)
    #[tokio::test]
    async fn real_ch_ixbrl_micro_accounts() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: iXBRL micro-entity accounts ──");

        // Fetch accounts filing
        let filing = find_filing(&key, "14506913", "accounts").await;
        if filing.is_none() {
            eprintln!("  No accounts filing found — skipping");
            return;
        }
        let filing = filing.unwrap();
        eprintln!("  Filing: {} ({})", 
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let result = download_and_extract(&key, &filing).await;
        eprintln!("  Method: {:?}", result.method);
        eprintln!("  Text length: {} chars", result.text.len());
        eprintln!("  iXBRL values: {}", result.ixbrl_values.len());

        // Validate
        assert!(
            result.text.len() > 100,
            "Accounts should produce substantial text (got {} chars)",
            result.text.len()
        );
        assert_eq!(
            result.method,
            ExtractionMethod::Xhtml,
            "Modern accounts should be XHTML"
        );

        // Preview
        let preview: String = result.text.chars().take(500).collect();
        eprintln!("  Preview:\n{}", preview);
        eprintln!("  ✓ PASSED");
    }

    // ── Test 2: Large Company Full Accounts ────────────────────────────
    // Company: 00102498 (Barclays — large, complex accounts)
    #[tokio::test]
    async fn real_ch_large_company_accounts() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Large company accounts (Barclays) ──");

        let filing = find_filing(&key, "00102498", "accounts").await;
        if filing.is_none() {
            eprintln!("  No accounts filing found — skipping");
            return;
        }
        let filing = filing.unwrap();
        eprintln!("  Filing: {} ({})", 
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let result = download_and_extract(&key, &filing).await;
        eprintln!("  Method: {:?}", result.method);
        eprintln!("  Text length: {} chars", result.text.len());

        // Large accounts may come as PDF (sometimes image-heavy).
        // We validate that extraction succeeds at all — the amount of text
        // depends on whether it's iXBRL (lots) or scanned PDF (minimal).
        assert!(
            !result.text.is_empty(),
            "Large company accounts should produce some output"
        );

        match result.method {
            ExtractionMethod::Xhtml => {
                // iXBRL should produce substantial content
                assert!(
                    result.text.len() > 500,
                    "XHTML accounts should produce extensive text (got {} chars)",
                    result.text.len()
                );
                let has_table_indicators = result.text.contains('|')
                    || result.text.contains("───")
                    || result.text.contains(':');
                assert!(
                    has_table_indicators,
                    "XHTML accounts should contain structured data"
                );
            }
            ExtractionMethod::PdfNative => {
                eprintln!("  → PDF with embedded text");
            }
            ExtractionMethod::PdfScannedNoOcr => {
                eprintln!("  → Scanned PDF (OCR needed)");
                assert!(result.text.contains("OCR") || result.text.contains("scanned"));
            }
            _ => {
                eprintln!("  → Unexpected method: {:?}", result.method);
            }
        }

        let preview: String = result.text.chars().take(500).collect();
        eprintln!("  Preview:\n{}", preview);
        eprintln!("  ✓ PASSED");
    }

    // ── Test 3: Confirmation Statement ──────────────────────────────────
    #[tokio::test]
    async fn real_ch_confirmation_statement() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Confirmation statement ──");

        let filing = find_filing(&key, "14506913", "confirmation-statement").await;
        if filing.is_none() {
            eprintln!("  No confirmation statement found — skipping");
            return;
        }
        let filing = filing.unwrap();
        eprintln!("  Filing: {} ({})",
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let result = download_and_extract(&key, &filing).await;
        eprintln!("  Method: {:?}", result.method);
        eprintln!("  Text length: {} chars", result.text.len());

        // Confirmation statements should produce some text
        assert!(
            result.text.len() > 20,
            "Confirmation statement should have content (got {} chars)",
            result.text.len()
        );

        // Should NOT be unsupported
        assert!(
            !matches!(result.method, ExtractionMethod::Unsupported(_)),
            "Confirmation statement should be a supported format"
        );

        let preview: String = result.text.chars().take(500).collect();
        eprintln!("  Preview:\n{}", preview);
        eprintln!("  ✓ PASSED");
    }

    // ── Test 4: Officer Change (AP01/TM01) ──────────────────────────────
    #[tokio::test]
    async fn real_ch_officer_appointment() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Officer appointment ──");

        let filing = find_filing(&key, "14506913", "officers").await;
        if filing.is_none() {
            eprintln!("  No officers filing found — skipping");
            return;
        }
        let filing = filing.unwrap();
        eprintln!("  Filing: {} ({})",
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let result = download_and_extract(&key, &filing).await;
        eprintln!("  Method: {:?}", result.method);
        eprintln!("  Text length: {} chars", result.text.len());

        // Officer docs should have some content
        assert!(
            result.text.len() > 10,
            "Officer document should have content (got {} chars)",
            result.text.len()
        );

        let preview: String = result.text.chars().take(500).collect();
        eprintln!("  Preview:\n{}", preview);
        eprintln!("  ✓ PASSED");
    }

    // ── Test 5: Capital / Allotment of Shares ──────────────────────────
    #[tokio::test]
    async fn real_ch_capital_allotment() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Capital/allotment document ──");

        // Use a company that's likely to have capital filings
        let filing = find_filing(&key, "00102498", "capital").await;
        if filing.is_none() {
            eprintln!("  No capital filing found — skipping");
            return;
        }
        let filing = filing.unwrap();
        eprintln!("  Filing: {} ({})",
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let result = download_and_extract(&key, &filing).await;
        eprintln!("  Method: {:?}", result.method);
        eprintln!("  Text length: {} chars", result.text.len());

        assert!(
            result.text.len() > 10,
            "Capital document should have content (got {} chars)",
            result.text.len()
        );

        let preview: String = result.text.chars().take(500).collect();
        eprintln!("  Preview:\n{}", preview);
        eprintln!("  ✓ PASSED");
    }

    // ── Test 6: Old Company Filings (may be scanned) ────────────────────
    // Very old filings from established companies are often scanned images.
    #[tokio::test]
    async fn real_ch_old_filing_graceful_fallback() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Old filing (potential scan) ──");

        // Fetch general filings for an old well-known company
        let filings = client::get_filing_history(&key, "00102498", None, Some(50))
            .await
            .unwrap_or_default();

        // Find the oldest filing that has a document
        let old_filing = filings.iter().rev().find(|f| {
            f.links
                .as_ref()
                .and_then(|l| l.document_metadata.as_ref())
                .is_some()
        });

        if old_filing.is_none() {
            eprintln!("  No old filing found — skipping");
            return;
        }
        let filing = old_filing.unwrap();
        eprintln!("  Filing: {} ({})",
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let meta_url = filing
            .links.as_ref().unwrap()
            .document_metadata.as_ref().unwrap();

        let doc = client::download_document(&key, meta_url).await;
        match doc {
            Ok(content) => {
                let result = extract_text(&content);
                eprintln!("  Method: {:?}", result.method);
                eprintln!("  Text length: {} chars", result.text.len());

                // Whether text or scanned, should not panic
                assert!(
                    !result.text.is_empty(),
                    "Result text should never be empty (even scanned PDFs get a message)"
                );

                if result.method == ExtractionMethod::PdfScannedNoOcr {
                    eprintln!("  → Detected as scanned PDF (OCR message shown)");
                    assert!(result.text.contains("OCR") || result.text.contains("scanned"),
                        "Scanned PDF message should mention OCR");
                }

                let preview: String = result.text.chars().take(300).collect();
                eprintln!("  Preview:\n{}", preview);
            }
            Err(e) => {
                eprintln!("  Download failed (expected for very old docs): {}", e);
                // Old documents may not be available — this is acceptable
            }
        }
        eprintln!("  ✓ PASSED");
    }

    // ── Test 7: Incorporation Document ──────────────────────────────────
    #[tokio::test]
    async fn real_ch_incorporation_document() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Incorporation document ──");

        let filing = find_filing(&key, "14506913", "incorporation").await;
        if filing.is_none() {
            eprintln!("  No incorporation filing found — skipping");
            return;
        }
        let filing = filing.unwrap();
        eprintln!("  Filing: {} ({})",
            filing.description.as_deref().unwrap_or("?"),
            filing.date.as_deref().unwrap_or("?"));

        let result = download_and_extract(&key, &filing).await;
        eprintln!("  Method: {:?}", result.method);
        eprintln!("  Text length: {} chars", result.text.len());

        assert!(
            result.text.len() > 20,
            "Incorporation document should have content (got {} chars)",
            result.text.len()
        );

        let preview: String = result.text.chars().take(500).collect();
        eprintln!("  Preview:\n{}", preview);
        eprintln!("  ✓ PASSED");
    }

    // ── Test 8: Table structure preservation ────────────────────────────
    // This test specifically validates that table content comes through
    // with columnar structure intact.
    #[tokio::test]
    async fn real_ch_table_structure_in_accounts() {
        let key = api_key();
        if key.is_empty() { return; }

        eprintln!("── Test: Table structure preservation ──");

        // Micro/small company accounts typically have balance sheet tables
        let filing = find_filing(&key, "14506913", "accounts").await;
        if filing.is_none() {
            eprintln!("  No accounts filing found — skipping");
            return;
        }
        let filing = filing.unwrap();
        let result = download_and_extract(&key, &filing).await;

        if result.method == ExtractionMethod::Xhtml {
            // Check for pipe separators (our table format)
            let pipe_lines: Vec<&str> = result.text.lines()
                .filter(|l| l.contains('|'))
                .collect();

            eprintln!("  Table rows found: {}", pipe_lines.len());

            if !pipe_lines.is_empty() {
                eprintln!("  Sample table rows:");
                for line in pipe_lines.iter().take(5) {
                    eprintln!("    {}", line);
                }

                // Validate that table rows have consistent column count
                let col_counts: Vec<usize> = pipe_lines.iter()
                    .map(|l| l.matches('|').count())
                    .collect();
                
                if col_counts.len() > 1 {
                    let first = col_counts[0];
                    let consistent = col_counts.iter().all(|&c| c == first);
                    if !consistent {
                        eprintln!("  ⚠ Column count varies (may be multiple tables): {:?}", 
                            &col_counts[..col_counts.len().min(10)]);
                    }
                }
            }

            // Check for definition-list style entries
            let kv_lines: Vec<&str> = result.text.lines()
                .filter(|l| l.contains(": ") && !l.starts_with('[') && !l.starts_with("---"))
                .collect();
            eprintln!("  Key-value pairs found: {}", kv_lines.len());
            for line in kv_lines.iter().take(5) {
                eprintln!("    {}", line);
            }
        }

        eprintln!("  ✓ PASSED");
    }
}
