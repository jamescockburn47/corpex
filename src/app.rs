use crate::config::{AppConfig, AiProviderConfig, OcrMode};
use crate::ch_api;
use crate::investigation::network::CorporateNetwork;
use crate::ui;

use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;

/// Search mode toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Company,
    Officer,
}

/// Messages sent from background tasks to the UI thread.
#[derive(Debug)]
pub enum BackgroundMessage {
    CompanyProfileLoaded {
        company_number: String,
        profile: ch_api::types::CompanyProfile,
    },
    SearchResults(Vec<ch_api::types::CompanySearchResult>),
    OfficerSearchResults(Vec<ch_api::types::OfficerSearchResult>),
    OfficerAppointmentsLoaded {
        officer_name: String,
        response: ch_api::types::OfficerAppointmentsResponse,
    },
    OfficersLoaded {
        company_number: String,
        officers: Vec<ch_api::types::Officer>,
    },
    PscsLoaded {
        company_number: String,
        pscs: Vec<ch_api::types::Psc>,
    },
    ChargesLoaded {
        company_number: String,
        charges: Vec<ch_api::types::Charge>,
    },
    InsolvencyLoaded {
        company_number: String,
        insolvency: Option<ch_api::types::InsolvencyData>,
    },
    FilingsLoaded {
        company_number: String,
        filings: Vec<ch_api::types::FilingHistoryItem>,
    },
    DocumentTextExtracted {
        company_number: String,
        filing_id: String,
        text: String,
    },
    AiAnalysisComplete {
        company_number: String,
        analysis: String,
        input_tokens: u32,
        output_tokens: u32,
        model: String,
    },
    AiChatResponse {
        company_number: String,
        response: String,
        input_tokens: u32,
        output_tokens: u32,
        model: String,
    },
    AiFilingSummary {
        filing_id: String,
        summary: String,
    },
    GroupSubsidiariesDiscovered {
        parent_number: String,
        subsidiaries: Vec<crate::investigation::group::SubsidiaryInfo>,
        has_consolidated: bool,
    },
    NetworkTraversalUpdate {
        message: String,
    },
    Error(String),
    StatusUpdate(String),
}

/// Top-level application state.
pub struct InvestigationApp {
    pub config: AppConfig,

    // UI state
    pub search_query: String,
    pub search_mode: SearchMode,
    pub search_results: Vec<ch_api::types::CompanySearchResult>,
    pub active_view: ui::View,
    pub sidebar_collapsed: bool,
    pub status_message: String,
    pub show_settings: bool,

    // Navigation history (for back button)
    pub view_history: Vec<(ui::View, Option<String>)>,
    pub scroll_reset_needed: bool,

    // Officer search state
    pub officer_search_results: Vec<ch_api::types::OfficerSearchResult>,
    pub selected_officer_name: Option<String>,
    pub selected_officer_appointments: Option<ch_api::types::OfficerAppointmentsResponse>,

    // Settings panel input fields
    pub settings_ai_provider: String,
    pub settings_ai_key: String,
    pub settings_ai_model: String,
    pub settings_ai_base_url: String,
    pub settings_kanon2_key: String,

    // Investigation data
    pub network: CorporateNetwork,
    pub selected_company: Option<String>,
    pub company_profiles: HashMap<String, ch_api::types::CompanyProfile>,
    pub company_officers: HashMap<String, Vec<ch_api::types::Officer>>,
    pub company_pscs: HashMap<String, Vec<ch_api::types::Psc>>,
    pub company_charges: HashMap<String, Vec<ch_api::types::Charge>>,
    pub company_insolvency: HashMap<String, Option<ch_api::types::InsolvencyData>>,
    pub company_filings: HashMap<String, Vec<ch_api::types::FilingHistoryItem>>,
    pub extracted_texts: HashMap<String, String>, // filing_id -> text
    pub ai_analyses: HashMap<String, String>,     // company_number -> analysis

    // Group structure
    pub group_info: HashMap<String, crate::investigation::group::GroupInfo>,

    // AI chat state
    pub ai_conversations: HashMap<String, Vec<(String, String)>>, // company -> [(role, msg)]
    pub ai_chat_input: String,
    pub filing_summaries: HashMap<String, String>, // filing_id -> AI summary

    // Analysis year range
    pub analysis_year_from: i32,
    pub analysis_year_to: i32,

    // Token/cost tracking
    pub session_input_tokens: u32,
    pub session_output_tokens: u32,
    pub session_cost_usd: f64,
    pub last_query_input_tokens: u32,
    pub last_query_output_tokens: u32,
    pub last_query_cost_usd: f64,

    // Background task communication
    pub bg_sender: Sender<BackgroundMessage>,
    pub bg_receiver: Receiver<BackgroundMessage>,

    // Tokio runtime handle
    pub runtime: tokio::runtime::Runtime,

    // Loading flags
    pub is_loading: bool,

    // Export/save dialog state
    pub show_save_dialog: bool,
    pub save_project_name: String,
    pub save_status_message: Option<String>,

    // Document viewer popup
    pub viewing_filing_id: Option<String>,
}

impl InvestigationApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load_from_env();
        let (tx, rx) = crossbeam_channel::unbounded();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
            config,
            search_query: String::new(),
            search_mode: SearchMode::Company,
            search_results: Vec::new(),
            active_view: ui::View::Welcome,
            sidebar_collapsed: false,
            status_message: "Ready. Enter a company name or number to begin.".to_string(),
            show_settings: false,
            view_history: Vec::new(),
            scroll_reset_needed: false,
            officer_search_results: Vec::new(),
            selected_officer_name: None,
            selected_officer_appointments: None,
            settings_ai_provider: "anthropic".to_string(),
            settings_ai_key: String::new(),
            settings_ai_model: "claude-haiku-4-5".to_string(),
            settings_ai_base_url: String::new(),
            settings_kanon2_key: String::new(),
            network: CorporateNetwork::new(),
            selected_company: None,
            company_profiles: HashMap::new(),
            company_officers: HashMap::new(),
            company_pscs: HashMap::new(),
            company_charges: HashMap::new(),
            company_insolvency: HashMap::new(),
            company_filings: HashMap::new(),
            extracted_texts: crate::cache::load_all_texts(),
            ai_analyses: HashMap::new(),
            group_info: HashMap::new(),
            ai_conversations: HashMap::new(),
            ai_chat_input: String::new(),
            filing_summaries: HashMap::new(),
            analysis_year_from: 2020,
            analysis_year_to: 2026,
            session_input_tokens: 0,
            session_output_tokens: 0,
            session_cost_usd: 0.0,
            last_query_input_tokens: 0,
            last_query_output_tokens: 0,
            last_query_cost_usd: 0.0,
            bg_sender: tx,
            bg_receiver: rx,
            runtime,
            is_loading: false,
            show_save_dialog: false,
            save_project_name: String::new(),
            save_status_message: None,
            viewing_filing_id: None,
        }
    }

    /// Process any pending messages from background tasks.
    pub fn poll_background(&mut self) {
        while let Ok(msg) = self.bg_receiver.try_recv() {
            match msg {
                BackgroundMessage::CompanyProfileLoaded { company_number, profile } => {
                    tracing::info!("Profile loaded for {}", company_number);
                    self.network.add_company(&company_number, &profile);
                    self.company_profiles.insert(company_number.clone(), profile);
                    // Always update selected company to the most recently loaded one
                    self.selected_company = Some(company_number);
                    self.is_loading = false;
                }
                BackgroundMessage::SearchResults(results) => {
                    self.search_results = results;
                    self.is_loading = false;
                }
                BackgroundMessage::OfficerSearchResults(results) => {
                    self.officer_search_results = results;
                    self.is_loading = false;
                }
                BackgroundMessage::OfficerAppointmentsLoaded { officer_name, response } => {
                    self.selected_officer_name = Some(officer_name);
                    self.selected_officer_appointments = Some(response);
                    self.is_loading = false;
                }
                BackgroundMessage::OfficersLoaded { company_number, officers } => {
                    self.company_officers.insert(company_number, officers);
                }
                BackgroundMessage::PscsLoaded { company_number, pscs } => {
                    self.network.process_pscs(&company_number, &pscs);
                    // Auto-detect group structure
                    if let Some(profile) = self.company_profiles.get(&company_number) {
                        let group = crate::investigation::group::detect_group_role(&pscs, profile);
                        self.group_info.insert(company_number.clone(), group);
                    }
                    self.company_pscs.insert(company_number, pscs);
                }
                BackgroundMessage::ChargesLoaded { company_number, charges } => {
                    self.company_charges.insert(company_number, charges);
                }
                BackgroundMessage::InsolvencyLoaded { company_number, insolvency } => {
                    self.company_insolvency.insert(company_number, insolvency);
                }
                BackgroundMessage::FilingsLoaded { company_number, filings } => {
                    self.company_filings.insert(company_number, filings);
                }
                BackgroundMessage::DocumentTextExtracted { company_number, filing_id, text } => {
                    // Save to disk cache for persistence across sessions
                    if let Err(e) = crate::cache::save_text(&company_number, &filing_id, &text) {
                        tracing::warn!("Failed to cache filing {}: {}", filing_id, e);
                    }
                    self.extracted_texts.insert(filing_id, text);
                }
                BackgroundMessage::AiAnalysisComplete { company_number, analysis, input_tokens, output_tokens, model } => {
                    let cost = crate::ai::provider::TokenUsage { input_tokens, output_tokens }.cost_usd(&model);
                    self.last_query_input_tokens = input_tokens;
                    self.last_query_output_tokens = output_tokens;
                    self.last_query_cost_usd = cost;
                    self.session_input_tokens += input_tokens;
                    self.session_output_tokens += output_tokens;
                    self.session_cost_usd += cost;
                    self.status_message = format!(
                        "✓ AI analysis complete | {}in + {}out tokens | ${:.4} (session: ${:.4})",
                        input_tokens, output_tokens, cost, self.session_cost_usd
                    );
                    self.ai_analyses.insert(company_number.clone(), analysis.clone());
                    self.ai_conversations
                        .entry(company_number)
                        .or_default()
                        .push(("assistant".to_string(), analysis));
                    self.is_loading = false;
                }
                BackgroundMessage::AiChatResponse { company_number, response, input_tokens, output_tokens, model } => {
                    let cost = crate::ai::provider::TokenUsage { input_tokens, output_tokens }.cost_usd(&model);
                    self.last_query_input_tokens = input_tokens;
                    self.last_query_output_tokens = output_tokens;
                    self.last_query_cost_usd = cost;
                    self.session_input_tokens += input_tokens;
                    self.session_output_tokens += output_tokens;
                    self.session_cost_usd += cost;
                    self.status_message = format!(
                        "✓ AI response | {}in + {}out tokens | ${:.4} (session: ${:.4})",
                        input_tokens, output_tokens, cost, self.session_cost_usd
                    );
                    self.ai_conversations
                        .entry(company_number)
                        .or_default()
                        .push(("assistant".to_string(), response));
                    self.is_loading = false;
                }
                BackgroundMessage::AiFilingSummary { filing_id, summary } => {
                    self.filing_summaries.insert(filing_id, summary);
                    self.is_loading = false;
                }
                BackgroundMessage::GroupSubsidiariesDiscovered { parent_number, subsidiaries, has_consolidated } => {
                    if let Some(gi) = self.group_info.get_mut(&parent_number) {
                        gi.subsidiaries = subsidiaries.clone();
                        gi.has_consolidated_accounts = has_consolidated;
                    }
                    // Also update any subsidiary that points to this parent
                    for (cn, gi) in self.group_info.iter_mut() {
                        if let Some(p) = &gi.parent {
                            if p.company_number == parent_number {
                                gi.subsidiaries = subsidiaries.clone();
                                gi.has_consolidated_accounts = has_consolidated;
                            }
                        }
                    }
                    self.is_loading = false;
                }
                BackgroundMessage::NetworkTraversalUpdate { message } => {
                    self.status_message = message;
                }
                BackgroundMessage::Error(e) => {
                    self.status_message = format!("⚠ Error: {}", e);
                    self.is_loading = false;
                }
                BackgroundMessage::StatusUpdate(msg) => {
                    self.status_message = msg;
                }
            }
        }
    }

    /// Push current view onto history before navigating.
    pub fn push_view(&mut self, new_view: ui::View) {
        self.view_history.push((self.active_view, self.selected_company.clone()));
        self.active_view = new_view;
        self.scroll_reset_needed = true;
    }

    /// Pop back to the previous view.
    pub fn pop_view(&mut self) {
        if let Some((view, company)) = self.view_history.pop() {
            self.active_view = view;
            self.selected_company = company;
        }
    }

    /// Kick off a company search in the background.
    pub fn search_companies(&self, query: String) {
        let tx = self.bg_sender.clone();
        let api_key = self.config.ch_api_key.clone().unwrap_or_default();
        self.runtime.spawn(async move {
            match ch_api::client::search_companies(&api_key, &query).await {
                Ok(results) => { let _ = tx.send(BackgroundMessage::SearchResults(results)); }
                Err(e) => { let _ = tx.send(BackgroundMessage::Error(format!("Search failed: {}", e))); }
            }
        });
    }

    /// Kick off an officer search in the background.
    pub fn search_officers(&self, query: String) {
        let tx = self.bg_sender.clone();
        let api_key = self.config.ch_api_key.clone().unwrap_or_default();
        self.runtime.spawn(async move {
            match ch_api::client::search_officers(&api_key, &query).await {
                Ok(results) => { let _ = tx.send(BackgroundMessage::OfficerSearchResults(results)); }
                Err(e) => { let _ = tx.send(BackgroundMessage::Error(format!("Officer search failed: {}", e))); }
            }
        });
    }

    /// Fetch all appointments for a specific officer.
    pub fn fetch_officer_appointments(&self, officer_name: String, appointments_path: String) {
        let tx = self.bg_sender.clone();
        let api_key = self.config.ch_api_key.clone().unwrap_or_default();
        self.runtime.spawn(async move {
            match ch_api::client::get_officer_appointments(&api_key, &appointments_path).await {
                Ok(response) => { let _ = tx.send(BackgroundMessage::OfficerAppointmentsLoaded { officer_name, response }); }
                Err(e) => { let _ = tx.send(BackgroundMessage::Error(format!("Appointments fetch failed: {}", e))); }
            }
        });
    }

    /// Kick off fetching a company profile + related data.
    pub fn investigate_company(&self, company_number: String) {
        let tx = self.bg_sender.clone();
        let api_key = self.config.ch_api_key.clone().unwrap_or_default();
        let cn = company_number.clone();

        self.runtime.spawn(async move {
            let _ = tx.send(BackgroundMessage::StatusUpdate(format!("Fetching profile for {}...", cn)));

            // Profile
            match ch_api::client::get_company_profile(&api_key, &cn).await {
                Ok(profile) => {
                    let _ = tx.send(BackgroundMessage::CompanyProfileLoaded {
                        company_number: cn.clone(),
                        profile,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::Error(format!("Profile fetch failed for {}: {}", cn, e)));
                    return;
                }
            }

            // Officers
            let _ = tx.send(BackgroundMessage::StatusUpdate(format!("Fetching officers for {}...", cn)));
            if let Ok(officers) = ch_api::client::get_officers(&api_key, &cn).await {
                let _ = tx.send(BackgroundMessage::OfficersLoaded {
                    company_number: cn.clone(),
                    officers,
                });
            }

            // PSCs
            let _ = tx.send(BackgroundMessage::StatusUpdate(format!("Fetching PSCs for {}...", cn)));
            if let Ok(pscs) = ch_api::client::get_pscs(&api_key, &cn).await {
                let _ = tx.send(BackgroundMessage::PscsLoaded {
                    company_number: cn.clone(),
                    pscs,
                });
            }

            // Charges
            let _ = tx.send(BackgroundMessage::StatusUpdate(format!("Fetching charges for {}...", cn)));
            if let Ok(charges) = ch_api::client::get_charges(&api_key, &cn).await {
                let _ = tx.send(BackgroundMessage::ChargesLoaded {
                    company_number: cn.clone(),
                    charges,
                });
            }

            // Insolvency
            if let Ok(insolvency) = ch_api::client::get_insolvency(&api_key, &cn).await {
                let _ = tx.send(BackgroundMessage::InsolvencyLoaded {
                    company_number: cn.clone(),
                    insolvency,
                });
            }

            // Filings
            let _ = tx.send(BackgroundMessage::StatusUpdate(format!("Fetching filings for {}...", cn)));
            if let Ok(filings) = ch_api::client::get_filing_history(&api_key, &cn, None, Some(25)).await {
                // Auto-extract accounts filings for financial analysis
                let accounts_to_extract: Vec<(String, String, String)> = filings.iter()
                    .filter(|f| {
                        // Only accounts-category filings
                        f.category.as_deref() == Some("accounts")
                    })
                    .filter(|f| {
                        // Must have both transaction_id and document_metadata link
                        f.transaction_id.is_some()
                            && f.links.as_ref().and_then(|l| l.document_metadata.as_ref()).is_some()
                    })
                    .filter(|f| {
                        // Skip already cached
                        let tid = f.transaction_id.as_ref().unwrap();
                        !crate::cache::is_cached(&cn, tid)
                    })
                    .take(10) // Extract up to 10 most recent accounts (covers ~10 years)
                    .map(|f| {
                        let tid = f.transaction_id.clone().unwrap();
                        let doc_url = f.links.as_ref().unwrap().document_metadata.clone().unwrap();
                        let desc = f.description.as_deref().unwrap_or("accounts").to_string();
                        (tid, doc_url, desc)
                    })
                    .collect();

                let _ = tx.send(BackgroundMessage::FilingsLoaded {
                    company_number: cn.clone(),
                    filings,
                });

                // Now auto-extract the accounts
                if !accounts_to_extract.is_empty() {
                    let _ = tx.send(BackgroundMessage::StatusUpdate(
                        format!("Auto-extracting {} accounts filings...", accounts_to_extract.len()),
                    ));

                    for (filing_id, doc_meta_url, desc) in accounts_to_extract {
                        let _ = tx.send(BackgroundMessage::StatusUpdate(
                            format!("Downloading & extracting {}...", filing_id),
                        ));

                        match ch_api::client::download_document(&api_key, &doc_meta_url).await {
                            Ok(doc_content) => {
                                let result = crate::extraction::extract_text(&doc_content);
                                tracing::info!(
                                    "Auto-extracted {} chars from accounts filing {} via {:?}",
                                    result.text.len(),
                                    filing_id,
                                    result.method
                                );
                                let _ = tx.send(BackgroundMessage::DocumentTextExtracted {
                                    company_number: cn.clone(),
                                    filing_id,
                                    text: result.text,
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Failed to auto-extract {}: {}", filing_id, e);
                                // Surface the failure so the AI and user see it
                                let _ = tx.send(BackgroundMessage::DocumentTextExtracted {
                                    company_number: cn.clone(),
                                    filing_id: filing_id.clone(),
                                    text: format!("[EXTRACTION FAILED: {} — {}]", desc, e),
                                });
                            }
                        }
                    }
                    let _ = tx.send(BackgroundMessage::StatusUpdate(
                        "✓ Accounts auto-extracted".to_string(),
                    ));
                }
            }

            let _ = tx.send(BackgroundMessage::StatusUpdate(format!("✓ Investigation data loaded for {}", cn)));
        });
    }

    /// Extract text from a filing document.
    /// Downloads the document and runs it through the extraction pipeline.
    pub fn extract_filing_text(&self, company_number: String, filing_id: String, document_metadata_url: String) {
        let tx = self.bg_sender.clone();
        let api_key = self.config.ch_api_key.clone().unwrap_or_default();

        self.runtime.spawn(async move {
            let _ = tx.send(BackgroundMessage::StatusUpdate(
                format!("Downloading document for {}...", filing_id),
            ));

            match ch_api::client::download_document(&api_key, &document_metadata_url).await {
                Ok(doc_content) => {
                    let result = crate::extraction::extract_text(&doc_content);
                    let method_label = format!("{:?}", result.method);
                    tracing::info!(
                        "Extracted {} chars from filing {} via {:?}",
                        result.text.len(),
                        filing_id,
                        result.method
                    );
                    let _ = tx.send(BackgroundMessage::DocumentTextExtracted {
                        company_number,
                        filing_id,
                        text: result.text,
                    });
                    let _ = tx.send(BackgroundMessage::StatusUpdate(
                        format!("✓ Text extracted ({})", method_label),
                    ));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::DocumentTextExtracted {
                        company_number,
                        filing_id: filing_id.clone(),
                        text: format!("[Download failed: {}]", e),
                    });
                    let _ = tx.send(BackgroundMessage::StatusUpdate(
                        format!("⚠ Document download failed for {}: {}", filing_id, e),
                    ));
                }
            }
        });
    }

    /// Run AI analysis for a company — builds context from all available data and sends to AI.
    pub fn run_ai_analysis(&mut self, company_number: String) {
        let tx = self.bg_sender.clone();
        let ai_config = self.config.ai_provider.clone();
        let cn = company_number.clone();

        // Build the context from all available data
        let context = crate::ai::prompts::build_company_context(
            &cn,
            self.company_profiles.get(&cn),
            self.company_officers.get(&cn),
            self.company_pscs.get(&cn),
            self.company_charges.get(&cn),
            self.company_insolvency.get(&cn),
            self.company_filings.get(&cn),
            &self.extracted_texts,
        );

        let user_prompt = crate::ai::prompts::build_analysis_prompt(
            &context,
            self.analysis_year_from,
            self.analysis_year_to,
        );

        // Store the user message in conversation history
        let year_label = if self.analysis_year_from == self.analysis_year_to {
            format!("Run full company analysis ({})", self.analysis_year_from)
        } else {
            format!("Run full company analysis ({}-{})", self.analysis_year_from, self.analysis_year_to)
        };
        self.ai_conversations
            .entry(cn.clone())
            .or_default()
            .push(("user".to_string(), year_label));

        self.is_loading = true;
        self.status_message = format!("Running AI analysis for {}...", cn);

        self.runtime.spawn(async move {
            let _ = tx.send(BackgroundMessage::StatusUpdate(
                format!("Sending data to AI for {}...", cn),
            ));

            let messages = vec![crate::ai::ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            }];

            match crate::ai::provider::chat_completion(
                &ai_config,
                crate::ai::prompts::SYSTEM_PROMPT,
                &messages,
            ).await {
                Ok(result) => {
                    let _ = tx.send(BackgroundMessage::AiAnalysisComplete {
                        company_number: cn.clone(),
                        analysis: result.text,
                        input_tokens: result.usage.input_tokens,
                        output_tokens: result.usage.output_tokens,
                        model: result.model,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::Error(
                        format!("AI analysis failed: {}", e),
                    ));
                }
            }
        });
    }

    /// Send a follow-up chat message in the context of a company's analysis.
    pub fn send_ai_chat(&mut self, company_number: String, user_message: String) {
        let tx = self.bg_sender.clone();
        let ai_config = self.config.ai_provider.clone();
        let cn = company_number.clone();

        // Add user message to conversation history
        self.ai_conversations
            .entry(cn.clone())
            .or_default()
            .push(("user".to_string(), user_message.clone()));

        // Build the full message history for the API call
        let conversation = self.ai_conversations.get(&cn).cloned().unwrap_or_default();

        // Include company context in the conversation
        let context = crate::ai::prompts::build_company_context(
            &cn,
            self.company_profiles.get(&cn),
            self.company_officers.get(&cn),
            self.company_pscs.get(&cn),
            self.company_charges.get(&cn),
            self.company_insolvency.get(&cn),
            self.company_filings.get(&cn),
            &self.extracted_texts,
        );

        self.is_loading = true;
        self.status_message = "Sending message to AI...".to_string();

        self.runtime.spawn(async move {
            // Build message list: context as first user message, then conversation history
            let mut messages = vec![crate::ai::ChatMessage {
                role: "user".to_string(),
                content: format!("Here is the company data for context:\n\n{}", context),
            }];

            // Add all conversation messages (skip the "Run full analysis" initial placeholder)
            for (role, content) in &conversation {
                messages.push(crate::ai::ChatMessage {
                    role: role.clone(),
                    content: content.clone(),
                });
            }

            match crate::ai::provider::chat_completion(
                &ai_config,
                crate::ai::prompts::SYSTEM_PROMPT,
                &messages,
            ).await {
                Ok(result) => {
                    let _ = tx.send(BackgroundMessage::AiChatResponse {
                        company_number: cn,
                        response: result.text,
                        input_tokens: result.usage.input_tokens,
                        output_tokens: result.usage.output_tokens,
                        model: result.model,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::Error(
                        format!("AI chat failed: {}", e),
                    ));
                }
            }
        });
    }

    /// Discover group structure for a company.
    /// If the company is a subsidiary, fetch the parent's PSCs to find siblings.
    /// If the company might be a parent, search for companies where it's listed as PSC.
    pub fn discover_group(&mut self, company_number: String) {
        let tx = self.bg_sender.clone();
        let api_key = self.config.ch_api_key.clone().unwrap_or_default();

        // Determine the parent company number to investigate
        let parent_number = if let Some(gi) = self.group_info.get(&company_number) {
            if let Some(p) = &gi.parent {
                p.company_number.clone()
            } else {
                company_number.clone() // This company might be the parent
            }
        } else {
            company_number.clone()
        };

        self.is_loading = true;
        self.status_message = format!("Discovering group structure from {}...", parent_number);

        self.runtime.spawn(async move {
            let _ = tx.send(BackgroundMessage::StatusUpdate(
                format!("Fetching PSCs for parent {}...", parent_number),
            ));

            // Get the parent's PSC data to find all subsidiaries
            let parent_pscs = match ch_api::client::get_pscs(&api_key, &parent_number).await {
                Ok(pscs) => pscs,
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::StatusUpdate(
                        format!("⚠ Couldn't fetch parent PSCs: {}", e),
                    ));
                    Vec::new()
                }
            };

            // The parent itself may appear as a corporate PSC on other companies
            // Search for companies using the parent's name
            let _ = tx.send(BackgroundMessage::StatusUpdate(
                "Searching for subsidiaries...".to_string(),
            ));

            // Get companies where this parent is a PSC by searching for subsidiaries
            // We do this by getting the parent's profile name and searching
            let parent_profile = ch_api::client::get_company_profile(&api_key, &parent_number).await.ok();
            let parent_name = parent_profile.as_ref()
                .and_then(|p| p.company_name.clone())
                .unwrap_or_else(|| parent_number.clone());

            // Check if parent has consolidated/group accounts
            let parent_filings = ch_api::client::get_filing_history(&api_key, &parent_number, Some("accounts"), Some(10)).await.unwrap_or_default();
            let has_consolidated = crate::investigation::group::has_consolidated_filings(&parent_filings);

            if has_consolidated {
                let _ = tx.send(BackgroundMessage::StatusUpdate(
                    format!("✓ {} files consolidated (group) accounts", parent_name),
                ));
            }

            // Search for companies where the parent company name appears
            // This helps find subsidiaries
            let search_results = ch_api::client::search_companies(&api_key, &parent_name).await.unwrap_or_default();

            let mut subsidiaries = Vec::new();

            // Also try to find subsidiaries by looking at companies where
            // the parent appears as a PSC — check companies from search results
            for result in &search_results {
                if let Some(cn) = &result.company_number {
                    if cn == &parent_number {
                        continue; // Skip the parent itself
                    }
                    // Check if this company has the parent as a PSC
                    let _ = tx.send(BackgroundMessage::StatusUpdate(
                        format!("Checking {}...", result.title.as_deref().unwrap_or(cn)),
                    ));

                    if let Ok(pscs) = ch_api::client::get_pscs(&api_key, cn).await {
                        let has_parent_psc = pscs.iter().any(|psc| {
                            psc.is_corporate()
                                && psc.is_active()
                                && psc.registration_number().map(|r| r == parent_number).unwrap_or(false)
                        });

                        if has_parent_psc {
                            let status = result.company_status.as_deref().unwrap_or("unknown").to_string();
                            let name = result.title.as_deref().unwrap_or(cn).to_string();
                            subsidiaries.push(crate::investigation::group::SubsidiaryInfo {
                                company_number: cn.clone(),
                                company_name: name,
                                status,
                            });
                        }
                    }

                    // Rate limiting — be gentle with the API
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                    // Limit to 10 checks
                    if subsidiaries.len() >= 10 {
                        break;
                    }
                }
            }

            let _ = tx.send(BackgroundMessage::StatusUpdate(
                format!("✓ Found {} subsidiaries", subsidiaries.len()),
            ));

            let _ = tx.send(BackgroundMessage::GroupSubsidiariesDiscovered {
                parent_number,
                subsidiaries,
                has_consolidated,
            });
        });
    }

    /// Run AI analysis across an entire corporate group.
    /// Uses consolidated accounts when available to save tokens.
    pub fn run_group_analysis(&mut self, company_number: String) {
        let tx = self.bg_sender.clone();
        let ai_config = self.config.ai_provider.clone();

        // Determine parent
        let parent_number = if let Some(gi) = self.group_info.get(&company_number) {
            if let Some(p) = &gi.parent {
                p.company_number.clone()
            } else {
                company_number.clone()
            }
        } else {
            company_number.clone()
        };

        // Build parent context
        let parent_context = crate::ai::prompts::build_company_context(
            &parent_number,
            self.company_profiles.get(&parent_number),
            self.company_officers.get(&parent_number),
            self.company_pscs.get(&parent_number),
            self.company_charges.get(&parent_number),
            self.company_insolvency.get(&parent_number),
            self.company_filings.get(&parent_number),
            &self.extracted_texts,
        );

        // Build subsidiary contexts (only for companies we already have data for)
        let mut sub_contexts = Vec::new();
        if let Some(gi) = self.group_info.get(&company_number) {
            for sub in &gi.subsidiaries {
                if sub.status != "active" {
                    continue;
                }
                let ctx = crate::ai::prompts::build_company_context(
                    &sub.company_number,
                    self.company_profiles.get(&sub.company_number),
                    self.company_officers.get(&sub.company_number),
                    self.company_pscs.get(&sub.company_number),
                    self.company_charges.get(&sub.company_number),
                    self.company_insolvency.get(&sub.company_number),
                    self.company_filings.get(&sub.company_number),
                    &self.extracted_texts,
                );
                sub_contexts.push((sub.company_name.clone(), ctx));
            }
        }

        let has_consolidated = self.group_info.get(&company_number)
            .map(|gi| gi.has_consolidated_accounts)
            .unwrap_or(false);

        let year_from = self.analysis_year_from;
        let year_to = self.analysis_year_to;

        let user_prompt = crate::ai::prompts::build_group_analysis_prompt(
            &parent_context,
            &sub_contexts,
            has_consolidated,
            year_from,
            year_to,
        );

        // Store the user message
        let label = format!("Run GROUP analysis ({}-{})", year_from, year_to);
        self.ai_conversations
            .entry(parent_number.clone())
            .or_default()
            .push(("user".to_string(), label));

        self.is_loading = true;
        self.status_message = format!("Running group analysis ({} subsidiaries)...", sub_contexts.len());

        let cn = parent_number;
        self.runtime.spawn(async move {
            let _ = tx.send(BackgroundMessage::StatusUpdate(
                "Sending group data to AI...".to_string(),
            ));

            let messages = vec![crate::ai::ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            }];

            match crate::ai::provider::chat_completion(
                &ai_config,
                crate::ai::prompts::SYSTEM_PROMPT,
                &messages,
            ).await {
                Ok(result) => {
                    let _ = tx.send(BackgroundMessage::AiAnalysisComplete {
                        company_number: cn,
                        analysis: result.text,
                        input_tokens: result.usage.input_tokens,
                        output_tokens: result.usage.output_tokens,
                        model: result.model,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::Error(
                        format!("Group analysis failed: {}", e),
                    ));
                }
            }
        });
    }

    /// Ask AI to summarise a single filing document.
    pub fn summarise_filing(&mut self, filing_id: String, filing_desc: String, text: String) {
        let tx = self.bg_sender.clone();
        let ai_config = self.config.ai_provider.clone();

        self.is_loading = true;
        self.status_message = format!("Summarising filing {}...", filing_id);

        self.runtime.spawn(async move {
            let prompt = format!(
                "Summarise this Companies House filing document concisely.\n\
                 Filing: {}\n\n\
                 Focus on:\n\
                 - Key financial figures (revenue, profit/loss, net assets, debt)\n\
                 - Auditor opinion and any going concern notes\n\
                 - Material changes from prior periods\n\
                 - Any red flags or notable items\n\n\
                 Keep the summary to 3-5 bullet points maximum.\n\n\
                 Document text:\n{}", filing_desc, text
            );

            let messages = vec![crate::ai::ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }];

            match crate::ai::provider::chat_completion(
                &ai_config,
                "You are a financial analyst reviewing Companies House filing documents. Be concise and factual.",
                &messages,
            ).await {
                Ok(result) => {
                    let _ = tx.send(BackgroundMessage::AiFilingSummary {
                        filing_id,
                        summary: result.text,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::AiFilingSummary {
                        filing_id,
                        summary: format!("⚠ Summary failed: {}", e),
                    });
                }
            }
        });
    }

    /// Apply settings from the settings panel.
    pub fn apply_settings(&mut self) {
        self.config.ai_provider = match self.settings_ai_provider.as_str() {
            "anthropic" => AiProviderConfig::Anthropic {
                api_key: self.settings_ai_key.clone(),
                model: self.settings_ai_model.clone(),
            },
            "openai" => AiProviderConfig::OpenAi {
                api_key: self.settings_ai_key.clone(),
                model: self.settings_ai_model.clone(),
            },
            "custom" => AiProviderConfig::Custom {
                api_key: self.settings_ai_key.clone(),
                model: self.settings_ai_model.clone(),
                base_url: self.settings_ai_base_url.clone(),
            },
            _ => AiProviderConfig::None,
        };
        if !self.settings_kanon2_key.is_empty() {
            self.config.kanon2_api_key = Some(self.settings_kanon2_key.clone());
        }
        self.status_message = "✓ Settings applied.".to_string();
    }
}

impl eframe::App for InvestigationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background();
        ui::render(self, ctx);
        // Request repaint while loading to poll messages
        if self.is_loading {
            ctx.request_repaint();
        }
    }
}
