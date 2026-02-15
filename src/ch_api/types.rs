use serde::Deserialize;

// ── Company Profile ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CompanyProfile {
    pub company_name: Option<String>,
    pub company_number: Option<String>,
    pub company_status: Option<String>,
    pub company_status_detail: Option<String>,
    #[serde(rename = "type")]
    pub company_type: Option<String>,
    pub date_of_creation: Option<String>,
    pub date_of_cessation: Option<String>,
    pub jurisdiction: Option<String>,
    pub registered_office_address: Option<Address>,
    pub sic_codes: Option<Vec<String>>,
    pub accounts: Option<AccountsInfo>,
    pub confirmation_statement: Option<ConfirmationStatementInfo>,
    pub has_charges: Option<bool>,
    pub has_insolvency_history: Option<bool>,
    pub has_been_liquidated: Option<bool>,
}

impl CompanyProfile {
    pub fn display_name(&self) -> String {
        self.company_name
            .clone()
            .unwrap_or_else(|| self.company_number.clone().unwrap_or("Unknown".into()))
    }
    pub fn number(&self) -> String {
        self.company_number.clone().unwrap_or_default()
    }
    pub fn is_active(&self) -> bool {
        self.company_status.as_deref() == Some("active")
    }
    pub fn accounts_overdue(&self) -> bool {
        self.accounts.as_ref().and_then(|a| a.overdue).unwrap_or(false)
    }
    pub fn confirmation_overdue(&self) -> bool {
        self.confirmation_statement.as_ref().and_then(|c| c.overdue).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Address {
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub premises: Option<String>,
}

impl Address {
    pub fn one_line(&self) -> String {
        [
            self.premises.as_deref(),
            self.address_line_1.as_deref(),
            self.address_line_2.as_deref(),
            self.locality.as_deref(),
            self.region.as_deref(),
            self.postal_code.as_deref(),
            self.country.as_deref(),
        ]
        .iter()
        .filter_map(|s| *s)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountsInfo {
    pub accounting_reference_date: Option<AccountingRefDate>,
    pub last_accounts: Option<LastAccounts>,
    pub next_due: Option<String>,
    pub next_made_up_to: Option<String>,
    pub overdue: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountingRefDate {
    pub day: Option<String>,
    pub month: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LastAccounts {
    pub made_up_to: Option<String>,
    #[serde(rename = "type")]
    pub accounts_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmationStatementInfo {
    pub last_made_up_to: Option<String>,
    pub next_due: Option<String>,
    pub next_made_up_to: Option<String>,
    pub overdue: Option<bool>,
}

// ── Company Search ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CompanySearchResponse {
    pub items: Option<Vec<CompanySearchResult>>,
    pub total_results: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompanySearchResult {
    pub title: Option<String>,
    pub company_number: Option<String>,
    pub company_status: Option<String>,
    #[serde(rename = "company_type")]
    pub company_type: Option<String>,
    pub date_of_creation: Option<String>,
    pub address_snippet: Option<String>,
    pub description: Option<String>,
}

// ── Officers ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerListResponse {
    pub items: Option<Vec<Officer>>,
    pub total_results: Option<u32>,
    pub active_count: Option<u32>,
    pub resigned_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Officer {
    pub name: Option<String>,
    pub officer_role: Option<String>,
    pub appointed_on: Option<String>,
    pub resigned_on: Option<String>,
    pub date_of_birth: Option<DateOfBirth>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub address: Option<Address>,
    pub links: Option<OfficerLinks>,
}

impl Officer {
    pub fn is_active(&self) -> bool {
        self.resigned_on.is_none()
    }
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or("Unknown".into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DateOfBirth {
    pub month: Option<u32>,
    pub year: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerLinks {
    #[serde(rename = "self")]
    pub self_link: Option<String>,
    pub officer: Option<OfficerAppointmentsLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerAppointmentsLink {
    pub appointments: Option<String>,
}

// ── Officer Search ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerSearchResponse {
    pub items: Option<Vec<OfficerSearchResult>>,
    pub total_results: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerSearchResult {
    pub title: Option<String>,
    pub description: Option<String>,
    pub appointment_count: Option<u32>,
    pub date_of_birth: Option<DateOfBirth>,
    pub address: Option<Address>,
    pub links: Option<OfficerSearchLinks>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerSearchLinks {
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

// ── Officer Appointments ────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerAppointmentsResponse {
    pub items: Option<Vec<OfficerAppointment>>,
    pub name: Option<String>,
    pub date_of_birth: Option<DateOfBirth>,
    pub active_count: Option<u32>,
    pub inactive_count: Option<u32>,
    pub resigned_count: Option<u32>,
    pub is_corporate_officer: Option<bool>,
    pub total_results: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfficerAppointment {
    pub name: Option<String>,
    pub officer_role: Option<String>,
    pub appointed_on: Option<String>,
    pub resigned_on: Option<String>,
    pub nationality: Option<String>,
    pub country_of_residence: Option<String>,
    pub occupation: Option<String>,
    pub address: Option<Address>,
    pub appointed_to: Option<AppointedTo>,
    pub name_elements: Option<NameElements>,
    pub is_pre_1992_appointment: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppointedTo {
    pub company_name: Option<String>,
    pub company_number: Option<String>,
    pub company_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NameElements {
    pub title: Option<String>,
    pub forename: Option<String>,
    pub surname: Option<String>,
    pub other_forenames: Option<String>,
}

// ── Persons with Significant Control ─────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct PscListResponse {
    pub items: Option<Vec<Psc>>,
    pub total_results: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Psc {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub natures_of_control: Option<Vec<String>>,
    pub notified_on: Option<String>,
    pub ceased_on: Option<String>,
    pub nationality: Option<String>,
    pub country_of_residence: Option<String>,
    pub address: Option<Address>,
    pub identification: Option<PscIdentification>,
}

impl Psc {
    pub fn is_corporate(&self) -> bool {
        self.kind.as_deref() == Some("corporate-entity-person-with-significant-control")
    }
    pub fn registration_number(&self) -> Option<&str> {
        self.identification
            .as_ref()
            .and_then(|id| id.registration_number.as_deref())
    }
    pub fn is_active(&self) -> bool {
        self.ceased_on.is_none()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PscIdentification {
    pub registration_number: Option<String>,
    pub legal_authority: Option<String>,
    pub legal_form: Option<String>,
    pub place_registered: Option<String>,
    pub country_registered: Option<String>,
}

// ── Charges ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ChargesListResponse {
    pub items: Option<Vec<Charge>>,
    pub total_count: Option<u32>,
    pub unfiltered_count: Option<u32>,
    pub part_satisfied_count: Option<u32>,
    pub satisfied_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Charge {
    pub charge_number: Option<u32>,
    pub status: Option<String>,
    pub created_on: Option<String>,
    pub delivered_on: Option<String>,
    pub satisfied_on: Option<String>,
    pub classification: Option<ChargeClassification>,
    pub particulars: Option<ChargeParticulars>,
    pub persons_entitled: Option<Vec<PersonEntitled>>,
    pub secured_details: Option<SecuredDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChargeClassification {
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub classification_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChargeParticulars {
    pub description: Option<String>,
    pub contains_negative_pledge: Option<bool>,
    pub contains_floating_charge: Option<bool>,
    pub floating_charge_covers_all: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonEntitled {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecuredDetails {
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub secured_type: Option<String>,
}

// ── Insolvency ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct InsolvencyResponse {
    pub status: Option<Vec<String>>,
    pub cases: Option<Vec<InsolvencyCase>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InsolvencyData {
    pub status: Vec<String>,
    pub cases: Vec<InsolvencyCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InsolvencyCase {
    #[serde(rename = "type")]
    pub case_type: Option<String>,
    pub number: Option<u32>,
    pub dates: Option<Vec<InsolvencyDate>>,
    pub practitioners: Option<Vec<InsolvencyPractitioner>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InsolvencyDate {
    #[serde(rename = "type")]
    pub date_type: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InsolvencyPractitioner {
    pub name: Option<String>,
    pub address: Option<Address>,
    pub role: Option<String>,
    pub appointed_on: Option<String>,
    pub ceased_to_act_on: Option<String>,
}

// ── Filing History ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct FilingHistoryResponse {
    pub items: Option<Vec<FilingHistoryItem>>,
    pub total_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilingHistoryItem {
    pub transaction_id: Option<String>,
    pub category: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub filing_type: Option<String>,
    pub links: Option<FilingLinks>,
    pub description_values: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilingLinks {
    #[serde(rename = "self")]
    pub self_link: Option<String>,
    pub document_metadata: Option<String>,
}
