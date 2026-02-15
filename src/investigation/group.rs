use crate::ch_api::types::{CompanyProfile, Psc};

/// Role of a company within a corporate group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupRole {
    /// Has no corporate PSC above it (could be ultimate parent or standalone)
    Parent,
    /// Has a corporate PSC — this is a subsidiary
    Subsidiary,
    /// No PSC data or cannot determine
    Unknown,
}

/// Info about a parent company discovered from PSC data.
#[derive(Debug, Clone)]
pub struct ParentInfo {
    pub company_number: String,
    pub name: String,
}

/// Info about a discovered subsidiary.
#[derive(Debug, Clone)]
pub struct SubsidiaryInfo {
    pub company_number: String,
    pub company_name: String,
    pub status: String,
}

/// Group structure information for a company.
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub role: GroupRole,
    pub parent: Option<ParentInfo>,
    pub subsidiaries: Vec<SubsidiaryInfo>,
    pub has_consolidated_accounts: bool,
}

impl GroupInfo {
    pub fn unknown() -> Self {
        Self {
            role: GroupRole::Unknown,
            parent: None,
            subsidiaries: Vec::new(),
            has_consolidated_accounts: false,
        }
    }
}

/// Detect group structure from a company's PSC data.
///
/// If there is a corporate PSC, this company is a subsidiary.
/// If there are no corporate PSCs, it could be a parent or standalone.
pub fn detect_group_role(pscs: &[Psc], _profile: &CompanyProfile) -> GroupInfo {
    let mut info = GroupInfo::unknown();

    // Look for corporate PSCs — these indicate a parent company
    for psc in pscs {
        if !psc.is_active() {
            continue;
        }
        if psc.is_corporate() {
            if let Some(reg_num) = psc.registration_number() {
                info.role = GroupRole::Subsidiary;
                info.parent = Some(ParentInfo {
                    company_number: reg_num.to_string(),
                    name: psc.name.clone().unwrap_or_else(|| reg_num.to_string()),
                });
                break; // Take the first active corporate PSC as the parent
            }
        }
    }

    // If no corporate PSC was found, this could be a parent or standalone
    if info.role == GroupRole::Unknown {
        info.role = GroupRole::Parent; // Assume parent for now; subsidiaries discovered later
    }

    info
}

/// Check if a company's filings suggest it files consolidated (group) accounts.
/// This is detected from filing descriptions containing "group" keywords.
pub fn has_consolidated_filings(filings: &[crate::ch_api::types::FilingHistoryItem]) -> bool {
    filings.iter().any(|f| {
        if let Some(desc) = &f.description {
            let d = desc.to_lowercase();
            d.contains("group") || d.contains("consolidated")
        } else {
            false
        }
    })
}
