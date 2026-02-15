use crate::ch_api::types::*;
use std::collections::HashMap;

/// System prompt for corporate investigation analysis.
pub const SYSTEM_PROMPT: &str = "\
You are a corporate intelligence analyst specialising in UK company investigations \
using Companies House data. You provide clear, structured analysis of company profiles, \
officer networks, financial filings, and risk indicators.

IMPORTANT FORMATTING RULES:
- Do NOT use markdown formatting (no #, ##, **, `, |, or --- syntax)
- Use PLAIN TEXT with clear section headings in CAPITALS
- Use simple dashes (-) for bullet points
- Use blank lines between sections for readability
- For tables, use aligned plain text columns instead of pipe-separated tables

COMPANY CLASSIFICATION - ALWAYS DO THIS FIRST:
Before any analysis, classify the company into one of these types based on SIC codes, \
filing patterns, balance sheet structure, and company name:
- TRADING COMPANY: has turnover/revenue, SIC in retail/manufacturing/services, files P&L
- PROPERTY HOLDING / SPV: SIC 68xxx, balance sheet dominated by investment property or \
  fixed assets, little or no turnover — these companies exist to hold property, NOT to trade
- DORMANT / SHELL: files dormant accounts only, no financial activity
- GROUP HOLDING COMPANY: SIC 70100, exists to hold subsidiaries, may file consolidated accounts
- PROFESSIONAL SERVICES: SIC 69-74 range, likely fee-based income
- NEWLY INCORPORATED: no filings yet, recently created

State the classification prominently at the start of your executive summary. \
This classification MUST drive the rest of your analysis — do not apply trading \
company analysis (P&L, turnover trends, margins) to a non-trading entity.

ANALYTICAL TONE:
- Be factual, precise, and consistent. Avoid sensationalism or dramatic framing.
- Present the same data the same way each time: state the numbers, then state what they mean.
- Distinguish clearly between FACTS (from the data) and INFERENCES (your professional judgment).
- When citing risk, quantify it where possible rather than using vague alarming language.
- When referencing data from a specific filing document, cite it as [REF:filing_id] where \
  filing_id is the ID shown in the source label.

Keep responses focused and actionable. Flag anything that warrants further investigation.";

/// Build a structured context string from all available company data.
/// This becomes the user message context that the AI analyses.
pub fn build_company_context(
    company_number: &str,
    profile: Option<&CompanyProfile>,
    officers: Option<&Vec<Officer>>,
    pscs: Option<&Vec<Psc>>,
    charges: Option<&Vec<Charge>>,
    insolvency: Option<&Option<InsolvencyData>>,
    filings: Option<&Vec<FilingHistoryItem>>,
    extracted_texts: &HashMap<String, String>,
) -> String {
    let mut ctx = String::with_capacity(8000);

    ctx.push_str(&format!("=== COMPANY: {} ===\n\n", company_number));

    // ── Profile ──────────────────────────────────────────────────────
    if let Some(p) = profile {
        ctx.push_str("## Company Profile\n");
        ctx.push_str(&format!("Name: {}\n", p.display_name()));
        ctx.push_str(&format!("Number: {}\n", company_number));
        if let Some(s) = &p.company_status {
            ctx.push_str(&format!("Status: {}\n", s));
        }
        if let Some(t) = &p.company_type {
            ctx.push_str(&format!("Type: {}\n", t));
        }
        if let Some(d) = &p.date_of_creation {
            ctx.push_str(&format!("Incorporated: {}\n", d));
        }
        if let Some(d) = &p.date_of_cessation {
            ctx.push_str(&format!("Dissolved: {}\n", d));
        }
        if let Some(addr) = &p.registered_office_address {
            ctx.push_str(&format!("Address: {}\n", addr.one_line()));
        }
        if let Some(sic) = &p.sic_codes {
            let enriched: Vec<String> = sic.iter().map(|code| {
                let desc = sic_description(code);
                if desc.is_empty() {
                    code.clone()
                } else {
                    format!("{} ({})", code, desc)
                }
            }).collect();
            ctx.push_str(&format!("SIC codes: {}\n", enriched.join(", ")));
        }
        if p.accounts.as_ref().and_then(|a| a.overdue) == Some(true) {
            ctx.push_str("⚠ ACCOUNTS OVERDUE\n");
        }
        ctx.push('\n');
    }

    // ── Officers ─────────────────────────────────────────────────────
    if let Some(officers) = officers {
        let active: Vec<_> = officers.iter().filter(|o| o.is_active()).collect();
        let resigned: Vec<_> = officers.iter().filter(|o| !o.is_active()).collect();

        ctx.push_str(&format!("## Officers ({} active, {} resigned)\n", active.len(), resigned.len()));

        for o in &active {
            ctx.push_str(&format!("  [ACTIVE] {} — {} (appointed {})\n",
                o.display_name(),
                o.officer_role.as_deref().unwrap_or("?"),
                o.appointed_on.as_deref().unwrap_or("?"),
            ));
            if let Some(nat) = &o.nationality {
                ctx.push_str(&format!("    Nationality: {}\n", nat));
            }
            if let Some(occ) = &o.occupation {
                ctx.push_str(&format!("    Occupation: {}\n", occ));
            }
        }
        if !resigned.is_empty() {
            ctx.push_str(&format!("  ({} resigned officers not shown in full)\n", resigned.len()));
            for o in resigned.iter().take(5) {
                ctx.push_str(&format!("  [RESIGNED] {} — {} (appointed {}, resigned {})\n",
                    o.display_name(),
                    o.officer_role.as_deref().unwrap_or("?"),
                    o.appointed_on.as_deref().unwrap_or("?"),
                    o.resigned_on.as_deref().unwrap_or("?"),
                ));
            }
        }
        ctx.push('\n');
    }

    // ── PSCs ─────────────────────────────────────────────────────────
    if let Some(pscs) = pscs {
        if !pscs.is_empty() {
            ctx.push_str(&format!("## Persons with Significant Control ({} entries)\n", pscs.len()));
            for psc in pscs {
                let name = psc.name.as_deref().unwrap_or("Unknown");
                ctx.push_str(&format!("  PSC: {}", name));
                if let Some(natures) = &psc.natures_of_control {
                    ctx.push_str(&format!(" — {}", natures.join(", ")));
                }
                ctx.push('\n');
                if let Some(nat) = &psc.nationality {
                    ctx.push_str(&format!("    Nationality: {}\n", nat));
                }
                if let Some(notified) = &psc.notified_on {
                    ctx.push_str(&format!("    Notified: {}\n", notified));
                }
                if let Some(ceased) = &psc.ceased_on {
                    ctx.push_str(&format!("    ⚠ Ceased: {}\n", ceased));
                }
            }
            ctx.push('\n');
        }
    }

    // ── Charges ──────────────────────────────────────────────────────
    if let Some(charges) = charges {
        if !charges.is_empty() {
            ctx.push_str(&format!("## Charges & Mortgages ({} entries)\n", charges.len()));
            for c in charges {
                let status_str = c.status.as_deref().unwrap_or("?");
                ctx.push_str(&format!("  Charge: {} (status: {})\n",
                    c.classification.as_ref()
                        .and_then(|cl| cl.description.as_deref())
                        .unwrap_or("unknown type"),
                    status_str,
                ));
                if let Some(holder) = &c.persons_entitled {
                    for h in holder {
                        ctx.push_str(&format!("    Holder: {}\n", h.name.as_deref().unwrap_or("?")));
                    }
                }
                if let Some(created) = &c.created_on {
                    ctx.push_str(&format!("    Created: {}\n", created));
                }
            }
            ctx.push('\n');
        }
    }

    // ── Insolvency ───────────────────────────────────────────────────
    if let Some(Some(insolvency)) = insolvency {
        ctx.push_str("## ⚠ INSOLVENCY DATA\n");
        ctx.push_str(&format!("  Status: {}\n", insolvency.status.join(", ")));
        for case in &insolvency.cases {
            ctx.push_str(&format!("  Case: {} (number {})\n",
                case.case_type.as_deref().unwrap_or("?"),
                case.number.unwrap_or(0),
            ));
            if let Some(dates) = &case.dates {
                for d in dates {
                    ctx.push_str(&format!("    {}: {}\n",
                        d.date_type.as_deref().unwrap_or("?"),
                        d.date.as_deref().unwrap_or("?"),
                    ));
                }
            }
            if let Some(practitioners) = &case.practitioners {
                for ip in practitioners {
                    ctx.push_str(&format!("    IP: {} ({})\n",
                        ip.name.as_deref().unwrap_or("?"),
                        ip.role.as_deref().unwrap_or("?"),
                    ));
                }
            }
        }
        ctx.push('\n');
    }

    // ── Recent Filings ───────────────────────────────────────────────
    if let Some(filings) = filings {
        if !filings.is_empty() {
            let recent: Vec<_> = filings.iter().take(15).collect();
            ctx.push_str(&format!("## Filing History (showing {} of {})\n", recent.len(), filings.len()));
            for f in &recent {
                ctx.push_str(&format!("  {} — {} ({})\n",
                    f.date.as_deref().unwrap_or("?"),
                    f.description.as_deref().unwrap_or("?"),
                    f.category.as_deref().unwrap_or("?"),
                ));
            }
            ctx.push('\n');
        }
    }

    // ── Extracted Document Text ──────────────────────────────────────
    // Match extracted texts to this company via the filing list's transaction IDs
    let mut company_texts: Vec<(&str, &str, &str)> = Vec::new(); // (filing_id, description, text)
    if let Some(filings) = filings {
        for f in filings {
            if let Some(tid) = &f.transaction_id {
                if let Some(text) = extracted_texts.get(tid.as_str()) {
                    let desc = f.description.as_deref().unwrap_or("Unknown filing");
                    company_texts.push((tid.as_str(), desc, text.as_str()));
                }
            }
        }
    }
    // Fallback: if no filings list but few total texts, include all
    if company_texts.is_empty() && extracted_texts.len() <= 3 {
        for (k, v) in extracted_texts {
            company_texts.push((k.as_str(), "Unknown filing", v.as_str()));
        }
    }

    if !company_texts.is_empty() {
        ctx.push_str(&format!("## Extracted Document Text ({} documents)\n", company_texts.len()));
        for (filing_id, description, text) in &company_texts {
            // Truncate very long texts to keep context manageable
            let max_len = 6000;
            let display_text = if text.len() > max_len {
                format!("{}...\n[truncated, {} chars total]", &text[..max_len], text.len())
            } else {
                text.to_string()
            };
            ctx.push_str(&format!("--- [SOURCE:{}] {} ---\n{}\n", filing_id, description, display_text));
        }
        ctx.push('\n');
    }

    ctx
}

/// Build the initial analysis prompt (first message from user).
pub fn build_analysis_prompt(company_context: &str, year_from: i32, year_to: i32) -> String {
    let year_instruction = if year_from == year_to {
        format!("Focus your analysis on the {} accounting period.", year_from)
    } else {
        format!(
            "Focus your analysis on the period {} to {} inclusive. \
             Compare year-on-year figures where data is available for multiple years in this range.",
            year_from, year_to
        )
    };

    format!(
        "Please analyse the following Companies House data for this company.\n\n\
         {}\n\n\
         STEP 1 — CLASSIFY THE COMPANY\n\
         First, determine the company type from the SIC codes, filing patterns, and balance \
         sheet structure. State your classification clearly (e.g. 'This is a PROPERTY HOLDING \
         COMPANY' or 'This is an ACTIVE TRADING COMPANY'). This classification drives the \
         entire analysis — do not apply irrelevant metrics.\n\n\
         STEP 2 — PLAIN LANGUAGE SUMMARY\n\
         Write a 3-4 paragraph summary for a non-expert reader covering:\n\
         - What this company is and what it does (based on your classification)\n\
         - The financial position in plain English, using ONLY metrics relevant to this \
           company type (see guidance below)\n\
         - Who runs it and anything notable about ownership\n\
         - Any genuine red flags — distinguish between real concerns and things that are \
           normal for this type of company\n\n\
         STEP 3 — DETAILED ASSESSMENT\n\n\
         1. COMPANY OVERVIEW — classification, current status, age, registered purpose\n\n\
         2. KEY PERSONNEL — officers, PSCs, appointment patterns, any concerns\n\n\
         3. FINANCIAL HEALTH — ADAPT THIS SECTION TO THE COMPANY TYPE:\n\n\
           FOR TRADING COMPANIES:\n\
           - Start with yearly turnover summary (Turnover 2024: £X, Turnover 2023: £Y, etc.)\n\
           - Full P&L breakdown: revenue, cost of sales, gross profit, admin expenses, \
             operating profit, profit before/after tax\n\
           - Balance sheet: net assets, fixed assets, current assets, cash, creditors\n\
           - Margins and trends year-on-year\n\n\
           FOR PROPERTY HOLDING / SPV COMPANIES:\n\
           - There is typically NO turnover and NO P&L — do not ask for or fabricate one\n\
           - Focus on: investment property valuation, total fixed assets, net asset value\n\
           - Debt position: mortgage/charge amounts vs property value (loan-to-value ratio)\n\
           - Rental income if disclosed, otherwise note it is a balance-sheet entity\n\
           - Equity position and changes year-on-year\n\n\
           FOR DORMANT / SHELL COMPANIES:\n\
           - Confirm dormant status from filing pattern\n\
           - Note any residual assets or liabilities\n\
           - Flag any outstanding charges despite dormancy\n\
           - Skip P&L analysis entirely\n\n\
           FOR GROUP HOLDING COMPANIES:\n\
           - Focus on investment in subsidiaries\n\
           - Intercompany balances and dividend income\n\
           - Consolidated vs entity-level figures\n\
           - Group structure and control\n\n\
           FOR ALL TYPES: state exact accounting period dates. If multiple years available \
           in the requested range, show side by side and comment on trends.\n\n\
         4. RISK INDICATORS — insolvency, overdue accounts, charges, unusual patterns. \
            Distinguish between risks that are NORMAL for this company type vs genuinely \
            concerning signals.\n\n\
         5. NETWORK CONNECTIONS — shared directors, PSC chains worth investigating\n\n\
         6. RECOMMENDATIONS — what to investigate further, tailored to company type\n\n\
         Remember: use plain text only, no markdown formatting. Be factual and consistent.\n\n\
         ---\n\n{}", year_instruction, company_context)
}

/// Build a group-level analysis prompt that analyses multiple companies in a corporate group.
///
/// If the parent has consolidated accounts, these are used as the primary source
/// to avoid token bloat from duplicated data across subsidiaries.
pub fn build_group_analysis_prompt(
    parent_context: &str,
    subsidiary_contexts: &[(String, String)], // (company_name, context)
    has_consolidated: bool,
    year_from: i32,
    year_to: i32,
) -> String {
    let year_range = if year_from == year_to {
        format!("{}", year_from)
    } else {
        format!("{}-{}", year_from, year_to)
    };

    let mut prompt = format!(
        "Analyse this CORPORATE GROUP structure ({} period).\n\n", year_range
    );

    if has_consolidated {
        prompt.push_str(
            "IMPORTANT: The parent company files CONSOLIDATED (GROUP) ACCOUNTS. \
             These already include the financial results of all subsidiaries. \
             Use the consolidated figures as the primary data source. \
             Only reference individual subsidiary data for specific operational breakdowns.\n\n"
        );
    }

    prompt.push_str("=== PARENT COMPANY ===\n");
    prompt.push_str(parent_context);
    prompt.push_str("\n\n");

    // Add subsidiary contexts (truncated to avoid token explosion)
    for (name, ctx) in subsidiary_contexts {
        prompt.push_str(&format!("=== SUBSIDIARY: {} ===\n", name));
        // Truncate individual subsidiary contexts to 3000 chars max
        if ctx.len() > 3000 {
            prompt.push_str(&ctx[..3000]);
            prompt.push_str("\n... [truncated for token efficiency]\n");
        } else {
            prompt.push_str(ctx);
        }
        prompt.push_str("\n\n");
    }

    prompt.push_str(
        "Provide a GROUP ANALYSIS with these sections:\n\n\
         1. GROUP OVERVIEW — what this group does, how many entities, hierarchy structure\n\n\
         2. GROUP FINANCIAL SUMMARY — consolidated turnover, profit, net assets for the \
            requested period. If consolidated accounts are available, lead with those figures. \
            Explain in plain English what the group earns and whether it's profitable.\n\n\
         3. SUBSIDIARY BREAKDOWN — for each active subsidiary:\n\
            - What it does (infer from SIC codes and name)\n\
            - Whether it's a trading entity or dormant/holding\n\
            - Key financials (if available and not already in consolidated figures)\n\
            - Status and any concerns\n\n\
         4. INTER-GROUP ANALYSIS — shared directors, common addresses, potential conflicts, \
            unusual structures\n\n\
         5. GROUP RISK ASSESSMENT — overall group health, contagion risks (if one entity fails \
            does it affect others?), concentration risks\n\n\
         6. RECOMMENDATIONS — what to investigate further across the group\n\n\
         Remember: use plain text only, no markdown formatting."
    );

    prompt
}

/// Look up a human-readable description for common SIC codes.
/// Returns empty string for unknown codes (the AI can still see the raw code).
fn sic_description(code: &str) -> &'static str {
    match code.trim() {
        // Property
        "68100" => "Buying and selling of own real estate",
        "68201" => "Renting and operating of Housing Association real estate",
        "68202" => "Letting and operating of conference and exhibition centres",
        "68209" => "Other letting and operating of own or leased real estate",
        "68310" => "Real estate agencies",
        "68320" => "Management of real estate on a fee or contract basis",
        // Holding / head office
        "64200" => "Activities of holding companies",
        "64209" => "Activities of other holding companies",
        "64910" => "Financial leasing",
        "64999" => "Financial intermediation not elsewhere classified",
        "70100" => "Activities of head offices",
        "70210" => "Public relations and communication activities",
        "70229" => "Management consultancy activities (other than financial management)",
        // Professional services
        "69101" => "Barristers at law",
        "69102" => "Solicitors",
        "69109" => "Activities of patent and copyright agents; other legal activities",
        "69201" => "Accounting and auditing activities",
        "69202" => "Bookkeeping activities",
        "69203" => "Tax consultancy",
        // Construction
        "41100" => "Development of building projects",
        "41201" => "Construction of commercial buildings",
        "41202" => "Construction of domestic buildings",
        "43120" => "Site preparation",
        "43290" => "Other construction installation",
        // Retail / wholesale
        "46900" => "Non-specialised wholesale trade",
        "47110" => "Retail sale in non-specialised stores with food, beverages or tobacco predominating",
        "47910" => "Retail sale via mail order houses or via Internet",
        // IT / tech
        "62011" => "Ready-made interactive leisure and entertainment software development",
        "62012" => "Business and domestic software development",
        "62020" => "Information technology consultancy activities",
        "62090" => "Other information technology service activities",
        "63110" => "Data processing, hosting and related activities",
        // Food / hospitality
        "56101" => "Licensed restaurants",
        "56102" => "Unlicensed restaurants and cafes",
        "56301" => "Licensed clubs",
        "55100" => "Hotels and similar accommodation",
        // Manufacturing
        "10000" | "10110" => "Processing and preserving of meat",
        "25000" => "Manufacture of fabricated metal products",
        // Other common
        "74909" => "Other professional, scientific and technical activities nec",
        "74990" => "Non-trading company",
        "82990" => "Other business support service activities nec",
        "85600" => "Educational support activities",
        "96090" => "Other service activities not elsewhere classified",
        _ => "",
    }
}
