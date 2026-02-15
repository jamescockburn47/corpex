use crate::ch_api::types::{CompanyProfile, Psc};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Relationship types between companies in the network.
#[derive(Debug, Clone)]
pub enum EdgeRelation {
    /// Corporate PSC ownership (parent → subsidiary)
    PscOwnership { natures: Vec<String> },
    /// Shared director between two companies
    SharedDirector { name: String },
    /// Parent company identified from filing text
    ParentFromFiling { source_filing: String },
    /// Subsidiary identified from filing text
    SubsidiaryFromFiling { source_filing: String },
    /// Charge holder relationship
    ChargeHolder { charge_desc: String },
}

impl EdgeRelation {
    pub fn label(&self) -> String {
        match self {
            Self::PscOwnership { natures } => {
                let ctrl: Vec<_> = natures
                    .iter()
                    .filter_map(|n| {
                        if n.contains("75") {
                            Some("75%+")
                        } else if n.contains("50") {
                            Some("50-75%")
                        } else if n.contains("25") {
                            Some("25-50%")
                        } else {
                            None
                        }
                    })
                    .collect();
                if ctrl.is_empty() {
                    "PSC ownership".to_string()
                } else {
                    format!("PSC ({})", ctrl.join(", "))
                }
            }
            Self::SharedDirector { name } => format!("Director: {}", name),
            Self::ParentFromFiling { .. } => "Parent (filing)".to_string(),
            Self::SubsidiaryFromFiling { .. } => "Subsidiary (filing)".to_string(),
            Self::ChargeHolder { charge_desc } => {
                format!("Charge: {}", truncate(charge_desc, 30))
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

/// Risk level for a company node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Per-company data stored at each node.
#[derive(Debug, Clone)]
pub struct CompanyNode {
    pub company_number: String,
    pub company_name: String,
    pub status: String,
    pub risk_level: RiskLevel,
    pub risk_signals: Vec<String>,
    pub sic_codes: Vec<String>,
    pub date_of_creation: Option<String>,
}

/// The corporate network graph.
pub struct CorporateNetwork {
    pub graph: DiGraph<CompanyNode, EdgeRelation>,
    /// Map from company number → node index
    pub node_map: HashMap<String, NodeIndex>,
}

impl CorporateNetwork {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add (or update) a company node from a profile.
    pub fn add_company(&mut self, company_number: &str, profile: &CompanyProfile) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(company_number) {
            // Update existing
            if let Some(node) = self.graph.node_weight_mut(idx) {
                node.company_name = profile.display_name();
                node.status = profile.company_status.clone().unwrap_or_default();
                node.sic_codes = profile.sic_codes.clone().unwrap_or_default();
                node.date_of_creation = profile.date_of_creation.clone();
                let (risk_level, signals) = assess_risk(profile);
                node.risk_level = risk_level;
                node.risk_signals = signals;
            }
            idx
        } else {
            let (risk_level, signals) = assess_risk(profile);
            let node = CompanyNode {
                company_number: company_number.to_string(),
                company_name: profile.display_name(),
                status: profile.company_status.clone().unwrap_or_default(),
                risk_level,
                risk_signals: signals,
                sic_codes: profile.sic_codes.clone().unwrap_or_default(),
                date_of_creation: profile.date_of_creation.clone(),
            };
            let idx = self.graph.add_node(node);
            self.node_map.insert(company_number.to_string(), idx);
            idx
        }
    }

    /// Add a placeholder node for a company we know the number of but haven't fetched yet.
    pub fn ensure_node(&mut self, company_number: &str, name: &str) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(company_number) {
            return idx;
        }
        let node = CompanyNode {
            company_number: company_number.to_string(),
            company_name: name.to_string(),
            status: "unknown".to_string(),
            risk_level: RiskLevel::Low,
            risk_signals: Vec::new(),
            sic_codes: Vec::new(),
            date_of_creation: None,
        };
        let idx = self.graph.add_node(node);
        self.node_map.insert(company_number.to_string(), idx);
        idx
    }

    /// Process PSC data and add ownership edges.
    pub fn process_pscs(&mut self, company_number: &str, pscs: &[Psc]) {
        let child_idx = self.ensure_node(company_number, company_number);

        for psc in pscs {
            if !psc.is_active() {
                continue;
            }
            if psc.is_corporate() {
                if let Some(reg_num) = psc.registration_number() {
                    let parent_name = psc.name.as_deref().unwrap_or(reg_num);
                    let parent_idx = self.ensure_node(reg_num, parent_name);
                    let relation = EdgeRelation::PscOwnership {
                        natures: psc.natures_of_control.clone().unwrap_or_default(),
                    };
                    // Add edge parent → child (ownership direction)
                    self.graph.add_edge(parent_idx, child_idx, relation);
                }
            }
        }
    }

    /// Add a shared director edge between two companies.
    pub fn add_shared_director(
        &mut self,
        company_a: &str,
        company_b: &str,
        director_name: &str,
    ) {
        let idx_a = self.ensure_node(company_a, company_a);
        let idx_b = self.ensure_node(company_b, company_b);
        self.graph.add_edge(
            idx_a,
            idx_b,
            EdgeRelation::SharedDirector {
                name: director_name.to_string(),
            },
        );
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn get_node(&self, company_number: &str) -> Option<&CompanyNode> {
        self.node_map
            .get(company_number)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    /// Get all company numbers in the network.
    pub fn all_companies(&self) -> Vec<String> {
        self.graph
            .node_weights()
            .map(|n| n.company_number.clone())
            .collect()
    }
}

/// Assess risk level from a company profile.
fn assess_risk(profile: &CompanyProfile) -> (RiskLevel, Vec<String>) {
    let mut signals = Vec::new();

    if profile.accounts_overdue() {
        signals.push("Accounts overdue".to_string());
    }
    if profile.confirmation_overdue() {
        signals.push("Confirmation statement overdue".to_string());
    }
    if profile.has_insolvency_history == Some(true) {
        signals.push("Insolvency history".to_string());
    }
    if profile.has_been_liquidated == Some(true) {
        signals.push("Has been liquidated".to_string());
    }
    if !profile.is_active() {
        let status = profile.company_status.as_deref().unwrap_or("inactive");
        signals.push(format!("Status: {}", status));
    }
    if profile.has_charges == Some(true) {
        // Not a risk itself but noteworthy
    }

    let level = if signals.iter().any(|s| {
        s.contains("Insolvency")
            || s.contains("liquidated")
            || s.contains("Accounts overdue")
    }) {
        RiskLevel::High
    } else if !signals.is_empty() {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    (level, signals)
}
