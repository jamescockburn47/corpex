//! Document text extraction pipeline for Companies House filings.
//!
//! Tiered approach:
//!   Tier 0: XHTML / iXBRL — most modern CH filings; parsed with `scraper`
//!   Tier 1: JSON — structured data filings; extract directly
//!   Tier 2: PDF with embedded text — `pdf_extract`
//!   Tier 3: Scanned PDF — OCR fallback (not yet implemented)

pub mod xhtml;
pub mod pdf;
pub mod pipeline;

pub use pipeline::{extract_text, ExtractionResult};
