//! PDF text extraction.
//!
//! Tier 1: Use `pdf_extract` for PDFs with embedded text.
//! Uses the CaseKit pattern: if extracted text < 50 chars, treat as scanned.



/// Result of a PDF text extraction attempt.
#[derive(Debug)]
pub enum PdfExtractionResult {
    /// Successfully extracted text from the PDF.
    Text(String),
    /// PDF appears to be scanned (text < 50 chars) — needs OCR.
    NeedsOcr,
    /// Extraction failed entirely.
    Error(String),
}

/// Extract text from a PDF byte slice.
///
/// Uses `pdf_extract` which handles PDFs with embedded text layers.
/// If the extracted text is very short (< 50 chars), the PDF is likely
/// a scanned image and would need OCR for meaningful extraction.
pub fn extract_text_from_pdf(pdf_bytes: &[u8]) -> PdfExtractionResult {
    match pdf_extract::extract_text_from_mem(pdf_bytes) {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.len() < 50 {
                tracing::info!(
                    "PDF text extraction yielded only {} chars — likely scanned",
                    trimmed.len()
                );
                if trimmed.is_empty() {
                    PdfExtractionResult::NeedsOcr
                } else {
                    // Return what we have, but flag that it might be incomplete
                    PdfExtractionResult::Text(format!(
                        "{}\n\n[Note: Very little text extracted — this may be a scanned document]",
                        trimmed
                    ))
                }
            } else {
                PdfExtractionResult::Text(clean_pdf_text(&trimmed))
            }
        }
        Err(e) => {
            tracing::warn!("PDF text extraction failed: {}", e);
            PdfExtractionResult::Error(format!("PDF extraction failed: {}", e))
        }
    }
}

/// Clean up PDF-extracted text.
///
/// PDF text extraction often produces:
///   - Excessive whitespace between characters (broken ligatures)
///   - Page break artifacts
///   - Header/footer repetition
fn clean_pdf_text(text: &str) -> String {
    // Collapse runs of spaces (but not newlines)
    let re_spaces = regex::Regex::new(r"[ \t]{3,}").unwrap();
    let text = re_spaces.replace_all(text, "  ").to_string();

    // Collapse excessive blank lines
    let re_blanks = regex::Regex::new(r"\n{4,}").unwrap();
    let text = re_blanks.replace_all(&text, "\n\n\n").to_string();

    // Remove form-feed characters (page breaks)
    let text = text.replace('\x0C', "\n---\n");

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_bytes_gives_error() {
        match extract_text_from_pdf(b"not a pdf") {
            PdfExtractionResult::Error(_) => {} // expected
            other => panic!("Expected Error, got {:?}", other),
        }
    }
}
