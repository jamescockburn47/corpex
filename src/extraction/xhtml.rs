//! XHTML / iXBRL text extraction.
//!
//! Companies House serves most modern filings as XHTML (often iXBRL).
//! This module strips tags and extracts readable text content,
//! with special handling for:
//!   - Tables → pipe-delimited rows preserving columnar structure
//!   - Definition lists (<dl>) → "Key: Value" pairs
//!   - Ordered/unordered lists → bulleted/numbered items
//!   - Nested structures → indented for readability

use scraper::{Html, Selector};

/// Extract readable text from an XHTML/HTML string.
///
/// Strategy:
///   1. Parse with `scraper`
///   2. Remove `<script>`, `<style>`, `<head>` content
///   3. Handle tables, definition lists, and lists specially
///   4. Extract remaining text nodes preserving paragraph breaks
///   5. Collapse excessive whitespace
pub fn extract_text_from_xhtml(xhtml: &str) -> String {
    let doc = Html::parse_document(xhtml);

    let body_sel = Selector::parse("body").unwrap();

    let mut output = String::new();

    // Find body or fall back to whole document
    let root_elements: Vec<scraper::ElementRef<'_>> = doc.select(&body_sel).collect();

    if root_elements.is_empty() {
        extract_node(&doc.root_element(), &mut output, 0);
    } else {
        for body in &root_elements {
            extract_node(body, &mut output, 0);
        }
    }

    clean_extracted_text(&output)
}

/// Recursively extract text from an element with structure awareness.
///
/// `depth` tracks nesting for indentation of lists, etc.
fn extract_node(
    element: &scraper::ElementRef<'_>,
    output: &mut String,
    depth: usize,
) {
    for child in element.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                let text = t.text.trim();
                if !text.is_empty() {
                    output.push_str(text);
                    output.push(' ');
                }
            }
            scraper::node::Node::Element(_) => {
                if let Some(el_ref) = scraper::ElementRef::wrap(child) {
                    let tag = el_ref.value().name();

                    // Skip script, style, head, noscript
                    if matches!(tag, "script" | "style" | "head" | "noscript") {
                        continue;
                    }

                    match tag {
                        // ── Tables ──────────────────────────────────────
                        "table" => {
                            output.push('\n');
                            extract_table(&el_ref, output);
                            output.push('\n');
                        }

                        // ── Definition Lists ────────────────────────────
                        "dl" => {
                            output.push('\n');
                            extract_definition_list(&el_ref, output);
                            output.push('\n');
                        }

                        // ── Ordered Lists ───────────────────────────────
                        "ol" => {
                            output.push('\n');
                            extract_ordered_list(&el_ref, output, depth);
                            output.push('\n');
                        }

                        // ── Unordered Lists ─────────────────────────────
                        "ul" => {
                            output.push('\n');
                            extract_unordered_list(&el_ref, output, depth);
                            output.push('\n');
                        }

                        // ── Block Elements ──────────────────────────────
                        "p" | "div" | "section" | "article" | "blockquote" | "pre"
                        | "header" | "footer" | "main" | "aside" | "figure"
                        | "figcaption" | "details" | "summary" => {
                            output.push('\n');
                            extract_node(&el_ref, output, depth);
                            output.push('\n');
                        }

                        // ── Headings ────────────────────────────────────
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let level = &tag[1..2];
                            let prefix = match level {
                                "1" => "\n═══ ",
                                "2" => "\n─── ",
                                "3" => "\n── ",
                                _ => "\n— ",
                            };
                            output.push_str(prefix);
                            let heading_text = collect_text(&el_ref);
                            output.push_str(&heading_text);
                            match level {
                                "1" => output.push_str(" ═══\n"),
                                "2" => output.push_str(" ───\n"),
                                "3" => output.push_str(" ──\n"),
                                _ => output.push_str(" —\n"),
                            };
                        }

                        // ── Line Breaks ─────────────────────────────────
                        "br" => output.push('\n'),
                        "hr" => output.push_str("\n────────────────────\n"),

                        // ── Skip table sub-elements (handled by extract_table) ──
                        "thead" | "tbody" | "tfoot" | "tr" | "th" | "td"
                        | "caption" | "colgroup" | "col" => {
                            // These are handled by the table extractor
                            // but might appear outside a <table> in malformed HTML
                            extract_node(&el_ref, output, depth);
                        }

                        // ── Skip dl sub-elements (handled by extract_definition_list) ──
                        "dt" | "dd" => {
                            extract_node(&el_ref, output, depth);
                        }

                        // ── Inline Elements ─────────────────────────────
                        _ => {
                            extract_node(&el_ref, output, depth);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// ─── Table Extraction ────────────────────────────────────────────────────────

/// Extract a table as pipe-delimited rows with a header separator.
///
/// Produces output like:
/// ```text
/// | Director Name | Appointed  | Resigned   |
/// |---------------|------------|------------|
/// | John Smith    | 2020-01-15 | —          |
/// | Jane Doe      | 2018-06-01 | 2023-03-10 |
/// ```
fn extract_table(table: &scraper::ElementRef<'_>, output: &mut String) {
    let tr_sel = Selector::parse("tr").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let caption_sel = Selector::parse("caption").unwrap();

    // Extract caption if present
    if let Some(caption) = table.select(&caption_sel).next() {
        let caption_text = collect_text(&caption).trim().to_string();
        if !caption_text.is_empty() {
            output.push_str(&format!("[Table: {}]\n", caption_text));
        }
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut has_header = false;

    for tr in table.select(&tr_sel) {
        let mut cells: Vec<String> = Vec::new();
        let mut row_is_header = false;

        // Check for <th> cells first
        let ths: Vec<scraper::ElementRef<'_>> = tr.select(&th_sel).collect();
        if !ths.is_empty() {
            row_is_header = true;
            for th in &ths {
                cells.push(collect_text(th).trim().to_string());
            }
        }

        // Collect <td> cells
        for td in tr.select(&td_sel) {
            cells.push(collect_text(&td).trim().to_string());
        }

        if !cells.is_empty() {
            if row_is_header && !has_header {
                has_header = true;
                rows.insert(0, cells); // Ensure header is first
            } else {
                rows.push(cells);
            }
        }
    }

    if rows.is_empty() {
        return;
    }

    // Calculate column widths for alignment
    let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths: Vec<usize> = vec![0; num_cols];

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(cell.len()).max(3);
            }
        }
    }

    // Render rows — no truncation, preserve all data
    for (row_idx, row) in rows.iter().enumerate() {
        output.push_str("| ");
        for (i, cell) in row.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(3);
            output.push_str(&format!("{:<width$}", cell, width = width));
            output.push_str(" | ");
        }
        // Pad missing columns
        for i in row.len()..num_cols {
            let width = col_widths.get(i).copied().unwrap_or(10);
            output.push_str(&format!("{:<width$}", "", width = width));
            output.push_str(" | ");
        }
        output.push('\n');

        // Add separator after header row
        if row_idx == 0 && has_header {
            output.push_str("| ");
            for i in 0..num_cols {
                let width = col_widths.get(i).copied().unwrap_or(10);
                output.push_str(&"-".repeat(width));
                output.push_str(" | ");
            }
            output.push('\n');
        }
    }
}

// ─── Definition List Extraction ──────────────────────────────────────────────

/// Extract a definition list as "Term: Definition" pairs.
///
/// Produces:
/// ```text
/// Registered Office: 123 Main Street, London
/// Company Status: Active
/// ```
fn extract_definition_list(dl: &scraper::ElementRef<'_>, output: &mut String) {
    let dt_sel = Selector::parse("dt").unwrap();
    let dd_sel = Selector::parse("dd").unwrap();

    let dts: Vec<scraper::ElementRef<'_>> = dl.select(&dt_sel).collect();
    let dds: Vec<scraper::ElementRef<'_>> = dl.select(&dd_sel).collect();

    // Pair up dt/dd elements
    let max_pairs = dts.len().max(dds.len());
    for i in 0..max_pairs {
        let term = dts
            .get(i)
            .map(|dt| collect_text(dt).trim().to_string())
            .unwrap_or_default();
        let def = dds
            .get(i)
            .map(|dd| collect_text(dd).trim().to_string())
            .unwrap_or_default();

        if !term.is_empty() && !def.is_empty() {
            output.push_str(&format!("{}: {}\n", term, def));
        } else if !term.is_empty() {
            output.push_str(&format!("{}:\n", term));
        } else if !def.is_empty() {
            output.push_str(&format!("  {}\n", def));
        }
    }
}

// ─── List Extraction ─────────────────────────────────────────────────────────

/// Extract an ordered list with numbered items.
fn extract_ordered_list(ol: &scraper::ElementRef<'_>, output: &mut String, depth: usize) {
    let li_sel = Selector::parse("li").unwrap();
    let indent = "  ".repeat(depth);

    for (i, li) in ol.select(&li_sel).enumerate() {
        let text = collect_text(&li).trim().to_string();
        if !text.is_empty() {
            output.push_str(&format!("{}{}. {}\n", indent, i + 1, text));
        }
    }
}

/// Extract an unordered list with bullet points.
fn extract_unordered_list(ul: &scraper::ElementRef<'_>, output: &mut String, depth: usize) {
    let li_sel = Selector::parse("li").unwrap();
    let indent = "  ".repeat(depth);

    for li in ul.select(&li_sel) {
        let text = collect_text(&li).trim().to_string();
        if !text.is_empty() {
            output.push_str(&format!("{}• {}\n", indent, text));
        }
    }
}

// ─── Utility Functions ───────────────────────────────────────────────────────

/// Collect all text content from an element (flattened, no structure).
/// Used for cells, headings, list items — places where we want the raw text.
fn collect_text(element: &scraper::ElementRef<'_>) -> String {
    element.text().collect::<Vec<_>>().join(" ")
}

/// Extract text from iXBRL-specific tags.
/// Many CH filings use `<ix:nonNumeric>` and `<ix:nonFraction>` tags.
pub fn extract_ixbrl_values(xhtml: &str) -> Vec<(String, String)> {
    let doc = Html::parse_document(xhtml);
    let mut values = Vec::new();

    // ix:nonNumeric contains textual data
    if let Ok(sel) = Selector::parse("ix\\:nonNumeric, ix\\:nonnumeric") {
        for el in doc.select(&sel) {
            let name = el
                .value()
                .attr("name")
                .unwrap_or("unknown")
                .to_string();
            let text: String = el.text().collect::<Vec<_>>().join(" ");
            let text = text.trim().to_string();
            if !text.is_empty() {
                values.push((name, text));
            }
        }
    }

    // ix:nonFraction contains numeric data
    if let Ok(sel) = Selector::parse("ix\\:nonFraction, ix\\:nonfraction") {
        for el in doc.select(&sel) {
            let name = el
                .value()
                .attr("name")
                .unwrap_or("unknown")
                .to_string();
            let text: String = el.text().collect::<Vec<_>>().join(" ");
            let text = text.trim().to_string();
            if !text.is_empty() {
                values.push((name, text));
            }
        }
    }

    values
}

/// Clean up extracted text: collapse whitespace, remove excessive blank lines.
fn clean_extracted_text(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();

    // Trim each line
    lines = lines.iter().map(|l| l.trim()).collect();

    // Collapse consecutive blank lines into at most one
    let mut result = Vec::new();
    let mut prev_blank = false;
    for line in &lines {
        if line.is_empty() {
            if !prev_blank && !result.is_empty() {
                result.push("");
                prev_blank = true;
            }
        } else {
            result.push(line);
            prev_blank = false;
        }
    }

    result.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_xhtml_extraction() {
        let html = r#"<html><body><h1>Company Report</h1><p>This is a test.</p></body></html>"#;
        let text = extract_text_from_xhtml(html);
        assert!(text.contains("Company Report"));
        assert!(text.contains("This is a test."));
    }

    #[test]
    fn test_script_removal() {
        let html = r#"<html><body><p>Hello</p><script>alert('x')</script><p>World</p></body></html>"#;
        let text = extract_text_from_xhtml(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn test_table_extraction() {
        let html = r#"
        <html><body>
        <table>
            <tr><th>Name</th><th>Role</th><th>Appointed</th></tr>
            <tr><td>John Smith</td><td>Director</td><td>2020-01-15</td></tr>
            <tr><td>Jane Doe</td><td>Secretary</td><td>2019-06-01</td></tr>
        </table>
        </body></html>"#;
        let text = extract_text_from_xhtml(html);
        // Should preserve columnar structure
        assert!(text.contains("Name"));
        assert!(text.contains("John Smith"));
        assert!(text.contains("|"));
        // Verify header separator exists
        assert!(text.contains("---"));
    }

    #[test]
    fn test_definition_list_extraction() {
        let html = r#"
        <html><body>
        <dl>
            <dt>Company Name</dt><dd>Acme Ltd</dd>
            <dt>Status</dt><dd>Active</dd>
        </dl>
        </body></html>"#;
        let text = extract_text_from_xhtml(html);
        assert!(text.contains("Company Name: Acme Ltd"));
        assert!(text.contains("Status: Active"));
    }

    #[test]
    fn test_unordered_list_extraction() {
        let html = r#"
        <html><body>
        <ul>
            <li>First item</li>
            <li>Second item</li>
        </ul>
        </body></html>"#;
        let text = extract_text_from_xhtml(html);
        assert!(text.contains("• First item"));
        assert!(text.contains("• Second item"));
    }

    #[test]
    fn test_ordered_list_extraction() {
        let html = r#"
        <html><body>
        <ol>
            <li>Step one</li>
            <li>Step two</li>
        </ol>
        </body></html>"#;
        let text = extract_text_from_xhtml(html);
        assert!(text.contains("1. Step one"));
        assert!(text.contains("2. Step two"));
    }

    #[test]
    fn test_heading_formatting() {
        let html = r#"<html><body><h1>Main Title</h1><h2>Section</h2><h3>Subsection</h3></body></html>"#;
        let text = extract_text_from_xhtml(html);
        assert!(text.contains("═══ Main Title ═══"));
        assert!(text.contains("─── Section ───"));
        assert!(text.contains("── Subsection ──"));
    }
}
