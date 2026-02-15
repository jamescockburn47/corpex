use super::types::*;
use anyhow::{Context, Result};
use base64::Engine;

const CH_API_BASE: &str = "https://api.company-information.service.gov.uk";

fn build_client(api_key: &str) -> Result<reqwest::Client> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{}:", api_key));
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Basic {}", encoded))?,
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")
}

pub async fn search_companies(api_key: &str, query: &str) -> Result<Vec<CompanySearchResult>> {
    let client = build_client(api_key)?;
    let url = format!("{}/search/companies", CH_API_BASE);
    let resp = client
        .get(&url)
        .query(&[("q", query), ("items_per_page", "20")])
        .send()
        .await
        .context("Company search request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for search", resp.status());
    }

    let body: CompanySearchResponse = resp.json().await.context("Failed to parse search response")?;
    Ok(body.items.unwrap_or_default())
}

/// Search for officers/directors by name.
pub async fn search_officers(api_key: &str, query: &str) -> Result<Vec<OfficerSearchResult>> {
    let client = build_client(api_key)?;
    let url = format!("{}/search/officers", CH_API_BASE);
    let resp = client
        .get(&url)
        .query(&[("q", query), ("items_per_page", "20")])
        .send()
        .await
        .context("Officer search request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for officer search", resp.status());
    }

    let body: OfficerSearchResponse = resp.json().await.context("Failed to parse officer search")?;
    Ok(body.items.unwrap_or_default())
}

/// Get all appointments for a specific officer.
/// `appointments_path` comes from OfficerSearchResult.links.self (e.g. "/officers/{id}/appointments")
pub async fn get_officer_appointments(
    api_key: &str,
    appointments_path: &str,
) -> Result<OfficerAppointmentsResponse> {
    let client = build_client(api_key)?;
    let url = if appointments_path.starts_with("http") {
        appointments_path.to_string()
    } else {
        format!("{}{}", CH_API_BASE, appointments_path)
    };
    let resp = client
        .get(&url)
        .query(&[("items_per_page", "100")])
        .send()
        .await
        .context("Officer appointments request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for officer appointments", resp.status());
    }

    resp.json()
        .await
        .context("Failed to parse officer appointments")
}

pub async fn get_company_profile(api_key: &str, company_number: &str) -> Result<CompanyProfile> {
    let client = build_client(api_key)?;
    let url = format!("{}/company/{}", CH_API_BASE, company_number);
    let resp = client.get(&url).send().await.context("Profile request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for profile {}", resp.status(), company_number);
    }

    resp.json().await.context("Failed to parse profile")
}

pub async fn get_officers(api_key: &str, company_number: &str) -> Result<Vec<Officer>> {
    let client = build_client(api_key)?;
    let url = format!("{}/company/{}/officers", CH_API_BASE, company_number);
    let resp = client
        .get(&url)
        .query(&[("items_per_page", "100")])
        .send()
        .await
        .context("Officers request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for officers {}", resp.status(), company_number);
    }

    let body: OfficerListResponse = resp.json().await.context("Failed to parse officers")?;
    Ok(body.items.unwrap_or_default())
}

pub async fn get_pscs(api_key: &str, company_number: &str) -> Result<Vec<Psc>> {
    let client = build_client(api_key)?;
    let url = format!(
        "{}/company/{}/persons-with-significant-control",
        CH_API_BASE, company_number
    );
    let resp = client
        .get(&url)
        .query(&[("items_per_page", "100")])
        .send()
        .await
        .context("PSC request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for PSCs {}", resp.status(), company_number);
    }

    let body: PscListResponse = resp.json().await.context("Failed to parse PSCs")?;
    Ok(body.items.unwrap_or_default())
}

pub async fn get_charges(api_key: &str, company_number: &str) -> Result<Vec<Charge>> {
    let client = build_client(api_key)?;
    let url = format!("{}/company/{}/charges", CH_API_BASE, company_number);
    let resp = client
        .get(&url)
        .query(&[("items_per_page", "100")])
        .send()
        .await
        .context("Charges request failed")?;

    if !resp.status().is_success() {
        // 404 is normal for companies without charges
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        anyhow::bail!("CH API returned {} for charges {}", resp.status(), company_number);
    }

    let body: ChargesListResponse = resp.json().await.context("Failed to parse charges")?;
    Ok(body.items.unwrap_or_default())
}

pub async fn get_insolvency(
    api_key: &str,
    company_number: &str,
) -> Result<Option<InsolvencyData>> {
    let client = build_client(api_key)?;
    let url = format!("{}/company/{}/insolvency", CH_API_BASE, company_number);
    let resp = client.get(&url).send().await.context("Insolvency request failed")?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        anyhow::bail!("CH API returned {} for insolvency {}", resp.status(), company_number);
    }

    let body: InsolvencyResponse = resp.json().await.context("Failed to parse insolvency")?;
    Ok(Some(InsolvencyData {
        status: body.status.unwrap_or_default(),
        cases: body.cases.unwrap_or_default(),
    }))
}

pub async fn get_filing_history(
    api_key: &str,
    company_number: &str,
    category: Option<&str>,
    items_per_page: Option<u32>,
) -> Result<Vec<FilingHistoryItem>> {
    let client = build_client(api_key)?;
    let url = format!("{}/company/{}/filing-history", CH_API_BASE, company_number);
    let mut query: Vec<(&str, String)> = vec![(
        "items_per_page",
        items_per_page.unwrap_or(25).to_string(),
    )];
    if let Some(cat) = category {
        query.push(("category", cat.to_string()));
    }

    let resp = client
        .get(&url)
        .query(&query)
        .send()
        .await
        .context("Filing history request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "CH API returned {} for filings {}",
            resp.status(),
            company_number
        );
    }

    let body: FilingHistoryResponse = resp.json().await.context("Failed to parse filings")?;
    Ok(body.items.unwrap_or_default())
}

/// The content type of a downloaded CH document.
#[derive(Debug, Clone)]
pub enum DocumentContent {
    Xhtml(String),
    Pdf(Vec<u8>),
    Json(String),
    Xml(String),
    Other { content_type: String, bytes: Vec<u8> },
}

/// Download a filing document. The `document_metadata_url` comes from
/// `FilingHistoryItem.links.document_metadata`.
///
/// CH Document API flow:
///   1. GET {document_metadata_url} → JSON with `links.document` URL
///   2. GET {document_url}/content with Accept header → actual bytes
pub async fn download_document(api_key: &str, document_metadata_url: &str) -> Result<DocumentContent> {
    let client = build_client(api_key)?;

    // The metadata URL from CH is relative (e.g. "/document/abc123")
    let meta_url = if document_metadata_url.starts_with("http") {
        document_metadata_url.to_string()
    } else {
        format!("https://document-api.company-information.service.gov.uk{}", document_metadata_url)
    };

    // Step 1: Get metadata
    let meta_resp = client
        .get(&meta_url)
        .send()
        .await
        .context("Document metadata request failed")?;

    if !meta_resp.status().is_success() {
        anyhow::bail!("Document metadata returned {}", meta_resp.status());
    }

    let meta: serde_json::Value = meta_resp
        .json()
        .await
        .context("Failed to parse document metadata")?;

    // Extract the document download link
    // NOTE: CH metadata `links.document` may already include `/content`
    let doc_url = meta
        .pointer("/links/document")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No document link in metadata"))?;

    let content_url = if doc_url.ends_with("/content") {
        doc_url.to_string()
    } else {
        format!("{}/content", doc_url)
    };

    // Step 2: Try XHTML first (structured, best for text extraction)
    let xhtml_resp = client
        .get(&content_url)
        .header("Accept", "application/xhtml+xml")
        .send()
        .await;

    if let Ok(resp) = xhtml_resp {
        if resp.status().is_success() {
            let ct = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            if ct.contains("xhtml") || ct.contains("html") || ct.contains("xml") {
                let text = resp.text().await.context("Failed to read XHTML body")?;
                return Ok(DocumentContent::Xhtml(text));
            }
        }
    }

    // Step 3: Try PDF
    let pdf_resp = client
        .get(&content_url)
        .header("Accept", "application/pdf")
        .send()
        .await
        .context("Document PDF request failed")?;

    if pdf_resp.status().is_success() {
        let ct = pdf_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let bytes = pdf_resp.bytes().await.context("Failed to read PDF body")?;
        if ct.contains("pdf") {
            return Ok(DocumentContent::Pdf(bytes.to_vec()));
        }
        if ct.contains("json") {
            let text = String::from_utf8_lossy(&bytes).to_string();
            return Ok(DocumentContent::Json(text));
        }
        if ct.contains("xml") {
            let text = String::from_utf8_lossy(&bytes).to_string();
            return Ok(DocumentContent::Xml(text));
        }
        return Ok(DocumentContent::Other {
            content_type: ct,
            bytes: bytes.to_vec(),
        });
    }

    anyhow::bail!("Could not download document (status {})", pdf_resp.status())
}
