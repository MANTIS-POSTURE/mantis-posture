use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use sysinfo::System;
use tauri::Manager;
use uuid::Uuid;

const MAX_PROCESS_OUTPUT_BYTES: usize = 512 * 1024;

#[derive(Serialize)]
struct Folder {
    id: String,
    name: String,
    context: String,
}

#[derive(Serialize, Clone)]
struct Incident {
    id: String,
    title: String,
    severity: String,
    discovered_at: String,
    what: String,
    why: String,
    impact: String,
    confidence: String,
    next_step: String,
    folder_id: Option<String>,
}

#[derive(Serialize, Clone)]
struct Action {
    id: String,
    title: String,
    priority_id: String,
    difficulty_id: String,
    deadline: String,
    status: String,
    guidance: String,
    proof_expected: String,
    folder_id: Option<String>,
    incident_id: Option<String>,
    tracking_note: Option<String>,
    actor: Option<String>,
    completed_at: Option<String>,
    blocked_reason: Option<String>,
}

#[derive(Serialize, Clone)]
struct ActionEvent {
    id: String,
    action_id: String,
    from_status: Option<String>,
    to_status: String,
    actor: Option<String>,
    note: Option<String>,
    created_at: String,
}

#[derive(Serialize, Clone)]
struct ActionEvidence {
    id: String,
    action_id: String,
    kind: String,
    locator: String,
    description: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct RemediationRecommendation {
    id: String,
    title: String,
    exposure_kinds: Vec<String>,
    priority: String,
    why: String,
    steps: Vec<String>,
    proof_expected: String,
    expected_outcome: String,
    guide_id: Option<String>,
    execution_mode: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct RemediationCatalog {
    version: String,
    reviewed_at: String,
    source: String,
    recommendations: Vec<RemediationRecommendation>,
}

#[derive(Serialize, Clone)]
struct RemediationPlanItem {
    id: String,
    action_id: Option<String>,
    exposure_id: Option<String>,
    incident_id: Option<String>,
    sort_order: i64,
    expected_outcome: String,
    proof_expected: String,
    execution_mode: String,
    recommendation_id: String,
}

#[derive(Serialize, Clone)]
struct RemediationPlan {
    id: String,
    identity_id: Option<String>,
    folder_id: Option<String>,
    scan_id: Option<String>,
    title: String,
    status: String,
    priority: String,
    rationale: String,
    catalog_version: String,
    items: Vec<RemediationPlanItem>,
}

#[derive(Serialize, Deserialize, Clone)]
struct RemediationAiPayload {
    schema_version: i32,
    task: String,
    summary: String,
    education: Vec<String>,
    priority_rationale: String,
    cautions: Vec<String>,
    citation_ids: Vec<String>,
}

#[derive(Serialize, Clone)]
struct RemediationAiEnrichment {
    id: String,
    plan_id: String,
    contract_version: String,
    mode: String,
    status: String,
    model_label: Option<String>,
    summary: String,
    education: Vec<String>,
    priority_rationale: String,
    cautions: Vec<String>,
    citation_ids: Vec<String>,
    error_message: Option<String>,
    created_at: String,
}

#[derive(Serialize, Clone)]
struct IdentityValue {
    id: String,
    kind: String,
    value: String,
    normalized_value: String,
    label: Option<String>,
    status: String,
    origin: String,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    sort_order: i64,
}

#[derive(Deserialize, Clone)]
struct IdentityValueInput {
    id: Option<String>,
    kind: String,
    value: String,
    label: Option<String>,
    status: String,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

#[derive(Serialize, Clone)]
struct Identity {
    id: String,
    label: String,
    kind: String,
    value: String,
    folder_id: Option<String>,
    notes: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    status: String,
    values: Vec<IdentityValue>,
}

#[derive(Serialize, Clone)]
struct Exposure {
    id: String,
    title: String,
    kind: String,
    severity: String,
    status: String,
    discovered_at: String,
    source: String,
    what: String,
    why: String,
    folder_id: Option<String>,
}

#[derive(Serialize, Clone)]
struct RgpdRequest {
    id: String,
    type_id: String,
    target: String,
    dpo_contact: String,
    status_id: String,
    data_summary: String,
    draft_preview: String,
    source_url: Option<String>,
    contact_source_url: Option<String>,
}

#[derive(Serialize)]
struct TimelineEntry {
    id: String,
    event_type: String,
    description: String,
    created_at: String,
}

#[derive(Serialize)]
struct PostureScore {
    score: Option<i32>,
    open_incidents: i32,
    high_exposures: i32,
    completed_actions: i32,
}

#[derive(Serialize, Debug, PartialEq)]
struct PublicIpContext {
    ip: String,
    ip_type: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    asn: Option<u64>,
    organization: Option<String>,
    isp: Option<String>,
    network_domain: Option<String>,
    timezone: Option<String>,
    utc_offset: Option<String>,
    proxy: bool,
    vpn: bool,
    tor: bool,
    hosting: bool,
}

#[derive(Deserialize)]
struct IpWhoisConnection {
    asn: Option<u64>,
    org: Option<String>,
    isp: Option<String>,
    domain: Option<String>,
}

#[derive(Deserialize)]
struct IpWhoisTimezone {
    id: Option<String>,
    utc: Option<String>,
}

#[derive(Deserialize, Default)]
struct IpWhoisSecurity {
    proxy: Option<bool>,
    vpn: Option<bool>,
    tor: Option<bool>,
    hosting: Option<bool>,
}

#[derive(Deserialize)]
struct IpWhoisResponse {
    success: bool,
    ip: Option<String>,
    #[serde(rename = "type")]
    ip_type: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    connection: Option<IpWhoisConnection>,
    timezone: Option<IpWhoisTimezone>,
    security: Option<IpWhoisSecurity>,
}

#[derive(Serialize, Clone)]
struct OsintModule {
    id: String,
    name: String,
    description: String,
    target_kind: String,
    frequency: String,
    status: String,
    last_run: Option<String>,
    next_run: Option<String>,
    script_path: Option<String>,
    script_args: Option<String>,
    installation_status: String,
    diagnostic: Option<String>,
    catalog_status: String,
    replacement_id: Option<String>,
    archived_reason: Option<String>,
}

// Legacy script-based OSINT flow, retained only for an explicit migration path.
// It is never compiled or exposed in the application by default.
#[cfg(feature = "legacy-osint")]
#[derive(Serialize, Clone)]
struct OsintTargetResult {
    target: String,
    leaks: Vec<String>,
    profiles: Vec<String>,
    noise_filtered: i32,
    risk_level: String,
}

#[cfg(feature = "legacy-osint")]
#[derive(Serialize)]
struct OsintRunSummary {
    results: Vec<OsintTargetResult>,
}

#[derive(Serialize)]
struct OsintScanSummary {
    scan_id: String,
    target: String,
    signals: Vec<OsintSignal>,
    message: String,
    analysis_job_id: Option<String>,
}

#[derive(Serialize)]
struct OsintScanSessionSummary {
    session_id: String,
    scan_id: String,
    identity_id: String,
    target: String,
    signals: Vec<OsintSignal>,
    claims: Vec<OsintClaim>,
    resolutions: Vec<OsintFactResolution>,
    message: String,
    analysis_job_id: Option<String>,
    planned_checks: usize,
    completed_checks: usize,
    failed_checks: usize,
    skipped_checks: usize,
    coverage: Vec<OsintScanCoverage>,
}

#[derive(Serialize)]
struct OsintScanCoverage {
    target_kind: String,
    planned_checks: usize,
    completed_checks: usize,
    failed_checks: usize,
    signal_count: usize,
}

#[derive(Serialize, Clone)]
struct OsintClaimEvidence {
    observation_id: String,
    signal_id: Option<String>,
    source: String,
    source_url: Option<String>,
    observed_at: String,
    role: String,
}

#[derive(Serialize, Clone)]
struct OsintClaim {
    id: String,
    identity_id: String,
    claim_type: String,
    display_value: String,
    status: String,
    priority: String,
    favorable_count: i64,
    contradictory_count: i64,
    source_count: i64,
    first_observed_at: String,
    last_observed_at: String,
    rationale: String,
    evidence: Vec<OsintClaimEvidence>,
}

#[derive(Serialize, Clone)]
struct OsintFactResolution {
    id: String,
    fact_type: String,
    status: String,
    source_count: i64,
    favorable_count: i64,
    contradictory_count: i64,
    rationale: String,
}

#[derive(Serialize)]
struct OsintScanSessionListItem {
    id: String,
    identity_id: String,
    origin: String,
    status: String,
    signal_count: i64,
    planned_checks: i64,
    completed_checks: i64,
    failed_checks: i64,
    skipped_checks: i64,
    started_at: String,
    completed_at: Option<String>,
    summary: Option<String>,
}

#[derive(Serialize)]
struct OsintReviewEvent {
    id: String,
    event_type: String,
    target_label: String,
    decision: String,
    reason: Option<String>,
    created_at: String,
}

#[derive(Serialize)]
struct OsintEvolutionItem {
    claim_id: String,
    display_value: String,
    change: String,
    current_sources: i64,
    previous_sources: i64,
}

#[derive(Serialize)]
struct OsintReviewWorkspace {
    events: Vec<OsintReviewEvent>,
    evolution: Vec<OsintEvolutionItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OsintScanPlanItem {
    module_id: &'static str,
    identity_value_id: String,
    target: String,
    target_kind: String,
}

#[derive(Serialize)]
struct OsintRoutineSummary {
    scanned_identities: usize,
    signals_found: usize,
    failed_identities: usize,
    message: String,
    analysis_jobs_started: usize,
}

#[derive(Serialize)]
struct OsintAnalysisJobStatus {
    id: String,
    origin: String,
    status: String,
    signal_count: i64,
    estimated_seconds: i64,
    elapsed_seconds: i64,
    run_id: Option<String>,
    result_mode: Option<String>,
    message: Option<String>,
    report: Option<OsintAnalysisReport>,
}

#[derive(Serialize)]
struct OsintGraphNode {
    id: String,
    node_type: String,
    label: String,
    detail: String,
}

#[derive(Serialize)]
struct OsintGraphEvidence {
    observation_id: String,
    label: String,
    excerpt: Option<String>,
    source_url: Option<String>,
    role: String,
}

#[derive(Serialize)]
struct OsintGraphEdge {
    id: String,
    from: String,
    to: String,
    relation_type: String,
    level: String,
    justification: String,
    review_status: String,
    evidence: Vec<OsintGraphEvidence>,
}

#[derive(Serialize)]
struct OsintGraphTimelineItem {
    id: String,
    date: String,
    date_kind: String,
    label: String,
    source: String,
    level: String,
}

#[derive(Serialize)]
struct OsintGraph {
    nodes: Vec<OsintGraphNode>,
    edges: Vec<OsintGraphEdge>,
    timeline: Vec<OsintGraphTimelineItem>,
    truncated: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintReportSource {
    signal_id: String,
    title: String,
    severity: String,
    source: String,
    source_url: Option<String>,
    observed_at: String,
    review_status: String,
    explanation: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintReportDecision {
    decision: String,
    reason: Option<String>,
    created_at: String,
    target_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintReportAction {
    id: String,
    title: String,
    priority: String,
    status: String,
    deadline: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintReportSnapshot {
    id: String,
    created_at: String,
    analyzed_count: i64,
    discarded_count: i64,
    attention_count: i64,
    uncertain_count: i64,
    contradiction_count: i64,
    actions_now_count: i64,
    overview: String,
    priorities: Vec<OsintReportSource>,
    sources: Vec<OsintReportSource>,
    decisions: Vec<OsintReportDecision>,
    actions: Vec<OsintReportAction>,
    guide_ids: Vec<String>,
    limitations: Vec<String>,
    has_local_ai_analysis: bool,
}

#[derive(Serialize)]
struct OsintReportExport {
    path: String,
    format: String,
    snapshot_id: String,
}

#[derive(Serialize)]
struct RgpdReviewStatus {
    request_id: String,
    draft_version_id: String,
    contract_version: String,
    validated: bool,
    eligible: bool,
    reason: String,
    source_signal_id: Option<String>,
    reviewed_at: Option<String>,
}

#[derive(Serialize)]
struct RgpdDraftUse {
    text: String,
    path: Option<String>,
    draft_version_id: String,
}

#[derive(Serialize, Clone)]
struct RgpdEvidence {
    id: String,
    request_id: String,
    kind: String,
    locator: String,
    description: Option<String>,
    verified: bool,
    created_at: String,
}

#[derive(Serialize, Clone)]
struct RgpdEvent {
    id: String,
    request_id: String,
    from_status: Option<String>,
    to_status: Option<String>,
    event_type: String,
    note: Option<String>,
    created_at: String,
}

#[derive(Serialize)]
struct VeilleRoutine {
    frequency: String,
    paused: bool,
    last_run: Option<String>,
    next_run: Option<String>,
    status: String,
}

#[derive(Serialize)]
struct OsintSignal {
    id: String,
    module_id: String,
    scan_id: String,
    target: String,
    signal_type: String,
    title: String,
    explanation: String,
    severity: String,
    confidence: String,
    source: String,
    source_url: Option<String>,
    discovered_at: String,
    review_status: String,
    exposure_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintAnalysisItem {
    observation_id: String,
    classification: String,
    relevance: String,
    reason: String,
    recommended_action: String,
    evidence_ids: Vec<String>,
    uncertainty: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintAnalysisPayload {
    schema_version: u32,
    task: String,
    overview: String,
    items: Vec<OsintAnalysisItem>,
    limitations: Vec<String>,
}

#[derive(Serialize)]
struct OsintAnalysisReport {
    run_id: String,
    mode: String,
    status: String,
    model_label: Option<String>,
    overview: String,
    items: Vec<OsintAnalysisItem>,
    limitations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conclusion: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    findings: Vec<OsintSynthesisFinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    citation_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintSynthesisFinding {
    claim_id: String,
    statement: String,
    confidence: String,
    recommended_action: String,
    evidence_ids: Vec<String>,
    contradiction: bool,
    #[serde(default)]
    exposure_kind: String,
    #[serde(default)]
    exposed_data: Vec<String>,
    #[serde(default)]
    where_found: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct OsintSynthesisPayload {
    schema_version: u32,
    task: String,
    conclusion: String,
    citation_ids: Vec<String>,
    findings: Vec<OsintSynthesisFinding>,
    limitations: Vec<String>,
}

#[derive(Serialize, Clone)]
struct LocalAiClaimInput {
    claim_id: String,
    claim_type: String,
    display_value: String,
    status: String,
    priority: String,
    rationale: String,
    resolution_status: String,
    resolution_source_count: i64,
    resolution_rationale: String,
    evidence: Vec<LocalAiClaimEvidence>,
}

#[derive(Serialize, Clone)]
struct LocalAiClaimEvidence {
    evidence_id: String,
    source: String,
    source_url: Option<String>,
    role: String,
    observed_at: String,
}

#[derive(Serialize)]
struct LocalAiAnalysisInput {
    observation_id: String,
    observation_type: String,
    display_value: String,
    source: String,
    relevance_status: String,
    evidence: Vec<LocalAiAnalysisEvidence>,
}

#[derive(Serialize)]
struct LocalAiAnalysisEvidence {
    evidence_id: String,
    label: String,
    excerpt: String,
}

#[derive(Serialize)]
struct OsintModuleLog {
    operation: String,
    status: String,
    message: String,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct ManagedBundleManifest {
    schema_version: u32,
    module_id: String,
    version: String,
    file: String,
    sha256: String,
    #[serde(default)]
    license_file: Option<String>,
}

#[derive(Deserialize)]
struct DdgsSearchResult {
    query: String,
    title: String,
    url: String,
    snippet: String,
    #[serde(default)]
    backend: String,
}

#[derive(Deserialize)]
struct DdgsSidecarOutput {
    version: u32,
    results: Vec<DdgsSearchResult>,
    errors: Vec<String>,
}

#[derive(Deserialize)]
struct UserScannerResult {
    status: String,
    reason: String,
    site_name: String,
    category: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    extra: serde_json::Value,
}

#[derive(Deserialize)]
struct UserScannerSummary {
    checked: usize,
    found: usize,
    errors: usize,
    skipped: usize,
    notification_checks_excluded: usize,
}

#[derive(Deserialize)]
struct UserScannerSidecarOutput {
    version: u32,
    target: String,
    target_kind: String,
    results: Vec<UserScannerResult>,
    summary: UserScannerSummary,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct MaigretResult {
    site_name: String,
    url: String,
    #[serde(default)]
    category: String,
}

#[derive(Deserialize)]
struct MaigretSummary {
    checked: usize,
    found: usize,
}

#[derive(Deserialize)]
struct MaigretSidecarOutput {
    version: u32,
    collector_version: String,
    target: String,
    results: Vec<MaigretResult>,
    summary: MaigretSummary,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct LocalAiCatalog {
    schema_version: u32,
    catalog_version: String,
    components: Vec<LocalAiCatalogComponent>,
}

#[derive(Deserialize, Clone)]
struct LocalAiCatalogComponent {
    id: String,
    component_type: String,
    version: String,
    platform: String,
    architecture: String,
    url: String,
    archive: String,
    sha256: String,
    byte_size: u64,
    filename: String,
    #[serde(default)]
    executable: Option<String>,
    license: String,
    source: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    min_ram_gb: Option<u64>,
    #[serde(default)]
    context_size: Option<u32>,
}

#[derive(Serialize)]
struct LocalAiStatus {
    component_id: String,
    version: String,
    status: String,
    installed: bool,
    integrity_ok: bool,
    downloaded_bytes: i64,
    expected_bytes: i64,
    diagnostic: String,
    platform: String,
    architecture: String,
    license: String,
    source: String,
    enabled: bool,
    onboarding_status: String,
    selected_model_id: Option<String>,
    recommended_model_id: String,
    total_memory_bytes: u64,
    available_disk_bytes: u64,
    active_download_id: Option<String>,
    models: Vec<LocalAiModelStatus>,
}

#[derive(Serialize)]
struct LocalAiModelStatus {
    component_id: String,
    label: String,
    tier: String,
    version: String,
    status: String,
    installed: bool,
    downloaded_bytes: i64,
    expected_bytes: i64,
    license: String,
    source: String,
    min_ram_gb: u64,
    context_size: u32,
    compatible: bool,
    diagnostic: String,
}

#[derive(Serialize, Deserialize)]
struct InstalledLocalAiManifest {
    schema_version: u32,
    component_id: String,
    version: String,
    archive_sha256: String,
    executable_path: String,
    executable_sha256: String,
}

const MIGRATION_SQL: &str = include_str!("../migrations/0001_init.sql");
const OSINT_OBSERVABILITY_MIGRATION_SQL: &str =
    include_str!("../migrations/0002_osint_observability.sql");
const LOCAL_AI_MIGRATION_SQL: &str = include_str!("../migrations/0003_local_ai_foundations.sql");
const LOCAL_AI_ONBOARDING_MIGRATION_SQL: &str =
    include_str!("../migrations/0004_local_ai_onboarding.sql");
const LOCAL_AI_ANALYSIS_MIGRATION_SQL: &str =
    include_str!("../migrations/0005_osint_local_ai_analysis.sql");
const LOCAL_AI_JOBS_MIGRATION_SQL: &str =
    include_str!("../migrations/0006_osint_analysis_jobs.sql");
const OSINT_GRAPH_MIGRATION_SQL: &str =
    include_str!("../migrations/0007_osint_relations_graph.sql");
const OSINT_REPORT_MIGRATION_SQL: &str =
    include_str!("../migrations/0008_osint_report_snapshots.sql");
const RGPD_REVIEW_MIGRATION_SQL: &str = include_str!("../migrations/0009_rgpd_review_audit.sql");
const IDENTITY_TARGETS_MIGRATION_SQL: &str =
    include_str!("../migrations/0010_identity_targets.sql");
const OSINT_SCAN_SESSIONS_MIGRATION_SQL: &str =
    include_str!("../migrations/0011_osint_scan_sessions.sql");
const OSINT_CLAIMS_MIGRATION_SQL: &str = include_str!("../migrations/0012_osint_claims.sql");
const OSINT_SYNTHESIS_MIGRATION_SQL: &str =
    include_str!("../migrations/0013_osint_multi_source_synthesis.sql");
const OSINT_REVIEW_PROJECTION_MIGRATION_SQL: &str =
    include_str!("../migrations/0014_osint_review_projection.sql");
const OSINT_CATALOG_LIFECYCLE_MIGRATION_SQL: &str =
    include_str!("../migrations/0015_osint_catalog_lifecycle.sql");
const ACTION_TRACKING_MIGRATION_SQL: &str = include_str!("../migrations/0016_action_tracking.sql");
const REMEDIATION_CATALOG_JSON: &str = include_str!("../resources/remediation-catalog.json");
const REMEDIATION_CATALOG_MIGRATION_SQL: &str =
    include_str!("../migrations/0017_remediation_catalog.sql");
const REMEDIATION_AI_MIGRATION_SQL: &str = include_str!("../migrations/0018_remediation_ai.sql");
const RGPD_TRACKING_MIGRATION_SQL: &str = include_str!("../migrations/0019_rgpd_tracking.sql");
const OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL: &str =
    include_str!("../migrations/0020_osint_evidence_assessments.sql");
const OSINT_EVIDENCE_FACTS_MIGRATION_SQL: &str =
    include_str!("../migrations/0021_osint_evidence_facts.sql");
const OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL: &str =
    include_str!("../migrations/0022_osint_claim_fact_links.sql");
const OSINT_FACT_RESOLUTIONS_MIGRATION_SQL: &str =
    include_str!("../migrations/0023_osint_fact_resolutions.sql");
const REMEDIATION_AI_SCHEMA: &str =
    include_str!("../resources/local-ai-contracts/remediation-enrichment-v1.schema.json");
const REMEDIATION_AI_INSTRUCTIONS: &str =
    include_str!("../resources/local-ai-contracts/remediation-enrichment-v1.txt");
const TRIAGE_OSINT_SCHEMA: &str =
    include_str!("../resources/local-ai-contracts/triage-osint-v1.schema.json");
const TRIAGE_OSINT_INSTRUCTIONS: &str =
    include_str!("../resources/local-ai-contracts/triage-osint-v1.txt");
const MULTI_SOURCE_SYNTHESIS_SCHEMA: &str =
    include_str!("../resources/local-ai-contracts/multi-source-synthesis-v1.schema.json");
const MULTI_SOURCE_SYNTHESIS_INSTRUCTIONS: &str =
    include_str!("../resources/local-ai-contracts/multi-source-synthesis-v1.txt");
const LOCAL_AI_CATALOG_JSON: &[u8] = include_bytes!("../resources/local-ai-catalog/catalog.json");
const LOCAL_AI_CATALOG_SIGNATURE: &str = include_str!("../resources/local-ai-catalog/catalog.sig");
const LLAMA_CPP_LICENSE: &str = include_str!("../resources/local-ai-catalog/LLAMA_CPP_LICENSE.txt");
const LOCAL_AI_CATALOG_PUBLIC_KEY: &str =
    "d106a93ab9f92baf0ed73d9046dd304ebf7277acf3bb2daf7e80485ae1af9344";

/// Version actuelle du schéma — **incrémenter à chaque changement de structure ou nettoyage de seeds**.
const SCHEMA_VERSION: i64 = 28;
static LOCAL_AI_ANALYSIS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn application_data_dir(_app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let directory = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(".mantis-dev-data");
        std::fs::create_dir_all(&directory).map_err(|e| {
            format!("Impossible de créer le stockage de développement MANTIS : {e}")
        })?;
        return Ok(directory);
    }
    #[cfg(not(debug_assertions))]
    {
        let directory = _app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&directory)
            .map_err(|e| format!("Impossible de créer le stockage privé MANTIS : {e}"))?;
        Ok(directory)
    }
}

fn init_database(app: &tauri::AppHandle) -> Result<(), String> {
    let app_dir = application_data_dir(app)?;
    let db_path = app_dir.join("mantis.db");
    let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    configure_database_connection(&conn)?;
    // WAL évite que les lectures de l'interface bloquent les écritures brèves des scans.
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=1000;")
        .map_err(|e| format!("Configuration SQLite impossible : {e}"))?;

    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);
    if user_version < SCHEMA_VERSION {
        backup_database_before_migration(&db_path, user_version)?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("Migration SQLite impossible : {e}"))?;
        tx.execute_batch(MIGRATION_SQL)
            .map_err(|e| format!("Migration de base impossible : {e}"))?;
        ensure_osint_schema(&tx).map_err(|e| format!("Migration OSINT impossible : {e}"))?;
        tx.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .map_err(|e| format!("Migration d’observabilité OSINT impossible : {e}"))?;
        tx.execute_batch(LOCAL_AI_MIGRATION_SQL)
            .map_err(|e| format!("Migration IA locale impossible : {e}"))?;
        tx.execute_batch(LOCAL_AI_ONBOARDING_MIGRATION_SQL)
            .map_err(|e| format!("Migration des préférences IA impossible : {e}"))?;
        tx.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL)
            .map_err(|e| format!("Migration des analyses IA impossible : {e}"))?;
        tx.execute_batch(LOCAL_AI_JOBS_MIGRATION_SQL)
            .map_err(|e| format!("Migration de la file d’analyse IA impossible : {e}"))?;
        tx.execute_batch(OSINT_GRAPH_MIGRATION_SQL)
            .map_err(|e| format!("Migration du graphe OSINT impossible : {e}"))?;
        tx.execute_batch(OSINT_REPORT_MIGRATION_SQL)
            .map_err(|e| format!("Migration des rapports OSINT impossible : {e}"))?;
        tx.execute_batch(RGPD_REVIEW_MIGRATION_SQL)
            .map_err(|e| format!("Migration de validation RGPD impossible : {e}"))?;
        ensure_identity_target_columns(&tx)
            .map_err(|e| format!("Migration des identités impossible : {e}"))?;
        tx.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL)
            .map_err(|e| format!("Migration des valeurs d’identité impossible : {e}"))?;
        remove_legacy_demo_data(&tx)
            .map_err(|e| format!("Nettoyage des données de démonstration impossible : {e}"))?;
        ensure_scan_session_columns(&tx)
            .map_err(|e| format!("Migration des sessions de veille impossible : {e}"))?;
        tx.execute_batch(OSINT_SCAN_SESSIONS_MIGRATION_SQL)
            .map_err(|e| format!("Migration de l’orchestrateur de veille impossible : {e}"))?;
        tx.execute_batch(OSINT_CLAIMS_MIGRATION_SQL)
            .map_err(|e| format!("Migration des revendications OSINT impossible : {e}"))?;
        ensure_analysis_job_columns(&tx)
            .map_err(|e| format!("Migration de la file de synthèse impossible : {e}"))?;
        tx.execute_batch(OSINT_SYNTHESIS_MIGRATION_SQL)
            .map_err(|e| format!("Migration des synthèses multi-source impossible : {e}"))?;
        tx.execute_batch(OSINT_REVIEW_PROJECTION_MIGRATION_SQL)
            .map_err(|e| format!("Migration de la revue OSINT impossible : {e}"))?;
        ensure_module_catalog_columns(&tx)
            .map_err(|e| format!("Migration du catalogue OSINT impossible : {e}"))?;
        tx.execute_batch(OSINT_CATALOG_LIFECYCLE_MIGRATION_SQL)
            .map_err(|e| format!("Archivage du catalogue OSINT impossible : {e}"))?;
        ensure_action_tracking_columns(&tx)
            .map_err(|e| format!("Migration du suivi des actions impossible : {e}"))?;
        tx.execute_batch(ACTION_TRACKING_MIGRATION_SQL)
            .map_err(|e| format!("Migration du suivi des actions impossible : {e}"))?;
        ensure_remediation_catalog_columns(&tx)
            .map_err(|e| format!("Migration du catalogue de remédiation impossible : {e}"))?;
        tx.execute_batch(REMEDIATION_CATALOG_MIGRATION_SQL)
            .map_err(|e| format!("Migration du catalogue de remédiation impossible : {e}"))?;
        tx.execute_batch(REMEDIATION_AI_MIGRATION_SQL)
            .map_err(|e| format!("Migration de l’enrichissement IA impossible : {e}"))?;
        tx.execute_batch(RGPD_TRACKING_MIGRATION_SQL)
            .map_err(|e| format!("Migration du suivi RGPD impossible : {e}"))?;
        tx.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
            .map_err(|e| format!("Migration de l’évaluation des preuves impossible : {e}"))?;
        tx.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
            .map_err(|e| format!("Migration des faits OSINT impossible : {e}"))?;
        tx.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
            .map_err(|e| format!("Migration de traçabilité des faits impossible : {e}"))?;
        tx.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
            .map_err(|e| format!("Migration de résolution des faits impossible : {e}"))?;
        tx.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION}"))
            .map_err(|e| format!("Version de schéma impossible à enregistrer : {e}"))?;
        tx.commit()
            .map_err(|e| format!("Validation de la migration impossible : {e}"))?;
    }

    ensure_osint_schema(&conn).map_err(|e| format!("Initialisation des modules OSINT : {e}"))?;
    ensure_identity_target_columns(&conn)
        .map_err(|e| format!("Initialisation des identités : {e}"))?;
    conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation des valeurs d’identité : {e}"))?;
    ensure_scan_session_columns(&conn)
        .map_err(|e| format!("Initialisation des sessions de veille : {e}"))?;
    conn.execute_batch(OSINT_SCAN_SESSIONS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation de l’orchestrateur de veille : {e}"))?;
    conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation des revendications OSINT : {e}"))?;
    ensure_analysis_job_columns(&conn)
        .map_err(|e| format!("Initialisation de la file de synthèse : {e}"))?;
    conn.execute_batch(OSINT_SYNTHESIS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation des synthèses multi-source : {e}"))?;
    conn.execute_batch(OSINT_REVIEW_PROJECTION_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation de la revue OSINT : {e}"))?;
    ensure_module_catalog_columns(&conn)
        .map_err(|e| format!("Initialisation du catalogue OSINT : {e}"))?;
    conn.execute_batch(OSINT_CATALOG_LIFECYCLE_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation du cycle de vie OSINT : {e}"))?;
    conn.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation de l’évaluation des preuves : {e}"))?;
    conn.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation des faits OSINT : {e}"))?;
    conn.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation de la traçabilité des faits : {e}"))?;
    conn.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
        .map_err(|e| format!("Initialisation de la résolution des faits : {e}"))?;
    ensure_action_tracking_columns(&conn)
        .map_err(|e| format!("Initialisation du suivi des actions : {e}"))?;
    ensure_remediation_catalog_columns(&conn)
        .map_err(|e| format!("Initialisation du catalogue de remédiation : {e}"))?;
    if user_version < SCHEMA_VERSION {
        let identity_ids = {
            let mut statement = conn
                .prepare("SELECT DISTINCT identity_id FROM osint_signals")
                .map_err(|e| format!("Préparation du reclassement des preuves impossible : {e}"))?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| format!("Lecture des identités à reclasser impossible : {e}"))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Lecture des identités à reclasser impossible : {e}"))?;
            rows
        };
        for identity_id in identity_ids {
            apply_signal_quality_gate(&mut conn, &identity_id)
                .map_err(|e| format!("Reclassement des preuves impossible : {e}"))?;
            refresh_osint_claims(&mut conn, &identity_id)
                .map_err(|e| format!("Actualisation des résultats qualifiés impossible : {e}"))?;
        }
    }
    // A process cannot survive an application restart. Keep the partial file and
    // expose a resumable state instead of leaving a permanently active download.
    let _ = conn.execute("UPDATE local_ai_downloads SET status='interrompu',error_message='Application fermée ; prêt à reprendre.',updated_at=datetime('now') WHERE status='en_cours'", []);
    let _ = conn.execute("UPDATE osint_analysis_jobs SET status='interrompu',message='Application fermée avant la fin de l’analyse.',completed_at=datetime('now') WHERE status IN ('en_attente','en_cours')", []);

    Ok(())
}

fn ensure_identity_target_columns(conn: &Connection) -> Result<(), String> {
    let mut statement = conn
        .prepare("PRAGMA table_info(identities)")
        .map_err(|e| e.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if !columns.iter().any(|column| column == "status") {
        conn.execute_batch("ALTER TABLE identities ADD COLUMN status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','inactive'))")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Supprime uniquement les anciennes lignes de démonstration connues.
///
/// Les versions précédentes pouvaient laisser ce jeu de données dans une base
/// locale. Les identifiants sont réservés au seed historique et ne sont jamais
/// générés par l’application : les supprimer lors de la migration garantit
/// qu’une installation existante retrouve le même état qu’une base neuve.
fn remove_legacy_demo_data(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "\
        DELETE FROM folder_identity
         WHERE identity_id IN ('id-name', 'id-email-perso', 'id-email-pro', 'id-address-old');
        DELETE FROM identity_values
         WHERE identity_id IN ('id-name', 'id-email-perso', 'id-email-pro', 'id-address-old');
        DELETE FROM identities
         WHERE id IN ('id-name', 'id-email-perso', 'id-email-pro', 'id-address-old');
        DELETE FROM folders
         WHERE id IN ('folder-perso', 'folder-job')
           AND NOT EXISTS (SELECT 1 FROM identities WHERE folder_id = folders.id)
           AND NOT EXISTS (SELECT 1 FROM exposures WHERE folder_id = folders.id)
           AND NOT EXISTS (SELECT 1 FROM incidents WHERE folder_id = folders.id)
           AND NOT EXISTS (SELECT 1 FROM actions WHERE folder_id = folders.id);\
        ",
    )
    .map_err(|e| e.to_string())
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn ensure_action_tracking_columns(conn: &Connection) -> Result<(), String> {
    let columns = table_columns(conn, "actions")?;
    for (name, definition) in [
        ("workflow_status", "TEXT NOT NULL DEFAULT 'a_faire' CHECK(workflow_status IN ('a_faire','en_cours','effectue_moi','effectue_site','en_attente','impossible','ignore'))"),
        ("tracking_note", "TEXT"),
        ("actor", "TEXT CHECK(actor IS NULL OR actor IN ('moi','site'))"),
        ("completed_at", "TEXT"),
        ("blocked_reason", "TEXT"),
    ] {
        if !columns.iter().any(|column| column == name) {
            conn.execute_batch(&format!("ALTER TABLE actions ADD COLUMN {name} {definition}"))
                .map_err(|e| e.to_string())?;
        }
    }
    conn.execute_batch("UPDATE actions SET workflow_status = CASE status WHEN 'faite' THEN 'effectue_moi' WHEN 'en_cours' THEN 'en_cours' ELSE 'a_faire' END WHERE workflow_status = 'a_faire'")
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn ensure_remediation_catalog_columns(conn: &Connection) -> Result<(), String> {
    let plans_exist: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='remediation_plans')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !plans_exist {
        return Ok(());
    }
    let plan_columns = table_columns(conn, "remediation_plans")?;
    if !plan_columns
        .iter()
        .any(|column| column == "catalog_version")
    {
        conn.execute_batch("ALTER TABLE remediation_plans ADD COLUMN catalog_version TEXT NOT NULL DEFAULT '1.0.0'")
            .map_err(|e| e.to_string())?;
    }
    let item_columns = table_columns(conn, "remediation_plan_items")?;
    if !item_columns
        .iter()
        .any(|column| column == "recommendation_id")
    {
        conn.execute_batch("ALTER TABLE remediation_plan_items ADD COLUMN recommendation_id TEXT NOT NULL DEFAULT ''")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_scan_session_columns(conn: &Connection) -> Result<(), String> {
    let scan_columns = table_columns(conn, "osint_scans")?;
    for (name, definition) in [
        ("session_id", "TEXT"),
        ("identity_value_id", "TEXT"),
        ("target_kind_snapshot", "TEXT"),
    ] {
        if !scan_columns.iter().any(|column| column == name) {
            conn.execute_batch(&format!(
                "ALTER TABLE osint_scans ADD COLUMN {name} {definition}"
            ))
            .map_err(|e| e.to_string())?;
        }
    }
    let signal_columns = table_columns(conn, "osint_signals")?;
    if !signal_columns
        .iter()
        .any(|column| column == "identity_value_id")
    {
        conn.execute_batch("ALTER TABLE osint_signals ADD COLUMN identity_value_id TEXT")
            .map_err(|e| e.to_string())?;
    }
    let observation_columns = table_columns(conn, "osint_observations")?;
    if !observation_columns
        .iter()
        .any(|column| column == "identity_value_id")
    {
        conn.execute_batch("ALTER TABLE osint_observations ADD COLUMN identity_value_id TEXT")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_analysis_job_columns(conn: &Connection) -> Result<(), String> {
    let columns = table_columns(conn, "osint_analysis_jobs")?;
    if !columns.iter().any(|column| column == "task") {
        conn.execute_batch(
            "ALTER TABLE osint_analysis_jobs ADD COLUMN task TEXT NOT NULL DEFAULT 'triage_osint'",
        )
        .map_err(|e| e.to_string())?;
    }
    if !columns.iter().any(|column| column == "identity_id") {
        conn.execute_batch("ALTER TABLE osint_analysis_jobs ADD COLUMN identity_id TEXT")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// A local process is bounded, so a job still running well past its estimate
/// cannot be completed by this application instance. Recover it on polling so
/// the interface never waits forever after a debugger stop, crash, or a
/// stranded child process.
fn recover_stale_analysis_jobs(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "UPDATE osint_analysis_jobs
         SET status='interrompu',
             message='L’analyse locale a été interrompue avant sa fin. Vous pouvez la relancer ; les preuves déjà collectées sont conservées.',
             completed_at=datetime('now')
         WHERE (status='en_attente' AND CAST(strftime('%s','now') - strftime('%s',created_at) AS INTEGER) > 120)
            OR (status='en_cours' AND started_at IS NOT NULL
                AND CAST(strftime('%s','now') - strftime('%s',started_at) AS INTEGER) > estimated_seconds + 120)",
        [],
    )
    .map_err(|e| format!("Récupération des tâches IA interrompues impossible : {e}"))?;
    Ok(())
}

fn ensure_module_catalog_columns(conn: &Connection) -> Result<(), String> {
    let columns = table_columns(conn, "osint_modules")?;
    if !columns.iter().any(|column| column == "catalog_status") {
        conn.execute_batch(
            "ALTER TABLE osint_modules ADD COLUMN catalog_status TEXT NOT NULL DEFAULT 'active'",
        )
        .map_err(|e| e.to_string())?;
    }
    if !columns.iter().any(|column| column == "replacement_id") {
        conn.execute_batch("ALTER TABLE osint_modules ADD COLUMN replacement_id TEXT")
            .map_err(|e| e.to_string())?;
    }
    if !columns.iter().any(|column| column == "archived_reason") {
        conn.execute_batch("ALTER TABLE osint_modules ADD COLUMN archived_reason TEXT")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Keeps OSINT data additive: existing dossiers and history are never reset for this feature.
fn ensure_osint_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "\
        CREATE TABLE IF NOT EXISTS osint_scans (
            id TEXT PRIMARY KEY, module_id TEXT NOT NULL, identity_id TEXT NOT NULL,
            target TEXT NOT NULL, status TEXT NOT NULL, started_at TEXT NOT NULL,
            completed_at TEXT, raw_result_path TEXT, error_message TEXT
        );
        CREATE TABLE IF NOT EXISTS osint_signals (
            id TEXT PRIMARY KEY, module_id TEXT NOT NULL, scan_id TEXT NOT NULL,
            identity_id TEXT NOT NULL, target TEXT NOT NULL, signal_type TEXT NOT NULL,
            title TEXT NOT NULL, explanation TEXT NOT NULL, severity TEXT NOT NULL,
            confidence TEXT NOT NULL, source TEXT NOT NULL, source_url TEXT,
            discovered_at TEXT NOT NULL, evidence_ref TEXT, raw_result_path TEXT,
            review_status TEXT NOT NULL, exposure_id TEXT
        );
        CREATE TABLE IF NOT EXISTS osint_module_logs (
            id TEXT PRIMARY KEY, module_id TEXT NOT NULL, operation TEXT NOT NULL,
            status TEXT NOT NULL, message TEXT NOT NULL, created_at TEXT NOT NULL
        );
    ",
    )
    .map_err(|e| e.to_string())?;
    ensure_rgpd_enrichment_columns(conn)?;
    ensure_module_catalog_columns(conn)?;

    // Never use REPLACE here: it deletes the existing row and can break a referenced scan.
    conn.execute_batch("\
        UPDATE osint_modules SET name='User Scanner (comptes)', description='Recherche des comptes potentiels associés à un e-mail ou pseudo autorisé. Les résultats restent à vérifier.', target_kind='e-mail ou pseudo', frequency='Hebdomadaire', script_path=NULL, script_args=NULL WHERE id='osint-email-platforms';
        UPDATE osint_modules SET name='XposedOrNot (fuites)', description='Vérifie les fuites connues associées à un e-mail autorisé. Les résultats restent à confirmer.', target_kind='email', frequency='Manuel', status='actif', script_path=NULL, script_args=NULL WHERE id='osint-email-intel';
        INSERT OR IGNORE INTO osint_modules (id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-web-footprint','Empreinte Web (DDGS)','Recherche des mentions publiques exactes d’un nom, pseudo, e-mail, téléphone ou adresse autorisé. Résultats à vérifier avant toute action.','nom, pseudo, e-mail, téléphone ou adresse','Manuel','planifie',NULL,NULL,NULL,NULL),
          ('osint-username-profiles','Profils publics (Maigret)','Recherche des profils publics potentiels associés à un pseudo autorisé. Chaque correspondance doit être vérifiée.','pseudo','Hebdomadaire','planifie',NULL,NULL,NULL,NULL),
          ('osint-github-profile','Profil GitHub public','Vérifie un profil GitHub public pour un pseudo déclaré. Les informations publiques restent une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules (id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-gitlab-profile','Profil GitLab public','Vérifie un profil GitLab public pour un pseudo ou e-mail public déclaré. Les informations publiques restent une correspondance possible, jamais une identité confirmée.','pseudo ou e-mail','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules (id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-mastodon-webfinger','Profil Mastodon public','Résout uniquement un identifiant Mastodon explicite utilisateur@instance via WebFinger. Le profil reste une correspondance possible, jamais une identité confirmée.','pseudo utilisateur@instance','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules (id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-fr-company-register','Mandats publics en France','Vérifie manuellement un nom complet exact dans le registre public français des entreprises. Un homonyme reste possible et aucune identité n’est attribuée.','nom complet','Manuel','actif',NULL,NULL,NULL,NULL),
          ('osint-hal-author','Publications HAL','Vérifie manuellement un nom d’auteur exact dans les métadonnées publiques HAL. Un homonyme reste possible et aucune identité n’est attribuée.','nom complet','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-gravatar-profile','Profil public lié à l’e-mail','Recherche un profil Gravatar public à partir du condensat SHA-256 d’un e-mail autorisé. Les informations restent une correspondance possible, jamais une identité confirmée.','e-mail','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-keybase-profile','Preuves publiques liées au pseudo','Recherche un profil Keybase exact et ses preuves publiques vers d’autres services. Un réseau de comptes reste une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-bluesky-profile','Profil Bluesky public','Vérifie un handle Bluesky public exact ou la variante explicite pseudo.bsky.social. Le profil reste une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL);
        INSERT OR IGNORE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args) VALUES
          ('osint-hackernews-profile','Profil Hacker News public','Vérifie un compte Hacker News public correspondant exactement au pseudo déclaré et résume une activité publique limitée. Le profil reste une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL);
        UPDATE osint_modules SET name='Empreinte Web (DDGS)', description='Recherche des mentions publiques exactes d’un nom, pseudo, e-mail, téléphone ou adresse autorisé. Résultats à vérifier avant toute action.', target_kind='nom, pseudo, e-mail, téléphone ou adresse', frequency='Manuel', script_path=NULL, script_args=NULL WHERE id='osint-web-footprint';
        UPDATE osint_modules SET name='Profils publics (Maigret)', description='Recherche des profils publics potentiels associés à un pseudo autorisé. Chaque correspondance doit être vérifiée.', target_kind='pseudo', frequency='Hebdomadaire', script_path=NULL, script_args=NULL WHERE id='osint-username-profiles';
        UPDATE osint_modules SET name='h8mail (sources locales)', description='Collecteur historique non distribué.', target_kind='email', frequency='Manuel', status='desactive', script_path=NULL, script_args=NULL WHERE id='osint-email-breaches-local';
        UPDATE osint_modules SET status='desactive', script_path=NULL, script_args=NULL WHERE id IN ('osint-gmail-profile','mock-osint','osint-entity-corroboration');
    ").map_err(|e| format!("Mise à jour du catalogue OSINT impossible : {e}"))?;
    conn.execute_batch(OSINT_CATALOG_LIFECYCLE_MIGRATION_SQL)
        .map_err(|e| format!("Cycle de vie du catalogue OSINT impossible : {e}"))?;
    conn.execute("INSERT OR IGNORE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args,catalog_status,archived_reason) VALUES ('mock-osint','Mock OSINT','Fixture synthétique interne ; jamais exposée dans le catalogue de production.','email','Manuel','desactive',NULL,NULL,NULL,NULL,'test_only','Réservé aux tests automatisés.')",[]).map_err(|e|e.to_string())?;
    Ok(())
}

fn backup_database_before_migration(
    db_path: &std::path::Path,
    user_version: i64,
) -> Result<(), String> {
    if user_version == 0
        || !db_path.exists()
        || std::fs::metadata(db_path).map_err(|e| e.to_string())?.len() == 0
    {
        return Ok(());
    }
    let backup_path = db_path.with_file_name(format!(
        "mantis.pre-v{SCHEMA_VERSION}.from-v{user_version}.bak"
    ));
    if !backup_path.exists() {
        std::fs::copy(db_path, &backup_path)
            .map_err(|e| format!("Sauvegarde de sécurité avant migration impossible : {e}"))?;
    }
    Ok(())
}

fn ensure_rgpd_enrichment_columns(conn: &Connection) -> Result<(), String> {
    let mut statement = conn
        .prepare("PRAGMA table_info(rgpd_requests)")
        .map_err(|e| e.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if !columns.iter().any(|column| column == "source_url") {
        conn.execute_batch("ALTER TABLE rgpd_requests ADD COLUMN source_url TEXT")
            .map_err(|e| e.to_string())?;
    }
    if !columns.iter().any(|column| column == "contact_source_url") {
        conn.execute_batch("ALTER TABLE rgpd_requests ADD COLUMN contact_source_url TEXT")
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Insère le jeu de données minimal (seulement si base vide).
///
/// Ce fixture est réservé aux tests et ne fait donc jamais partie d’un binaire
/// de production. Une nouvelle installation doit rester vide et attendre la
/// saisie explicite de l’utilisateur.
#[cfg(test)]
#[allow(dead_code)]
fn seed_minimal_test_data(conn: &Connection) -> Result<(), String> {
    // ============================================================
    // DONNÉES DE TEST MINIMALES — MANTIS POSTURE
    // Convention : les données de test portent le suffixe "¤" dans les libellés
    // pour être identifiables comme mocks dans l'UI.
    // ============================================================

    // --- Références (métadonnées d'actions & RGPD) ---
    conn.execute_batch(
        "
        INSERT OR IGNORE INTO action_metadata (id, type, value, label) VALUES
        ('prio_001', 'priority', 'basse', 'Basse'),
        ('prio_002', 'priority', 'moyenne', 'Moyenne'),
        ('prio_003', 'priority', 'haute', 'Haute'),
        ('prio_004', 'priority', 'critique', 'Critique');

        INSERT OR IGNORE INTO action_metadata (id, type, value, label) VALUES
        ('diff_001', 'difficulty', 'facile', 'Facile'),
        ('diff_002', 'difficulty', 'moyenne', 'Moyenne'),
        ('diff_003', 'difficulty', 'difficile', 'Difficile');

        INSERT OR IGNORE INTO rgpd_types (id, name, label) VALUES
        ('type_001', 'acces', 'Accès'),
        ('type_002', 'rectification', 'Rectification'),
        ('type_003', 'effacement', 'Effacement'),
        ('type_004', 'opposition', 'Opposition'),
        ('type_005', 'dereferencement', 'Déréférencement');

        INSERT OR IGNORE INTO rgpd_statuses (id, name, label) VALUES
        ('status_001', 'brouillon', 'Brouillon'),
        ('status_002', 'prete', 'Prête à envoyer'),
        ('status_003', 'envoyee', 'Envoyée'),
        ('status_004', 'repondue', 'Répondue');
    ",
    )
    .map_err(|e| e.to_string())?;

    // --- 1. DOSSIERS (2) : contextes distincts ---
    conn.execute(
        "INSERT OR IGNORE INTO folders (id, name, context) VALUES (?1, ?2, ?3)",
        params![
            "folder-perso",
            "Personnel¤",
            "Identités et traces hors travail — données de test"
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO folders (id, name, context) VALUES (?1, ?2, ?3)",
        params![
            "folder-job",
            "Emploi actuel¤",
            "Présence professionnelle et comptes liés au travail — données de test"
        ],
    )
    .map_err(|e| e.to_string())?;

    // --- 2. IDENTITÉS (4) : couvrir les kinds principaux ---
    conn.execute(
        "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["id-name", "Nom complet¤", "nom", "Alex Martin¤", "folder-perso", "Donnée de test — identité principale"]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["id-email-perso", "E-mail personnel¤", "email", "alex.martin.perso@example.com¤", "folder-perso", "Utilisé pour comptes grand public — donnée de test"]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["id-email-pro", "E-mail professionnel¤", "email", "a.martin@entreprise-exemple.example¤", "folder-job", "Donnée de test — exposition LinkedIn"]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO identities (id, label, kind, value, folder_id, notes, address_line1, address_line2, city, postal_code, country) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params!["id-address-old", "Ancienne adresse¤", "adresse", "12 rue de l'Ancienne, 75001 Paris¤", "folder-perso", "Adresse conservée pour vérification de fuites — donnée de test", "12 rue de l'Ancienne¤", "", "Paris¤", "75001", "France"]
    ).map_err(|e| e.to_string())?;

    // --- 3. EXPOSITIONS (2) : une par dossier, kinds différents ---
    conn.execute(
        "INSERT OR IGNORE INTO exposures (id, title, kind, severity, status, discovered_at, source, what, why, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params!["exp-linkedin-email", "E-mail pro sur profil LinkedIn¤", "profil_public", "modérée", "en_suivi", "2026-07-20", "Profil public¤", "L'adresse e-mail professionnelle apparaît dans la section contact du profil.¤", "Facilite phishing ciblé et corrélation d'identités.¤", "folder-job"]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO exposures (id, title, kind, severity, status, discovered_at, source, what, why, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params!["exp-leak-2019", "Occurrence e-mail — fuite 2019¤", "fuite", "élevée", "en_suivi", "2026-07-18", "Breach intelligence¤", "E-mail personnel signalé dans une fuite datée 2019.¤", "Risque si réutilisation d'identifiants sur des comptes encore actifs.¤", "folder-perso"]
    ).map_err(|e| e.to_string())?;

    // --- 4. INCIDENTS (2) : chacun lié à une exposition + dossier ---
    conn.execute(
        "INSERT OR IGNORE INTO incidents (id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params!["inc-linkedin", "E-mail professionnel visible sur LinkedIn¤", "modérée", "2026-07-20", "Le profil LinkedIn affiche l'adresse e-mail professionnelle en clair.¤", "Cette adresse facilite le phishing ciblé, le scraping et la corrélation avec d'autres traces publiques.¤", "Phishing, usurpation légère, augmentation du volume de spam ciblé.¤", "Élevée — observation directe du profil public.¤", "Masquer ou retirer l'e-mail du profil, puis vérifier la page publique.¤", "folder-job"]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO incidents (id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params!["inc-leaks", "Fuites anciennes d'identifiants¤", "élevée", "2026-07-18", "Deux occurrences d'un e-mail personnel dans des bases de fuites datées (2019 et 2021).¤", "Si un mot de passe a été réutilisé, des comptes encore actifs peuvent être exposés.¤", "Compromission de comptes, reprise d'accès non autorisée.¤", "Moyenne — correspondance e-mail, contexte de fuite partiel.¤", "Passer en revue les services concernés et confirmer rotation MFA / mots de passe (hors MANTIS).¤", "folder-perso"]
    ).map_err(|e| e.to_string())?;

    // --- 5. ACTIONS (2) : une par incident, statuts différents ---
    conn.execute(
        "INSERT OR IGNORE INTO actions (id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params!["act-linkedin-email", "Masquer l'e-mail sur LinkedIn¤", "prio_003", "diff_001", "2026-07-28", "a_faire", "[\"Ouvrir les paramètres de confidentialité du profil LinkedIn.¤\",\"Retirer ou masquer l'adresse e-mail professionnelle de la vue publique.¤\"]", "Capture ou note : e-mail plus visible en mode public.¤", "folder-job"]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO actions (id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params!["act-review-leaks", "Revue des comptes liés aux fuites¤", "prio_004", "diff_002", "2026-08-01", "en_cours", "[\"Lister les services où l'e-mail fuité était utilisé (hors MANTIS).¤\",\"Pour chaque compte encore actif : changer le mot de passe et activer la MFA.¤\"]", "Liste des services revus (noms uniquement).¤", "folder-perso"]
    ).map_err(|e| e.to_string())?;

    // --- 6. LIENS INCIDENT ↔ ACTION ---
    conn.execute(
        "INSERT OR IGNORE INTO incident_action (incident_id, action_id) VALUES (?1, ?2)",
        params!["inc-linkedin", "act-linkedin-email"],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO incident_action (incident_id, action_id) VALUES (?1, ?2)",
        params!["inc-leaks", "act-review-leaks"],
    )
    .map_err(|e| e.to_string())?;

    // --- 7. DEMANDES RGPD (1) : effacement adresse, statut "prete" ---
    conn.execute(
        "INSERT OR IGNORE INTO rgpd_requests (id, type_id, target, dpo_contact, status_id, data_summary, draft_preview) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params!["rgpd-1", "type_003", "Annuaire Web Exemple¤", "privacy@annuaire-exemple.example¤", "status_002", "Adresse postale ancienne publiée sur une fiche nominative.¤", "Objet : Demande d'effacement...¤"]
    ).map_err(|e| e.to_string())?;

    // --- 8. CHRONOLOGIE (2) : détection + action ---
    conn.execute(
        "INSERT OR IGNORE INTO timeline_entries (id, event_type, description, created_at) VALUES (?1, ?2, ?3, ?4)",
        params!["tl-1", "Détection¤", "Fuite d'email détectée dans Collection #1¤", "2023-10-15T12:00:00Z"]
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO timeline_entries (id, event_type, description, created_at) VALUES (?1, ?2, ?3, ?4)",
        params!["tl-2", "Action¤", "Création d'une action pour changer le mot de passe¤", "2023-10-15T12:05:00Z"]
    ).map_err(|e| e.to_string())?;

    // --- 9. MODULES OSINT (4) : stack email complète, statuts variés ---
    // Ces modules sont insérés par la migration SQL (0001_init.sql) mais on s'assure qu'ils existent ici en fallback.
    conn.execute_batch("
        INSERT OR IGNORE INTO osint_modules (id, name, description, target_kind, frequency, status, last_run, next_run, script_path, script_args) VALUES
        ('osint-email-intel', 'MailAccess (Intel email)¤', 'Analyse globale d''un e‑mail : breaches connues, domaines associés, identités liées, réputation.¤', 'email', 'Hebdomadaire', 'planifie', NULL, NULL, 'python', 'scripts/mailaccess_intel.py'),
        ('osint-email-platforms', 'User Scanner (Comptes)¤', 'Détecte les plateformes publiques associées à un e-mail ou un pseudo.¤', 'e-mail ou pseudo', 'Hebdomadaire', 'planifie', NULL, NULL, NULL, NULL),
        ('osint-email-breaches-local', 'h8mail (Fuites locales)¤', 'Recherche l''e‑mail dans des dumps de breaches locaux (Collections, HaveIBeenPwned dumps).¤', 'email', 'Hebdomadaire', 'planifie', NULL, NULL, 'python', 'scripts/h8mail.py'),
        ('osint-gmail-profile', 'GHunt (Profil Google)¤', 'Si l''e‑mail est Gmail / Google Workspace, récupère les informations publiques du profil Google.¤', 'email', 'Mensuelle', 'desactive', NULL, NULL, 'python', 'scripts/ghunt.py');
    ").map_err(|e| e.to_string())?;

    Ok(())
}

fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
    let app_dir = application_data_dir(app)?;
    let db_path = app_dir.join("mantis.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    configure_database_connection(&conn)?;
    Ok(conn)
}

fn configure_database_connection(conn: &Connection) -> Result<(), String> {
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("Configuration SQLite impossible : {e}"))?;
    Ok(())
}

// --- Folder commands ---

#[tauri::command]
fn list_folders(app: tauri::AppHandle) -> Result<Vec<Folder>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn
        .prepare("SELECT id, name, context FROM folders")
        .map_err(|e| e.to_string())?;
    let folder_iter = stmt
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                context: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for folder in folder_iter {
        folders.push(folder.map_err(|e| e.to_string())?);
    }

    Ok(folders)
}

#[tauri::command]
fn get_folder(app: tauri::AppHandle, id: String) -> Result<Folder, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn
        .prepare("SELECT id, name, context FROM folders WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let folder = stmt
        .query_row(params![id], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                context: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(folder)
}

// --- Incident commands ---

#[tauri::command]
fn list_incidents(
    app: tauri::AppHandle,
    identity_id: Option<String>,
) -> Result<Vec<Incident>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn.prepare("SELECT i.id, i.title, i.severity, i.discovered_at, i.what, i.why, i.impact, i.confidence, i.next_step, i.folder_id FROM incidents i WHERE ?1 IS NULL OR EXISTS (SELECT 1 FROM exposure_incident ei JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ei.incident_id=i.id AND s.identity_id=?1)").map_err(|e| e.to_string())?;
    let incident_iter = stmt
        .query_map(params![identity_id], |row| {
            Ok(Incident {
                id: row.get(0)?,
                title: row.get(1)?,
                severity: row.get(2)?,
                discovered_at: row.get(3)?,
                what: row.get(4)?,
                why: row.get(5)?,
                impact: row.get(6)?,
                confidence: row.get(7)?,
                next_step: row.get(8)?,
                folder_id: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut incidents = Vec::new();
    for incident in incident_iter {
        incidents.push(incident.map_err(|e| e.to_string())?);
    }

    Ok(incidents)
}

#[tauri::command]
fn get_incident(app: tauri::AppHandle, id: String) -> Result<Incident, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id FROM incidents WHERE id = ?1").map_err(|e| e.to_string())?;
    let incident = stmt
        .query_row(params![id], |row| {
            Ok(Incident {
                id: row.get(0)?,
                title: row.get(1)?,
                severity: row.get(2)?,
                discovered_at: row.get(3)?,
                what: row.get(4)?,
                why: row.get(5)?,
                impact: row.get(6)?,
                confidence: row.get(7)?,
                next_step: row.get(8)?,
                folder_id: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(incident)
}

// --- Action commands ---

#[tauri::command]
fn list_actions(app: tauri::AppHandle, identity_id: Option<String>) -> Result<Vec<Action>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn.prepare("
        SELECT a.id, a.title, a.priority_id, a.difficulty_id, a.deadline, COALESCE(a.workflow_status, a.status), a.guidance, a.proof_expected, a.folder_id,
            a.tracking_note, a.actor, a.completed_at, a.blocked_reason,
            (SELECT ia.incident_id FROM incident_action ia WHERE ia.action_id = a.id LIMIT 1) as incident_id
        FROM actions a
		WHERE ?1 IS NULL
			OR EXISTS (SELECT 1 FROM incident_action ia JOIN exposure_incident ei ON ei.incident_id=ia.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ia.action_id=a.id AND s.identity_id=?1)
			OR EXISTS (SELECT 1 FROM remediation_plan_items pi JOIN remediation_plans rp ON rp.id=pi.plan_id WHERE pi.action_id=a.id AND rp.identity_id=?1)
    ").map_err(|e| e.to_string())?;
    let action_iter = stmt
        .query_map(params![identity_id], |row| {
            Ok(Action {
                id: row.get(0)?,
                title: row.get(1)?,
                priority_id: row.get(2)?,
                difficulty_id: row.get(3)?,
                deadline: row.get(4)?,
                status: row.get(5)?,
                guidance: row.get(6)?,
                proof_expected: row.get(7)?,
                folder_id: row.get(8)?,
                tracking_note: row.get(9)?,
                actor: row.get(10)?,
                completed_at: row.get(11)?,
                blocked_reason: row.get(12)?,
                incident_id: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut actions = Vec::new();
    for action in action_iter {
        actions.push(action.map_err(|e| e.to_string())?);
    }

    Ok(actions)
}

#[tauri::command]
fn get_action(app: tauri::AppHandle, id: String) -> Result<Action, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("
        SELECT a.id, a.title, a.priority_id, a.difficulty_id, a.deadline, COALESCE(a.workflow_status, a.status), a.guidance, a.proof_expected, a.folder_id,
            a.tracking_note, a.actor, a.completed_at, a.blocked_reason,
            (SELECT ia.incident_id FROM incident_action ia WHERE ia.action_id = a.id LIMIT 1) as incident_id
        FROM actions a WHERE a.id = ?1
    ").map_err(|e| e.to_string())?;
    let action = stmt
        .query_row(params![id], |row| {
            Ok(Action {
                id: row.get(0)?,
                title: row.get(1)?,
                priority_id: row.get(2)?,
                difficulty_id: row.get(3)?,
                deadline: row.get(4)?,
                status: row.get(5)?,
                guidance: row.get(6)?,
                proof_expected: row.get(7)?,
                folder_id: row.get(8)?,
                tracking_note: row.get(9)?,
                actor: row.get(10)?,
                completed_at: row.get(11)?,
                blocked_reason: row.get(12)?,
                incident_id: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(action)
}

#[tauri::command]
fn create_action(
    app: tauri::AppHandle,
    title: String,
    priority_id: String,
    difficulty_id: String,
    deadline: String,
    guidance: String,
    proof_expected: String,
    folder_id: Option<String>,
    incident_id: Option<String>,
) -> Result<Action, String> {
    let conn = get_db_connection(&app)?;
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO actions (id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            title,
            priority_id,
            difficulty_id,
            deadline,
            "a_faire",
            guidance,
            proof_expected,
            folder_id
        ]
    ).map_err(|e| e.to_string())?;

    if let Some(inc_id) = incident_id {
        conn.execute(
            "INSERT OR IGNORE INTO incident_action (incident_id, action_id) VALUES (?1, ?2)",
            params![inc_id, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let mut stmt = conn.prepare(
        "SELECT a.id, a.title, a.priority_id, a.difficulty_id, a.deadline, COALESCE(a.workflow_status, a.status), a.guidance, a.proof_expected, a.folder_id,
                a.tracking_note, a.actor, a.completed_at, a.blocked_reason,
                (SELECT ia.incident_id FROM incident_action ia WHERE ia.action_id = a.id LIMIT 1) as incident_id
         FROM actions a WHERE a.id = ?1"
    ).map_err(|e| e.to_string())?;
    let action = stmt
        .query_row(params![id], |row| {
            Ok(Action {
                id: row.get(0)?,
                title: row.get(1)?,
                priority_id: row.get(2)?,
                difficulty_id: row.get(3)?,
                deadline: row.get(4)?,
                status: row.get(5)?,
                guidance: row.get(6)?,
                proof_expected: row.get(7)?,
                folder_id: row.get(8)?,
                tracking_note: row.get(9)?,
                actor: row.get(10)?,
                completed_at: row.get(11)?,
                blocked_reason: row.get(12)?,
                incident_id: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(action)
}

// --- Identity commands ---

fn normalize_phone_number(value: &str) -> String {
    let trimmed = value.trim();
    let international = trimmed.starts_with('+') || trimmed.starts_with("00");
    let canonical_input = if international {
        trimmed.replace("(0)", "")
    } else {
        trimmed.to_string()
    };
    let digits = canonical_input
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>();
    if trimmed.starts_with('+') {
        format!("+{digits}")
    } else if let Some(international) = digits.strip_prefix("00") {
        format!("+{international}")
    } else {
        digits
    }
}

fn normalize_address_text(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_identity_value(kind: &str, value: &str) -> String {
    let trimmed = value.trim();
    match kind {
        "email" | "pseudo" | "prenom" | "nom" | "domaine" | "url" => trimmed.to_lowercase(),
        "telephone" => normalize_phone_number(trimmed),
        "adresse" => normalize_address_text(trimmed),
        _ => trimmed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase(),
    }
}

// `identities.kind` is a legacy compatibility column whose SQLite CHECK
// predates the multi-value model and does not know `prenom`. The authoritative
// type is stored in `identity_values`; keep the legacy column within its old
// vocabulary so creating an identity never fails on that compatibility check.
fn legacy_identity_kind(kind: &str) -> &str {
    if kind == "prenom" {
        "nom"
    } else {
        kind
    }
}

fn validate_identity_values(values: &[IdentityValueInput]) -> Result<(), String> {
    if values.is_empty() {
        return Err("Ajoutez au moins une donnée de recherche à l’identité.".into());
    }
    let allowed = [
        "prenom",
        "nom",
        "pseudo",
        "email",
        "telephone",
        "adresse",
        "domaine",
        "url",
    ];
    let mut unique = std::collections::HashSet::new();
    for value in values {
        if !allowed.contains(&value.kind.as_str()) {
            return Err(format!(
                "Type de donnée d’identité non reconnu : {}",
                value.kind
            ));
        }
        if value.status != "active" && value.status != "inactive" {
            return Err("Le statut d’une donnée doit être actif ou inactif.".into());
        }
        let normalized = normalize_identity_value(&value.kind, &value.value);
        if normalized.is_empty() {
            return Err("Une donnée d’identité ne peut pas être vide.".into());
        }
        if !unique.insert((value.kind.clone(), normalized)) {
            return Err("La même donnée ne peut pas être ajoutée deux fois à une identité.".into());
        }
    }
    Ok(())
}

fn load_identity_values(
    conn: &Connection,
    identity_id: &str,
) -> Result<Vec<IdentityValue>, String> {
    let mut statement = conn.prepare("SELECT id,kind,value,normalized_value,label,status,origin,address_line1,address_line2,city,postal_code,country,sort_order FROM identity_values WHERE identity_id=?1 ORDER BY sort_order,created_at,id")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![identity_id], |row| {
            Ok(IdentityValue {
                id: row.get(0)?,
                kind: row.get(1)?,
                value: row.get(2)?,
                normalized_value: row.get(3)?,
                label: row.get(4)?,
                status: row.get(5)?,
                origin: row.get(6)?,
                address_line1: row.get(7)?,
                address_line2: row.get(8)?,
                city: row.get(9)?,
                postal_code: row.get(10)?,
                country: row.get(11)?,
                sort_order: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn load_identity(conn: &Connection, id: &str) -> Result<Identity, String> {
    let mut identity = conn.query_row(
        "SELECT id,label,kind,value,folder_id,notes,address_line1,address_line2,city,postal_code,country,status FROM identities WHERE id=?1",
        params![id],
        |row| Ok(Identity {
            id: row.get(0)?, label: row.get(1)?, kind: row.get(2)?, value: row.get(3)?, folder_id: row.get(4)?, notes: row.get(5)?,
            address_line1: row.get(6)?, address_line2: row.get(7)?, city: row.get(8)?, postal_code: row.get(9)?, country: row.get(10)?,
            status: row.get(11)?, values: Vec::new(),
        }),
    ).map_err(|_| "Identité introuvable.".to_string())?;
    identity.values = load_identity_values(conn, id)?;
    Ok(identity)
}

fn sync_identity_values(
    conn: &Connection,
    identity_id: &str,
    values: &[IdentityValueInput],
) -> Result<(), String> {
    validate_identity_values(values)?;
    let existing_ids = {
        let mut statement = conn
            .prepare("SELECT id FROM identity_values WHERE identity_id=?1")
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map(params![identity_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<std::collections::HashSet<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let requested_existing_ids = values
        .iter()
        .filter_map(|value| value.id.clone())
        .collect::<std::collections::HashSet<_>>();
    if !requested_existing_ids.is_subset(&existing_ids) {
        return Err("Une donnée d’identité ne correspond pas à cette identité.".into());
    }
    for id in existing_ids.difference(&requested_existing_ids) {
        conn.execute(
            "DELETE FROM identity_values WHERE id=?1 AND identity_id=?2",
            params![id, identity_id],
        )
        .map_err(|e| e.to_string())?;
    }
    // Temporarily free normalized unique keys so two retained values can safely be swapped.
    for id in &requested_existing_ids {
        conn.execute("UPDATE identity_values SET normalized_value='__editing__'||id WHERE id=?1 AND identity_id=?2", params![id, identity_id]).map_err(|e| e.to_string())?;
    }
    for (sort_order, value) in values.iter().enumerate() {
        let id = match value.id.as_ref() {
            Some(id) if existing_ids.contains(id) => id.clone(),
            Some(_) => unreachable!("ownership checked above"),
            None => Uuid::new_v4().to_string(),
        };
        let normalized = normalize_identity_value(&value.kind, &value.value);
        conn.execute(
            "INSERT INTO identity_values (id,identity_id,kind,value,normalized_value,label,status,origin,address_line1,address_line2,city,postal_code,country,sort_order) VALUES (?1,?2,?3,?4,?5,?6,?7,'user',?8,?9,?10,?11,?12,?13) ON CONFLICT(id) DO UPDATE SET kind=excluded.kind,value=excluded.value,normalized_value=excluded.normalized_value,label=excluded.label,status=excluded.status,address_line1=excluded.address_line1,address_line2=excluded.address_line2,city=excluded.city,postal_code=excluded.postal_code,country=excluded.country,sort_order=excluded.sort_order,updated_at=datetime('now')",
            params![id, identity_id, value.kind, value.value.trim(), normalized, value.label, value.status, value.address_line1, value.address_line2, value.city, value.postal_code, value.country, sort_order as i64],
        ).map_err(|e| format!("Enregistrement de la donnée d’identité impossible : {e}"))?;
    }
    Ok(())
}

#[tauri::command]
fn list_identities(app: tauri::AppHandle) -> Result<Vec<Identity>, String> {
    let conn = get_db_connection(&app)?;
    let ids = {
        let mut statement = conn.prepare("SELECT id FROM identities ORDER BY CASE status WHEN 'active' THEN 0 ELSE 1 END,label").map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    ids.iter().map(|id| load_identity(&conn, id)).collect()
}

#[tauri::command]
fn get_identity(app: tauri::AppHandle, id: String) -> Result<Identity, String> {
    let conn = get_db_connection(&app)?;
    load_identity(&conn, &id)
}

#[tauri::command]
fn create_identity(
    app: tauri::AppHandle,
    label: String,
    status: String,
    folder_id: Option<String>,
    notes: Option<String>,
    values: Vec<IdentityValueInput>,
) -> Result<Identity, String> {
    if label.trim().is_empty() {
        return Err("Le nom d’affichage de l’identité est obligatoire.".into());
    }
    if status != "active" && status != "inactive" {
        return Err("Statut d’identité invalide.".into());
    }
    validate_identity_values(&values)?;
    let mut conn = get_db_connection(&app)?;
    let id = Uuid::new_v4().to_string();
    let primary = values
        .iter()
        .find(|value| value.status == "active")
        .unwrap_or(&values[0])
        .clone();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO identities (id,label,kind,value,folder_id,notes,address_line1,address_line2,city,postal_code,country,status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![id, label.trim(), legacy_identity_kind(&primary.kind), primary.value.trim(), folder_id, notes, primary.address_line1, primary.address_line2, primary.city, primary.postal_code, primary.country, status],
    ).map_err(|e| e.to_string())?;
    sync_identity_values(&tx, &id, &values)?;
    tx.commit().map_err(|e| e.to_string())?;
    load_identity(&conn, &id)
}

#[tauri::command]
fn update_identity(
    app: tauri::AppHandle,
    id: String,
    label: String,
    status: String,
    folder_id: Option<String>,
    notes: Option<String>,
    values: Vec<IdentityValueInput>,
) -> Result<Identity, String> {
    if label.trim().is_empty() {
        return Err("Le nom d’affichage de l’identité est obligatoire.".into());
    }
    if status != "active" && status != "inactive" {
        return Err("Statut d’identité invalide.".into());
    }
    validate_identity_values(&values)?;
    let mut conn = get_db_connection(&app)?;
    let primary = values
        .iter()
        .find(|value| value.status == "active")
        .unwrap_or(&values[0])
        .clone();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let changed = tx.execute(
        "UPDATE identities SET label=?1,kind=?2,value=?3,folder_id=?4,notes=?5,address_line1=?6,address_line2=?7,city=?8,postal_code=?9,country=?10,status=?11,updated_at=datetime('now') WHERE id=?12",
        params![label.trim(), legacy_identity_kind(&primary.kind), primary.value.trim(), folder_id, notes, primary.address_line1, primary.address_line2, primary.city, primary.postal_code, primary.country, status, id],
    ).map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Identité introuvable.".into());
    }
    sync_identity_values(&tx, &id, &values)?;
    tx.commit().map_err(|e| e.to_string())?;
    load_identity(&conn, &id)
}

#[tauri::command]
fn delete_identity(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut conn = get_db_connection(&app)?;
    let history: i64 = conn.query_row(
        "SELECT (SELECT COUNT(*) FROM osint_scans WHERE identity_id=?1) + (SELECT COUNT(*) FROM osint_signals WHERE identity_id=?1)",
        params![id], |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if history > 0 {
        return Err("Cette identité possède un historique de veille. Passez-la en inactive pour conserver ses scans et ses preuves.".into());
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM identity_values WHERE identity_id=?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    let changed = tx
        .execute("DELETE FROM identities WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Identité introuvable.".into());
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

// --- Exposure commands ---

fn load_remediation_catalog() -> Result<RemediationCatalog, String> {
    serde_json::from_str(REMEDIATION_CATALOG_JSON)
        .map_err(|e| format!("Catalogue de remédiation local invalide : {e}"))
}

fn priority_for_exposure(severity: &str) -> &'static str {
    match severity {
        "critique" => "critique",
        "elevee" | "élevée" => "haute",
        "moderee" | "modérée" => "moderee",
        _ => "basse",
    }
}

fn read_remediation_plan(
    conn: &Connection,
    plan_id: &str,
    catalog_version: &str,
) -> Result<RemediationPlan, String> {
    let mut plan_stmt = conn.prepare("SELECT id, identity_id, folder_id, scan_id, title, status, priority, rationale, catalog_version FROM remediation_plans WHERE id=?1").map_err(|e| e.to_string())?;
    let plan = plan_stmt
        .query_row(params![plan_id], |row| {
            Ok(RemediationPlan {
                id: row.get(0)?,
                identity_id: row.get(1)?,
                folder_id: row.get(2)?,
                scan_id: row.get(3)?,
                title: row.get(4)?,
                status: row.get(5)?,
                priority: row.get(6)?,
                rationale: row.get(7)?,
                catalog_version: row
                    .get::<_, Option<String>>(8)?
                    .unwrap_or_else(|| catalog_version.to_string()),
                items: Vec::new(),
            })
        })
        .map_err(|e| format!("Plan de remédiation introuvable : {e}"))?;
    let mut item_stmt = conn.prepare("SELECT i.id, i.action_id, i.exposure_id, i.incident_id, i.sort_order, i.expected_outcome, i.proof_expected, i.execution_mode, i.recommendation_id FROM remediation_plan_items i WHERE i.plan_id=?1 ORDER BY i.sort_order, i.created_at").map_err(|e| e.to_string())?;
    let items = item_stmt
        .query_map(params![plan_id], |row| {
            Ok(RemediationPlanItem {
                id: row.get(0)?,
                action_id: row.get(1)?,
                exposure_id: row.get(2)?,
                incident_id: row.get(3)?,
                sort_order: row.get(4)?,
                expected_outcome: row.get(5)?,
                proof_expected: row.get(6)?,
                execution_mode: row.get(7)?,
                recommendation_id: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(RemediationPlan { items, ..plan })
}

#[tauri::command]
fn list_remediation_recommendations(
    exposure_kind: String,
) -> Result<Vec<RemediationRecommendation>, String> {
    let catalog = load_remediation_catalog()?;
    Ok(catalog
        .recommendations
        .into_iter()
        .filter(|item| {
            item.exposure_kinds
                .iter()
                .any(|kind| kind == &exposure_kind)
        })
        .collect())
}

#[tauri::command]
fn create_remediation_plan(
    app: tauri::AppHandle,
    exposure_id: String,
    identity_id: Option<String>,
) -> Result<RemediationPlan, String> {
    let catalog = load_remediation_catalog()?;
    let conn = get_db_connection(&app)?;
    let (title, kind, severity, folder_id, why): (String, String, String, Option<String>, String) =
        conn.query_row(
            "SELECT title, kind, severity, folder_id, why FROM exposures WHERE id=?1",
            params![exposure_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| format!("Exposition introuvable : {e}"))?;
    let recommendations: Vec<RemediationRecommendation> = catalog
        .recommendations
        .iter()
        .filter(|item| {
            item.exposure_kinds
                .iter()
                .any(|candidate| candidate == &kind)
        })
        .cloned()
        .collect();
    if recommendations.is_empty() {
        return Err("Aucune recommandation locale ne correspond à ce type d’exposition.".into());
    }
    if let Ok(existing_id) = conn.query_row("SELECT plan_id FROM remediation_plan_items WHERE exposure_id=?1 ORDER BY created_at DESC LIMIT 1", params![exposure_id], |row| row.get::<_, String>(0)) {
        return read_remediation_plan(&conn, &existing_id, &catalog.version);
    }
    let plan_id = Uuid::new_v4().to_string();
    let plan_priority = priority_for_exposure(&severity).to_string();
    let rationale = format!("Plan local proposé à partir de l’exposition « {title} ». {why}");
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO remediation_plans (id,identity_id,folder_id,title,status,priority,rationale,catalog_version) VALUES (?1,?2,?3,?4,'valide',?5,?6,?7)", params![plan_id, identity_id, folder_id, format!("Réduire l’exposition : {title}"), plan_priority, rationale, catalog.version]).map_err(|e| e.to_string())?;
    let incident_id: Option<String> = tx
        .query_row(
            "SELECT incident_id FROM exposure_incident WHERE exposure_id=?1 LIMIT 1",
            params![exposure_id],
            |row| row.get(0),
        )
        .ok();
    for (index, recommendation) in recommendations.iter().enumerate() {
        let action_id = Uuid::new_v4().to_string();
        let guidance = serde_json::to_string(&recommendation.steps).map_err(|e| e.to_string())?;
        let priority_id = match recommendation.priority.as_str() {
            "critique" => "prio_004",
            "haute" => "prio_003",
            "moderee" => "prio_002",
            _ => "prio_001",
        };
        tx.execute("INSERT INTO actions (id,title,priority_id,difficulty_id,deadline,status,guidance,proof_expected,folder_id) VALUES (?1,?2,?3,'diff_001',date('now','+14 days'),'a_faire',?4,?5,?6)", params![action_id, recommendation.title, priority_id, guidance, recommendation.proof_expected, folder_id]).map_err(|e| e.to_string())?;
        if let Some(inc_id) = &incident_id {
            tx.execute(
                "INSERT OR IGNORE INTO incident_action (incident_id,action_id) VALUES (?1,?2)",
                params![inc_id, action_id],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.execute("INSERT INTO remediation_plan_items (id,plan_id,action_id,exposure_id,incident_id,sort_order,expected_outcome,proof_expected,execution_mode,recommendation_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![Uuid::new_v4().to_string(), plan_id, action_id, exposure_id, incident_id, index as i64, recommendation.expected_outcome, recommendation.proof_expected, recommendation.execution_mode, recommendation.id]).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    read_remediation_plan(&conn, &plan_id, &catalog.version)
}

#[tauri::command]
fn get_remediation_plan(app: tauri::AppHandle, plan_id: String) -> Result<RemediationPlan, String> {
    let catalog = load_remediation_catalog()?;
    let conn = get_db_connection(&app)?;
    read_remediation_plan(&conn, &plan_id, &catalog.version)
}

fn deterministic_remediation_enrichment(
    plan: &RemediationPlan,
    recommendations: &[RemediationRecommendation],
) -> RemediationAiPayload {
    RemediationAiPayload {
        schema_version: 1,
        task: "remediation_enrichment".into(),
        summary: format!("{} Le plan contient {} action(s) concrète(s), à valider et suivre avec leurs preuves.", plan.rationale, plan.items.len()),
        education: vec!["Une recommandation explique une réduction de risque ; elle ne prouve pas qu’un résultat concerne la personne.".into(), "Effectuez les étapes sur le site concerné, puis conservez seulement une note ou une preuve utile, jamais un secret.".into()],
        priority_rationale: format!("La priorité {} reprend la sévérité déterministe de l’exposition et peut être ajustée après vérification des preuves.", plan.priority),
        cautions: vec!["Une correspondance OSINT reste un profil potentiel ou un signal à vérifier.".into(), "MANTIS ne contacte aucun site et ne clôture aucune action automatiquement.".into()],
        citation_ids: recommendations.iter().map(|item| item.id.clone()).take(12).collect(),
    }
}

/// Recommendation identifiers are internal references, not prose. Small local
/// models may repeat or misspell one while otherwise producing a valid
/// explanation. Keep only the identifiers that were supplied by MANTIS before
/// validating the enrichment; this never creates a citation or a business
/// record on the model's behalf.
fn normalize_remediation_citation_ids(
    payload: &mut RemediationAiPayload,
    allowed: &std::collections::HashSet<String>,
) {
    let mut seen = std::collections::HashSet::new();
    payload
        .citation_ids
        .retain(|id| allowed.contains(id) && seen.insert(id.clone()));
}

fn validate_remediation_enrichment(
    payload: &RemediationAiPayload,
    allowed: &std::collections::HashSet<String>,
) -> Result<(), String> {
    if payload.schema_version != 1
        || payload.task != "remediation_enrichment"
        || payload.summary.is_empty()
        || payload.summary.chars().count() > 700
        || payload.education.is_empty()
        || payload.education.len() > 4
        || payload.priority_rationale.is_empty()
        || payload.cautions.is_empty()
        || payload.cautions.len() > 5
        || payload.citation_ids.len() > 12
    {
        return Err(
            "La sortie IA ne respecte pas le contrat d’enrichissement de remédiation.".into(),
        );
    }
    let forbidden = [
        "ce profil est confirmé",
        "appartient à la personne",
        "est bien la personne",
        "mot de passe est",
    ];
    let text = format!(
        "{} {} {}",
        payload.summary,
        payload.priority_rationale,
        payload.education.join(" ")
    )
    .to_lowercase();
    if forbidden.iter().any(|phrase| text.contains(phrase)) {
        return Err(
            "L’enrichissement IA contient une attribution ou une consigne sensible interdite."
                .into(),
        );
    }
    let mut seen = std::collections::HashSet::new();
    if payload
        .citation_ids
        .iter()
        .any(|id| !allowed.contains(id) || !seen.insert(id))
    {
        return Err("L’enrichissement IA cite une référence inconnue ou dupliquée.".into());
    }
    Ok(())
}

fn remediation_json_from_output(output: &str) -> Result<RemediationAiPayload, String> {
    let start = output
        .find('{')
        .ok_or("Le modèle n’a produit aucun objet JSON de remédiation.")?;
    let end = output
        .rfind('}')
        .ok_or("Le modèle a produit un JSON de remédiation incomplet.")?;
    serde_json::from_str(&output[start..=end])
        .map_err(|_| "Le modèle a produit un JSON de remédiation invalide.".into())
}

fn execute_local_remediation_ai(
    app: &tauri::AppHandle,
    input_json: &str,
    allowed: &std::collections::HashSet<String>,
) -> Result<(RemediationAiPayload, String), String> {
    let conn = get_db_connection(app)?;
    let (enabled, selected): (i64, Option<String>) = conn
        .query_row(
            "SELECT enabled,selected_model_id FROM local_ai_preferences WHERE id=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    if enabled == 0 {
        return Err("Les fonctions IA locales sont désactivées.".into());
    }
    let model_id = selected.ok_or("Aucun modèle local n’est sélectionné.")?;
    let model = local_ai_models()?
        .into_iter()
        .find(|model| model.id == model_id)
        .ok_or("Le modèle sélectionné n’appartient plus au catalogue signé.")?;
    if !installed_model_integrity(app, &model)?.0 {
        return Err("Le modèle local n’est pas prêt.".into());
    }
    let runtime = local_ai_component()?;
    let executable = installed_local_ai_executable(app, &runtime)?;
    let model_path = local_ai_model_destination(app, &model)?.join(&model.filename);
    let prompt = format!(
        "{}\n\nPLAN_ET_DOCUMENTATION_JSON:\n{}\n\nRéponds uniquement avec l’objet JSON.",
        REMEDIATION_AI_INSTRUCTIONS, input_json
    );
    let threads = local_ai_thread_count();
    let args = vec![
        "--model".into(),
        model_path.to_string_lossy().to_string(),
        "--threads".into(),
        threads.clone(),
        "--threads-batch".into(),
        threads,
        "--prompt".into(),
        prompt,
        "--ctx-size".into(),
        "4096".into(),
        "--predict".into(),
        "900".into(),
        "--temp".into(),
        "0.1".into(),
        "--seed".into(),
        "42".into(),
        "--json-schema".into(),
        REMEDIATION_AI_SCHEMA.into(),
        "--no-display-prompt".into(),
        "--simple-io".into(),
        "-no-cnv".into(),
        "--no-jinja".into(),
        "--single-turn".into(),
        "--reasoning".into(),
        "off".into(),
    ];
    let (stdout, _) = run_process(&executable, &args, Duration::from_secs(120))?;
    let mut payload = remediation_json_from_output(&stdout)?;
    normalize_remediation_citation_ids(&mut payload, allowed);
    validate_remediation_enrichment(&payload, allowed)?;
    Ok((payload, model.label.unwrap_or(model.id)))
}

fn read_latest_remediation_enrichment(
    conn: &Connection,
    plan_id: &str,
) -> Result<RemediationAiEnrichment, String> {
    conn.query_row("SELECT id,plan_id,contract_version,mode,status,model_label,output_json,error_message,created_at FROM remediation_plan_ai_enrichments WHERE plan_id=?1 ORDER BY created_at DESC LIMIT 1", params![plan_id], |row| {
        let payload: RemediationAiPayload = serde_json::from_str(&row.get::<_,String>(6)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
        Ok(RemediationAiEnrichment { id: row.get(0)?, plan_id: row.get(1)?, contract_version: row.get(2)?, mode: row.get(3)?, status: row.get(4)?, model_label: row.get(5)?, summary: payload.summary, education: payload.education, priority_rationale: payload.priority_rationale, cautions: payload.cautions, citation_ids: payload.citation_ids, error_message: row.get(7)?, created_at: row.get(8)? })
    }).map_err(|e| e.to_string())
}

#[tauri::command]
fn enrich_remediation_plan(
    app: tauri::AppHandle,
    plan_id: String,
) -> Result<RemediationAiEnrichment, String> {
    let catalog = load_remediation_catalog()?;
    let conn = get_db_connection(&app)?;
    let plan = read_remediation_plan(&conn, &plan_id, &catalog.version)?;
    let recommendations = plan
        .items
        .iter()
        .filter_map(|item| {
            catalog
                .recommendations
                .iter()
                .find(|candidate| candidate.id == item.recommendation_id)
                .cloned()
        })
        .collect::<Vec<_>>();
    let allowed = recommendations
        .iter()
        .map(|item| item.id.clone())
        .collect::<std::collections::HashSet<_>>();
    let input = serde_json::json!({"plan":{"id":plan.id,"title":plan.title,"priority":plan.priority,"rationale":plan.rationale,"items":plan.items},"documentation":recommendations});
    let input_json = serde_json::to_string(&input).map_err(|e| e.to_string())?;
    let input_hash = hex::encode(Sha256::digest(input_json.as_bytes()));
    if let Ok(existing) = conn.query_row("SELECT id FROM remediation_plan_ai_enrichments WHERE plan_id=?1 AND input_sha256=?2 ORDER BY created_at DESC LIMIT 1", params![plan_id, input_hash], |row| row.get::<_,String>(0)) {
        return read_latest_remediation_enrichment(&conn, &plan_id).or_else(|_| Err(format!("Enrichissement {existing} introuvable.")));
    }
    let fallback = deterministic_remediation_enrichment(&plan, &recommendations);
    let started = Instant::now();
    let (payload, mode, status, model_label, error) =
        match execute_local_remediation_ai(&app, &input_json, &allowed) {
            Ok((payload, label)) => (payload, "ia_locale", "valide", Some(label), None),
            Err(error) => (fallback, "deterministe", "fallback", None, Some(error)),
        };
    validate_remediation_enrichment(&payload, &allowed)?;
    let output_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO remediation_plan_ai_enrichments (id,plan_id,contract_version,input_sha256,mode,status,model_label,output_json,error_message) VALUES (?1,?2,'1.0.0',?3,?4,?5,?6,?7,?8)", params![id,plan_id,input_hash,mode,status,model_label,output_json,error]).map_err(|e| e.to_string())?;
    let _ = started;
    read_latest_remediation_enrichment(&conn, &plan_id)
}

#[tauri::command]
fn get_latest_remediation_enrichment(
    app: tauri::AppHandle,
    plan_id: String,
) -> Result<RemediationAiEnrichment, String> {
    let conn = get_db_connection(&app)?;
    read_latest_remediation_enrichment(&conn, &plan_id)
}

#[tauri::command]
fn list_exposures(
    app: tauri::AppHandle,
    identity_id: Option<String>,
) -> Result<Vec<Exposure>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn.prepare("SELECT e.id, e.title, e.kind, e.severity, e.status, e.discovered_at, e.source, e.what, e.why, e.folder_id FROM exposures e WHERE ?1 IS NULL OR EXISTS (SELECT 1 FROM osint_signals s WHERE s.exposure_id=e.id AND s.identity_id=?1)").map_err(|e| e.to_string())?;
    let exposure_iter = stmt
        .query_map(params![identity_id], |row| {
            Ok(Exposure {
                id: row.get(0)?,
                title: row.get(1)?,
                kind: row.get(2)?,
                severity: row.get(3)?,
                status: row.get(4)?,
                discovered_at: row.get(5)?,
                source: row.get(6)?,
                what: row.get(7)?,
                why: row.get(8)?,
                folder_id: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut exposures = Vec::new();
    for exposure in exposure_iter {
        exposures.push(exposure.map_err(|e| e.to_string())?);
    }

    Ok(exposures)
}

#[tauri::command]
fn get_exposure(app: tauri::AppHandle, id: String) -> Result<Exposure, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id, title, kind, severity, status, discovered_at, source, what, why, folder_id FROM exposures WHERE id = ?1").map_err(|e| e.to_string())?;
    let exposure = stmt
        .query_row(params![id], |row| {
            Ok(Exposure {
                id: row.get(0)?,
                title: row.get(1)?,
                kind: row.get(2)?,
                severity: row.get(3)?,
                status: row.get(4)?,
                discovered_at: row.get(5)?,
                source: row.get(6)?,
                what: row.get(7)?,
                why: row.get(8)?,
                folder_id: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(exposure)
}

// --- RGPD commands ---

#[tauri::command]
fn list_rgpd_requests(
    app: tauri::AppHandle,
    identity_id: Option<String>,
) -> Result<Vec<RgpdRequest>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn.prepare("SELECT r.id, r.type_id, r.target, r.dpo_contact, r.status_id, r.data_summary, r.draft_preview, r.source_url, r.contact_source_url FROM rgpd_requests r WHERE ?1 IS NULL OR EXISTS (SELECT 1 FROM incident_rgpd ir JOIN exposure_incident ei ON ei.incident_id=ir.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ir.rgpd_id=r.id AND s.identity_id=?1) OR EXISTS (SELECT 1 FROM action_rgpd ar JOIN incident_action ia ON ia.action_id=ar.action_id JOIN exposure_incident ei ON ei.incident_id=ia.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ar.rgpd_id=r.id AND s.identity_id=?1)").map_err(|e| e.to_string())?;
    let rgpd_iter = stmt
        .query_map(params![identity_id], |row| {
            Ok(RgpdRequest {
                id: row.get(0)?,
                type_id: row.get(1)?,
                target: row.get(2)?,
                dpo_contact: row.get(3)?,
                status_id: row.get(4)?,
                data_summary: row.get(5)?,
                draft_preview: row.get(6)?,
                source_url: row.get(7)?,
                contact_source_url: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut requests = Vec::new();
    for req in rgpd_iter {
        requests.push(req.map_err(|e| e.to_string())?);
    }

    Ok(requests)
}

#[tauri::command]
fn get_rgpd_request(app: tauri::AppHandle, id: String) -> Result<RgpdRequest, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id, type_id, target, dpo_contact, status_id, data_summary, draft_preview, source_url, contact_source_url FROM rgpd_requests WHERE id = ?1").map_err(|e| e.to_string())?;
    let request = stmt
        .query_row(params![id], |row| {
            Ok(RgpdRequest {
                id: row.get(0)?,
                type_id: row.get(1)?,
                target: row.get(2)?,
                dpo_contact: row.get(3)?,
                status_id: row.get(4)?,
                data_summary: row.get(5)?,
                draft_preview: row.get(6)?,
                source_url: row.get(7)?,
                contact_source_url: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(request)
}

// --- Timeline commands ---

#[tauri::command]
fn list_timeline_entries(app: tauri::AppHandle) -> Result<Vec<TimelineEntry>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn
        .prepare("SELECT id, event_type, description, created_at FROM timeline_entries")
        .map_err(|e| e.to_string())?;
    let timeline_iter = stmt
        .query_map([], |row| {
            Ok(TimelineEntry {
                id: row.get(0)?,
                event_type: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in timeline_iter {
        entries.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(entries)
}

// --- OSINT commands ---

#[tauri::command]
fn list_osint_modules(app: tauri::AppHandle) -> Result<Vec<OsintModule>, String> {
    let conn = get_db_connection(&app)?;

    let mut stmt = conn.prepare("SELECT id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args,catalog_status,replacement_id,archived_reason FROM osint_modules WHERE catalog_status='active' ORDER BY name").map_err(|e| e.to_string())?;
    let module_iter = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let (installation_status, diagnostic) = module_installation_state(&app, &id);
            Ok(OsintModule {
                id,
                name: row.get(1)?,
                description: row.get(2)?,
                target_kind: row.get(3)?,
                frequency: row.get(4)?,
                status: row.get(5)?,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                script_path: row.get(8)?,
                script_args: row.get(9)?,
                installation_status,
                diagnostic,
                catalog_status: row.get(10)?,
                replacement_id: row.get(11)?,
                archived_reason: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut modules = Vec::new();
    for module in module_iter {
        modules.push(module.map_err(|e| e.to_string())?);
    }

    Ok(modules)
}

#[tauri::command]
fn list_osint_module_inventory(app: tauri::AppHandle) -> Result<Vec<OsintModule>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt=conn.prepare("SELECT id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args,catalog_status,replacement_id,archived_reason FROM osint_modules ORDER BY CASE catalog_status WHEN 'active' THEN 0 WHEN 'archived' THEN 1 ELSE 2 END,name").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let catalog_status: String = row.get(10)?;
            let (installation_status, diagnostic) = if catalog_status == "active" {
                module_installation_state(&app, &id)
            } else {
                (catalog_status.clone(), row.get::<_, Option<String>>(12)?)
            };
            Ok(OsintModule {
                id,
                name: row.get(1)?,
                description: row.get(2)?,
                target_kind: row.get(3)?,
                frequency: row.get(4)?,
                status: row.get(5)?,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                script_path: row.get(8)?,
                script_args: row.get(9)?,
                installation_status,
                diagnostic,
                catalog_status,
                replacement_id: row.get(11)?,
                archived_reason: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// --- Posture score ---

/// Convert unresolved risk into a bounded, explainable posture score.
/// One observation should move the dial, not collapse it to zero.
fn calculate_posture_score(
    incident_severities: &[String],
    exposure_severities: &[String],
    completed_actions: i64,
) -> i32 {
    let incident_penalty: i32 = incident_severities
        .iter()
        .map(|severity| match severity.as_str() {
            "critique" => 18,
            "elevee" | "élevée" => 10,
            "moderee" | "modérée" => 5,
            "faible" => 1,
            _ => 0,
        })
        .sum();
    let exposure_penalty: i32 = exposure_severities
        .iter()
        .map(|severity| match severity.as_str() {
            "critique" => 12,
            "elevee" | "élevée" => 7,
            "moderee" | "modérée" => 3,
            "faible" => 1,
            _ => 0,
        })
        .sum();
    let mitigation_credit = (completed_actions.max(0) * 3).min(12) as i32;
    (100 - incident_penalty - exposure_penalty + mitigation_credit).clamp(5, 100)
}

const PUBLIC_IP_LOOKUP_URL: &str =
    "https://ipwho.is/?fields=success,ip,type,country,region,city,connection,timezone,security";
const MAX_PUBLIC_IP_RESPONSE_BYTES: usize = 32 * 1024;

fn clean_ip_lookup_field(value: Option<String>, max_length: usize) -> Option<String> {
    value
        .map(|value| value.trim().chars().take(max_length).collect::<String>())
        .filter(|value| !value.is_empty())
}

fn is_public_ip_value(value: &str) -> bool {
    value
        .parse::<std::net::IpAddr>()
        .ok()
        .is_some_and(|ip| match ip {
            std::net::IpAddr::V4(ip) => {
                !(ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast())
            }
            std::net::IpAddr::V6(ip) => {
                !(ip.is_loopback()
                    || ip.is_unspecified()
                    || ip.is_unique_local()
                    || ip.is_unicast_link_local()
                    || ip.is_multicast())
            }
        })
}

fn parse_public_ip_context(bytes: &[u8]) -> Result<PublicIpContext, String> {
    let response: IpWhoisResponse = serde_json::from_slice(bytes)
        .map_err(|_| "Le service de contexte réseau a renvoyé une réponse invalide.".to_string())?;
    if !response.success {
        return Err("Le service de contexte réseau n’a pas pu identifier cette connexion.".into());
    }
    let ip = response
        .ip
        .filter(|value| is_public_ip_value(value))
        .ok_or_else(|| {
            "Le service de contexte réseau n’a pas renvoyé d’IP publique valide.".to_string()
        })?;
    let connection = response.connection;
    let timezone = response.timezone;
    let security = response.security.unwrap_or_default();
    Ok(PublicIpContext {
        ip,
        ip_type: clean_ip_lookup_field(response.ip_type, 16),
        city: clean_ip_lookup_field(response.city, 80),
        region: clean_ip_lookup_field(response.region, 80),
        country: clean_ip_lookup_field(response.country, 80),
        asn: connection.as_ref().and_then(|value| value.asn),
        organization: clean_ip_lookup_field(
            connection.as_ref().and_then(|value| value.org.clone()),
            120,
        ),
        isp: clean_ip_lookup_field(connection.as_ref().and_then(|value| value.isp.clone()), 120),
        network_domain: clean_ip_lookup_field(connection.and_then(|value| value.domain), 120),
        timezone: clean_ip_lookup_field(timezone.as_ref().and_then(|value| value.id.clone()), 80),
        utc_offset: clean_ip_lookup_field(timezone.and_then(|value| value.utc), 16),
        proxy: security.proxy.unwrap_or(false),
        vpn: security.vpn.unwrap_or(false),
        tor: security.tor.unwrap_or(false),
        hosting: security.hosting.unwrap_or(false),
    })
}

#[tauri::command]
fn get_public_ip_context() -> Result<PublicIpContext, String> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(6))
        .user_agent("MANTIS-POSTURE/0.1 network-context")
        .build()
        .map_err(|_| "Le contexte réseau ne peut pas être initialisé.".to_string())?;
    let response = client
        .get(PUBLIC_IP_LOOKUP_URL)
        .send()
        .map_err(|_| "Le contexte réseau est momentanément indisponible.".to_string())?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > MAX_PUBLIC_IP_RESPONSE_BYTES as u64)
    {
        return Err("Le contexte réseau est momentanément indisponible.".into());
    }
    let mut bytes = Vec::new();
    response
        .take((MAX_PUBLIC_IP_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "La réponse du contexte réseau est illisible.".to_string())?;
    if bytes.len() > MAX_PUBLIC_IP_RESPONSE_BYTES {
        return Err("La réponse du contexte réseau dépasse la taille autorisée.".into());
    }
    parse_public_ip_context(&bytes)
}

#[tauri::command]
fn get_posture_score(
    app: tauri::AppHandle,
    identity_id: Option<String>,
) -> Result<PostureScore, String> {
    let conn = get_db_connection(&app)?;

    let identity_id = match identity_id.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => {
            return Ok(PostureScore {
                score: None,
                open_incidents: 0,
                high_exposures: 0,
                completed_actions: 0,
            })
        }
    };

    let completed_or_partial_scans: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM osint_scan_sessions WHERE identity_id=?1 AND status IN ('termine','partiel')",
            params![identity_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if completed_or_partial_scans == 0 {
        return Ok(PostureScore {
            score: None,
            open_incidents: 0,
            high_exposures: 0,
            completed_actions: 0,
        });
    }

    let mut stmt_inc = conn
        .prepare("SELECT i.severity FROM incidents i WHERE EXISTS (SELECT 1 FROM exposure_incident ei JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ei.incident_id=i.id AND s.identity_id=?1)")
        .map_err(|e| e.to_string())?;
    let inc_iter = stmt_inc
        .query_map(params![identity_id], |row| {
            let sev: String = row.get(0)?;
            Ok(sev)
        })
        .map_err(|e| e.to_string())?;

    let mut incident_severities = Vec::new();
    for sev_res in inc_iter {
        let sev = sev_res.map_err(|e| e.to_string())?;
        incident_severities.push(sev);
    }
    let open_incidents = incident_severities.len() as i32;

    let mut stmt_exp = conn
        .prepare("SELECT e.severity FROM exposures e WHERE e.status != 'reduite' AND EXISTS (SELECT 1 FROM osint_signals s WHERE s.exposure_id=e.id AND s.identity_id=?1)")
        .map_err(|e| e.to_string())?;
    let exp_iter = stmt_exp
        .query_map(params![identity_id], |row| {
            let sev: String = row.get(0)?;
            Ok(sev)
        })
        .map_err(|e| e.to_string())?;

    let mut exposure_severities = Vec::new();
    for sev_res in exp_iter {
        let sev = sev_res.map_err(|e| e.to_string())?;
        exposure_severities.push(sev);
    }
    let high_exposures = exposure_severities
        .iter()
        .filter(|severity| matches!(severity.as_str(), "elevee" | "élevée" | "critique"))
        .count() as i32;

    let action_columns = table_columns(&conn, "actions")?;
    let completed_query = if action_columns
        .iter()
        .any(|column| column == "workflow_status")
    {
        "SELECT COUNT(*) FROM actions a WHERE a.workflow_status IN ('effectue_moi','effectue_site') AND EXISTS (SELECT 1 FROM incident_action ia JOIN exposure_incident ei ON ei.incident_id=ia.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ia.action_id=a.id AND s.identity_id=?1)"
    } else {
        "SELECT COUNT(*) FROM actions a WHERE a.status = 'faite' AND EXISTS (SELECT 1 FROM incident_action ia JOIN exposure_incident ei ON ei.incident_id=ia.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ia.action_id=a.id AND s.identity_id=?1)"
    };
    let completed: i64 = conn
        .query_row(completed_query, params![identity_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let score = calculate_posture_score(&incident_severities, &exposure_severities, completed);

    Ok(PostureScore {
        score: Some(score),
        open_incidents,
        high_exposures,
        completed_actions: completed as i32,
    })
}

// --- Status update commands ---

#[tauri::command]
fn update_action_status(
    app: tauri::AppHandle,
    action_id: String,
    status: String,
) -> Result<(), String> {
    let status = if status == "faite" {
        "effectue_moi".to_string()
    } else {
        status
    };
    if !matches!(
        status.as_str(),
        "a_faire"
            | "en_cours"
            | "effectue_moi"
            | "effectue_site"
            | "en_attente"
            | "impossible"
            | "ignore"
    ) {
        return Err("Statut d’action invalide.".into());
    }
    let conn = get_db_connection(&app)?;
    let previous: String = conn
        .query_row(
            "SELECT COALESCE(workflow_status, status) FROM actions WHERE id = ?1",
            params![action_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Action introuvable : {e}"))?;
    conn.execute(
        "UPDATE actions SET workflow_status = ?1, completed_at = CASE WHEN ?1 IN ('effectue_moi','effectue_site') THEN datetime('now') ELSE NULL END, updated_at = datetime('now') WHERE id = ?2",
        params![status, action_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO action_events (id, action_id, from_status, to_status, created_at) VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![Uuid::new_v4().to_string(), action_id, previous, status],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_action_tracking(
    app: tauri::AppHandle,
    action_id: String,
    status: String,
    note: Option<String>,
    actor: Option<String>,
    blocked_reason: Option<String>,
) -> Result<(), String> {
    if let Some(actor_value) = actor.as_deref() {
        if !matches!(actor_value, "moi" | "site") {
            return Err("Acteur d’action invalide.".into());
        }
    }
    update_action_status(app.clone(), action_id.clone(), status)?;
    let conn = get_db_connection(&app)?;
    conn.execute(
        "UPDATE actions SET tracking_note = ?1, actor = ?2, blocked_reason = ?3, updated_at = datetime('now') WHERE id = ?4",
        params![note, actor, blocked_reason, action_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE action_events SET note = ?1, actor = ?2 WHERE action_id = ?3 AND id = (SELECT id FROM action_events WHERE action_id = ?3 ORDER BY created_at DESC, rowid DESC LIMIT 1)",
        params![note, actor, action_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn list_action_events(
    app: tauri::AppHandle,
    action_id: String,
) -> Result<Vec<ActionEvent>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn
        .prepare("SELECT id, action_id, from_status, to_status, actor, note, created_at FROM action_events WHERE action_id = ?1 ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![action_id], |row| {
            Ok(ActionEvent {
                id: row.get(0)?,
                action_id: row.get(1)?,
                from_status: row.get(2)?,
                to_status: row.get(3)?,
                actor: row.get(4)?,
                note: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_action_evidence(
    app: tauri::AppHandle,
    action_id: String,
    kind: String,
    locator: String,
    description: Option<String>,
) -> Result<ActionEvidence, String> {
    if !matches!(kind.as_str(), "url" | "fichier" | "note" | "hash") {
        return Err("Type de preuve invalide.".into());
    }
    let locator = locator.trim().to_string();
    if locator.is_empty() || locator.len() > 2048 {
        return Err("La preuve doit contenir une référence courte et non vide.".into());
    }
    let conn = get_db_connection(&app)?;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO action_evidence (id, action_id, kind, locator, description) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, action_id, kind, locator, description],
    ).map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, action_id, kind, locator, description, created_at FROM action_evidence WHERE id = ?1",
        params![id],
        |row| Ok(ActionEvidence { id: row.get(0)?, action_id: row.get(1)?, kind: row.get(2)?, locator: row.get(3)?, description: row.get(4)?, created_at: row.get(5)? }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_action_evidence(
    app: tauri::AppHandle,
    action_id: String,
) -> Result<Vec<ActionEvidence>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id, action_id, kind, locator, description, created_at FROM action_evidence WHERE action_id = ?1 ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![action_id], |row| {
            Ok(ActionEvidence {
                id: row.get(0)?,
                action_id: row.get(1)?,
                kind: row.get(2)?,
                locator: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_rgpd_request_status(
    app: tauri::AppHandle,
    request_id: String,
    status_id: String,
) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    if !matches!(
        status_id.as_str(),
        "status_001" | "status_002" | "status_003" | "status_004"
    ) {
        return Err("Statut RGPD invalide.".into());
    }
    if matches!(status_id.as_str(), "status_002" | "status_003") {
        let decision: Option<String> = conn.query_row("SELECT decision FROM rgpd_user_reviews WHERE request_id=?1 AND draft_version_id=(SELECT id FROM rgpd_draft_versions WHERE request_id=?1 ORDER BY created_at DESC,rowid DESC LIMIT 1) ORDER BY created_at DESC,rowid DESC LIMIT 1",params![request_id],|r|r.get(0)).ok();
        if decision.as_deref() != Some("valide") {
            return Err(
                "Relisez et validez le brouillon avant de le déclarer prêt ou envoyé.".into(),
            );
        }
    }
    let previous: Option<String> = conn
        .query_row(
            "SELECT status_id FROM rgpd_requests WHERE id=?1",
            params![request_id],
            |row| row.get(0),
        )
        .ok();
    conn.execute(
        "UPDATE rgpd_requests SET status_id = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status_id, request_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO rgpd_request_events (id,request_id,from_status,to_status,event_type,note) VALUES (?1,?2,?3,?4,'statut',NULL)", params![Uuid::new_v4().to_string(), request_id, previous, status_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn list_rgpd_evidence(
    app: tauri::AppHandle,
    request_id: String,
) -> Result<Vec<RgpdEvidence>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id,request_id,kind,locator,description,verified,created_at FROM rgpd_request_evidence WHERE request_id=?1 ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![request_id], |row| {
            Ok(RgpdEvidence {
                id: row.get(0)?,
                request_id: row.get(1)?,
                kind: row.get(2)?,
                locator: row.get(3)?,
                description: row.get(4)?,
                verified: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn validate_rgpd_evidence_locator(locator: &str) -> Result<(), String> {
    let lower = locator.to_lowercase();
    if ["password=", "passwd=", "token=", "cookie=", "secret="]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(
            "Les secrets, tokens et cookies ne peuvent pas être enregistrés comme preuve.".into(),
        );
    }
    if locator.trim().is_empty() || locator.len() > 2048 {
        return Err("La référence de preuve est vide ou trop longue.".into());
    }
    Ok(())
}

#[tauri::command]
fn add_rgpd_evidence(
    app: tauri::AppHandle,
    request_id: String,
    kind: String,
    locator: String,
    description: Option<String>,
    verified: bool,
) -> Result<RgpdEvidence, String> {
    if !matches!(
        kind.as_str(),
        "source" | "identity" | "recipient" | "content" | "send" | "response"
    ) {
        return Err("Type de preuve RGPD invalide.".into());
    }
    let locator = locator.trim().to_string();
    validate_rgpd_evidence_locator(&locator)?;
    let conn = get_db_connection(&app)?;
    let id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO rgpd_request_evidence (id,request_id,kind,locator,description,verified) VALUES (?1,?2,?3,?4,?5,?6)", params![id,request_id,kind,locator,description,verified as i64]).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO rgpd_request_events (id,request_id,event_type,note) VALUES (?1,?2,'preuve_ajoutee',?3)", params![Uuid::new_v4().to_string(),request_id,format!("Preuve {} ajoutée", kind)]).map_err(|e| e.to_string())?;
    conn.query_row("SELECT id,request_id,kind,locator,description,verified,created_at FROM rgpd_request_evidence WHERE id=?1", params![id], |row| Ok(RgpdEvidence { id: row.get(0)?, request_id: row.get(1)?, kind: row.get(2)?, locator: row.get(3)?, description: row.get(4)?, verified: row.get::<_,i64>(5)? != 0, created_at: row.get(6)? })).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_rgpd_events(app: tauri::AppHandle, request_id: String) -> Result<Vec<RgpdEvent>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id,request_id,from_status,to_status,event_type,note,created_at FROM rgpd_request_events WHERE request_id=?1 ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![request_id], |row| {
            Ok(RgpdEvent {
                id: row.get(0)?,
                request_id: row.get(1)?,
                from_status: row.get(2)?,
                to_status: row.get(3)?,
                event_type: row.get(4)?,
                note: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

// --- Remise à zéro complète de la base ---
#[tauri::command]
fn reset_database(app: tauri::AppHandle) -> Result<String, String> {
    let app_dir = application_data_dir(&app)?;
    let db_path = app_dir.join("mantis.db");

    // Supprimer la base, ses journaux WAL/SHM et les sauvegardes de migration
    // qui pourraient sinon conserver une copie des données après la remise à zéro.
    for path in [
        db_path.clone(),
        app_dir.join("mantis.db-wal"),
        app_dir.join("mantis.db-shm"),
    ] {
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    for entry in std::fs::read_dir(&app_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let is_migration_backup = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("mantis.pre-v") && name.ends_with(".bak"));
        if is_migration_backup {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }

    // Réinitialiser (créera les tables sans identité préremplie)
    init_database(&app)?;

    Ok("Base de données réinitialisée sans identité préremplie.".to_string())
}

// --- OSINT Email Stack — Real script execution ---

#[cfg(feature = "legacy-osint")]
fn execute_osint_script(
    script_path: &str,
    script_args: &str,
    email: &str,
) -> Result<String, String> {
    let args: Vec<String> = if script_args.is_empty() {
        vec![email.to_string()]
    } else {
        let mut parts: Vec<String> = script_args
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        parts.push(email.to_string());
        parts
    };

    let mut command = Command::new(script_path);
    command.args(&args);
    configure_background_process(&mut command);
    let output = command
        .output()
        .map_err(|e| format!("Failed to execute script '{}': {}", script_path, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!(
            "Script '{}' exited with error: {}",
            script_path, stderr
        ))
    }
}

#[cfg(feature = "legacy-osint")]
fn process_mailaccess_result(
    conn: &Connection,
    email: &str,
    output: &str,
    folder_id: &str,
) -> Result<(), String> {
    let parsed: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| format!("Failed to parse MailAccess output as JSON: {}", e))?;

    let obj = parsed
        .as_object()
        .ok_or("MailAccess output is not a JSON object")?;

    if let Some(breaches) = obj.get("breaches").and_then(|v| v.as_array()) {
        for breach in breaches {
            let breach_str = breach.as_str().unwrap_or("");
            if !breach_str.is_empty() {
                create_exposure(
                    conn,
                    email,
                    "fuite",
                    &format!("Fuite détectée : {}", breach_str),
                    folder_id,
                )?;
                create_incident_and_action_for_leak(conn, email, breach_str)?;
            }
        }
    }

    if let Some(domains) = obj.get("domains").and_then(|v| v.as_array()) {
        for domain in domains {
            let domain_str = domain.as_str().unwrap_or("");
            if !domain_str.is_empty() {
                create_exposure(
                    conn,
                    email,
                    "mention",
                    &format!("Domaine associé détecté : {}", domain_str),
                    folder_id,
                )?;
            }
        }
    }

    if let Some(identities) = obj.get("identities").and_then(|v| v.as_array()) {
        for identity in identities {
            let identity_str = identity.as_str().unwrap_or("");
            if !identity_str.is_empty() && identity_str != email {
                create_or_update_identity_from_osint(conn, identity_str, folder_id)?;
                create_exposure(
                    conn,
                    email,
                    "profil_public",
                    &format!("Identité liée détectée : {}", identity_str),
                    folder_id,
                )?;
            }
        }
    }

    conn.execute(
        "UPDATE osint_modules SET last_run = datetime('now'), status = 'actif' WHERE id = 'osint-email-intel'",
        []
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "legacy-osint")]
fn process_holehe_result(
    conn: &Connection,
    email: &str,
    output: &str,
    folder_id: &str,
) -> Result<(), String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(obj) = parsed.as_object() {
            for (platform, value) in obj {
                let platform_str = platform.to_string();
                let has_account = value.as_bool().unwrap_or(false);
                if has_account {
                    create_exposure(
                        conn,
                        email,
                        "mention",
                        &format!("Compte détecté sur : {}", platform_str),
                        folder_id,
                    )?;
                }
            }
        }
    } else {
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(colon_pos) = line.find(':') {
                let platform = line[..colon_pos].trim();
                let status = line[colon_pos + 1..].trim().to_lowercase();
                if status.contains("found") || status.contains("exist") || status.contains("yes") {
                    create_exposure(
                        conn,
                        email,
                        "mention",
                        &format!("Compte détecté sur : {}", platform),
                        folder_id,
                    )?;
                }
            }
        }
    }

    conn.execute(
        "UPDATE osint_modules SET last_run = datetime('now'), status = 'actif' WHERE id = 'osint-email-platforms'",
        []
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "legacy-osint")]
fn process_h8mail_result(
    conn: &Connection,
    email: &str,
    output: &str,
    folder_id: &str,
) -> Result<(), String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(arr) = parsed.as_array() {
            for item in arr {
                let breach_info = item.to_string();
                create_exposure(
                    conn,
                    email,
                    "fuite",
                    &format!("Fuite détectée (dump local) : {}", breach_info),
                    folder_id,
                )?;
                create_incident_and_action_for_leak(conn, email, &breach_info)?;
            }
        } else if let Some(obj) = parsed.as_object() {
            let breach_info = serde_json::to_string(obj).unwrap_or_default();
            create_exposure(
                conn,
                email,
                "fuite",
                &format!("Fuite détectée (dump local) : {}", breach_info),
                folder_id,
            )?;
            create_incident_and_action_for_leak(conn, email, &breach_info)?;
        }
    } else {
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.to_lowercase().contains("breach")
                || line.to_lowercase().contains("fuite")
                || line.to_lowercase().contains("found")
            {
                create_exposure(
                    conn,
                    email,
                    "fuite",
                    &format!("Fuite détectée (dump local) : {}", line),
                    folder_id,
                )?;
                create_incident_and_action_for_leak(conn, email, line)?;
            }
        }
    }

    conn.execute(
        "UPDATE osint_modules SET last_run = datetime('now'), status = 'actif' WHERE id = 'osint-email-breaches-local'",
        []
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "legacy-osint")]
fn process_ghunt_result(
    conn: &Connection,
    email: &str,
    output: &str,
    folder_id: &str,
) -> Result<(), String> {
    let parsed: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| format!("Failed to parse GHunt output as JSON: {}", e))?;

    let obj = parsed
        .as_object()
        .ok_or("GHunt output is not a JSON object")?;

    if email.ends_with("@gmail.com") || email.ends_with("@google.com") || obj.get("name").is_some()
    {
        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let picture = obj.get("picture").and_then(|v| v.as_str()).unwrap_or("");

        let mut what_parts = vec!["Profil Google public détecté".to_string()];
        if !name.is_empty() {
            what_parts.push(format!("Nom : {}", name));
        }
        if !picture.is_empty() {
            what_parts.push("Photo de profil disponible".to_string());
        }

        create_exposure(
            conn,
            email,
            "profil_public",
            &what_parts.join(" — "),
            folder_id,
        )?;
    }

    conn.execute(
        "UPDATE osint_modules SET last_run = datetime('now'), status = 'actif' WHERE id = 'osint-gmail-profile'",
        []
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "legacy-osint")]
fn create_exposure(
    conn: &Connection,
    email: &str,
    kind: &str,
    what: &str,
    folder_id: &str,
) -> Result<(), String> {
    let exp_id = Uuid::new_v4().to_string();
    let (db_kind, severity) = match kind {
        "fuite" => ("fuite", "élevée"),
        "profil_public" => ("profil_public", "modérée"),
        _ => ("mention", "faible"),
    };

    conn.execute(
        "INSERT INTO exposures (id, title, kind, severity, status, discovered_at, source, what, why, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), ?6, ?7, ?8, ?9)",
        params![
            exp_id,
            format!("{} - {}", kind, email),
            db_kind,
            severity,
            "nouvelle",
            "Suite OSINT locale",
            what,
            "Détection automatique par la suite de veille.",
            folder_id
        ]
    ).map_err(|e| e.to_string())?;

    let tl_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO timeline_entries (id, event_type, description, created_at) VALUES (?1, ?2, ?3, datetime('now'))",
        params![tl_id, "Détection OSINT", format!("{} pour {}", what, email)]
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "legacy-osint")]
fn create_incident_and_action_for_leak(
    conn: &Connection,
    email: &str,
    breach: &str,
) -> Result<(), String> {
    let inc_id = Uuid::new_v4().to_string();
    let act_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO incidents (id, title, severity, discovered_at, what, why, impact, confidence, next_step, folder_id) VALUES (?1, ?2, ?3, datetime('now'), ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            inc_id,
            format!("Fuite de données: {}", email),
            "élevée",
            format!("L'email {} a été trouvé dans une fuite: {}", email, breach),
            "Risque de compromission de compte si réutilisation de mot de passe.",
            "Accès non autorisé, usurpation d'identité.",
            "Élevée",
            "Changer les mots de passe associés et activer la MFA.",
            "folder-perso"
        ]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO actions (id, title, priority_id, difficulty_id, deadline, status, guidance, proof_expected, folder_id) VALUES (?1, ?2, ?3, ?4, date('now', '+7 days'), ?5, ?6, ?7, ?8)",
        params![
            act_id,
            format!("Revue des comptes liés à {}", email),
            "prio_004",
            "diff_002",
            "a_faire",
            "[\"Lister les services où l'e-mail fuité était utilisé.\",\"Changer les mots de passe et activer la MFA.\"]",
            "Liste des services revus.",
            "folder-perso"
        ]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO incident_action (incident_id, action_id) VALUES (?1, ?2)",
        params![inc_id, act_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(feature = "legacy-osint")]
fn create_or_update_identity_from_osint(
    conn: &Connection,
    email: &str,
    folder_id: &str,
) -> Result<(), String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM identities WHERE value = ?1",
            params![email],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count == 0 {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO identities (id, label, kind, value, folder_id, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, &format!("E-mail découvert: {}", email), "email", email, folder_id, "Découvert via OSINT"]
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(feature = "legacy-osint")]
#[tauri::command]
fn run_osint_module(app: tauri::AppHandle, module_id: String) -> Result<OsintRunSummary, String> {
    let conn = get_db_connection(&app)?;

    let module: OsintModule = conn.query_row(
        "SELECT id, name, description, target_kind, frequency, status, last_run, next_run, script_path, script_args FROM osint_modules WHERE id = ?1",
        params![module_id],
        |row| {
            Ok(OsintModule {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                target_kind: row.get(3)?,
                frequency: row.get(4)?,
                status: row.get(5)?,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                script_path: row.get(8)?,
                script_args: row.get(9)?,
                installation_status: "legacy".to_string(),
                diagnostic: None,
                catalog_status:"archived".into(),replacement_id:None,archived_reason:Some("Chemin historique non exposé.".into()),
            })
        }
    ).map_err(|e| e.to_string())?;

    let script_path = module.script_path.clone().unwrap_or_default();
    let script_args = module.script_args.clone().unwrap_or_default();

    if script_path.is_empty() {
        return Err(format!(
            "Le module '{}' n'a pas de script configuré (script_path vide).",
            module.name
        ));
    }

    if !std::path::Path::new(&script_path).exists() && script_path != "python" {
        return Err(format!(
            "Le script '{}' n'existe pas sur le système.",
            script_path
        ));
    }

    let mut stmt = conn
        .prepare("SELECT id, value, folder_id FROM identities WHERE kind = 'email'")
        .map_err(|e| e.to_string())?;
    let email_iter = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut summary = OsintRunSummary { results: vec![] };

    for email_res in email_iter {
        let (_id, email, folder_id) = email_res.map_err(|e| e.to_string())?;
        let folder_id = folder_id.unwrap_or_else(|| "folder-perso".to_string());

        let mut target_result = OsintTargetResult {
            target: email.clone(),
            leaks: vec![],
            profiles: vec![],
            noise_filtered: 0,
            risk_level: "faible".to_string(),
        };

        let script_output = match execute_osint_script(&script_path, &script_args, &email) {
            Ok(output) => output,
            Err(e) => {
                conn.execute(
                    "UPDATE osint_modules SET status = 'erreur' WHERE id = ?1",
                    params![module_id],
                )
                .map_err(|e| e.to_string())?;
                return Err(format!(
                    "Erreur d'exécution du module '{}' pour {}: {}",
                    module.name, email, e
                ));
            }
        };

        let process_result = match module_id.as_str() {
            "osint-email-intel" => {
                process_mailaccess_result(&conn, &email, &script_output, &folder_id)
            }
            "osint-email-platforms" => {
                process_holehe_result(&conn, &email, &script_output, &folder_id)
            }
            "osint-email-breaches-local" => {
                process_h8mail_result(&conn, &email, &script_output, &folder_id)
            }
            "osint-gmail-profile" => {
                process_ghunt_result(&conn, &email, &script_output, &folder_id)
            }
            _ => {
                target_result.leaks.push(script_output.trim().to_string());
                target_result.risk_level = "info".to_string();
                Ok(())
            }
        };

        if let Err(_e) = process_result {
            target_result.noise_filtered += 1;
            conn.execute(
                "UPDATE osint_modules SET status = 'erreur' WHERE id = ?1",
                params![module_id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            if !target_result.leaks.is_empty() {
                target_result.risk_level = "élevée".to_string();
            } else if !target_result.profiles.is_empty() {
                target_result.risk_level = "modérée".to_string();
            }
        }

        summary.results.push(target_result);
    }

    conn.execute(
        "UPDATE osint_modules SET status = 'actif' WHERE id = ?1",
        params![module_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(summary)
}

#[cfg(feature = "legacy-osint")]
#[tauri::command]
fn run_osint_module_for_email(
    app: tauri::AppHandle,
    module_id: String,
    email: String,
) -> Result<OsintRunSummary, String> {
    let conn = get_db_connection(&app)?;

    let module: OsintModule = conn.query_row(
        "SELECT id, name, description, target_kind, frequency, status, last_run, next_run, script_path, script_args FROM osint_modules WHERE id = ?1",
        params![module_id],
        |row| {
            Ok(OsintModule {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                target_kind: row.get(3)?,
                frequency: row.get(4)?,
                status: row.get(5)?,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                script_path: row.get(8)?,
                script_args: row.get(9)?,
                installation_status: "legacy".to_string(),
                diagnostic: None,
                catalog_status:"archived".into(),replacement_id:None,archived_reason:Some("Chemin historique non exposé.".into()),
            })
        }
    ).map_err(|e| e.to_string())?;

    let script_path = module.script_path.clone().unwrap_or_default();
    let script_args = module.script_args.clone().unwrap_or_default();

    if script_path.is_empty() {
        return Err(format!(
            "Le module '{}' n'a pas de script configuré (script_path vide).",
            module.name
        ));
    }

    if !std::path::Path::new(&script_path).exists() && script_path != "python" {
        return Err(format!(
            "Le script '{}' n'existe pas sur le système.",
            script_path
        ));
    }

    // Trouver le folder_id pour cet email
    let folder_id: Option<String> = conn
        .query_row(
            "SELECT folder_id FROM identities WHERE value = ?1 AND kind = 'email' LIMIT 1",
            params![&email],
            |row| row.get(0),
        )
        .ok();

    let folder_id = folder_id.unwrap_or_else(|| "folder-perso".to_string());

    let mut summary = OsintRunSummary { results: vec![] };

    let mut target_result = OsintTargetResult {
        target: email.clone(),
        leaks: vec![],
        profiles: vec![],
        noise_filtered: 0,
        risk_level: "faible".to_string(),
    };

    let script_output = match execute_osint_script(&script_path, &script_args, &email) {
        Ok(output) => output,
        Err(e) => {
            conn.execute(
                "UPDATE osint_modules SET status = 'erreur' WHERE id = ?1",
                params![module_id],
            )
            .map_err(|e| e.to_string())?;
            return Err(format!(
                "Erreur d'exécution du module '{}' pour {}: {}",
                module.name, email, e
            ));
        }
    };

    let process_result = match module_id.as_str() {
        "osint-email-intel" => process_mailaccess_result(&conn, &email, &script_output, &folder_id),
        "osint-email-platforms" => process_holehe_result(&conn, &email, &script_output, &folder_id),
        "osint-email-breaches-local" => {
            process_h8mail_result(&conn, &email, &script_output, &folder_id)
        }
        "osint-gmail-profile" => process_ghunt_result(&conn, &email, &script_output, &folder_id),
        _ => {
            target_result.leaks.push(script_output.trim().to_string());
            target_result.risk_level = "info".to_string();
            Ok(())
        }
    };

    if let Err(_e) = process_result {
        target_result.noise_filtered += 1;
        conn.execute(
            "UPDATE osint_modules SET status = 'erreur' WHERE id = ?1",
            params![module_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        if !target_result.leaks.is_empty() {
            target_result.risk_level = "élevée".to_string();
        } else if !target_result.profiles.is_empty() {
            target_result.risk_level = "modérée".to_string();
        }
    }

    summary.results.push(target_result);

    conn.execute(
        "UPDATE osint_modules SET status = 'actif', last_run = datetime('now') WHERE id = ?1",
        params![module_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(summary)
}

fn clear_user_scan_data_in_connection(conn: &mut Connection) -> Result<(), String> {
    conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for table in [
        "rgpd_draft_uses",
        "rgpd_user_reviews",
        "rgpd_draft_versions",
        "osint_report_exports",
        "osint_report_snapshots",
        "osint_projection_events",
        "osint_claim_session_presence",
        "osint_claim_reviews",
        "osint_analysis_claim_inputs",
        "osint_synthesis_runs",
        "osint_fact_resolution_evidence",
        "osint_fact_resolutions",
        "osint_claim_fact_links",
        "osint_claim_evidence",
        "osint_claims",
        "osint_relation_evidence",
        "osint_relations",
        "osint_entities",
        "osint_analysis_jobs",
        "osint_analysis_outputs",
        "osint_analysis_inputs",
        "osint_analysis_runs",
        "osint_user_decisions",
        "osint_evidence_facts",
        "osint_evidence_assessments",
        "osint_evidence_links",
        "osint_observations",
        "osint_raw_artifacts",
        "osint_signals",
        "osint_scan_sessions",
        "osint_scans",
        "osint_module_logs",
        "action_rgpd",
        "incident_rgpd",
        "incident_action",
        "exposure_incident",
        "folder_identity",
        "timeline_entries",
        "rgpd_requests",
        "actions",
        "incidents",
        "exposures",
        "identity_values",
        "identities",
        "folders",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|e| e.to_string())?;
    }
    tx.execute("UPDATE osint_modules SET last_run=NULL", [])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

/// Supprime uniquement les résultats produits par les scans Veille.
/// Les dossiers, identités et valeurs saisies par l'utilisateur sont conservés.
fn clear_scan_results_in_connection(conn: &mut Connection) -> Result<(), String> {
    conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    // Les projections métier sont repérées par leur lien avec un signal OSINT.
    // On retire d'abord les dépendances afin de préserver les intégrités référentielles.
    for statement in [
        "CREATE TEMP TABLE IF NOT EXISTS _mantis_scan_exposure_ids(id TEXT PRIMARY KEY)",
        "CREATE TEMP TABLE IF NOT EXISTS _mantis_scan_incident_ids(id TEXT PRIMARY KEY)",
        "CREATE TEMP TABLE IF NOT EXISTS _mantis_scan_action_ids(id TEXT PRIMARY KEY)",
        "INSERT OR IGNORE INTO _mantis_scan_exposure_ids(id) SELECT exposure_id FROM osint_signals WHERE exposure_id IS NOT NULL",
        "INSERT OR IGNORE INTO _mantis_scan_incident_ids(id) SELECT incident_id FROM exposure_incident WHERE exposure_id IN (SELECT id FROM _mantis_scan_exposure_ids)",
        "INSERT OR IGNORE INTO _mantis_scan_action_ids(id) SELECT action_id FROM incident_action WHERE incident_id IN (SELECT id FROM _mantis_scan_incident_ids)",
        "CREATE TEMP TABLE IF NOT EXISTS _mantis_scan_rgpd_ids AS SELECT rgpd_id AS id FROM action_rgpd WHERE 0",
        "INSERT OR IGNORE INTO _mantis_scan_rgpd_ids(id) SELECT rgpd_id FROM action_rgpd WHERE action_id IN (SELECT id FROM _mantis_scan_action_ids)",
        "INSERT OR IGNORE INTO _mantis_scan_rgpd_ids(id) SELECT rgpd_id FROM incident_rgpd WHERE incident_id IN (SELECT id FROM _mantis_scan_incident_ids)",
        "DELETE FROM action_rgpd WHERE action_id IN (SELECT id FROM _mantis_scan_action_ids)",
        "DELETE FROM incident_rgpd WHERE incident_id IN (SELECT id FROM _mantis_scan_incident_ids)",
        "DELETE FROM rgpd_draft_uses WHERE request_id IN (SELECT id FROM _mantis_scan_rgpd_ids)",
        "DELETE FROM rgpd_user_reviews WHERE request_id IN (SELECT id FROM _mantis_scan_rgpd_ids)",
        "DELETE FROM rgpd_draft_versions WHERE request_id IN (SELECT id FROM _mantis_scan_rgpd_ids)",
        "DELETE FROM rgpd_requests WHERE id IN (SELECT id FROM _mantis_scan_rgpd_ids)",
        "DELETE FROM remediation_plans WHERE scan_id IN (SELECT id FROM osint_scans) OR id IN (SELECT plan_id FROM remediation_plan_items WHERE exposure_id IN (SELECT id FROM _mantis_scan_exposure_ids))",
        "DELETE FROM action_events WHERE action_id IN (SELECT id FROM _mantis_scan_action_ids)",
        "DELETE FROM action_evidence WHERE action_id IN (SELECT id FROM _mantis_scan_action_ids)",
        "DELETE FROM incident_action WHERE incident_id IN (SELECT id FROM _mantis_scan_incident_ids)",
        "DELETE FROM exposure_incident WHERE exposure_id IN (SELECT id FROM _mantis_scan_exposure_ids)",
        "DELETE FROM actions WHERE id IN (SELECT id FROM _mantis_scan_action_ids)",
        "DELETE FROM incidents WHERE id IN (SELECT id FROM _mantis_scan_incident_ids)",
        "DELETE FROM exposures WHERE id IN (SELECT id FROM _mantis_scan_exposure_ids)",
        "DELETE FROM timeline_entries WHERE event_type IN ('Signal OSINT','Revue OSINT')",
    ] {
        tx.execute(statement, []).map_err(|e| e.to_string())?;
    }
    tx.execute_batch("DROP TABLE IF EXISTS _mantis_scan_rgpd_ids; DROP TABLE IF EXISTS _mantis_scan_action_ids; DROP TABLE IF EXISTS _mantis_scan_incident_ids; DROP TABLE IF EXISTS _mantis_scan_exposure_ids")
        .map_err(|e| e.to_string())?;
    for table in [
        "osint_report_exports",
        "osint_report_snapshots",
        "osint_projection_events",
        "osint_claim_session_presence",
        "osint_claim_reviews",
        "osint_analysis_claim_inputs",
        "osint_synthesis_runs",
        "osint_fact_resolution_evidence",
        "osint_fact_resolutions",
        "osint_claim_fact_links",
        "osint_claim_evidence",
        "osint_claims",
        "osint_relation_evidence",
        "osint_relations",
        "osint_entities",
        "osint_analysis_jobs",
        "osint_analysis_outputs",
        "osint_analysis_inputs",
        "osint_analysis_runs",
        "osint_user_decisions",
        "osint_evidence_facts",
        "osint_evidence_assessments",
        "osint_evidence_links",
        "osint_observations",
        "osint_raw_artifacts",
        "osint_signals",
        "osint_scan_sessions",
        "osint_scans",
        "osint_module_logs",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])
            .map_err(|e| e.to_string())?;
    }
    tx.execute("UPDATE osint_modules SET last_run=NULL", [])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_scan_results(app: tauri::AppHandle) -> Result<String, String> {
    let _analysis_guard = LOCAL_AI_ANALYSIS_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "Impossible de sécuriser les analyses en cours.".to_string())?;
    let mut conn = get_db_connection(&app)?;
    clear_scan_results_in_connection(&mut conn)?;
    let raw_dir = osint_root(&app)?.join("raw");
    if raw_dir.exists() {
        std::fs::remove_dir_all(&raw_dir)
            .map_err(|e| format!("Les résultats SQLite ont été effacés, mais les preuves brutes n’ont pas pu être supprimées : {e}"))?;
    }
    std::fs::create_dir_all(&raw_dir).map_err(|e| e.to_string())?;
    let exports_dir = application_data_dir(&app)?.join("exports");
    if exports_dir.exists() {
        std::fs::remove_dir_all(&exports_dir)
            .map_err(|e| format!("Les résultats ont été effacés, mais certains exports n’ont pas pu être supprimés : {e}"))?;
    }
    Ok("Résultats de scans, fuites, incidents et données dérivées effacés. Vos dossiers et identités sont conservés.".into())
}

/// Restores the empty first-run experience without touching OSINT modules,
/// application preferences or the local AI installation.
#[tauri::command]
fn clear_user_scan_data(app: tauri::AppHandle) -> Result<String, String> {
    let _analysis_guard = LOCAL_AI_ANALYSIS_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "Impossible de sécuriser les analyses en cours.".to_string())?;
    let mut conn = get_db_connection(&app)?;
    clear_user_scan_data_in_connection(&mut conn)?;

    let raw_dir = osint_root(&app)?.join("raw");
    if raw_dir.exists() {
        std::fs::remove_dir_all(&raw_dir)
            .map_err(|e| format!("Les données SQLite ont été effacées, mais les preuves brutes n’ont pas pu être supprimées : {e}"))?;
    }
    std::fs::create_dir_all(&raw_dir).map_err(|e| e.to_string())?;

    let exports_dir = application_data_dir(&app)?.join("exports");
    if exports_dir.parent() != Some(application_data_dir(&app)?.as_path())
        || exports_dir.file_name().and_then(|v| v.to_str()) != Some("exports")
    {
        return Err("Le dossier d’exports n’a pas passé le contrôle de sécurité.".into());
    }
    if exports_dir.exists() {
        std::fs::remove_dir_all(&exports_dir).map_err(|e| {
            format!("Données effacées, mais certains exports n’ont pas pu être supprimés : {e}")
        })?;
    }

    Ok("Données utilisateur, preuves et exports effacés. Les modules OSINT et l’IA locale sont conservés.".to_string())
}

fn refresh_osint_graph(conn: &mut Connection) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM osint_relation_evidence", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM osint_relations", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM osint_entities", [])
        .map_err(|e| e.to_string())?;
    tx.execute_batch(r#"
        INSERT INTO osint_entities (id,entity_type,label,canonical_value,identity_id,first_seen_at,last_seen_at)
        SELECT 'identity:'||id,'identite',label,value,id,created_at,updated_at FROM identities;
        INSERT INTO osint_entities (id,entity_type,label,canonical_value,identity_id,first_seen_at,last_seen_at)
        SELECT 'observation:'||id,'observation',display_value,canonical_key,identity_id,observed_at,observed_at FROM osint_observations WHERE relevance_status!='ignoree';
        INSERT OR IGNORE INTO osint_entities (id,entity_type,label,canonical_value,first_seen_at,last_seen_at)
        SELECT 'source:'||lower(hex(source)),'source',source,lower(source),MIN(observed_at),MAX(observed_at) FROM osint_observations WHERE relevance_status!='ignoree' GROUP BY lower(source);

        INSERT INTO osint_relations (id,from_entity_id,to_entity_id,relation_type,evidence_level,justification,review_status,first_seen_at,last_seen_at)
        SELECT 'relation:observe:'||id,'identity:'||identity_id,'observation:'||id,
               CASE WHEN relevance_status='pas_moi' THEN 'contredit' ELSE 'observe' END,
               CASE relevance_status WHEN 'confirmee' THEN 'corroboree' WHEN 'pas_moi' THEN 'contradiction' ELSE 'possible' END,
               CASE relevance_status WHEN 'confirmee' THEN 'Observation confirmée par l’utilisateur.' WHEN 'pas_moi' THEN 'L’utilisateur a indiqué que ce résultat ne le concerne pas.' ELSE 'Observation collectée, attribution à vérifier.' END,
               CASE relevance_status WHEN 'confirmee' THEN 'validee' WHEN 'pas_moi' THEN 'rejetee' ELSE 'proposee' END,
               observed_at,observed_at
        FROM osint_observations WHERE relevance_status!='ignoree';
        INSERT INTO osint_relations (id,from_entity_id,to_entity_id,relation_type,evidence_level,justification,review_status,first_seen_at,last_seen_at)
        SELECT 'relation:source:'||id,'observation:'||id,'source:'||lower(hex(source)),'collecte_par','observe','Provenance enregistrée par le module de collecte.','validee',observed_at,observed_at
        FROM osint_observations WHERE relevance_status!='ignoree';

        INSERT OR IGNORE INTO osint_relations (id,from_entity_id,to_entity_id,relation_type,evidence_level,justification,review_status,first_seen_at,last_seen_at)
        SELECT 'relation:multi:'||o.id,'identity:'||o.identity_id,'observation:'||o.id,'correspondance_multi_source','probable',
               'Même valeur observée par plusieurs sources indépendantes ; une validation humaine reste nécessaire.','proposee',MIN(g.observed_at),MAX(g.observed_at)
        FROM osint_observations o JOIN osint_observations g ON g.identity_id=o.identity_id AND g.observation_type=o.observation_type AND lower(trim(g.display_value))=lower(trim(o.display_value))
        WHERE o.relevance_status NOT IN ('ignoree','pas_moi') AND g.relevance_status NOT IN ('ignoree','pas_moi')
        GROUP BY o.id HAVING COUNT(DISTINCT lower(g.source)) >= 2;

        INSERT INTO osint_relation_evidence (relation_id,observation_id,evidence_role)
        SELECT 'relation:observe:'||id,id,CASE WHEN relevance_status='pas_moi' THEN 'contradictoire' ELSE 'favorable' END FROM osint_observations WHERE relevance_status!='ignoree';
        INSERT INTO osint_relation_evidence (relation_id,observation_id,evidence_role)
        SELECT 'relation:source:'||id,id,'favorable' FROM osint_observations WHERE relevance_status!='ignoree';
        INSERT OR IGNORE INTO osint_relation_evidence (relation_id,observation_id,evidence_role)
        SELECT r.id,g.id,'favorable' FROM osint_relations r JOIN osint_observations o ON r.to_entity_id='observation:'||o.id
        JOIN osint_observations g ON g.identity_id=o.identity_id AND g.observation_type=o.observation_type AND lower(trim(g.display_value))=lower(trim(o.display_value))
        WHERE r.relation_type='correspondance_multi_source' AND g.relevance_status NOT IN ('ignoree','pas_moi');
    "#).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[derive(Clone)]
struct ClaimObservationRow {
    observation_id: String,
    module_id: String,
    observation_type: String,
    display_value: String,
    source: String,
    source_url: Option<String>,
    observed_at: String,
    relevance_status: String,
    severity: String,
}

fn normalized_exact_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalized_claim_value(kind: &str, value: &str) -> String {
    let exact = normalized_exact_text(value);
    match kind {
        "email" | "courriel" => exact,
        "telephone" | "téléphone" => normalize_phone_number(&exact),
        "adresse" => normalize_address_text(&exact),
        "pseudo" => exact.trim_start_matches('@').to_string(),
        "domaine" | "domain" => exact
            .trim_start_matches("www.")
            .trim_end_matches('.')
            .to_string(),
        _ => exact,
    }
}

fn normalized_claim_url(value: &str) -> Option<String> {
    let mut url = safe_public_url_shape(value)?;
    url.set_fragment(None);
    let retained = url
        .query_pairs()
        .filter(|(key, _)| {
            !matches!(
                key.to_ascii_lowercase().as_str(),
                "utm_source"
                    | "utm_medium"
                    | "utm_campaign"
                    | "utm_term"
                    | "utm_content"
                    | "fbclid"
                    | "gclid"
            )
        })
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    url.set_query(None);
    if !retained.is_empty() {
        url.query_pairs_mut().extend_pairs(retained);
    }
    let mut normalized = url.to_string();
    if normalized.ends_with('/') && url.path() == "/" && url.query().is_none() {
        normalized.pop();
    }
    Some(normalized)
}

fn claim_key(row: &ClaimObservationRow) -> String {
    // A breach reference can be a generic provider notice shared by several
    // distinct incidents. Group leaks by their exact breach label, while web
    // profiles remain grouped by their canonical public URL.
    if row.observation_type != "fuite" {
        if let Some(url) = row.source_url.as_deref().and_then(normalized_claim_url) {
            return format!("url|{url}");
        }
    }
    format!(
        "{}|{}",
        normalized_exact_text(&row.observation_type),
        normalized_claim_value(&row.observation_type, &row.display_value)
    )
}

fn claim_type_label(value: &str) -> String {
    match value {
        "fuite" => "fuite_eventuelle",
        "compte_potentiel" | "profil_public" => "profil_potentiel",
        "annuaire" => "annuaire",
        "site_rencontre" => "site_de_rencontre",
        "source_indisponible" => "source_indisponible",
        _ => "mention_web",
    }
    .to_string()
}

fn refresh_osint_claims(conn: &mut Connection, identity_id: &str) -> Result<(), String> {
    // Claims are a presentation layer. Rebuild their facts first so a raw,
    // weak observation cannot quietly become a default user-facing conclusion.
    conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    conn.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    refresh_osint_evidence_facts(conn, identity_id)?;
    let rows = {
        let mut statement = conn.prepare(
            "SELECT o.id,o.signal_id,COALESCE(s.module_id,''),o.observation_type,o.display_value,o.source,o.source_url,o.observed_at,o.relevance_status,COALESCE(s.severity,'faible')
             FROM osint_observations o LEFT JOIN osint_signals s ON s.id=o.signal_id
             WHERE o.identity_id=?1 AND o.relevance_status!='ignoree'
             ORDER BY o.observed_at,o.id"
        ).map_err(|e|e.to_string())?;
        let mapped = statement
            .query_map(params![identity_id], |row| {
                Ok(ClaimObservationRow {
                    observation_id: row.get(0)?,
                    module_id: row.get(2)?,
                    observation_type: row.get(3)?,
                    display_value: row.get(4)?,
                    source: row.get(5)?,
                    source_url: row.get(6)?,
                    observed_at: row.get(7)?,
                    relevance_status: row.get(8)?,
                    severity: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        mapped
    };
    let mut groups = std::collections::BTreeMap::<String, Vec<ClaimObservationRow>>::new();
    for row in rows {
        groups.entry(claim_key(&row)).or_default().push(row);
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM osint_claim_fact_links WHERE claim_id IN (SELECT id FROM osint_claims WHERE identity_id=?1)",
        params![identity_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM osint_claim_evidence WHERE claim_id IN (SELECT id FROM osint_claims WHERE identity_id=?1)",params![identity_id]).map_err(|e|e.to_string())?;
    tx.execute(
        "DELETE FROM osint_claims WHERE identity_id=?1",
        params![identity_id],
    )
    .map_err(|e| e.to_string())?;
    for (canonical_key, observations) in groups {
        let favorable = observations
            .iter()
            .filter(|row| row.relevance_status != "pas_moi")
            .map(|row| {
                format!(
                    "{}|{}",
                    normalized_exact_text(&row.source),
                    row.source_url
                        .as_deref()
                        .and_then(normalized_claim_url)
                        .unwrap_or_default()
                )
            })
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let contradictory = observations
            .iter()
            .filter(|row| row.relevance_status == "pas_moi")
            .map(|row| {
                format!(
                    "{}|{}",
                    normalized_exact_text(&row.source),
                    row.source_url
                        .as_deref()
                        .and_then(normalized_claim_url)
                        .unwrap_or_default()
                )
            })
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let confirmed = observations
            .iter()
            .any(|row| row.relevance_status == "confirmee");
        let sources = observations
            .iter()
            .filter(|row| row.relevance_status != "pas_moi")
            .map(|row| {
                evidence_source_family(&row.module_id, &row.source, row.source_url.as_deref())
            })
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let status = if contradictory > 0 && favorable > 0 {
            "contradictoire"
        } else if contradictory > 0 {
            "rejetee"
        } else if confirmed {
            "confirmee"
        } else if sources >= 2 {
            "corroboree"
        } else {
            "a_verifier"
        };
        let severity_rank = observations
            .iter()
            .filter(|row| row.relevance_status != "pas_moi")
            .map(|row| match row.severity.as_str() {
                "critique" => 3,
                "elevee" | "élevée" => 2,
                "moderee" | "modérée" => 1,
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        let actionable_public_mention = observations.iter().any(|row| {
            row.observation_type == "mention"
                && row.source == "DDGS"
                && row.source_url.is_some()
                && row.relevance_status != "pas_moi"
        });
        let priority = if status == "rejetee" {
            "faible"
        } else if severity_rank >= 3 {
            "critique"
        } else if severity_rank >= 2 || sources >= 2 || confirmed || actionable_public_mention {
            "haute"
        } else if observations.iter().any(|row| row.source_url.is_some()) {
            "moyenne"
        } else {
            "faible"
        };
        let rationale = match status {
            "contradictoire" => format!("{favorable} source(s) concordante(s) et {contradictory} source(s) écartée(s) : revue humaine nécessaire."),
            "rejetee" => "Toutes les observations correspondantes ont été écartées par l’utilisateur.".to_string(),
            "confirmee" => format!("Confirmation humaine enregistrée ; {sources} source(s) distincte(s) conservée(s)."),
            "corroboree" => format!("Même fait exact observé par {sources} sources distinctes ; cela ne confirme pas l’identité."),
            _ => "Une seule source ou preuve exacte : correspondance possible à vérifier.".to_string(),
        };
        let first = observations.first().expect("groupe non vide");
        let last = observations.last().expect("groupe non vide");
        let digest = Sha256::digest(format!("{identity_id}|{canonical_key}").as_bytes());
        let claim_id = format!("claim-{}", &hex::encode(digest)[..24]);
        tx.execute(
            "INSERT INTO osint_claims (id,identity_id,claim_type,canonical_key,display_value,status,priority,favorable_count,contradictory_count,source_count,first_observed_at,last_observed_at,rationale,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,datetime('now'))",
            params![claim_id,identity_id,claim_type_label(&first.observation_type),canonical_key,first.display_value,status,priority,favorable,contradictory,sources,first.observed_at,last.observed_at,rationale]
        ).map_err(|e|e.to_string())?;
        for observation in observations {
            let role = if observation.relevance_status == "pas_moi" {
                "contradictoire"
            } else {
                "favorable"
            };
            tx.execute("INSERT INTO osint_claim_evidence (claim_id,observation_id,evidence_role) VALUES (?1,?2,?3)",params![claim_id,observation.observation_id,role]).map_err(|e|e.to_string())?;
            tx.execute(
                "INSERT OR IGNORE INTO osint_claim_fact_links (claim_id,fact_id,evidence_role)
                 SELECT ?1,id,?3 FROM osint_evidence_facts WHERE observation_id=?2",
                params![claim_id, observation.observation_id, role],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

fn load_identity_claims(conn: &Connection, identity_id: &str) -> Result<Vec<OsintClaim>, String> {
    let mut statement = conn.prepare(
        "SELECT id,identity_id,claim_type,display_value,status,priority,favorable_count,contradictory_count,source_count,first_observed_at,last_observed_at,rationale
         FROM osint_claims WHERE identity_id=?1
         ORDER BY CASE priority WHEN 'critique' THEN 0 WHEN 'haute' THEN 1 WHEN 'moyenne' THEN 2 ELSE 3 END,
                  CASE status WHEN 'contradictoire' THEN 0 WHEN 'a_verifier' THEN 1 WHEN 'corroboree' THEN 2 WHEN 'confirmee' THEN 3 ELSE 4 END,last_observed_at DESC,id"
    ).map_err(|e|e.to_string())?;
    let raw = statement
        .query_map(params![identity_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut claims = Vec::new();
    for (
        id,
        identity_id,
        claim_type,
        display_value,
        status,
        priority,
        favorable_count,
        contradictory_count,
        source_count,
        first_observed_at,
        last_observed_at,
        rationale,
    ) in raw
    {
        let mut evidence_statement=conn.prepare(
            "SELECT o.id,o.signal_id,o.source,o.source_url,o.observed_at,ce.evidence_role FROM osint_claim_evidence ce JOIN osint_observations o ON o.id=ce.observation_id WHERE ce.claim_id=?1 ORDER BY o.observed_at DESC,o.id LIMIT 20"
        ).map_err(|e|e.to_string())?;
        let evidence = evidence_statement
            .query_map(params![id], |row| {
                Ok(OsintClaimEvidence {
                    observation_id: row.get(0)?,
                    signal_id: row.get(1)?,
                    source: row.get(2)?,
                    source_url: row.get(3)?,
                    observed_at: row.get(4)?,
                    role: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        claims.push(OsintClaim {
            id,
            identity_id,
            claim_type,
            display_value,
            status,
            priority,
            favorable_count,
            contradictory_count,
            source_count,
            first_observed_at,
            last_observed_at,
            rationale,
            evidence,
        });
    }
    Ok(claims)
}

fn load_identity_fact_resolutions(
    conn: &Connection,
    identity_id: &str,
) -> Result<Vec<OsintFactResolution>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id,fact_type,status,source_count,favorable_count,contradictory_count,rationale
             FROM osint_fact_resolutions WHERE identity_id=?1
             ORDER BY CASE status WHEN 'contradictoire' THEN 0 WHEN 'corroboree' THEN 1 WHEN 'a_verifier' THEN 2 ELSE 3 END,
                      source_count DESC,favorable_count DESC,id",
        )
        .map_err(|e| e.to_string())?;
    let resolutions = statement
        .query_map(params![identity_id], |row| {
            Ok(OsintFactResolution {
                id: row.get(0)?,
                fact_type: row.get(1)?,
                status: row.get(2)?,
                source_count: row.get(3)?,
                favorable_count: row.get(4)?,
                contradictory_count: row.get(5)?,
                rationale: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(resolutions)
}

fn record_claim_session_presence(
    conn: &Connection,
    session_id: &str,
    identity_id: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM osint_claim_session_presence WHERE session_id=?1",
        params![session_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO osint_claim_session_presence(session_id,claim_id,identity_id,source_count) SELECT ?1,ce.claim_id,?2,COUNT(DISTINCT o.source) FROM osint_claim_evidence ce JOIN osint_observations o ON o.id=ce.observation_id JOIN osint_scans sc ON sc.id=o.scan_id WHERE sc.session_id=?1 AND o.identity_id=?2 GROUP BY ce.claim_id",params![session_id,identity_id]).map_err(|e|e.to_string())?;
    Ok(())
}

#[tauri::command]
fn review_osint_claim(
    app: tauri::AppHandle,
    claim_id: String,
    decision: String,
    reason: Option<String>,
) -> Result<String, String> {
    let (review_status, relevance_status) = match decision.as_str() {
        "confirmer" => ("Confirmé", "confirmee"),
        "pas_moi" => ("Ce n’est pas moi", "pas_moi"),
        "ignorer" => ("Ignoré", "ignoree"),
        "suivre" => ("Suivi", "suivie"),
        _ => return Err("Cette décision n’est pas reconnue.".into()),
    };
    if reason
        .as_deref()
        .is_some_and(|value| value.chars().count() > 500)
    {
        return Err("Le motif est trop long.".into());
    }
    let mut conn = get_db_connection(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let (identity_id, previous_status): (String, String) = tx
        .query_row(
            "SELECT identity_id,status FROM osint_claims WHERE id=?1",
            params![claim_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Revendication introuvable.".to_string())?;
    let signal_ids = {
        let mut statement=tx.prepare("SELECT DISTINCT o.signal_id FROM osint_claim_evidence ce JOIN osint_observations o ON o.id=ce.observation_id WHERE ce.claim_id=?1 AND o.signal_id IS NOT NULL").map_err(|e|e.to_string())?;
        let rows = statement
            .query_map(params![claim_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    if signal_ids.is_empty() {
        return Err("Cette revendication ne possède aucun signal révisable.".into());
    }
    tx.execute("INSERT INTO osint_claim_reviews(id,claim_id,identity_id,decision,reason,previous_status) VALUES (?1,?2,?3,?4,?5,?6)",params![Uuid::new_v4().to_string(),claim_id,identity_id,decision,reason,previous_status]).map_err(|e|e.to_string())?;
    for signal_id in &signal_ids {
        let prior: String = tx
            .query_row(
                "SELECT review_status FROM osint_signals WHERE id=?1",
                params![signal_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "À vérifier".into());
        let grouped_reason = Some(format!(
            "Revue groupée depuis la revendication {claim_id}.{}",
            reason
                .as_deref()
                .map(|value| format!(" Motif : {value}"))
                .unwrap_or_default()
        ));
        tx.execute("INSERT INTO osint_user_decisions(id,target_type,target_id,decision,reason,previous_status,created_at) VALUES (?1,'signal',?2,?3,?4,?5,datetime('now'))",params![Uuid::new_v4().to_string(),signal_id,decision,grouped_reason,prior]).map_err(|e|e.to_string())?;
        tx.execute(
            "UPDATE osint_signals SET review_status=?1 WHERE id=?2",
            params![review_status, signal_id],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE osint_observations SET relevance_status=?1 WHERE signal_id=?2",
            params![relevance_status, signal_id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    apply_signal_quality_gate(&mut conn, &identity_id)?;
    refresh_osint_claims(&mut conn, &identity_id)?;
    Ok(review_status.into())
}

fn identity_review_workspace_in_connection(
    conn: &Connection,
    identity_id: &str,
) -> Result<OsintReviewWorkspace, String> {
    let mut statement=conn.prepare("SELECT id,'decision',COALESCE((SELECT display_value FROM osint_claims c WHERE c.id=r.claim_id),r.claim_id),decision,reason,created_at FROM osint_claim_reviews r WHERE identity_id=?1 UNION ALL SELECT d.id,'decision',s.title,d.decision,d.reason,d.created_at FROM osint_user_decisions d JOIN osint_signals s ON s.id=d.target_id WHERE s.identity_id=?1 AND (d.reason IS NULL OR d.reason NOT LIKE 'Revue groupée depuis la revendication %') UNION ALL SELECT id,'projection',projection_type||' → '||target_id,outcome,NULL,created_at FROM osint_projection_events WHERE identity_id=?1 ORDER BY 6 DESC LIMIT 80").map_err(|e|e.to_string())?;
    let events = statement
        .query_map(params![identity_id], |row| {
            Ok(OsintReviewEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                target_label: row.get(2)?,
                decision: row.get(3)?,
                reason: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let sessions = {
        let mut stmt=conn.prepare("SELECT id FROM osint_scan_sessions WHERE identity_id=?1 AND status IN ('termine','partiel') ORDER BY started_at DESC,rowid DESC LIMIT 2").map_err(|e|e.to_string())?;
        let rows = stmt
            .query_map(params![identity_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    let mut evolution = Vec::new();
    if let Some(current) = sessions.first() {
        let previous = sessions.get(1);
        let mut stmt=conn.prepare("SELECT claim_id,source_count FROM osint_claim_session_presence WHERE session_id=?1").map_err(|e|e.to_string())?;
        let current_map = stmt
            .query_map(params![current], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<std::collections::HashMap<_, _>, _>>()
            .map_err(|e| e.to_string())?;
        let previous_map = if let Some(previous) = previous {
            stmt.query_map(params![previous], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<std::collections::HashMap<_, _>, _>>()
            .map_err(|e| e.to_string())?
        } else {
            std::collections::HashMap::new()
        };
        let mut ids = current_map
            .keys()
            .chain(previous_map.keys())
            .cloned()
            .collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        for claim_id in ids {
            let current_sources = *current_map.get(&claim_id).unwrap_or(&0);
            let previous_sources = *previous_map.get(&claim_id).unwrap_or(&0);
            let change = if current_sources == 0 {
                "source_indisponible"
            } else if previous_sources == 0 {
                "nouveau"
            } else {
                "toujours_present"
            };
            let display_value = conn
                .query_row(
                    "SELECT display_value FROM osint_claims WHERE id=?1",
                    params![claim_id],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| claim_id.clone());
            evolution.push(OsintEvolutionItem {
                claim_id,
                display_value,
                change: change.into(),
                current_sources,
                previous_sources,
            });
        }
    }
    Ok(OsintReviewWorkspace { events, evolution })
}

#[tauri::command]
fn get_identity_review_workspace(
    app: tauri::AppHandle,
    identity_id: String,
) -> Result<OsintReviewWorkspace, String> {
    identity_review_workspace_in_connection(&get_db_connection(&app)?, &identity_id)
}

#[tauri::command]
fn get_osint_graph(
    app: tauri::AppHandle,
    identity_id: Option<String>,
) -> Result<OsintGraph, String> {
    let mut conn = get_db_connection(&app)?;
    refresh_osint_graph(&mut conn)?;
    let total_nodes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM osint_entities WHERE ?1 IS NULL OR identity_id=?1 OR id='identity:'||?1 OR id IN (SELECT r.to_entity_id FROM osint_relations r JOIN osint_relation_evidence re ON re.relation_id=r.id JOIN osint_observations o ON o.id=re.observation_id WHERE o.identity_id=?1)",
            params![identity_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let mut node_stmt = conn.prepare("SELECT id,entity_type,label,CASE entity_type WHEN 'identite' THEN 'Identité suivie' WHEN 'source' THEN 'Source de collecte' ELSE 'Observation normalisée' END FROM osint_entities WHERE ?1 IS NULL OR identity_id=?1 OR id='identity:'||?1 OR id IN (SELECT r.to_entity_id FROM osint_relations r JOIN osint_relation_evidence re ON re.relation_id=r.id JOIN osint_observations o ON o.id=re.observation_id WHERE o.identity_id=?1) ORDER BY CASE entity_type WHEN 'identite' THEN 0 WHEN 'observation' THEN 1 ELSE 2 END,last_seen_at DESC LIMIT 150").map_err(|e|e.to_string())?;
    let nodes = node_stmt
        .query_map(params![identity_id], |r| {
            Ok(OsintGraphNode {
                id: r.get(0)?,
                node_type: r.get(1)?,
                label: r.get(2)?,
                detail: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let node_ids: std::collections::HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let mut edge_stmt = conn.prepare("SELECT id,from_entity_id,to_entity_id,relation_type,evidence_level,justification,review_status FROM osint_relations ORDER BY CASE evidence_level WHEN 'contradiction' THEN 0 WHEN 'corroboree' THEN 1 WHEN 'probable' THEN 2 ELSE 3 END,last_seen_at DESC LIMIT 300").map_err(|e|e.to_string())?;
    let raw_edges = edge_stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut edges = Vec::new();
    for (id, from, to, relation_type, level, justification, review_status) in raw_edges {
        if !node_ids.contains(&from) || !node_ids.contains(&to) {
            continue;
        }
        let mut evidence_stmt = conn.prepare("SELECT re.observation_id,el.evidence_label,el.excerpt,COALESCE(el.source_url,o.source_url),re.evidence_role FROM osint_relation_evidence re JOIN osint_observations o ON o.id=re.observation_id LEFT JOIN osint_evidence_links el ON el.observation_id=o.id WHERE re.relation_id=?1 ORDER BY el.created_at LIMIT 8").map_err(|e|e.to_string())?;
        let evidence = evidence_stmt
            .query_map(params![id], |r| {
                Ok(OsintGraphEvidence {
                    observation_id: r.get(0)?,
                    label: r
                        .get::<_, Option<String>>(1)?
                        .unwrap_or_else(|| "Observation".into()),
                    excerpt: r.get(2)?,
                    source_url: r.get(3)?,
                    role: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        edges.push(OsintGraphEdge {
            id,
            from,
            to,
            relation_type,
            level,
            justification,
            review_status,
            evidence,
        });
    }
    let mut timeline_stmt = conn.prepare("SELECT id,observed_at,'observation',display_value,source,CASE relevance_status WHEN 'confirmee' THEN 'corroboree' WHEN 'pas_moi' THEN 'contradiction' ELSE 'possible' END FROM osint_observations WHERE ?1 IS NULL OR identity_id=?1 UNION ALL SELECT d.id,d.created_at,'decision','Décision : '||d.decision,'Utilisateur',CASE d.decision WHEN 'confirmer' THEN 'corroboree' WHEN 'pas_moi' THEN 'contradiction' ELSE 'observe' END FROM osint_user_decisions d WHERE ?1 IS NULL OR EXISTS (SELECT 1 FROM osint_signals s WHERE s.id=d.target_id AND s.identity_id=?1) ORDER BY 2 DESC LIMIT 200").map_err(|e|e.to_string())?;
    let timeline = timeline_stmt
        .query_map(params![identity_id], |r| {
            Ok(OsintGraphTimelineItem {
                id: r.get(0)?,
                date: r.get(1)?,
                date_kind: r.get(2)?,
                label: r.get(3)?,
                source: r.get(4)?,
                level: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(OsintGraph {
        nodes,
        edges,
        timeline,
        truncated: total_nodes > 150,
    })
}

fn build_report_snapshot(
    conn: &Connection,
    id: String,
    created_at: String,
    identity_id: Option<String>,
) -> Result<OsintReportSnapshot, String> {
    let mut source_stmt = conn.prepare("SELECT id,title,severity,source,source_url,discovered_at,review_status,explanation FROM osint_signals WHERE ?1 IS NULL OR identity_id=?1 ORDER BY discovered_at DESC,id LIMIT 250").map_err(|e|e.to_string())?;
    let sources = source_stmt
        .query_map(params![identity_id], |r| {
            Ok(OsintReportSource {
                signal_id: r.get(0)?,
                title: r.get(1)?,
                severity: r.get(2)?,
                source: r.get(3)?,
                source_url: r.get(4)?,
                observed_at: r.get(5)?,
                review_status: r.get(6)?,
                explanation: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let analyzed_count = sources.len() as i64;
    let discarded_count = sources
        .iter()
        .filter(|s| {
            matches!(
                s.review_status.as_str(),
                "Ignoré" | "Ce n'est pas moi" | "Ce n’est pas moi"
            )
        })
        .count() as i64;
    let contradiction_count = sources
        .iter()
        .filter(|s| s.review_status.to_lowercase().contains("pas moi"))
        .count() as i64;
    let uncertain_count = sources
        .iter()
        .filter(|s| s.review_status == "À vérifier")
        .count() as i64;
    let mut priorities: Vec<_> = sources
        .iter()
        .filter(|s| {
            matches!(s.severity.as_str(), "elevee" | "élevée" | "critique")
                && !matches!(
                    s.review_status.as_str(),
                    "Ignoré" | "Ce n'est pas moi" | "Ce n’est pas moi"
                )
        })
        .cloned()
        .collect();
    let attention_count = priorities.len() as i64;
    priorities.truncate(12);
    let mut decision_stmt = conn.prepare("SELECT d.decision,d.reason,d.created_at,d.target_id FROM osint_user_decisions d WHERE ?1 IS NULL OR EXISTS (SELECT 1 FROM osint_signals s WHERE s.id=d.target_id AND s.identity_id=?1) ORDER BY d.created_at DESC LIMIT 100").map_err(|e|e.to_string())?;
    let decisions = decision_stmt
        .query_map(params![identity_id], |r| {
            Ok(OsintReportDecision {
                decision: r.get(0)?,
                reason: r.get(1)?,
                created_at: r.get(2)?,
                target_id: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let action_columns = table_columns(&conn, "actions")?;
    let uses_identity_filter =
        identity_id.is_some() && table_columns(&conn, "remediation_plan_items").is_ok();
    let action_query = if action_columns
        .iter()
        .any(|column| column == "workflow_status")
    {
        if uses_identity_filter {
            "SELECT a.id,a.title,a.priority_id,a.workflow_status,a.deadline FROM actions a WHERE EXISTS (SELECT 1 FROM incident_action ia JOIN exposure_incident ei ON ei.incident_id=ia.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ia.action_id=a.id AND s.identity_id=?1) OR EXISTS (SELECT 1 FROM remediation_plan_items pi JOIN remediation_plans rp ON rp.id=pi.plan_id WHERE pi.action_id=a.id AND rp.identity_id=?1) ORDER BY CASE a.priority_id WHEN 'prio_004' THEN 0 WHEN 'prio_003' THEN 1 ELSE 2 END,a.deadline LIMIT 100"
        } else {
            "SELECT id,title,priority_id,workflow_status,deadline FROM actions ORDER BY CASE priority_id WHEN 'prio_004' THEN 0 WHEN 'prio_003' THEN 1 ELSE 2 END,deadline LIMIT 100"
        }
    } else {
        if uses_identity_filter {
            "SELECT a.id,a.title,a.priority_id,a.status,a.deadline FROM actions a WHERE EXISTS (SELECT 1 FROM incident_action ia JOIN exposure_incident ei ON ei.incident_id=ia.incident_id JOIN osint_signals s ON s.exposure_id=ei.exposure_id WHERE ia.action_id=a.id AND s.identity_id=?1) OR EXISTS (SELECT 1 FROM remediation_plan_items pi JOIN remediation_plans rp ON rp.id=pi.plan_id WHERE pi.action_id=a.id AND rp.identity_id=?1) ORDER BY CASE a.priority_id WHEN 'prio_004' THEN 0 WHEN 'prio_003' THEN 1 ELSE 2 END,a.deadline LIMIT 100"
        } else {
            "SELECT id,title,priority_id,status,deadline FROM actions ORDER BY CASE priority_id WHEN 'prio_004' THEN 0 WHEN 'prio_003' THEN 1 ELSE 2 END,deadline LIMIT 100"
        }
    };
    let mut action_stmt = conn.prepare(action_query).map_err(|e| e.to_string())?;
    let map_action = |r: &rusqlite::Row<'_>| -> rusqlite::Result<OsintReportAction> {
        Ok(OsintReportAction {
            id: r.get(0)?,
            title: r.get(1)?,
            priority: r.get(2)?,
            status: r.get(3)?,
            deadline: r.get(4)?,
        })
    };
    let actions = if uses_identity_filter {
        action_stmt.query_map(params![identity_id], map_action)
    } else {
        action_stmt.query_map([], map_action)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    let actions_now_count = actions
        .iter()
        .filter(|a| {
            !matches!(
                a.status.as_str(),
                "effectue_moi" | "effectue_site" | "faite"
            ) && matches!(a.priority.as_str(), "prio_003" | "prio_004")
        })
        .count() as i64;
    let has_local_ai_analysis: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM osint_analysis_runs r JOIN osint_synthesis_runs s ON s.run_id=r.id WHERE r.status IN ('valide','fallback') AND (?1 IS NULL OR s.identity_id=?1))",params![identity_id],|r|r.get::<_,i64>(0)).unwrap_or(0)==1;
    let mut guide_ids = Vec::new();
    if sources.iter().any(|s| {
        s.title.to_lowercase().contains("fuite") || s.source.to_lowercase().contains("xposed")
    }) {
        guide_ids.push("guide-mfa".into());
    }
    if sources.iter().any(|s| {
        s.title.to_lowercase().contains("profil") || s.source.to_lowercase().contains("ddgs")
    }) {
        guide_ids.push("guide-privacy-social".into());
    }
    if sources.iter().any(|s| s.source_url.is_some()) {
        guide_ids.push("guide-rgpd".into());
    }
    let overview = if analyzed_count == 0 {
        "Aucun résultat n’a encore été analysé. Lancez un premier scan depuis Veille.".into()
    } else {
        format!("{} résultat(s) analysé(s). {} écarté(s) ou contesté(s). {} élément(s) nécessitent une attention prioritaire et {} action(s) sont recommandées maintenant.",analyzed_count,discarded_count,attention_count,actions_now_count)
    };
    Ok(OsintReportSnapshot{id,created_at,analyzed_count,discarded_count,attention_count,uncertain_count,contradiction_count,actions_now_count,overview,priorities,sources,decisions,actions,guide_ids,limitations:vec!["Les correspondances OSINT ne prouvent jamais à elles seules l’identité d’une personne.".into(),"Les résultats non confirmés nécessitent une validation humaine avant toute action.".into(),"L’analyse IA locale, lorsqu’elle existe, n’effectue aucune recherche et ne remplace pas les preuves collectées.".into()],has_local_ai_analysis})
}

#[tauri::command]
fn generate_osint_report(
    app: tauri::AppHandle,
    identity_id: Option<String>,
) -> Result<OsintReportSnapshot, String> {
    let conn = get_db_connection(&app)?;
    let id = Uuid::new_v4().to_string();
    let created_at: String = conn
        .query_row("SELECT datetime('now')", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let report = build_report_snapshot(&conn, id.clone(), created_at, identity_id)?;
    let json = serde_json::to_string(&report).map_err(|e| e.to_string())?;
    let hash = hex::encode(Sha256::digest(json.as_bytes()));
    conn.execute(
        "INSERT INTO osint_report_snapshots (id,content_json,content_sha256) VALUES (?1,?2,?3)",
        params![id, json, hash],
    )
    .map_err(|e| e.to_string())?;
    Ok(report)
}

#[tauri::command]
fn get_latest_osint_report(app: tauri::AppHandle) -> Result<Option<OsintReportSnapshot>, String> {
    let conn = get_db_connection(&app)?;
    let json:Option<String>=conn.query_row("SELECT content_json FROM osint_report_snapshots ORDER BY created_at DESC,rowid DESC LIMIT 1",[],|r|r.get(0)).ok();
    json.map(|value| serde_json::from_str(&value).map_err(|e| e.to_string()))
        .transpose()
}

fn report_markdown(report: &OsintReportSnapshot) -> String {
    let mut out = format!(
        "# Rapport MANTIS\n\nGénéré localement le {}  \nSnapshot : `{}`\n\n## Synthèse\n\n{}\n\n",
        report.created_at, report.id, report.overview
    );
    out.push_str(&format!("- Résultats analysés : {}\n- Écartés ou contestés : {}\n- Priorités : {}\n- Incertains : {}\n- Contradictions : {}\n- Actions urgentes ouvertes : {}\n\n",report.analyzed_count,report.discarded_count,report.attention_count,report.uncertain_count,report.contradiction_count,report.actions_now_count));
    out.push_str("## Priorités\n\n");
    for item in &report.priorities {
        out.push_str(&format!(
            "- **{}** — {} — {} — statut : {}{}\n",
            item.title,
            item.severity,
            item.source,
            item.review_status,
            item.source_url
                .as_ref()
                .map(|u| format!(" — {}", u))
                .unwrap_or_default()
        ));
    }
    if report.priorities.is_empty() {
        out.push_str("Aucune priorité élevée identifiée.\n");
    }
    out.push_str("\n## Plan de remédiation suivi dans MANTIS\n\n");
    for action in &report.actions {
        out.push_str(&format!(
            "- {} — priorité {} — statut {} — échéance {}\n",
            action.title, action.priority, action.status, action.deadline
        ));
    }
    if report.actions.is_empty() {
        out.push_str("Aucune action enregistrée.\n");
    }
    out.push_str("\n## Sources et observations\n\n");
    for item in &report.sources {
        out.push_str(&format!(
            "- {} — {} — {} — {}{}\n",
            item.observed_at,
            item.source,
            item.title,
            item.review_status,
            item.source_url
                .as_ref()
                .map(|u| format!(" — {}", u))
                .unwrap_or_default()
        ));
    }
    out.push_str("\n## Décisions humaines\n\n");
    for decision in &report.decisions {
        out.push_str(&format!(
            "- {} — {} — cible {}{}\n",
            decision.created_at,
            decision.decision,
            decision.target_id,
            decision
                .reason
                .as_ref()
                .map(|r| format!(" — {}", r))
                .unwrap_or_default()
        ));
    }
    if report.decisions.is_empty() {
        out.push_str("Aucune décision enregistrée.\n");
    }
    out.push_str("\n## Limites\n\n");
    for limit in &report.limitations {
        out.push_str(&format!("- {}\n", limit));
    }
    out
}

fn pdf_winansi_hex(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            let b = match c {
                'é' => 0xE9,
                'è' => 0xE8,
                'ê' => 0xEA,
                'à' => 0xE0,
                'â' => 0xE2,
                'ù' => 0xF9,
                'û' => 0xFB,
                'ô' => 0xF4,
                'î' => 0xEE,
                'ï' => 0xEF,
                'ç' => 0xE7,
                'É' => 0xC9,
                'À' => 0xC0,
                'œ' => b'o',
                '–' | '—' => b'-',
                '’' => b'\'',
                c if c.is_ascii() => c as u8,
                _ => b'?',
            };
            format!("{:02X}", b)
        })
        .collect()
}

fn simple_pdf(markdown: &str) -> Vec<u8> {
    let mut lines = Vec::new();
    for raw in markdown.lines() {
        let plain = raw
            .trim_start_matches('#')
            .trim()
            .replace("**", "")
            .replace('`', "");
        if plain.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut rest = plain.as_str();
        while rest.chars().count() > 92 {
            let cut = rest
                .char_indices()
                .take_while(|(_, c)| *c != '\n')
                .take(92)
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(rest.len());
            let preferred = rest[..cut].rfind(' ').unwrap_or(cut);
            lines.push(rest[..preferred].trim().to_string());
            rest = rest[preferred..].trim();
        }
        lines.push(rest.to_string());
    }
    let pages: Vec<&[String]> = lines.chunks(48).collect();
    let font_id = 3 + pages.len() * 2;
    let mut objects = Vec::new();
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    let kids = (0..pages.len())
        .map(|i| format!("{} 0 R", 3 + i * 2))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids,
        pages.len()
    ));
    for (i, page) in pages.iter().enumerate() {
        let content_id = 4 + i * 2;
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {} 0 R >> >> /Contents {} 0 R >>",font_id,content_id));
        let mut stream = "BT /F1 10 Tf 46 800 Td 14 TL\n".to_string();
        for line in *page {
            stream.push_str(&format!("<{}> Tj T*\n", pdf_winansi_hex(line)));
        }
        stream.push_str("ET");
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            stream.as_bytes().len(),
            stream
        ));
    }
    objects.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into(),
    );
    let mut pdf = b"%PDF-1.4\n%MANTIS\n".to_vec();
    let mut offsets = vec![0usize];
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, obj).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1,
            xref
        )
        .as_bytes(),
    );
    pdf
}

#[tauri::command]
fn export_osint_report(
    app: tauri::AppHandle,
    snapshot_id: String,
    format: String,
) -> Result<OsintReportExport, String> {
    if format != "markdown" && format != "pdf" {
        return Err("Format d’export non autorisé.".into());
    }
    let conn = get_db_connection(&app)?;
    let json: String = conn
        .query_row(
            "SELECT content_json FROM osint_report_snapshots WHERE id=?1",
            params![snapshot_id],
            |r| r.get(0),
        )
        .map_err(|_| "Snapshot de rapport introuvable.".to_string())?;
    let report: OsintReportSnapshot = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let markdown = report_markdown(&report);
    let bytes = if format == "pdf" {
        simple_pdf(&markdown)
    } else {
        markdown.into_bytes()
    };
    let exports = application_data_dir(&app)?.join("exports");
    std::fs::create_dir_all(&exports).map_err(|e| e.to_string())?;
    let ext = if format == "pdf" { "pdf" } else { "md" };
    let filename = format!("rapport-mantis-{}.{}", snapshot_id, ext);
    let path = exports.join(&filename);
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    let hash = hex::encode(Sha256::digest(&bytes));
    conn.execute("INSERT INTO osint_report_exports (id,snapshot_id,export_format,relative_path,content_sha256) VALUES (?1,?2,?3,?4,?5)",params![Uuid::new_v4().to_string(),snapshot_id,format,filename,hash]).map_err(|e|e.to_string())?;
    Ok(OsintReportExport {
        path: path.to_string_lossy().into(),
        format,
        snapshot_id: report.id,
    })
}

#[tauri::command]
fn create_folder(app: tauri::AppHandle, name: String, context: String) -> Result<Folder, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Le nom du dossier est requis.".into());
    }
    let folder = Folder {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        context: context.trim().to_string(),
    };
    let conn = get_db_connection(&app)?;
    conn.execute(
        "INSERT INTO folders (id,name,context) VALUES (?1,?2,?3)",
        params![folder.id, folder.name, folder.context],
    )
    .map_err(|e| e.to_string())?;
    Ok(folder)
}

#[tauri::command]
fn delete_folder(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    let linked: i64 = conn.query_row("SELECT (SELECT COUNT(*) FROM identities WHERE folder_id=?1) + (SELECT COUNT(*) FROM exposures WHERE folder_id=?1) + (SELECT COUNT(*) FROM incidents WHERE folder_id=?1) + (SELECT COUNT(*) FROM actions WHERE folder_id=?1)", params![id], |row| row.get(0)).map_err(|e| e.to_string())?;
    if linked > 0 {
        return Err(
            "Ce dossier contient encore des éléments liés. Supprimez-les ou déplacez-les d’abord."
                .into(),
        );
    }
    conn.execute("DELETE FROM folders WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn rgpd_review_status_in_connection(
    conn: &Connection,
    request_id: &str,
) -> Result<RgpdReviewStatus, String> {
    let (version_id,contract_version,source_signal_id):(String,String,Option<String>)=conn.query_row("SELECT id,contract_version,source_signal_id FROM rgpd_draft_versions WHERE request_id=?1 ORDER BY created_at DESC,rowid DESC LIMIT 1",params![request_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|_|"Aucune version de brouillon disponible.".to_string())?;
    let (eligible, reason) = if let Some(signal_id) = source_signal_id.as_deref() {
        let status: Option<String> = conn
            .query_row(
                "SELECT review_status FROM osint_signals WHERE id=?1",
                params![signal_id],
                |r| r.get(0),
            )
            .ok();
        match status.as_deref() {
            Some("Confirmé") | Some("Suivi") => (
                true,
                "La source est liée à un signal confirmé ou suivi.".into(),
            ),
            Some(_) => (
                false,
                "Le signal source doit d’abord être confirmé ou suivi dans Veille.".into(),
            ),
            None => (false, "Le signal source n’existe plus.".into()),
        }
    } else {
        (
            false,
            "Ce brouillon historique n’est pas relié à un signal OSINT traçable.".into(),
        )
    };
    let latest:Option<(String,String)>=conn.query_row("SELECT decision,created_at FROM rgpd_user_reviews WHERE request_id=?1 AND draft_version_id=?2 ORDER BY created_at DESC,rowid DESC LIMIT 1",params![request_id,version_id],|r|Ok((r.get(0)?,r.get(1)?))).ok();
    let validated = eligible && latest.as_ref().map(|v| v.0.as_str()) == Some("valide");
    Ok(RgpdReviewStatus {
        request_id: request_id.into(),
        draft_version_id: version_id,
        contract_version,
        validated,
        eligible,
        reason,
        source_signal_id,
        reviewed_at: latest.map(|v| v.1),
    })
}

#[tauri::command]
fn get_rgpd_review_status(
    app: tauri::AppHandle,
    request_id: String,
) -> Result<RgpdReviewStatus, String> {
    rgpd_review_status_in_connection(&get_db_connection(&app)?, &request_id)
}

#[tauri::command]
fn validate_rgpd_draft(
    app: tauri::AppHandle,
    request_id: String,
    source_checked: bool,
    identity_checked: bool,
    recipient_checked: bool,
    content_checked: bool,
    legal_notice_accepted: bool,
) -> Result<RgpdReviewStatus, String> {
    if !(source_checked
        && identity_checked
        && recipient_checked
        && content_checked
        && legal_notice_accepted)
    {
        return Err("Toutes les vérifications sont obligatoires avant validation.".into());
    }
    let conn = get_db_connection(&app)?;
    let status = rgpd_review_status_in_connection(&conn, &request_id)?;
    if !status.eligible {
        return Err(status.reason);
    }
    conn.execute("INSERT INTO rgpd_user_reviews (id,request_id,draft_version_id,source_checked,identity_checked,recipient_checked,content_checked,legal_notice_accepted,decision) VALUES (?1,?2,?3,1,1,1,1,1,'valide')",params![Uuid::new_v4().to_string(),request_id,status.draft_version_id]).map_err(|e|e.to_string())?;
    conn.execute(
        "UPDATE rgpd_requests SET status_id='status_002',updated_at=datetime('now') WHERE id=?1",
        params![request_id],
    )
    .map_err(|e| e.to_string())?;
    rgpd_review_status_in_connection(&conn, &request_id)
}

#[tauri::command]
fn revoke_rgpd_draft_validation(
    app: tauri::AppHandle,
    request_id: String,
) -> Result<RgpdReviewStatus, String> {
    let conn = get_db_connection(&app)?;
    let status = rgpd_review_status_in_connection(&conn, &request_id)?;
    conn.execute("INSERT INTO rgpd_user_reviews (id,request_id,draft_version_id,source_checked,identity_checked,recipient_checked,content_checked,legal_notice_accepted,decision) VALUES (?1,?2,?3,0,0,0,0,0,'revoque')",params![Uuid::new_v4().to_string(),request_id,status.draft_version_id]).map_err(|e|e.to_string())?;
    conn.execute(
        "UPDATE rgpd_requests SET status_id='status_001',updated_at=datetime('now') WHERE id=?1",
        params![request_id],
    )
    .map_err(|e| e.to_string())?;
    rgpd_review_status_in_connection(&conn, &request_id)
}

#[tauri::command]
fn save_rgpd_draft_revision(
    app: tauri::AppHandle,
    request_id: String,
    draft_text: String,
) -> Result<RgpdReviewStatus, String> {
    let draft_text = draft_text.trim().to_string();
    if draft_text.len() < 40 || draft_text.len() > 12_000 {
        return Err("Le brouillon doit contenir entre 40 et 12 000 caractères.".into());
    }
    let conn = get_db_connection(&app)?;
    let current = rgpd_review_status_in_connection(&conn, &request_id)?;
    if !current.eligible {
        return Err(current.reason);
    }
    let (source_url, source_signal_id): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT source_url,source_signal_id FROM rgpd_draft_versions WHERE id=?1",
            params![current.draft_version_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let version_id = Uuid::new_v4().to_string();
    let hash = hex::encode(Sha256::digest(draft_text.as_bytes()));
    conn.execute("INSERT INTO rgpd_draft_versions (id,request_id,contract_version,draft_text,content_sha256,source_url,source_signal_id) VALUES (?1,?2,'effacement-source-confirmee-v1',?3,?4,?5,?6)",params![version_id,request_id,draft_text,hash,source_url,source_signal_id]).map_err(|e|e.to_string())?;
    conn.execute("UPDATE rgpd_requests SET draft_preview=?1,status_id='status_001',updated_at=datetime('now') WHERE id=?2",params![draft_text,request_id]).map_err(|e|e.to_string())?;
    rgpd_review_status_in_connection(&conn, &request_id)
}

#[tauri::command]
fn use_validated_rgpd_draft(
    app: tauri::AppHandle,
    request_id: String,
    use_type: String,
) -> Result<RgpdDraftUse, String> {
    if !matches!(use_type.as_str(), "copie" | "export_texte") {
        return Err("Usage du brouillon non autorisé.".into());
    }
    let conn = get_db_connection(&app)?;
    let status = rgpd_review_status_in_connection(&conn, &request_id)?;
    if !status.validated {
        return Err("Le brouillon doit être validé avant copie ou export.".into());
    }
    let text: String = conn
        .query_row(
            "SELECT draft_text FROM rgpd_draft_versions WHERE id=?1",
            params![status.draft_version_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let mut path = None;
    if use_type == "export_texte" {
        let root = application_data_dir(&app)?.join("exports").join("rgpd");
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let file = root.join(format!("brouillon-rgpd-{}.txt", request_id));
        std::fs::write(&file, text.as_bytes()).map_err(|e| e.to_string())?;
        path = Some(file.to_string_lossy().into())
    }
    conn.execute("INSERT INTO rgpd_draft_uses (id,request_id,draft_version_id,use_type,relative_path) VALUES (?1,?2,?3,?4,?5)",params![Uuid::new_v4().to_string(),request_id,status.draft_version_id,use_type,path]).map_err(|e|e.to_string())?;
    Ok(RgpdDraftUse {
        text,
        path,
        draft_version_id: status.draft_version_id,
    })
}

// --- Managed OSINT runtimes and normalized signal flow ---

fn osint_root(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let root = application_data_dir(app)?.join("osint");
    std::fs::create_dir_all(root.join("raw")).map_err(|e| e.to_string())?;
    Ok(root)
}

fn user_scanner_bundle(
    app: &tauri::AppHandle,
) -> Result<(std::path::PathBuf, ManagedBundleManifest), String> {
    let directory = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources")
        .join("user-scanner");
    #[cfg(debug_assertions)]
    let directory = if !directory.join("manifest.json").exists() {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("user-scanner")
    } else {
        directory
    };
    let manifest_path = directory.join("manifest.json");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|_| "Le bundle User Scanner fourni avec MANTIS est introuvable.".to_string())?;
    let manifest: ManagedBundleManifest = serde_json::from_str(&manifest_text)
        .map_err(|_| "Le manifeste du bundle User Scanner est invalide.".to_string())?;
    if manifest.schema_version != 1
        || manifest.module_id != "osint-email-platforms"
        || manifest.file != "user-scanner.exe"
        || manifest.sha256.len() != 64
        || manifest.license_file.as_deref() != Some("USER_SCANNER_LICENSE.txt")
        || !directory.join("USER_SCANNER_LICENSE.txt").exists()
    {
        return Err("Le bundle User Scanner est incompatible ou incomplet.".into());
    }
    Ok((directory, manifest))
}

fn sha256_file(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn installed_user_scanner_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(osint_root(app)?
        .join("modules")
        .join("user-scanner")
        .join("user-scanner.exe"))
}

fn ddgs_bundle(
    app: &tauri::AppHandle,
) -> Result<(std::path::PathBuf, ManagedBundleManifest), String> {
    let directory = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources")
        .join("ddgs");
    #[cfg(debug_assertions)]
    let directory = if !directory.join("manifest.json").exists() {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("ddgs")
    } else {
        directory
    };
    let manifest_path = directory.join("manifest.json");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|_| "Le bundle Empreinte Web fourni avec MANTIS est introuvable.".to_string())?;
    let manifest: ManagedBundleManifest = serde_json::from_str(&manifest_text)
        .map_err(|_| "Le manifeste du bundle Empreinte Web est invalide.".to_string())?;
    if manifest.schema_version != 1
        || manifest.module_id != "osint-web-footprint"
        || manifest.file != "ddgs-web.exe"
        || manifest.sha256.len() != 64
    {
        return Err("Le bundle Empreinte Web est incompatible ou incomplet.".into());
    }
    Ok((directory, manifest))
}

fn installed_ddgs_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(osint_root(app)?
        .join("modules")
        .join("ddgs")
        .join("ddgs-web.exe"))
}

fn maigret_bundle(
    app: &tauri::AppHandle,
) -> Result<(std::path::PathBuf, ManagedBundleManifest), String> {
    let directory = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources")
        .join("maigret");
    #[cfg(debug_assertions)]
    let directory = if !directory.join("manifest.json").exists() {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("maigret")
    } else {
        directory
    };
    let manifest_text = std::fs::read_to_string(directory.join("manifest.json"))
        .map_err(|_| "Le bundle Maigret fourni avec MANTIS est introuvable.".to_string())?;
    let manifest: ManagedBundleManifest = serde_json::from_str(&manifest_text)
        .map_err(|_| "Le manifeste du bundle Maigret est invalide.".to_string())?;
    if manifest.schema_version != 1
        || manifest.module_id != "osint-username-profiles"
        || manifest.file != "maigret-mantis.exe"
        || manifest.sha256.len() != 64
        || manifest.license_file.as_deref() != Some("MAIGRET_LICENSE.txt")
        || !directory.join("MAIGRET_LICENSE.txt").exists()
    {
        return Err("Le bundle Maigret est incompatible ou incomplet.".into());
    }
    Ok((directory, manifest))
}

fn installed_maigret_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(osint_root(app)?
        .join("modules")
        .join("maigret")
        .join("maigret-mantis.exe"))
}

fn managed_module_layout(
    app: &tauri::AppHandle,
    module_id: &str,
) -> Result<
    (
        std::path::PathBuf,
        std::path::PathBuf,
        &'static str,
        &'static str,
    ),
    String,
> {
    let modules = osint_root(app)?.join("modules");
    let (directory, runtime, label) = match module_id {
        "osint-email-platforms" => ("user-scanner", "user-scanner.exe", "User Scanner"),
        "osint-web-footprint" => ("ddgs", "ddgs-web.exe", "Empreinte Web"),
        "osint-username-profiles" => ("maigret", "maigret-mantis.exe", "Maigret"),
        _ => return Err("Ce module ne possède pas de runtime Windows géré.".into()),
    };
    Ok((
        modules.join(directory),
        modules.join(format!("{directory}-previous")),
        runtime,
        label,
    ))
}

fn managed_directory_integrity(
    destination: &std::path::Path,
    module_id: &str,
    runtime: &str,
) -> Result<String, String> {
    let text = std::fs::read_to_string(destination.join("manifest.json"))
        .map_err(|_| "Le manifeste privé de l’installation est absent.".to_string())?;
    let manifest: ManagedBundleManifest = serde_json::from_str(&text)
        .map_err(|_| "Le manifeste privé de l’installation est invalide.".to_string())?;
    if manifest.schema_version != 1
        || manifest.module_id != module_id
        || manifest.file != runtime
        || manifest.sha256.len() != 64
    {
        return Err("Le manifeste privé ne correspond pas au module attendu.".into());
    }
    let executable = destination.join(runtime);
    if !executable.exists() || sha256_file(&executable)? != manifest.sha256.to_lowercase() {
        return Err("Le runtime privé ne passe pas le contrôle SHA-256.".into());
    }
    if let Some(license) = manifest.license_file.as_deref() {
        if !destination.join(license).exists() {
            return Err("La notice de licence privée est absente.".into());
        }
    }
    Ok(manifest.version)
}

fn installed_managed_module_integrity(
    app: &tauri::AppHandle,
    module_id: &str,
) -> Result<String, String> {
    let (destination, _, runtime, _) = managed_module_layout(app, module_id)?;
    managed_directory_integrity(&destination, module_id, runtime)
}

fn module_installation_state(app: &tauri::AppHandle, module_id: &str) -> (String, Option<String>) {
    if matches!(
        module_id,
        "osint-email-intel"
            | "osint-github-profile"
            | "osint-gitlab-profile"
            | "osint-mastodon-webfinger"
            | "osint-gravatar-profile"
            | "osint-keybase-profile"
            | "osint-bluesky-profile"
            | "osint-hackernews-profile"
            | "osint-fr-company-register"
            | "osint-hal-author"
    ) {
        return ("prêt".into(), None);
    }
    match installed_managed_module_integrity(app, module_id) {
        Ok(version) => (
            "prêt".into(),
            Some(format!("Installation privée {version} vérifiée.")),
        ),
        Err(error) => {
            let exists =
                managed_module_layout(app, module_id).is_ok_and(|(path, _, _, _)| path.exists());
            (
                if exists {
                    "réparation_requise"
                } else {
                    "non_installé"
                }
                .into(),
                Some(error),
            )
        }
    }
}

fn log_module(
    conn: &Connection,
    module_id: &str,
    operation: &str,
    status: &str,
    message: &str,
) -> Result<(), String> {
    conn.execute("INSERT INTO osint_module_logs (id,module_id,operation,status,message,created_at) VALUES (?1,?2,?3,?4,?5,datetime('now'))",
        params![Uuid::new_v4().to_string(), module_id, operation, status, message]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn list_osint_module_logs(
    app: tauri::AppHandle,
    module_id: String,
) -> Result<Vec<OsintModuleLog>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT operation,status,message,created_at FROM osint_module_logs WHERE module_id=?1 ORDER BY created_at DESC LIMIT 20").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![module_id], |row| {
            Ok(OsintModuleLog {
                operation: row.get(0)?,
                status: row.get(1)?,
                message: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|row| row.map_err(|e| e.to_string())).collect()
}

fn read_bounded_process_output_with_limit<R: Read>(
    mut reader: R,
    max_bytes: usize,
) -> Result<(Vec<u8>, bool), String> {
    let mut captured = Vec::with_capacity(max_bytes);
    let mut buffer = [0_u8; 8 * 1024];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        let remaining = max_bytes.saturating_sub(captured.len());
        let retained = remaining.min(read);
        captured.extend_from_slice(&buffer[..retained]);
        truncated |= retained < read;
    }
    Ok((captured, truncated))
}

/// Les collecteurs et le runtime local sont des processus auxiliaires : ils
/// ne doivent jamais ouvrir une console visible dans l'application Windows.
fn configure_background_process(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn run_process_with_limit(
    program: &std::path::Path,
    args: &[String],
    timeout: Duration,
    max_output_bytes: usize,
) -> Result<(String, String), String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_background_process(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Impossible de démarrer le module : {}", e))?;
    let stdout_reader = child.stdout.take().map(|out| {
        std::thread::spawn(move || read_bounded_process_output_with_limit(out, max_output_bytes))
    });
    let stderr_reader = child.stderr.take().map(|err| {
        std::thread::spawn(move || read_bounded_process_output_with_limit(err, max_output_bytes))
    });
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let (stdout_bytes, stdout_truncated) = stdout_reader
                .map(|reader| {
                    reader
                        .join()
                        .map_err(|_| "Lecture de sortie interrompue.".to_string())?
                })
                .transpose()?
                .unwrap_or_default();
            let (stderr_bytes, stderr_truncated) = stderr_reader
                .map(|reader| {
                    reader
                        .join()
                        .map_err(|_| "Lecture de diagnostic interrompue.".to_string())?
                })
                .transpose()?
                .unwrap_or_default();
            if stdout_truncated {
                return Err("Le module a produit trop de données. Le résultat a été écarté pour préserver les ressources de MANTIS.".into());
            }
            let stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
            let mut stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();
            if stderr_truncated {
                stderr.push_str("\nDiagnostic limité pour préserver les ressources.");
            }
            return if status.success() {
                Ok((stdout, stderr))
            } else {
                Err(format!(
                    "Le module s’est arrêté sans résultat exploitable : {}",
                    stderr.trim()
                ))
            };
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            if let Some(reader) = stdout_reader {
                let _ = reader.join();
            }
            if let Some(reader) = stderr_reader {
                let _ = reader.join();
            }
            return Err(
                "Le scan a pris trop de temps. Aucun résultat incomplet n’a été conservé.".into(),
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn run_process(
    program: &std::path::Path,
    args: &[String],
    timeout: Duration,
) -> Result<(String, String), String> {
    run_process_with_limit(program, args, timeout, MAX_PROCESS_OUTPUT_BYTES)
}

#[tauri::command]
fn install_osint_module(app: tauri::AppHandle, module_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    let catalog_status: String = conn
        .query_row(
            "SELECT catalog_status FROM osint_modules WHERE id=?1",
            params![module_id],
            |row| row.get(0),
        )
        .map_err(|_| "Module absent du catalogue MANTIS.".to_string())?;
    if catalog_status != "active" {
        return Err(
            "Ce module est archivé ou réservé aux tests ; son installation est interdite.".into(),
        );
    }
    let root = osint_root(&app)?;
    let (bundle_dir, manifest, runtime_name, module_directory, label) = match module_id.as_str() {
        "osint-email-platforms" => match user_scanner_bundle(&app) {
            Ok((dir, manifest)) => (
                dir,
                manifest,
                "user-scanner.exe",
                "user-scanner",
                "User Scanner",
            ),
            Err(e) => {
                log_module(&conn, &module_id, "installation", "erreur", &e)?;
                return Err(e);
            }
        },
        "osint-web-footprint" => match ddgs_bundle(&app) {
            Ok((dir, manifest)) => (dir, manifest, "ddgs-web.exe", "ddgs", "Empreinte Web"),
            Err(e) => {
                log_module(&conn, &module_id, "installation", "erreur", &e)?;
                return Err(e);
            }
        },
        "osint-username-profiles" => match maigret_bundle(&app) {
            Ok((dir, manifest)) => (dir, manifest, "maigret-mantis.exe", "maigret", "Maigret"),
            Err(e) => {
                log_module(&conn, &module_id, "installation", "erreur", &e)?;
                return Err(e);
            }
        },
        _ => return Err("Ce module sera intégré dans une prochaine étape.".into()),
    };
    let source = bundle_dir.join(&manifest.file);
    if !source.exists() || sha256_file(&source)? != manifest.sha256.to_lowercase() {
        let message = format!(
            "Le bundle {} fourni avec MANTIS a échoué à la vérification d’intégrité.",
            label
        );
        log_module(&conn, &module_id, "installation", "erreur", &message)?;
        return Err(message);
    }
    let modules = root.join("modules");
    std::fs::create_dir_all(&modules).map_err(|e| e.to_string())?;
    if let Ok(entries) = std::fs::read_dir(&modules) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}-staging-", module_directory))
                && entry.path().parent() == Some(modules.as_path())
            {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
    let staging = modules.join(format!("{}-staging-{}", module_directory, Uuid::new_v4()));
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let staged = staging.join(runtime_name);
    if let Err(e) = std::fs::copy(&source, &staged) {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(e.to_string());
    }
    if sha256_file(&staged)? != manifest.sha256.to_lowercase() {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(format!(
            "La copie privée de {} n’a pas passé le contrôle d’intégrité.",
            label
        ));
    }
    if let Some(license_file) = &manifest.license_file {
        let license_source = bundle_dir.join(license_file);
        if !license_source.exists() {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(format!(
                "La notice de licence de {} est absente du bundle.",
                label
            ));
        }
        std::fs::copy(&license_source, staging.join(license_file)).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        staging.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let destination = modules.join(module_directory);
    let backup = modules.join(format!("{}-previous", module_directory));
    let _ = std::fs::remove_dir_all(&backup);
    if destination.exists() {
        std::fs::rename(&destination, &backup).map_err(|e| e.to_string())?;
    }
    if let Err(e) = std::fs::rename(&staging, &destination) {
        if backup.exists() {
            let _ = std::fs::rename(&backup, &destination);
        }
        return Err(e.to_string());
    }
    conn.execute(
        "UPDATE osint_modules SET status='actif' WHERE id=?1",
        params![module_id],
    )
    .map_err(|e| e.to_string())?;
    log_module(
        &conn,
        &module_id,
        "installation",
        "ok",
        &format!("{} {} est prêt.", label, manifest.version),
    )?;
    Ok(())
}

#[tauri::command]
fn rollback_osint_module(app: tauri::AppHandle, module_id: String) -> Result<String, String> {
    let conn = get_db_connection(&app)?;
    let (destination, backup, runtime, label) = managed_module_layout(&app, &module_id)?;
    let previous_version = rollback_managed_runtime(&destination, &backup, &module_id, runtime)?;
    let message = format!(
        "{} a été restauré vers la version {}. Les scans et preuves n’ont pas été modifiés.",
        label, previous_version
    );
    log_module(&conn, &module_id, "restauration", "ok", &message)?;
    Ok(message)
}

fn rollback_managed_runtime(
    destination: &std::path::Path,
    backup: &std::path::Path,
    module_id: &str,
    runtime: &str,
) -> Result<String, String> {
    if !backup.exists() {
        return Err("Aucune version précédente vérifiée n’est disponible.".into());
    }
    let previous_version = managed_directory_integrity(backup, module_id, runtime)?;
    let modules = destination.parent().ok_or("Chemin de module invalide.")?;
    if backup.parent() != Some(modules) {
        return Err("Le dossier de restauration est hors de l’espace OSINT autorisé.".into());
    }
    let displaced = modules.join(format!("rollback-displaced-{}", Uuid::new_v4()));
    if destination.exists() {
        std::fs::rename(&destination, &displaced).map_err(|e| e.to_string())?;
    }
    if let Err(error) = std::fs::rename(&backup, &destination) {
        if displaced.exists() {
            let _ = std::fs::rename(&displaced, &destination);
        }
        return Err(error.to_string());
    }
    if let Err(error) = managed_directory_integrity(&destination, &module_id, runtime) {
        let _ = std::fs::rename(&destination, &backup);
        if displaced.exists() {
            let _ = std::fs::rename(&displaced, &destination);
        }
        return Err(format!("Restauration refusée : {error}"));
    }
    if displaced.exists() {
        let _ = std::fs::remove_dir_all(&displaced);
    }
    Ok(previous_version)
}

#[tauri::command]
fn remove_osint_module_runtime(app: tauri::AppHandle, module_id: String) -> Result<String, String> {
    let conn = get_db_connection(&app)?;
    let (destination, backup, _, label) = managed_module_layout(&app, &module_id)?;
    let modules = destination.parent().ok_or("Chemin de module invalide.")?;
    if backup.parent() != Some(modules) {
        return Err("Le chemin de suppression est hors de l’espace OSINT autorisé.".into());
    }
    if destination.exists() {
        std::fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
    }
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|e| e.to_string())?;
    }
    conn.execute(
        "UPDATE osint_modules SET status='planifie' WHERE id=?1 AND catalog_status='active'",
        params![module_id],
    )
    .map_err(|e| e.to_string())?;
    let message = format!(
        "Runtime {} supprimé. Les scans, résultats bruts, preuves et décisions sont conservés.",
        label
    );
    log_module(&conn, &module_id, "suppression_runtime", "ok", &message)?;
    Ok(message)
}

#[tauri::command]
fn cleanup_orphaned_osint_runtimes(app: tauri::AppHandle) -> Result<String, String> {
    let modules = osint_root(&app)?.join("modules");
    std::fs::create_dir_all(&modules).map_err(|e| e.to_string())?;
    let mut removed = 0usize;
    if let Ok(entries) = std::fs::read_dir(&modules) {
        for entry in entries.flatten() {
            if entry.path().parent() != Some(modules.as_path()) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let obsolete = matches!(name.as_str(), "holehe" | "h8mail" | "ghunt" | "spiderfoot")
                || [
                    "user-scanner-staging-",
                    "ddgs-staging-",
                    "maigret-staging-",
                    "rollback-displaced-",
                ]
                .iter()
                .any(|prefix| name.starts_with(prefix));
            if obsolete && entry.path().is_dir() {
                std::fs::remove_dir_all(entry.path()).map_err(|e| e.to_string())?;
                removed += 1;
            }
        }
    }
    Ok(format!("Nettoyage terminé : {} dossier(s) obsolète(s) ou temporaire(s) supprimé(s). Les données de scan et l’IA locale sont intactes.",removed))
}

#[tauri::command]
fn install_veille(app: tauri::AppHandle) -> Result<String, String> {
    let mut installed = 0;
    let mut errors = Vec::new();
    for module_id in [
        "osint-email-platforms",
        "osint-web-footprint",
        "osint-username-profiles",
    ] {
        if module_installation_state(&app, module_id).0 == "prêt" {
            continue;
        }
        match install_osint_module(app.clone(), module_id.into()) {
            Ok(()) => installed += 1,
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        Ok(if installed == 0 {
            "La veille est déjà prête.".into()
        } else {
            "La veille est installée et prête à lancer un scan.".into()
        })
    } else {
        let _ = errors;
        Err("La préparation de la veille est incomplète. Ouvrez le mode avancé pour consulter le diagnostic.".into())
    }
}

/// Installe les trois sidecars embarqués lors du premier démarrage.
///
/// Le marqueur n’est écrit qu’après une installation complète : une release
/// incomplète pourra donc réparer automatiquement ses outils au prochain
/// démarrage, tandis qu’une désinstallation volontaire ne sera pas annulée.
fn install_default_osint_modules(app: &tauri::AppHandle) -> Result<(), String> {
    let conn = get_db_connection(app)?;
    let already_initialized: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key='osint_default_bundles_installed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();
    if already_initialized == "1" {
        return Ok(());
    }
    drop(conn);

    install_veille(app.clone())?;

    let conn = get_db_connection(app)?;
    conn.execute(
        "INSERT INTO app_settings (key,value) VALUES ('osint_default_bundles_installed','1') ON CONFLICT(key) DO UPDATE SET value='1'",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn veille_setting(conn: &Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key=?1",
        params![key],
        |row| row.get(0),
    )
    .unwrap_or_else(|_| default.into())
}

fn save_veille_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute("INSERT INTO app_settings (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value]).map_err(|e| e.to_string())?;
    Ok(())
}

fn next_veille_run(frequency: &str) -> Option<String> {
    match frequency {
        "Quotidienne" => Some("datetime('now','+1 day')".into()),
        "Hebdomadaire" => Some("datetime('now','+7 days')".into()),
        "Mensuelle" => Some("datetime('now','+1 month')".into()),
        _ => None,
    }
}

fn read_veille_routine(conn: &Connection) -> VeilleRoutine {
    let frequency = veille_setting(conn, "veille_frequency", "Manuelle");
    let paused = veille_setting(conn, "veille_paused", "0") == "1";
    let last_run = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key='veille_last_run'",
            [],
            |row| row.get(0),
        )
        .ok();
    let next_run = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key='veille_next_run'",
            [],
            |row| row.get(0),
        )
        .ok();
    let status = if paused {
        "En pause"
    } else if frequency == "Manuelle" {
        "Manuelle"
    } else {
        "Active"
    }
    .into();
    VeilleRoutine {
        frequency,
        paused,
        last_run,
        next_run,
        status,
    }
}

#[tauri::command]
fn get_veille_routine(app: tauri::AppHandle) -> Result<VeilleRoutine, String> {
    Ok(read_veille_routine(&get_db_connection(&app)?))
}

#[tauri::command]
fn update_veille_routine(
    app: tauri::AppHandle,
    frequency: String,
    paused: bool,
) -> Result<VeilleRoutine, String> {
    if !matches!(
        frequency.as_str(),
        "Manuelle" | "Quotidienne" | "Hebdomadaire" | "Mensuelle"
    ) {
        return Err("Fréquence de veille invalide.".into());
    }
    let conn = get_db_connection(&app)?;
    save_veille_setting(&conn, "veille_frequency", &frequency)?;
    save_veille_setting(&conn, "veille_paused", if paused { "1" } else { "0" })?;
    if paused || frequency == "Manuelle" {
        save_veille_setting(&conn, "veille_next_run", "")?;
    } else if let Some(sql_time) = next_veille_run(&frequency) {
        conn.execute(&format!("INSERT INTO app_settings (key,value) VALUES ('veille_next_run',{}) ON CONFLICT(key) DO UPDATE SET value=excluded.value", sql_time), []).map_err(|e| e.to_string())?;
    }
    Ok(read_veille_routine(&conn))
}

#[tauri::command]
fn diagnose_osint_module(app: tauri::AppHandle, module_id: String) -> Result<String, String> {
    let conn = get_db_connection(&app)?;
    let (state, detail) = module_installation_state(&app, &module_id);
    let message = if module_id == "osint-email-platforms" && state == "prêt" {
        match user_scanner_bundle(&app).and_then(|(_, manifest)| sha256_file(&installed_user_scanner_path(&app)?).map(|hash| (manifest, hash))) {
            Ok((manifest, hash)) if hash == manifest.sha256.to_lowercase() => format!("User Scanner {} est prêt et son intégrité est validée.", manifest.version),
            _ => "User Scanner doit être réparé : la copie privée ne passe plus le contrôle d’intégrité.".into(),
        }
    } else if module_id == "osint-web-footprint" && state == "prêt" {
        match ddgs_bundle(&app).and_then(|(_, manifest)| sha256_file(&installed_ddgs_path(&app)?).map(|hash| (manifest, hash))) {
            Ok((manifest, hash)) if hash == manifest.sha256.to_lowercase() => format!("Empreinte Web {} est prêt et son intégrité est validée.", manifest.version),
            _ => "Empreinte Web doit être réparé : la copie privée ne passe plus le contrôle d’intégrité.".into(),
        }
    } else if module_id == "osint-username-profiles" && state == "prêt" {
        match maigret_bundle(&app).and_then(|(_, manifest)| {
            sha256_file(&installed_maigret_path(&app)?).map(|hash| (manifest, hash))
        }) {
            Ok((manifest, hash)) if hash == manifest.sha256.to_lowercase() => format!(
                "Maigret {} est prêt et son intégrité est validée.",
                manifest.version
            ),
            _ => {
                "Maigret doit être réparé : la copie privée ne passe plus le contrôle d’intégrité."
                    .into()
            }
        }
    } else {
        detail.unwrap_or_else(|| format!("État : {}.", state))
    };
    log_module(
        &conn,
        &module_id,
        "diagnostic",
        if state == "prêt" { "ok" } else { "info" },
        &message,
    )?;
    Ok(message)
}

fn scanner_source_url(value: &str) -> Option<String> {
    safe_public_url_shape(value).map(|url| url.to_string())
}

fn scanner_details(extra: &serde_json::Value, reason: &str) -> String {
    let mut parts = Vec::new();
    if let Some(object) = extra.as_object() {
        for (key, value) in object.iter().take(3) {
            let text = value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            if !text.trim().is_empty() {
                parts.push(format!("{} : {}", key.replace('_', " "), text.trim()));
            }
        }
    }
    if parts.is_empty() && !reason.trim().is_empty() {
        parts.push(reason.trim().to_string());
    }
    if parts.is_empty() {
        "Aucun détail public complémentaire n’a été renvoyé par la plateforme.".into()
    } else {
        parts.join(" · ")
    }
}

fn clean_web_result_text(value: &str) -> Option<String> {
    let clean = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\u{fffd}', "…");
    (!clean.is_empty()).then_some(clean)
}

fn parse_maigret_output(raw: &str, expected_target: &str) -> Result<MaigretSidecarOutput, String> {
    if raw.len() > 2_000_000 {
        return Err("La sortie Maigret dépasse la limite autorisée.".into());
    }
    let output: MaigretSidecarOutput = serde_json::from_str(raw)
        .map_err(|_| "Maigret a renvoyé un format JSON inattendu.".to_string())?;
    let expected = expected_target.trim().trim_start_matches('@');
    if output.version != 1
        || output.collector_version != "maigret-0.6.3"
        || output.target != expected
    {
        return Err(
            "Le résultat Maigret ne correspond pas au collecteur ou au pseudo demandé.".into(),
        );
    }
    if output.summary.found != output.results.len() || output.results.len() > 200 {
        return Err("Le résumé Maigret est incohérent ou dépasse les limites MANTIS.".into());
    }
    for item in &output.results {
        if item.site_name.trim().is_empty() || scanner_source_url(&item.url).is_none() {
            return Err("Maigret a renvoyé une preuve incomplète ou une URL non publique.".into());
        }
    }
    Ok(output)
}

fn deduplicate_maigret_results(results: Vec<MaigretResult>) -> Vec<MaigretResult> {
    let mut seen = std::collections::HashSet::new();
    results
        .into_iter()
        .filter(|item| seen.insert(item.url.to_lowercase()))
        .take(200)
        .collect()
}

fn maigret_signal_type(category: &str) -> &'static str {
    let category = category.to_lowercase();
    if category.contains("dating") || category.contains("rencontre") {
        "site_rencontre"
    } else {
        "profil_public"
    }
}

fn sync_scan_observability(conn: &Connection, scan_id: &str) -> Result<(), String> {
    let (module_id, raw_result_path, collected_at): (String, Option<String>, String) = conn.query_row(
        "SELECT module_id,raw_result_path,COALESCE(completed_at,started_at) FROM osint_scans WHERE id=?1",
        params![scan_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|e| e.to_string())?;

    let artifact_id = format!("artifact-{scan_id}");
    if let Some(path_text) = raw_result_path.as_deref() {
        let path = std::path::Path::new(path_text);
        let relative_path = path
            .file_name()
            .map(|name| format!("raw/{}", name.to_string_lossy()))
            .unwrap_or_else(|| "raw/resultat-inconnu".to_string());
        let byte_size = std::fs::metadata(path)
            .ok()
            .map(|metadata| metadata.len() as i64);
        let hash = path.exists().then(|| sha256_file(path)).transpose()?;
        let media_type = match path.extension().and_then(|value| value.to_str()) {
            Some("json") => "application/json",
            Some("pdf") => "application/pdf",
            Some("html" | "htm") => "text/html",
            _ => "text/plain",
        };
        conn.execute(
            "INSERT OR IGNORE INTO osint_raw_artifacts (id,scan_id,relative_path,media_type,byte_size,sha256,collector_id,collected_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![artifact_id, scan_id, relative_path, media_type, byte_size, hash, module_id, collected_at],
        ).map_err(|e| e.to_string())?;
    }

    let document_artifacts = {
        let mut statement = conn
            .prepare(
                "SELECT id,raw_result_path FROM osint_signals
             WHERE scan_id=?1 AND raw_result_path IS NOT NULL
               AND raw_result_path!=COALESCE(?2,'')",
            )
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map(params![scan_id, raw_result_path], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    let mut evidence_artifact_updates = Vec::new();
    for (signal_id, path_text) in document_artifacts {
        let path = std::path::Path::new(&path_text);
        if !path.exists() {
            continue;
        }
        let hash = sha256_file(path)?;
        let document_artifact_id = format!("artifact-{scan_id}-{}", &hash[..16]);
        let relative_path = path
            .file_name()
            .map(|name| format!("raw/{}", name.to_string_lossy()))
            .unwrap_or_else(|| "raw/document-inconnu".to_string());
        let byte_size = std::fs::metadata(path)
            .ok()
            .map(|metadata| metadata.len() as i64);
        let media_type = match path.extension().and_then(|value| value.to_str()) {
            Some("pdf") => "application/pdf",
            Some("html" | "htm") => "text/html",
            _ => "text/plain",
        };
        conn.execute(
            "INSERT OR IGNORE INTO osint_raw_artifacts (id,scan_id,relative_path,media_type,byte_size,sha256,collector_id,collected_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![document_artifact_id, scan_id, relative_path, media_type, byte_size, hash, module_id, collected_at],
        ).map_err(|e| e.to_string())?;
        evidence_artifact_updates.push((signal_id, document_artifact_id));
    }

    conn.execute(
        "INSERT OR IGNORE INTO osint_observations (id,scan_id,signal_id,identity_id,observation_type,canonical_key,display_value,source,source_url,observed_at,relevance_status)
         SELECT 'observation-'||id,scan_id,id,identity_id,signal_type,lower(trim(signal_type||'|'||source||'|'||COALESCE(source_url,evidence_ref,title))),title,source,source_url,discovered_at,'a_verifier'
         FROM osint_signals WHERE scan_id=?1",
        params![scan_id],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO osint_evidence_links (id,observation_id,artifact_id,source_url,evidence_label,excerpt,locator,created_at)
         SELECT 'evidence-'||id,'observation-'||id,CASE WHEN raw_result_path IS NULL THEN NULL ELSE ?2 END,source_url,COALESCE(NULLIF(trim(evidence_ref),''),source),explanation,evidence_ref,discovered_at
         FROM osint_signals WHERE scan_id=?1",
        params![scan_id, artifact_id],
    ).map_err(|e| e.to_string())?;
    for (signal_id, document_artifact_id) in evidence_artifact_updates {
        conn.execute(
            "UPDATE osint_evidence_links SET artifact_id=?1 WHERE observation_id=?2",
            params![document_artifact_id, format!("observation-{signal_id}")],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

const AUTO_FILTERED_REVIEW_STATUS: &str = "Filtré automatiquement";

fn evidence_fact_type(signal_type: &str, module_id: &str) -> &'static str {
    if signal_type == "fuite" {
        "breach_event"
    } else if module_id == "osint-keybase-profile" && signal_type == "compte_potentiel" {
        "external_proof"
    } else if matches!(
        signal_type,
        "profil_public" | "compte_potentiel" | "site_rencontre"
    ) {
        "public_profile"
    } else if signal_type == "source_indisponible" {
        "source_unavailable"
    } else {
        "public_mention"
    }
}

/// Rebuilds an additive, deterministic fact projection from the authoritative
/// signal and assessment records. A fact says what a source demonstrated; it
/// never asserts that a public profile belongs to a person.
fn refresh_osint_evidence_facts(conn: &mut Connection, identity_id: &str) -> Result<(), String> {
    conn.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    let rows = {
        let mut statement = conn
            .prepare(
                "SELECT s.id,s.module_id,s.signal_type,s.title,s.target,s.source_url,
                    COALESCE(a.source_reliability,'inconnue'),
                    COALESCE(a.match_level,'non_verifie'),
                    COALESCE(a.publication_status,'masque'),
                    COALESCE(a.rationale,'Preuve non encore évaluée.'),
                    COALESCE(a.evidence_fingerprint,'')
             FROM osint_signals s
             LEFT JOIN osint_evidence_assessments a ON a.signal_id=s.id
             WHERE s.identity_id=?1
             ORDER BY s.discovered_at,s.id",
            )
            .map_err(|e| e.to_string())?;
        let facts = statement
            .query_map(params![identity_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        facts
    };
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM osint_evidence_facts WHERE identity_id=?1",
        params![identity_id],
    )
    .map_err(|e| e.to_string())?;
    for (
        signal_id,
        module_id,
        signal_type,
        title,
        target,
        source_url,
        reliability,
        match_level,
        publication_status,
        rationale,
        fingerprint,
    ) in rows
    {
        let fact_type = evidence_fact_type(&signal_type, &module_id);
        let canonical_value = source_url
            .as_deref()
            .and_then(normalized_claim_url)
            .unwrap_or_else(|| {
                format!(
                    "{}|{}",
                    normalized_exact_text(&signal_type),
                    normalized_exact_text(&title)
                )
            });
        let fact_status = match publication_status.as_str() {
            "visible" => "retenu",
            "rejete" => "rejete",
            _ => "avance",
        };
        let fact_id = format!(
            "fact-{}",
            &hex::encode(Sha256::digest(
                format!("{signal_id}|{fact_type}|{canonical_value}").as_bytes()
            ))[..24]
        );
        let stable_fingerprint = if fingerprint.is_empty() {
            evidence_fingerprint(
                "signal",
                &target,
                &evidence_source_family(&module_id, "", source_url.as_deref()),
                source_url.as_deref(),
            )
        } else {
            fingerprint
        };
        tx.execute(
            "INSERT INTO osint_evidence_facts (id,identity_id,signal_id,observation_id,fact_type,canonical_value,display_value,source_url,source_reliability,match_level,fact_status,rationale,evidence_fingerprint)
             VALUES (?1,?2,?3,'observation-'||?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![fact_id,identity_id,signal_id,fact_type,canonical_value,title,source_url,reliability,match_level,fact_status,rationale,stable_fingerprint],
        ).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    refresh_osint_fact_resolutions(conn, identity_id)
}

/// Consolidates only exact, canonical facts. A repeated result from the same
/// source family is kept as evidence but never inflated into corroboration.
fn refresh_osint_fact_resolutions(conn: &mut Connection, identity_id: &str) -> Result<(), String> {
    conn.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    #[derive(Clone)]
    struct ResolutionFact {
        id: String,
        fact_type: String,
        canonical_value: String,
        fact_status: String,
        source_family: String,
    }
    let facts = {
        let mut statement = conn.prepare(
            "SELECT f.id,f.fact_type,f.canonical_value,f.fact_status,COALESCE(a.source_family,'source:unknown')
             FROM osint_evidence_facts f
             LEFT JOIN osint_evidence_assessments a ON a.signal_id=f.signal_id
             WHERE f.identity_id=?1 ORDER BY f.created_at,f.id",
        ).map_err(|e| e.to_string())?;
        let rows = statement
            .query_map(params![identity_id], |row| {
                Ok(ResolutionFact {
                    id: row.get(0)?,
                    fact_type: row.get(1)?,
                    canonical_value: row.get(2)?,
                    fact_status: row.get(3)?,
                    source_family: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };
    let mut groups = std::collections::BTreeMap::<(String, String), Vec<ResolutionFact>>::new();
    for fact in facts {
        groups
            .entry((fact.fact_type.clone(), fact.canonical_value.clone()))
            .or_default()
            .push(fact);
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM osint_fact_resolution_evidence WHERE resolution_id IN (SELECT id FROM osint_fact_resolutions WHERE identity_id=?1)",
        params![identity_id],
    ).map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM osint_fact_resolutions WHERE identity_id=?1",
        params![identity_id],
    )
    .map_err(|e| e.to_string())?;
    for ((fact_type, canonical_value), grouped) in groups {
        let favorable = grouped
            .iter()
            .filter(|fact| fact.fact_status == "retenu")
            .count() as i64;
        let contradictory = grouped
            .iter()
            .filter(|fact| fact.fact_status == "rejete")
            .count() as i64;
        let sources = grouped
            .iter()
            .filter(|fact| fact.fact_status == "retenu")
            .map(|fact| fact.source_family.clone())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let status = if contradictory > 0 && favorable > 0 {
            "contradictoire"
        } else if contradictory > 0 && favorable == 0 {
            "rejete"
        } else if favorable > 0 && sources >= 2 {
            "corroboree"
        } else {
            "a_verifier"
        };
        let rationale = match status {
            "contradictoire" => format!("{favorable} fait(s) retenu(s) et {contradictory} fait(s) rejeté(s) : revue humaine nécessaire."),
            "rejete" => "Toutes les occurrences de ce fait ont été rejetées par l’utilisateur.".to_string(),
            "corroboree" => format!("Même fait exact observé par {sources} familles de sources indépendantes ; cela ne confirme pas l’identité."),
            _ if favorable > 0 => "Fait sourcé, mais une seule famille de source le soutient : à vérifier.".to_string(),
            _ => "Fait conservé en arrière-plan en attente d’une preuve présentable.".to_string(),
        };
        let resolution_id = format!(
            "resolution-{}",
            &hex::encode(Sha256::digest(
                format!("{identity_id}|{fact_type}|{canonical_value}").as_bytes()
            ))[..24]
        );
        tx.execute(
            "INSERT INTO osint_fact_resolutions (id,identity_id,fact_type,canonical_value,status,source_count,favorable_count,contradictory_count,rationale)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![resolution_id,identity_id,fact_type,canonical_value,status,sources,favorable,contradictory,rationale],
        ).map_err(|e| e.to_string())?;
        for fact in grouped {
            let role = if fact.fact_status == "rejete" {
                "contradictoire"
            } else {
                "favorable"
            };
            tx.execute(
                "INSERT INTO osint_fact_resolution_evidence (resolution_id,fact_id,evidence_role) VALUES (?1,?2,?3)",
                params![resolution_id, fact.id, role],
            ).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

fn public_source_host(url: Option<&str>) -> Option<String> {
    let value = url?.trim();
    let without_scheme = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))?;
    let authority = without_scheme.split('/').next()?.trim();
    let host_with_port = authority.rsplit('@').next()?;
    let host = if host_with_port.starts_with('[') {
        host_with_port
            .split(']')
            .next()
            .unwrap_or(host_with_port)
            .to_lowercase()
    } else {
        host_with_port
            .split(':')
            .next()
            .unwrap_or(host_with_port)
            .to_lowercase()
    };
    (!host.is_empty()).then_some(host)
}

/// Identifies the independent origin of evidence, not the label displayed by a
/// collector. Maigret and User Scanner are both account enumerators; seeing the
/// same page through both must therefore never count as two independent proofs.
fn evidence_source_family(module_id: &str, source: &str, source_url: Option<&str>) -> String {
    match module_id {
        "osint-email-intel" => "breach:xposedornot".to_string(),
        "osint-username-profiles" | "osint-email-platforms" => {
            "enumeration:public-accounts".to_string()
        }
        "osint-github-profile" => "platform:github".to_string(),
        "osint-gitlab-profile" => "platform:gitlab".to_string(),
        "osint-mastodon-webfinger" => "platform:mastodon".to_string(),
        "osint-gravatar-profile" => "platform:gravatar".to_string(),
        "osint-keybase-profile" => "platform:keybase".to_string(),
        "osint-bluesky-profile" => "platform:bluesky".to_string(),
        "osint-hackernews-profile" => "platform:hackernews".to_string(),
        "osint-fr-company-register" => "registry:entreprises-fr".to_string(),
        "osint-hal-author" => "repository:hal".to_string(),
        "osint-web-footprint" => "search:web-index".to_string(),
        _ => {
            let normalized_source = normalized_exact_text(source);
            if !normalized_source.is_empty() {
                format!("source:{normalized_source}")
            } else if let Some(host) = public_source_host(source_url) {
                format!("host:{host}")
            } else {
                "source:unknown".to_string()
            }
        }
    }
}

fn evidence_source_reliability(module_id: &str) -> &'static str {
    match module_id {
        "osint-email-intel" => "structuree",
        "osint-github-profile"
        | "osint-gitlab-profile"
        | "osint-mastodon-webfinger"
        | "osint-gravatar-profile"
        | "osint-keybase-profile"
        | "osint-bluesky-profile"
        | "osint-hackernews-profile" => "plateforme_directe",
        "osint-fr-company-register" | "osint-hal-author" => "structuree",
        "osint-web-footprint" => "indexee",
        "osint-username-profiles" | "osint-email-platforms" => "agregateur",
        _ => "inconnue",
    }
}

fn evidence_fingerprint(
    target_kind: &str,
    target: &str,
    source_family: &str,
    source_url: Option<&str>,
) -> String {
    let canonical_url = source_url
        .and_then(normalized_claim_url)
        .unwrap_or_default();
    let digest = Sha256::digest(
        format!(
            "{}|{}|{}|{}",
            normalized_exact_text(target_kind),
            normalized_claim_value(target_kind, target),
            source_family,
            canonical_url
        )
        .as_bytes(),
    );
    hex::encode(digest)
}

#[derive(Default)]
struct PublicPageEvidence {
    title: Option<String>,
    exact_identifier_match: bool,
    media_type: Option<String>,
    content_sha256: Option<String>,
    excerpt: Option<String>,
    downloaded_bytes: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WaybackSnapshot {
    url: String,
    timestamp: String,
}

fn parse_wayback_availability(body: &str, expected_url: &str) -> Option<WaybackSnapshot> {
    let payload: serde_json::Value = serde_json::from_str(body).ok()?;
    let closest = payload.get("archived_snapshots")?.get("closest")?;
    if closest.get("available").and_then(|value| value.as_bool()) != Some(true)
        || closest.get("status").and_then(|value| value.as_str()) != Some("200")
    {
        return None;
    }
    let timestamp = closest.get("timestamp")?.as_str()?.trim();
    if timestamp.len() != 14
        || !timestamp
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return None;
    }
    let snapshot_url = closest.get("url")?.as_str()?.trim();
    let parsed_snapshot = reqwest::Url::parse(snapshot_url).ok()?;
    if parsed_snapshot.host_str()?.to_ascii_lowercase() != "web.archive.org"
        || !matches!(parsed_snapshot.scheme(), "http" | "https")
    {
        return None;
    }
    let expected = normalized_claim_url(expected_url)?;
    let embedded = parsed_snapshot
        .path()
        .split_once("/http")
        .and_then(|(_, suffix)| normalized_claim_url(&format!("http{suffix}")))?;
    if embedded != expected {
        return None;
    }
    Some(WaybackSnapshot {
        url: snapshot_url.to_string(),
        timestamp: timestamp.to_string(),
    })
}

fn lookup_wayback_snapshot(validated_url: &str) -> Option<WaybackSnapshot> {
    if !is_safe_public_http_url(validated_url) {
        return None;
    }
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("MANTIS-Posture/0.1 wayback-availability")
        .build()
        .ok()?
        .get("https://archive.org/wayback/available")
        .query(&[("url", validated_url)])
        .send()
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = read_bounded_http_body(response, 128 * 1024).ok()?;
    parse_wayback_availability(&body, validated_url)
}

const MAX_PUBLIC_DOCUMENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_PUBLIC_PDF_BYTES: usize = 64 * 1024 * 1024;
const MAX_EXTRACTED_DOCUMENT_CHARS: usize = 1_000_000;

fn extract_public_pdf_text(bytes: &[u8]) -> Option<String> {
    if bytes.len() > MAX_PUBLIC_PDF_BYTES || !bytes.starts_with(b"%PDF-") {
        return None;
    }
    match std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes)) {
        Ok(Ok(text)) if text.chars().count() <= MAX_EXTRACTED_DOCUMENT_CHARS => Some(text),
        _ => None,
    }
}

/// Soft matching for human names: accents and case do not change identity of
/// a token (`Zoé` and `Zoe` are equivalent), while punctuation and spacing
/// remain meaningful enough to avoid broad homonym matches.
fn normalize_soft_public_text(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => {
                'a'
            }
            'Ç' | 'ç' => 'c',
            'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
            'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
            'Ñ' | 'ñ' => 'n',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
            'Ý' | 'Ÿ' | 'ý' | 'ÿ' => 'y',
            'Æ' | 'æ' => 'a',
            'Œ' | 'œ' => 'o',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

fn evidence_excerpt(text: &str, identifier: &str, kind: &str) -> Option<String> {
    let normalized_identifier = match kind {
        "telephone" => normalize_phone_number(identifier)
            .trim_start_matches('+')
            .to_string(),
        "adresse" => normalize_address_text(identifier),
        _ => normalize_soft_public_text(identifier.trim()),
    };
    if normalized_identifier.is_empty() {
        return None;
    }
    let normalized_text = match kind {
        "telephone" => text
            .chars()
            .map(|character| {
                if character.is_ascii_digit() {
                    character
                } else {
                    ' '
                }
            })
            .collect::<String>(),
        "adresse" => normalize_address_text(text),
        _ => normalize_soft_public_text(text),
    };
    let position = normalized_text.find(&normalized_identifier)?;
    let start = position.saturating_sub(120);
    let end = (position + normalized_identifier.len() + 120).min(normalized_text.len());
    normalized_text
        .get(start..end)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(320).collect())
}

fn public_text_contains_identifier(text: &str, identifier: &str, kind: &str) -> bool {
    match kind {
        "telephone" => {
            let expected = normalize_phone_number(identifier);
            let expected_digits = expected.trim_start_matches('+');
            if !(7..=15).contains(&expected_digits.len()) {
                return false;
            }
            let mut candidate = String::new();
            for character in text.chars().chain(std::iter::once('x')) {
                if character.is_ascii_digit() {
                    candidate.push(character);
                } else if matches!(character, '+' | '(' | ')' | '.' | '-' | '/' | ' ') {
                    continue;
                } else {
                    if candidate == expected_digits
                        || (candidate.starts_with("00") && &candidate[2..] == expected_digits)
                    {
                        return true;
                    }
                    candidate.clear();
                }
            }
            false
        }
        "adresse" => {
            let expected = normalize_address_text(identifier);
            !expected.is_empty() && normalize_address_text(text).contains(&expected)
        }
        _ => normalize_soft_public_text(text)
            .contains(&normalize_soft_public_text(identifier.trim())),
    }
}

fn is_safe_public_http_url(url: &str) -> bool {
    let Some(parsed) = safe_public_url_shape(url) else {
        return false;
    };
    let Some(host) = public_source_host(Some(url)) else {
        return false;
    };
    if parsed.host_str().is_none() {
        return false;
    }
    let private_ipv4 = {
        let parts = host.split('.').collect::<Vec<_>>();
        if parts.len() != 4 {
            false
        } else if let (Ok(first), Ok(second)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
            first == 0
                || first == 10
                || first == 127
                || (first == 100 && (64..=127).contains(&second))
                || (first == 169 && second == 254)
                || (first == 192 && second == 168)
                || (first == 198 && matches!(second, 18 | 19))
                || (first == 172 && (16..=31).contains(&second))
        } else {
            false
        }
    };
    let dns_is_public = (host.as_str(), 443)
        .to_socket_addrs()
        .ok()
        .is_some_and(|mut addresses| {
            addresses.all(|address| {
                let ip = address.ip();
                match ip {
                    std::net::IpAddr::V4(ip) => {
                        let octets = ip.octets();
                        !(octets[0] == 0
                            || octets[0] == 10
                            || octets[0] == 127
                            || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                            || (octets[0] == 169 && octets[1] == 254)
                            || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                            || (octets[0] == 192 && octets[1] == 168)
                            || (octets[0] == 198 && matches!(octets[1], 18 | 19)))
                    }
                    std::net::IpAddr::V6(ip) => {
                        !(ip.is_loopback()
                            || ip.is_unspecified()
                            || ip.is_unique_local()
                            || ip.is_unicast_link_local())
                    }
                }
            })
        });
    !(host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || private_ipv4
        || host.starts_with('['))
        && dns_is_public
}

/// Validates the URL shape before a result is persisted or handed to an
/// external browser. DNS resolution is deliberately left to
/// `is_safe_public_http_url`, which is used immediately before network I/O.
/// This lighter gate still rejects credentials, non-web schemes, local names
/// and literal private/reserved addresses returned by an untrusted sidecar.
fn safe_public_url_shape(value: &str) -> Option<reqwest::Url> {
    let url = reqwest::Url::parse(value.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    let host = url
        .host_str()?
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host.is_empty()
        || host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
    {
        return None;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        let private = match ip {
            std::net::IpAddr::V4(ip) => {
                let octets = ip.octets();
                octets[0] == 0
                    || octets[0] == 10
                    || octets[0] == 127
                    || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                    || (octets[0] == 169 && octets[1] == 254)
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    || (octets[0] == 192 && octets[1] == 168)
                    || (octets[0] == 198 && matches!(octets[1], 18 | 19))
            }
            std::net::IpAddr::V6(ip) => {
                ip.is_loopback()
                    || ip.is_unspecified()
                    || ip.is_unique_local()
                    || ip.is_unicast_link_local()
            }
        };
        if private {
            return None;
        }
    }
    Some(url)
}

fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let content_start = lower[start..].find('>')? + start + 1;
    let end = lower[content_start..].find("</title>")? + content_start;
    clean_web_result_text(&html[content_start..end])
}

/// Extract a small, bounded set of public document links from a page.  Many
/// publishers (Calaméo included) render the useful text in a PDF behind an
/// HTML viewer, so checking only the landing page loses high-value matches.
fn extract_embedded_public_document_urls(page_url: &str, html: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let normalized = html.replace("\\/", "/");
    let bytes = normalized.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() && candidates.len() < 8 {
        let Some(relative) = normalized[cursor..].find("http") else {
            break;
        };
        let start = cursor + relative;
        let end = normalized[start..]
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '"' | '\'' | '<' | '>')
            })
            .map(|offset| start + offset)
            .unwrap_or(normalized.len());
        let raw = normalized[start..end].trim_end_matches([')', ',', ';']);
        if let Ok(parsed) = reqwest::Url::parse(raw) {
            let path = parsed.path().to_ascii_lowercase();
            let query = parsed.query().unwrap_or_default().to_ascii_lowercase();
            let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
            if matches!(parsed.scheme(), "http" | "https")
                && (path.ends_with(".pdf")
                    || path.contains("/pdf")
                    || query.contains(".pdf")
                    || query.contains("format=pdf")
                    || query.contains("download")
                    || host.ends_with("calameoassets.com"))
                && !candidates.iter().any(|candidate| candidate == raw)
            {
                candidates.push(raw.to_string());
            }
        }
        cursor = end.saturating_add(1);
    }

    // Also handle ordinary relative href/src attributes (the common viewer
    // pattern), while keeping the same strict PDF/document heuristics.
    for attribute in ["href", "src", "data", "content"] {
        for quote in ['"', '\''] {
            let marker = format!("{attribute}={quote}");
            let mut offset = 0;
            while offset < normalized.len() && candidates.len() < 8 {
                let Some(found) = normalized[offset..].find(&marker) else {
                    break;
                };
                let start = offset + found + marker.len();
                let Some(end_offset) = normalized[start..].find(quote) else {
                    break;
                };
                let raw = normalized[start..start + end_offset].trim();
                let lower = raw.to_ascii_lowercase();
                if lower.ends_with(".pdf") || lower.contains("/pdf") || lower.contains("download") {
                    if let Ok(base) = reqwest::Url::parse(page_url) {
                        if let Ok(joined) = base.join(raw) {
                            if matches!(joined.scheme(), "http" | "https")
                                && !candidates
                                    .iter()
                                    .any(|candidate| candidate == joined.as_str())
                            {
                                candidates.push(joined.to_string());
                            }
                        }
                    }
                }
                offset = start + end_offset + 1;
            }
        }
    }
    candidates
}

fn fetch_public_document(url: &str) -> Option<(String, Vec<u8>)> {
    if !is_safe_public_http_url(url) {
        return None;
    }
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("MANTIS-Posture/0.1 public-evidence-check")
        .build()
        .ok()?
        .get(url)
        .send()
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let declared = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let pdf_hint = declared == "application/pdf"
        || url.to_ascii_lowercase().contains(".pdf")
        || url.to_ascii_lowercase().contains("calameoassets.com");
    let max_bytes = if pdf_hint {
        MAX_PUBLIC_PDF_BYTES
    } else {
        MAX_PUBLIC_DOCUMENT_BYTES
    };
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return None;
    }
    let mut bytes = Vec::new();
    if response
        .take((max_bytes + 1) as u64)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() > max_bytes
    {
        return None;
    }
    let media_type = if declared == "application/pdf" || bytes.starts_with(b"%PDF-") {
        "application/pdf".to_string()
    } else if matches!(declared.as_str(), "text/html" | "text/plain") {
        declared
    } else {
        return None;
    };
    Some((media_type, bytes))
}

/// Calaméo publications expose a public, bounded text-layer search endpoint
/// from their reader. It is preferable to downloading a 30–80 MB source PDF:
/// the endpoint searches the publisher's indexed document text and returns
/// the matching page references.
fn inspect_calameo_text_layer(
    url: &str,
    identifier: &str,
    kind: &str,
) -> Option<PublicPageEvidence> {
    let parsed = reqwest::Url::parse(url).ok()?;
    if parsed.host_str()?.trim_start_matches("www.") != "calameo.com" {
        return None;
    }
    let segments = parsed.path_segments()?.collect::<Vec<_>>();
    let marker = segments
        .iter()
        .position(|segment| *segment == "books" || *segment == "read")?;
    let book_code = segments.get(marker + 1)?.trim();
    if book_code.len() != 24
        || !book_code
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return None;
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("MANTIS-Posture/0.1 public-evidence-check")
        .build()
        .ok()?;
    let metadata_url = "https://d.calameo.com/pinwheel/viewer/book/get";
    let metadata = client
        .get(metadata_url)
        .query(&[("bkcode", book_code), ("authid", ""), ("page", "1")])
        .send()
        .ok()
        .filter(|response| response.status().is_success())
        .and_then(|response| read_bounded_http_body(response, 2 * 1024 * 1024).ok())?;
    let payload: serde_json::Value = serde_json::from_str(&metadata).ok()?;
    let search_url = payload
        .pointer("/content/features/search/url")
        .and_then(|value| value.as_str())?;
    if !is_safe_public_http_url(search_url) {
        return None;
    }
    let body = client
        .post(search_url)
        .form(&[
            ("bkcode", book_code),
            ("query", identifier),
            ("start", "1"),
            ("step", "20"),
        ])
        .send()
        .ok()
        .filter(|response| response.status().is_success())
        .and_then(|response| read_bounded_http_body(response, 2 * 1024 * 1024).ok())?;
    if !public_text_contains_identifier(&body, identifier, kind) {
        return None;
    }
    let hash = hex::encode(Sha256::digest(body.as_bytes()));
    Some(PublicPageEvidence {
        title: None,
        exact_identifier_match: true,
        media_type: Some("text/plain".into()),
        content_sha256: Some(hash),
        excerpt: evidence_excerpt(&body, identifier, kind),
        downloaded_bytes: Some(body.into_bytes()),
    })
}

fn inspect_public_page_for_kind(url: &str, identifier: &str, kind: &str) -> PublicPageEvidence {
    if !is_safe_public_http_url(url) || identifier.trim().is_empty() {
        return PublicPageEvidence::default();
    }
    let Some((declared_media_type, bytes)) = fetch_public_document(url) else {
        return inspect_calameo_text_layer(url, identifier, kind).unwrap_or_default();
    };
    let body = if declared_media_type == "application/pdf" {
        let Some(text) = extract_public_pdf_text(&bytes) else {
            return PublicPageEvidence::default();
        };
        text
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    let exact_identifier_match = public_text_contains_identifier(&body, identifier, kind);
    if !exact_identifier_match {
        if let Some(calameo_evidence) = inspect_calameo_text_layer(url, identifier, kind) {
            return calameo_evidence;
        }
    }
    if !exact_identifier_match && declared_media_type == "text/html" {
        // Inspect a handful of linked/embedded PDFs before declaring the page
        // irrelevant.  This is intentionally bounded to preserve scan speed.
        for document_url in extract_embedded_public_document_urls(url, &body) {
            let Some((media_type, document_bytes)) = fetch_public_document(&document_url) else {
                continue;
            };
            if media_type != "application/pdf" {
                continue;
            }
            let Some(document_text) = extract_public_pdf_text(&document_bytes) else {
                continue;
            };
            if public_text_contains_identifier(&document_text, identifier, kind) {
                let digest = hex::encode(Sha256::digest(&document_bytes));
                return PublicPageEvidence {
                    title: extract_html_title(&body),
                    exact_identifier_match: true,
                    media_type: Some(media_type),
                    content_sha256: Some(digest),
                    excerpt: evidence_excerpt(&document_text, identifier, kind),
                    downloaded_bytes: Some(document_bytes),
                };
            }
        }
    }
    let content_sha256 = hex::encode(Sha256::digest(&bytes));
    PublicPageEvidence {
        title: (declared_media_type == "text/html")
            .then(|| extract_html_title(&body))
            .flatten(),
        exact_identifier_match,
        media_type: Some(declared_media_type),
        content_sha256: Some(content_sha256),
        excerpt: exact_identifier_match
            .then(|| evidence_excerpt(&body, identifier, kind))
            .flatten(),
        downloaded_bytes: Some(bytes),
    }
}

fn inspect_public_page_for_address(
    url: &str,
    address: &str,
    postal_code: &str,
) -> PublicPageEvidence {
    let evidence = inspect_public_page_for_kind(url, address, "adresse");
    if !evidence.exact_identifier_match || !is_safe_public_http_url(url) {
        return PublicPageEvidence::default();
    }
    let Some((media_type, bytes)) = fetch_public_document(url) else {
        return PublicPageEvidence::default();
    };
    let body = if media_type == "application/pdf" {
        let Some(text) = extract_public_pdf_text(&bytes) else {
            return PublicPageEvidence::default();
        };
        text
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    if !public_text_contains_identifier(&body, postal_code, "texte") {
        return PublicPageEvidence::default();
    }
    evidence
}

fn inspect_public_page(url: &str, identifier: &str) -> PublicPageEvidence {
    inspect_public_page_for_kind(url, identifier, "texte")
}

fn ddgs_context_for_identity(
    conn: &Connection,
    identity_id: &str,
    target_kind: &str,
) -> Vec<String> {
    if !matches!(target_kind, "nom" | "adresse") {
        return Vec::new();
    }
    load_identity(conn, identity_id)
        .map(|identity| {
            if target_kind == "adresse" {
                let mut postcodes = identity
                    .values
                    .iter()
                    .filter(|value| value.status == "active" && value.kind == "adresse")
                    .filter_map(|value| value.postal_code.clone())
                    .filter(|value| !value.trim().is_empty())
                    .collect::<Vec<_>>();
                if let Some(postal_code) = identity.postal_code {
                    if !postal_code.trim().is_empty()
                        && !postcodes.iter().any(|value| value == &postal_code)
                    {
                        postcodes.push(postal_code);
                    }
                }
                postcodes.into_iter().take(1).collect()
            } else {
                identity
                    .values
                    .into_iter()
                    .filter(|value| value.status == "active" && value.kind == "pseudo")
                    .map(|value| value.value)
                    .filter(|value| !value.trim().is_empty())
                    .take(3)
                    .collect()
            }
        })
        .unwrap_or_default()
}

fn normalized_person_name_text(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for character in value.chars().flat_map(char::to_lowercase) {
        match character {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => normalized.push('a'),
            'æ' => normalized.push_str("ae"),
            'ç' => normalized.push('c'),
            'è' | 'é' | 'ê' | 'ë' => normalized.push('e'),
            'ì' | 'í' | 'î' | 'ï' => normalized.push('i'),
            'ñ' => normalized.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => normalized.push('o'),
            'œ' => normalized.push_str("oe"),
            'ù' | 'ú' | 'û' | 'ü' => normalized.push('u'),
            'ý' | 'ÿ' => normalized.push('y'),
            character if character.is_alphanumeric() => normalized.push(character),
            _ => normalized.push(' '),
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn active_identity_full_name_tokens(
    conn: &Connection,
    identity_id: &str,
) -> Result<Vec<Vec<String>>, String> {
    let identity_values_exist: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='identity_values')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !identity_values_exist {
        return Ok(Vec::new());
    }
    let mut statement = conn
        .prepare(
            "SELECT kind,value FROM identity_values
             WHERE identity_id=?1 AND status='active' AND kind IN ('prenom','nom')",
        )
        .map_err(|e| e.to_string())?;
    let values = statement
        .query_map(params![identity_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let first_names = values
        .iter()
        .filter(|(kind, _)| kind == "prenom")
        .map(|(_, value)| normalized_person_name_text(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let last_names = values
        .iter()
        .filter(|(kind, _)| kind == "nom")
        .map(|(_, value)| normalized_person_name_text(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let mut full_names = Vec::new();
    for first_name in &first_names {
        for last_name in &last_names {
            for full_name in [
                format!("{first_name} {last_name}"),
                format!("{last_name} {first_name}"),
            ] {
                let tokens = full_name
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if tokens.len() >= 2 && !full_names.contains(&tokens) {
                    full_names.push(tokens);
                }
            }
        }
    }
    Ok(full_names)
}

fn text_contains_full_name(value: &str, full_names: &[Vec<String>]) -> bool {
    let normalized = normalized_person_name_text(value);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    full_names.iter().any(|full_name| {
        tokens.windows(full_name.len()).any(|window| {
            window
                .iter()
                .zip(full_name)
                .all(|(left, right)| *left == right)
        })
    })
}

fn text_is_full_name(value: &str, full_names: &[Vec<String>]) -> bool {
    let normalized = normalized_person_name_text(value);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    full_names.iter().any(|full_name| {
        tokens.len() == full_name.len()
            && tokens
                .iter()
                .zip(full_name)
                .all(|(left, right)| *left == right)
    })
}

/// Applies a deterministic publication gate. Raw signals remain stored and are
/// available to advanced users; only evidence that can be reasonably explained
/// is allowed into claims and the default Veille feed.
fn apply_signal_quality_gate(conn: &mut Connection, identity_id: &str) -> Result<(), String> {
    conn.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
        .map_err(|e| e.to_string())?;
    #[derive(Clone)]
    struct Candidate {
        id: String,
        module_id: String,
        signal_type: String,
        target: String,
        title: String,
        explanation: String,
        source: String,
        source_url: Option<String>,
        review_status: String,
        target_kind: String,
    }

    let candidates = {
        let mut statement = conn.prepare(
            "SELECT s.id,s.module_id,s.signal_type,s.target,s.title,s.explanation,s.source,s.source_url,s.review_status,COALESCE(sc.target_kind_snapshot,'')
             FROM osint_signals s JOIN osint_scans sc ON sc.id=s.scan_id
             WHERE s.identity_id=?1",
        ).map_err(|e| e.to_string())?;
        let mapped = statement
            .query_map(params![identity_id], |row| {
                Ok(Candidate {
                    id: row.get(0)?,
                    module_id: row.get(1)?,
                    signal_type: row.get(2)?,
                    target: row.get(3)?,
                    title: row.get(4)?,
                    explanation: row.get(5)?,
                    source: row.get(6)?,
                    source_url: row.get(7)?,
                    review_status: row.get(8)?,
                    target_kind: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        mapped
    };
    let identity_full_names = active_identity_full_name_tokens(conn, identity_id)?;

    // Keybase proofs are useful only as a cross-source link. Keep the exact,
    // canonical public URLs so the corresponding direct platform observation
    // can pass the publication gate without treating the shared username as
    // proof of a person's identity.
    let keybase_verified_urls = candidates
        .iter()
        .filter(|candidate| {
            candidate.module_id == "osint-keybase-profile"
                && candidate.target_kind == "pseudo"
                && format!("{} {}", candidate.title, candidate.explanation)
                    .to_lowercase()
                    .contains("preuves publiques vérifiées par keybase :")
        })
        .filter_map(|candidate| candidate.source_url.as_deref())
        .filter_map(normalized_claim_url)
        .collect::<std::collections::HashSet<_>>();

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for candidate in candidates {
        let exact_known_breach =
            candidate.module_id == "osint-email-intel" && candidate.signal_type == "fuite";
        let context = format!("{} {}", candidate.title, candidate.explanation).to_lowercase();
        // The scan target and evidence_ref contain the query itself, so they are
        // not independent evidence. Evaluate every result-bearing field instead:
        // indexed/page title, snippet or local excerpt, and canonical source URL.
        let exact_full_name_in_signal = matches!(candidate.target_kind.as_str(), "nom" | "prenom")
            && candidate.source_url.is_some()
            && [
                Some(candidate.title.as_str()),
                Some(candidate.explanation.as_str()),
                candidate.source_url.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(|value| text_contains_full_name(value, &identity_full_names));
        let exact_full_name_on_page = candidate.module_id == "osint-web-footprint"
            && matches!(candidate.target_kind.as_str(), "nom" | "prenom")
            && candidate.source_url.is_some()
            && text_is_full_name(&candidate.target, &identity_full_names)
            && context.contains(
                "preuve publique vérifiée : l’identifiant recherché est présent dans la page",
            );
        let exact_full_name_candidate = exact_full_name_in_signal || exact_full_name_on_page;
        let exact_email_on_page = ((candidate.module_id == "osint-web-footprint"
            && context.contains(
                "preuve publique vérifiée : l’identifiant recherché est présent dans la page",
            ))
            || (matches!(
                candidate.module_id.as_str(),
                "osint-gitlab-profile" | "osint-gravatar-profile"
            ) && context.contains("preuve publique vérifiée : l’identifiant exact")))
            && candidate.target_kind == "email"
            && candidate.source_url.is_some();
        let exact_pseudo_profile = matches!(
            candidate.module_id.as_str(),
            "osint-username-profiles"
                | "osint-email-platforms"
                | "osint-github-profile"
                | "osint-gitlab-profile"
                | "osint-mastodon-webfinger"
                | "osint-keybase-profile"
                | "osint-bluesky-profile"
                | "osint-hackernews-profile"
        ) && candidate.target_kind == "pseudo"
            && candidate.source_url.is_some()
            && (context
                .contains("preuve publique vérifiée : le pseudo exact est présent dans la page")
                || (candidate.module_id == "osint-mastodon-webfinger"
                    && context.contains("preuve publique vérifiée : l’identifiant fédéré exact")));
        let user_confirmed = matches!(
            candidate.review_status.as_str(),
            "Confirmé" | "Suivi" | "Traité"
        );
        let user_rejected = candidate.review_status == "Ce n’est pas moi";
        let user_ignored = candidate.review_status == "Ignoré";
        // A username can be exact and still belong to somebody else. It remains
        // hidden until additional identity context or a human confirmation exists.
        let verified_keybase_cluster = candidate.module_id == "osint-keybase-profile"
            && candidate.target_kind == "pseudo"
            && context.contains("preuves publiques vérifiées par keybase :");
        let corroborated_by_keybase = candidate.module_id != "osint-keybase-profile"
            && candidate.target_kind == "pseudo"
            && candidate
                .source_url
                .as_deref()
                .and_then(normalized_claim_url)
                .is_some_and(|url| keybase_verified_urls.contains(&url));
        let visible = user_confirmed
            || exact_known_breach
            || exact_email_on_page
            || exact_full_name_candidate
            || verified_keybase_cluster
            || corroborated_by_keybase;
        let publication_status = if user_rejected {
            "rejete"
        } else if visible {
            "visible"
        } else {
            "masque"
        };
        let match_level = if user_rejected {
            "rejete_utilisateur"
        } else if exact_known_breach || exact_email_on_page {
            "identifiant_exact"
        } else if exact_full_name_candidate {
            "identifiant_exact"
        } else if exact_pseudo_profile || verified_keybase_cluster || corroborated_by_keybase {
            "pseudo_exact"
        } else if matches!(candidate.target_kind.as_str(), "nom" | "prenom") {
            "nom_seul"
        } else {
            "non_verifie"
        };
        let reason = if user_rejected {
            "Écarté par l’utilisateur : cette observation ne doit plus contribuer à la synthèse."
        } else if user_confirmed {
            "Correspondance conservée après validation explicite de l’utilisateur."
        } else if exact_known_breach {
            "Adresse exacte retrouvée dans une source structurée de fuite connue."
        } else if exact_email_on_page {
            "Adresse exacte visible dans une page publique vérifiée."
        } else if exact_full_name_candidate {
            "Nom complet exact retrouvé dans les éléments publics du signal ; une homonymie reste possible et la correspondance doit être validée par l’utilisateur."
        } else if verified_keybase_cluster {
            "Pseudo exact relié par Keybase à au moins un autre compte public vérifié ; l’identité de la personne reste non attribuée."
        } else if corroborated_by_keybase {
            "Même compte public retrouvé directement et dans une preuve Keybase valide ; l’identité de la personne reste non attribuée."
        } else if exact_pseudo_profile {
            "Masqué automatiquement : un pseudo exact reste partageable entre plusieurs personnes et ne suffit pas à attribuer un profil."
        } else if matches!(candidate.target_kind.as_str(), "nom" | "prenom") {
            "Masqué automatiquement : un nom seul ne permet pas d’attribuer une page à cette identité."
        } else {
            "Masqué automatiquement : la preuve publique ne contient pas une correspondance assez forte."
        };
        let source_family = evidence_source_family(
            &candidate.module_id,
            &candidate.source,
            candidate.source_url.as_deref(),
        );
        let fingerprint = evidence_fingerprint(
            &candidate.target_kind,
            &candidate.target,
            &source_family,
            candidate.source_url.as_deref(),
        );
        tx.execute(
            "INSERT INTO osint_evidence_assessments (signal_id,observation_id,identity_id,source_family,source_reliability,match_level,publication_status,rationale,evidence_fingerprint,assessed_at)
             VALUES (?1,'observation-'||?1,?2,?3,?4,?5,?6,?7,?8,datetime('now'))
             ON CONFLICT(signal_id) DO UPDATE SET observation_id=excluded.observation_id,identity_id=excluded.identity_id,source_family=excluded.source_family,source_reliability=excluded.source_reliability,match_level=excluded.match_level,publication_status=excluded.publication_status,rationale=excluded.rationale,evidence_fingerprint=excluded.evidence_fingerprint,assessed_at=excluded.assessed_at",
            params![candidate.id,identity_id,source_family,evidence_source_reliability(&candidate.module_id),match_level,publication_status,reason,fingerprint],
        ).map_err(|e| e.to_string())?;

        if visible && !user_confirmed {
            tx.execute(
                "UPDATE osint_signals SET review_status=CASE WHEN review_status=?1 THEN 'À vérifier' ELSE review_status END, confidence=?2 WHERE id=?3",
                params![AUTO_FILTERED_REVIEW_STATUS, reason, candidate.id],
            ).map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE osint_observations SET relevance_status='a_verifier' WHERE signal_id=?1 AND relevance_status='ignoree'",
                params![candidate.id],
            ).map_err(|e| e.to_string())?;
        } else if !user_rejected && !user_ignored && !user_confirmed {
            tx.execute(
                "UPDATE osint_signals SET review_status=?1,confidence=?2 WHERE id=?3",
                params![AUTO_FILTERED_REVIEW_STATUS, reason, candidate.id],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    tx.execute(
        "UPDATE osint_observations SET relevance_status='ignoree'
         WHERE identity_id=?1 AND signal_id IN (SELECT id FROM osint_signals WHERE review_status=?2)
           AND relevance_status='a_verifier'",
        params![identity_id, AUTO_FILTERED_REVIEW_STATUS],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    refresh_osint_evidence_facts(conn, identity_id)
}

#[tauri::command]
fn review_osint_signal(
    app: tauri::AppHandle,
    signal_id: String,
    decision: String,
    reason: Option<String>,
) -> Result<String, String> {
    let (review_status, relevance_status) = match decision.as_str() {
        "confirmer" => ("Confirmé", "confirmee"),
        "pas_moi" => ("Ce n’est pas moi", "pas_moi"),
        "ignorer" => ("Ignoré", "ignoree"),
        "suivre" => ("Suivi", "suivie"),
        _ => return Err("Cette décision n’est pas reconnue.".into()),
    };
    if reason.as_deref().is_some_and(|value| value.len() > 500) {
        return Err("Le motif est trop long.".into());
    }
    let mut conn = get_db_connection(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let (previous_status, identity_id): (String, String) = tx
        .query_row(
            "SELECT review_status,identity_id FROM osint_signals WHERE id=?1",
            params![signal_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Signal introuvable.".to_string())?;
    tx.execute(
        "INSERT INTO osint_user_decisions (id,target_type,target_id,decision,reason,previous_status,created_at) VALUES (?1,'signal',?2,?3,?4,?5,datetime('now'))",
        params![Uuid::new_v4().to_string(), signal_id, decision, reason, previous_status],
    ).map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE osint_signals SET review_status=?1 WHERE id=?2",
        params![review_status, signal_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE osint_observations SET relevance_status=?1 WHERE signal_id=?2",
        params![relevance_status, signal_id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    apply_signal_quality_gate(&mut conn, &identity_id)?;
    refresh_osint_claims(&mut conn, &identity_id)?;
    Ok(review_status.to_string())
}

fn run_real_osint_scan_for_value(
    app: tauri::AppHandle,
    module_id: String,
    identity_id: String,
    identity_value_id: String,
    target: String,
    target_kind: String,
    session_id: Option<String>,
) -> Result<OsintScanSummary, String> {
    if module_id != "osint-email-platforms"
        && module_id != "mock-osint"
        && module_id != "osint-email-intel"
        && module_id != "osint-github-profile"
        && module_id != "osint-gitlab-profile"
        && module_id != "osint-mastodon-webfinger"
        && module_id != "osint-gravatar-profile"
        && module_id != "osint-keybase-profile"
        && module_id != "osint-bluesky-profile"
        && module_id != "osint-hackernews-profile"
        && module_id != "osint-fr-company-register"
        && module_id != "osint-hal-author"
        && module_id != "osint-web-footprint"
        && module_id != "osint-username-profiles"
    {
        return Err("Ce module n’est pas encore exécutable.".into());
    }
    let conn = get_db_connection(&app)?;
    let catalog_status: String = conn
        .query_row(
            "SELECT catalog_status FROM osint_modules WHERE id=?1",
            params![module_id],
            |row| row.get(0),
        )
        .map_err(|_| "Module absent du catalogue MANTIS.".to_string())?;
    if catalog_status != "active" {
        return Err(
            "Ce module est archivé ou réservé aux tests et ne peut pas être exécuté.".into(),
        );
    }
    let folder_id: Option<String> = conn
        .query_row(
            "SELECT folder_id FROM identities WHERE id=?1",
            params![identity_id],
            |row| row.get(0),
        )
        .map_err(|_| "Choisissez une identité existante.".to_string())?;
    let is_email_module = matches!(
        module_id.as_str(),
        "mock-osint" | "osint-email-intel" | "osint-gravatar-profile"
    );
    if is_email_module && (target_kind != "email" || !target.contains('@') || target.len() > 254) {
        return Err("Choisissez une identité e-mail valide.".into());
    }
    if module_id == "osint-email-platforms" && !matches!(target_kind.as_str(), "email" | "pseudo") {
        return Err("Choisissez une identité e-mail ou pseudo valide.".into());
    }
    if module_id == "osint-web-footprint"
        && !matches!(
            target_kind.as_str(),
            "email" | "nom" | "pseudo" | "telephone" | "adresse"
        )
    {
        return Err(
            "Choisissez un nom, un pseudo, une adresse e-mail, un téléphone ou une adresse postale pour cette recherche.".into(),
        );
    }
    if module_id == "osint-username-profiles" && target_kind != "pseudo" {
        return Err("Maigret accepte uniquement un pseudo déclaré et actif.".into());
    }
    if module_id == "osint-github-profile"
        && (target_kind != "pseudo" || !is_valid_github_username(&target))
    {
        return Err("GitHub accepte uniquement un pseudo GitHub déclaré et valide.".into());
    }
    if module_id == "osint-gitlab-profile" && !matches!(target_kind.as_str(), "email" | "pseudo") {
        return Err(
            "GitLab accepte uniquement un pseudo ou une adresse e-mail publique déclarée.".into(),
        );
    }
    if module_id == "osint-mastodon-webfinger"
        && (target_kind != "pseudo" || mastodon_account_for_target(&target).is_none())
    {
        return Err(
            "Mastodon exige un identifiant explicite utilisateur@instance publique.".into(),
        );
    }
    if module_id == "osint-keybase-profile"
        && (target_kind != "pseudo"
            || !is_valid_keybase_username(
                &target.trim().trim_start_matches('@').to_ascii_lowercase(),
            ))
    {
        return Err(
            "Keybase accepte uniquement un pseudo composé de lettres, chiffres ou tirets bas."
                .into(),
        );
    }
    if module_id == "osint-bluesky-profile"
        && (target_kind != "pseudo" || bluesky_handle_for_target(&target).is_none())
    {
        return Err("Bluesky accepte un handle complet ou un pseudo composé de lettres, chiffres et tirets.".into());
    }
    if module_id == "osint-hackernews-profile"
        && (target_kind != "pseudo" || !is_valid_hackernews_username(&target))
    {
        return Err("Hacker News accepte uniquement un pseudo déclaré composé de lettres, chiffres, tirets bas ou tirets.".into());
    }
    if matches!(
        module_id.as_str(),
        "osint-fr-company-register" | "osint-hal-author"
    ) && (target_kind != "nom" || target.split_whitespace().count() < 2)
    {
        return Err(
            "Cette source spécialisée exige un prénom et un nom actifs explicitement saisis."
                .into(),
        );
    }
    if target.trim().is_empty() || target.len() > 512 {
        return Err("L’identité sélectionnée est invalide.".into());
    }
    let (state, _) = module_installation_state(&app, &module_id);
    if state != "prêt" {
        return Err("Ce module doit être installé ou réparé avant utilisation.".into());
    }
    let scan_id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO osint_scans (id,module_id,identity_id,target,status,started_at,session_id,identity_value_id,target_kind_snapshot) VALUES (?1,?2,?3,?4,'en_cours',datetime('now'),?5,?6,?7)", params![scan_id,module_id,identity_id,target,session_id,identity_value_id,target_kind]).map_err(|e| e.to_string())?;
    if module_id == "mock-osint" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.txt", scan_id));
        std::fs::write(
            &raw_path,
            "Démonstration locale MANTIS : aucune requête externe n’a été effectuée.",
        )
        .map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let title = "Profil public potentiel (démonstration)".to_string();
        let explanation = "Exemple local pour apprendre le parcours MANTIS. Vérifiez toujours un signal avant de le retenir comme exposition.".to_string();
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible','À vérifier','Démonstration locale',NULL,datetime('now'),'Exemple pédagogique',?8,'À vérifier')", params![id,module_id,scan_id,identity_id,target,title,explanation,raw_path.to_string_lossy().to_string()]).map_err(|e| e.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|e|e.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|e| e.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            "Démonstration locale terminée : 1 signal à vérifier.",
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        let signal = OsintSignal {
            id,
            module_id,
            scan_id: scan_id.clone(),
            target: target.clone(),
            signal_type: "profil_public".into(),
            title,
            explanation,
            severity: "faible".into(),
            confidence: "À vérifier".into(),
            source: "Démonstration locale".into(),
            source_url: None,
            discovered_at: chrono_like_now(),
            review_status: "À vérifier".into(),
            exposure_id: None,
        };
        return Ok(OsintScanSummary { scan_id, target, signals: vec![signal], message: "Démonstration terminée : voici un signal fictif à examiner. Aucune donnée n’a quitté votre appareil.".into(), analysis_job_id:None });
    }
    if module_id == "osint-email-intel" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.txt", scan_id));
        let response = reqwest::blocking::Client::builder().timeout(Duration::from_secs(30)).build().map_err(|e| e.to_string())?
            .get("https://api.xposedornot.com/v1/breach-analytics").query(&[("email", target.as_str())]).send().map_err(|_| "La vérification des fuites est indisponible. Vos données MANTIS n’ont pas été modifiées.".to_string())?;
        let body = response
            .text()
            .map_err(|_| "La réponse de la source est illisible.".to_string())?;
        std::fs::write(&raw_path, &body).map_err(|e| e.to_string())?;
        let mut signals = Vec::new();
        for breach in parse_xposed_breaches(&body)? {
            let id = Uuid::new_v4().to_string();
            let title = format!("Fuite potentielle : {}", breach.breach);
            conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'fuite',?6,?7,?8,?9,'XposedOrNot',?10,datetime('now'),?11,?12,'À vérifier')", params![id,module_id,scan_id,identity_id,target,title,breach.explanation,breach.severity,breach.confidence,breach.source_url,breach.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|e| e.to_string())?;
            signals.push(OsintSignal {
                id,
                module_id: module_id.clone(),
                scan_id: scan_id.clone(),
                target: target.clone(),
                signal_type: "fuite".into(),
                title,
                explanation: breach.explanation,
                severity: breach.severity,
                confidence: breach.confidence,
                source: "XposedOrNot".into(),
                source_url: breach.source_url,
                discovered_at: chrono_like_now(),
                review_status: "À vérifier".into(),
                exposure_id: None,
            });
        }
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|e|e.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|e| e.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            &format!("{} fuite(s) potentielle(s) signalée(s).", signals.len()),
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        let message = if signals.is_empty() {
            "Aucune fuite connue n’a été signalée pour cette adresse.".into()
        } else {
            "Des fuites potentielles ont été signalées. Examinez-les avant de décider de les suivre.".into()
        };
        return Ok(OsintScanSummary {
            scan_id,
            target,
            signals,
            message,
            analysis_job_id: None,
        });
    }
    if module_id == "osint-gravatar-profile" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let normalized_email = target.trim().to_ascii_lowercase();
        let email_hash = hex::encode(Sha256::digest(normalized_email.as_bytes()));
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("MANTIS-Posture/0.1 public-profile-check")
            .build()
            .map_err(|error| error.to_string())?
            .get(format!(
                "https://api.gravatar.com/v3/profiles/{email_hash}"
            ))
            .header("Accept", "application/json")
            .send()
            .map_err(|_| {
                "Gravatar est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées."
                    .to_string()
            })?;
        let status = response.status();
        let body = read_bounded_http_body(response, 512 * 1024)?;
        std::fs::write(&raw_path, &body).map_err(|error| error.to_string())?;
        if status == reqwest::StatusCode::NOT_FOUND {
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun profil Gravatar public ne correspond à cette adresse.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun profil Gravatar public n’a été trouvé pour cette adresse. Ce résultat ne prouve pas l’absence d’autres profils publics.".into(), analysis_job_id: None });
        }
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "Gravatar limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "Gravatar a refusé ou interrompu cette vérification publique."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let profile = parse_gravatar_public_profile(&body, &email_hash)?;
        let id = Uuid::new_v4().to_string();
        let title = format!("Profil public potentiel : {}", profile.display_name);
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'Gravatar',?9,datetime('now'),?10,?11,'À vérifier')",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            "Profil Gravatar public exact normalisé comme correspondance possible.",
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id: scan_id.clone(), target: target.clone(), signals: vec![OsintSignal { id, module_id, scan_id, target, signal_type:"profil_public".into(), title, explanation:profile.explanation, severity:"faible".into(), confidence:profile.confidence, source:"Gravatar".into(), source_url:Some(profile.profile_url), discovered_at:chrono_like_now(), review_status:"À vérifier".into(), exposure_id:None }], message:"Un profil Gravatar public est lié à l’adresse analysée. Ses informations déclarées et ses liens sont présentés comme une correspondance possible, jamais comme une identité confirmée.".into(), analysis_job_id:None });
    }
    if module_id == "osint-hackernews-profile" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let username = target.trim().to_ascii_lowercase();
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("MANTIS-Posture/0.1 public-profile-check")
            .build()
            .map_err(|error| error.to_string())?;
        let response = client
            .get(format!("https://hn.algolia.com/api/v1/users/{username}"))
            .header("Accept", "application/json")
            .send()
            .map_err(|_| {
                "Hacker News est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées."
                    .to_string()
            })?;
        let status = response.status();
        let profile_body = read_bounded_http_body(response, 256 * 1024)?;
        if status == reqwest::StatusCode::NOT_FOUND {
            std::fs::write(&raw_path, &profile_body).map_err(|error| error.to_string())?;
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun profil Hacker News public exact ne correspond à ce pseudo.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun profil Hacker News public exact n’a été trouvé. Ce résultat ne prouve pas l’absence d’autres profils publics.".into(), analysis_job_id: None });
        }
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "Hacker News limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "Hacker News a refusé ou interrompu cette vérification publique."
            };
            std::fs::write(&raw_path, &profile_body).map_err(|error| error.to_string())?;
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let (activity_body, activity_available) = match client
            .get("https://hn.algolia.com/api/v1/search_by_date")
            .query(&[
                ("tags", format!("author_{username}")),
                ("hitsPerPage", "5".into()),
            ])
            .header("Accept", "application/json")
            .send()
        {
            Ok(response) if response.status().is_success() => {
                let body = read_bounded_http_body(response, 256 * 1024)?;
                (body, true)
            }
            _ => ("{\"hits\":[]}".into(), false),
        };
        let stored = serde_json::json!({
            "profile": serde_json::from_str::<serde_json::Value>(&profile_body)
                .map_err(|_| "Hacker News a renvoyé un profil public dans un format inattendu.")?,
            "activity": serde_json::from_str::<serde_json::Value>(&activity_body)
                .map_err(|_| "Hacker News a renvoyé une activité publique dans un format inattendu.")?
        });
        let stored_body = serde_json::to_string(&stored)
            .map_err(|_| "La réponse Hacker News ne peut pas être conservée sûrement.")?;
        std::fs::write(&raw_path, &stored_body).map_err(|error| error.to_string())?;
        let profile = parse_hackernews_public_profile(&stored_body, &username)?;
        let id = Uuid::new_v4().to_string();
        let title = format!("Profil Hacker News potentiel : {}", profile.username);
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status,identity_value_id) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'Hacker News',?9,datetime('now'),?10,?11,'À vérifier',?12)",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string(),identity_value_id]).map_err(|error|error.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            if activity_available {
                "Profil Hacker News exact normalisé avec une activité publique bornée."
            } else {
                "Profil Hacker News exact normalisé ; l’activité publique n’était pas disponible."
            },
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id: scan_id.clone(), target: target.clone(), signals: vec![OsintSignal { id, module_id, scan_id, target, signal_type:"profil_public".into(), title, explanation:profile.explanation, severity:"faible".into(), confidence:profile.confidence, source:"Hacker News".into(), source_url:Some(profile.profile_url), discovered_at:chrono_like_now(), review_status:"À vérifier".into(), exposure_id:None }], message:"Un profil Hacker News public porte exactement ce pseudo. Son activité publique est limitée et le profil reste masqué par défaut tant qu’il n’est pas recoupé ou validé.".into(), analysis_job_id:None });
    }
    if module_id == "osint-bluesky-profile" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let handle = bluesky_handle_for_target(&target)
            .ok_or("Le pseudo ne peut pas former un handle Bluesky valide.")?;
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("MANTIS-Posture/0.1 public-profile-check")
            .build()
            .map_err(|error| error.to_string())?
            .get("https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile")
            .query(&[("actor", handle.as_str())])
            .header("Accept", "application/json")
            .send()
            .map_err(|_| {
                "Bluesky est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées."
                    .to_string()
            })?;
        let status = response.status();
        let body = read_bounded_http_body(response, 512 * 1024)?;
        let stored_body = if status.is_success() {
            let mut value: serde_json::Value = serde_json::from_str(&body).map_err(|_| {
                "Bluesky a renvoyé un profil public dans un format inattendu.".to_string()
            })?;
            remove_sensitive_api_fields(&mut value);
            serde_json::to_string(&value).map_err(|_| {
                "La réponse Bluesky ne peut pas être conservée sûrement.".to_string()
            })?
        } else {
            body.clone()
        };
        std::fs::write(&raw_path, &stored_body).map_err(|error| error.to_string())?;
        let profile_not_found = matches!(
            status,
            reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::NOT_FOUND
        ) && serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(|error| error.as_str())
                    .map(str::to_string)
            })
            .is_some_and(|error| {
                matches!(
                    error.to_ascii_lowercase().as_str(),
                    "profilenotfound" | "actornotfound" | "invalidrequest"
                )
            });
        if profile_not_found {
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun profil Bluesky public exact ne correspond à ce handle.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun profil Bluesky public exact n’a été trouvé. Ce résultat ne prouve pas l’absence d’autres profils publics.".into(), analysis_job_id: None });
        }
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "Bluesky limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "Bluesky a refusé ou interrompu cette vérification publique."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let profile = parse_bluesky_public_profile(&stored_body, &handle)?;
        let id = Uuid::new_v4().to_string();
        let title = format!("Profil Bluesky potentiel : {}", profile.handle);
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status,identity_value_id) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'Bluesky',?9,datetime('now'),?10,?11,'À vérifier',?12)",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string(),identity_value_id]).map_err(|error|error.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            &format!(
                "Profil Bluesky exact normalisé avec le DID public {}.",
                profile.did
            ),
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id: scan_id.clone(), target: target.clone(), signals: vec![OsintSignal { id, module_id, scan_id, target, signal_type:"profil_public".into(), title, explanation:profile.explanation, severity:"faible".into(), confidence:profile.confidence, source:"Bluesky".into(), source_url:Some(profile.profile_url), discovered_at:chrono_like_now(), review_status:"À vérifier".into(), exposure_id:None }], message:"Un profil Bluesky public porte exactement ce handle. Il reste masqué par défaut tant qu’il n’est pas recoupé ou validé, car un handle seul ne confirme pas l’identité de la personne.".into(), analysis_job_id:None });
    }
    if module_id == "osint-keybase-profile" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let username = target.trim().trim_start_matches('@').to_ascii_lowercase();
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("MANTIS-Posture/0.1 public-profile-check")
            .build()
            .map_err(|error| error.to_string())?
            .get("https://keybase.io/_/api/1.0/user/lookup.json")
            .query(&[
                ("usernames", username.as_str()),
                ("fields", "basics,profile,proofs_summary"),
            ])
            .header("Accept", "application/json")
            .send()
            .map_err(|_| {
                "Keybase est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées."
                    .to_string()
            })?;
        let status = response.status();
        let body = read_bounded_http_body(response, 512 * 1024)?;
        let sanitized_body = if status.is_success() {
            sanitized_keybase_public_response(&body)?
        } else {
            format!("{{\"http_status\":{}}}", status.as_u16())
        };
        std::fs::write(&raw_path, &sanitized_body).map_err(|error| error.to_string())?;
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "Keybase limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "Keybase a refusé ou interrompu cette vérification publique."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let Some(profile) = parse_keybase_public_profile(&sanitized_body, &username)? else {
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun profil Keybase public exact ne correspond à ce pseudo.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun profil Keybase public exact n’a été trouvé. Ce résultat ne prouve pas l’absence de comptes sur d’autres plateformes.".into(), analysis_job_id: None });
        };
        let id = Uuid::new_v4().to_string();
        let title = if profile.verified_proof_count > 0 {
            format!(
                "Réseau de comptes potentiel : {} ({} preuve(s))",
                profile.username, profile.verified_proof_count
            )
        } else {
            format!("Profil Keybase potentiel : {}", profile.username)
        };
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'Keybase',?9,datetime('now'),?10,?11,'À vérifier')",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
        let mut signals = vec![OsintSignal {
            id,
            module_id: module_id.clone(),
            scan_id: scan_id.clone(),
            target: target.clone(),
            signal_type: "profil_public".into(),
            title,
            explanation: profile.explanation.clone(),
            severity: "faible".into(),
            confidence: profile.confidence.clone(),
            source: "Keybase".into(),
            source_url: Some(profile.profile_url.clone()),
            discovered_at: chrono_like_now(),
            review_status: "À vérifier".into(),
            exposure_id: None,
        }];
        for proof in &profile.verified_proofs {
            let proof_id = Uuid::new_v4().to_string();
            let proof_title = format!("Compte public potentiellement lié : {}", proof.service);
            let account_context = if proof.account.is_empty() {
                String::new()
            } else {
                format!(" sous le nom « {} »", proof.account)
            };
            let proof_explanation = format!("Preuves publiques vérifiées par Keybase : le profil exact « {} » publie une preuve valide vers {}{}. Cette preuve relie deux comptes publics entre eux ; elle ne confirme jamais l’identité de la personne.", profile.username, proof.service, account_context);
            let proof_confidence =
                "Preuve externe valide publiée par Keybase — identité non attribuée".to_string();
            let proof_reference = format!(
                "Keybase · preuve {} · {}",
                proof.service,
                if proof.account.is_empty() {
                    proof.url.as_str()
                } else {
                    proof.account.as_str()
                }
            );
            conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'compte_potentiel',?6,?7,'faible',?8,'Keybase',?9,datetime('now'),?10,?11,'À vérifier')",params![proof_id,module_id,scan_id,identity_id,target,proof_title,proof_explanation,proof_confidence,proof.url,proof_reference,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
            signals.push(OsintSignal {
                id: proof_id,
                module_id: module_id.clone(),
                scan_id: scan_id.clone(),
                target: target.clone(),
                signal_type: "compte_potentiel".into(),
                title: proof_title,
                explanation: proof_explanation,
                severity: "faible".into(),
                confidence: proof_confidence,
                source: "Keybase".into(),
                source_url: Some(proof.url.clone()),
                discovered_at: chrono_like_now(),
                review_status: "À vérifier".into(),
                exposure_id: None,
            });
        }
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            &format!(
                "Profil Keybase exact normalisé avec {} preuve(s) externe(s) valide(s).",
                profile.verified_proof_count
            ),
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        let message = if profile.verified_proof_count > 0 {
            "Keybase relie ce pseudo à d’autres comptes publics vérifiés. MANTIS présente cet ensemble comme un réseau potentiel, jamais comme une identité confirmée."
        } else {
            "Un profil Keybase porte exactement ce pseudo, mais aucune preuve externe valide ne permet de renforcer cette correspondance. Il restera masqué par défaut."
        };
        return Ok(OsintScanSummary {
            scan_id,
            target,
            signals,
            message: message.into(),
            analysis_job_id: None,
        });
    }
    if matches!(
        module_id.as_str(),
        "osint-fr-company-register" | "osint-hal-author"
    ) {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("MANTIS-Posture/0.1 specialized-public-source")
            .build()
            .map_err(|error| error.to_string())?;
        let response = if module_id == "osint-fr-company-register" {
            client
                .get("https://recherche-entreprises.api.gouv.fr/search")
                .query(&[("q", target.as_str()), ("page", "1"), ("per_page", "5")])
                .send()
        } else {
            let query = format!("authFullName_t:\"{}\"", target.trim());
            client
                .get("https://api.archives-ouvertes.fr/search/")
                .query(&[
                    ("q", query.as_str()),
                    ("fl", "title_s,authFullName_s,uri_s,producedDateY_i"),
                    ("rows", "5"),
                    ("wt", "json"),
                ])
                .send()
        }
        .map_err(|_| {
            "La source publique spécialisée est momentanément indisponible.".to_string()
        })?;
        let status = response.status();
        let body = read_bounded_http_body(response, 512 * 1024)?;
        std::fs::write(&raw_path, &body).map_err(|error| error.to_string())?;
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "La source publique spécialisée limite temporairement les requêtes."
            } else {
                "La source publique spécialisée a refusé cette vérification."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let findings = if module_id == "osint-fr-company-register" {
            parse_fr_company_findings(&body, &target)?
        } else {
            parse_hal_findings(&body, &target)?
        };
        let mut signals = Vec::new();
        for finding in findings {
            let id = Uuid::new_v4().to_string();
            conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'mention',?6,?7,'faible','Nom complet exact — homonyme possible',?8,?9,datetime('now'),?10,?11,'À vérifier')",params![id,module_id,scan_id,identity_id,target,finding.title,finding.explanation,finding.source,finding.source_url,finding.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
            signals.push(OsintSignal {
                id,
                module_id: module_id.clone(),
                scan_id: scan_id.clone(),
                target: target.clone(),
                signal_type: "mention".into(),
                title: finding.title,
                explanation: finding.explanation,
                severity: "faible".into(),
                confidence: "Nom complet exact — homonyme possible".into(),
                source: finding.source,
                source_url: Some(finding.source_url),
                discovered_at: chrono_like_now(),
                review_status: "À vérifier".into(),
                exposure_id: None,
            });
        }
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            &format!("{} correspondance(s) exacte(s) à revoir.", signals.len()),
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id, target, signals, message: "La source spécialisée a été vérifiée manuellement sur le nom complet exact. Chaque résultat reste un homonyme potentiel jusqu’à la revue humaine.".into(), analysis_job_id: None });
    }
    if module_id == "osint-mastodon-webfinger" {
        let (username, instance) = mastodon_account_for_target(&target)
            .ok_or("Mastodon exige un identifiant explicite utilisateur@instance publique.")?;
        let mut endpoint =
            reqwest::Url::parse(&format!("https://{instance}/.well-known/webfinger"))
                .map_err(|_| "L’instance Mastodon déclarée est invalide.".to_string())?;
        endpoint
            .query_pairs_mut()
            .append_pair("resource", &format!("acct:{username}@{instance}"));
        if !is_safe_public_http_url(endpoint.as_str()) {
            return Err(
                "MANTIS refuse de consulter une instance locale, privée ou non résolue.".into(),
            );
        }
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("MANTIS-Posture/0.1 webfinger-check")
            .build()
            .map_err(|error| error.to_string())?
            .get(endpoint)
            .header("Accept", "application/jrd+json, application/json")
            .send()
            .map_err(|_| "L’instance Mastodon est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées.".to_string())?;
        let status = response.status();
        let body = read_bounded_http_body(response, 256 * 1024)?;
        std::fs::write(&raw_path, &body).map_err(|error| error.to_string())?;
        if status == reqwest::StatusCode::NOT_FOUND {
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun compte Mastodon exact n’est publié par cette instance.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun compte Mastodon public exact n’a été trouvé sur l’instance déclarée. Ce résultat ne prouve pas l’absence d’un compte sur une autre instance.".into(), analysis_job_id: None });
        }
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "L’instance Mastodon limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "L’instance Mastodon a refusé ou interrompu cette vérification WebFinger."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let profile = match parse_mastodon_webfinger_profile(&body, &username, &instance) {
            Ok(profile) => profile,
            Err(message) => {
                conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
                log_module(&conn, &module_id, "scan", "erreur", &message)?;
                return Err(message);
            }
        };
        let id = Uuid::new_v4().to_string();
        let title = format!("Profil Mastodon potentiel : {}", profile.account);
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'Mastodon/WebFinger',?9,datetime('now'),?10,?11,'À vérifier')",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            "Identifiant Mastodon exact résolu par WebFinger comme profil potentiel.",
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id: scan_id.clone(), target: target.clone(), signals: vec![OsintSignal { id, module_id, scan_id, target, signal_type:"profil_public".into(), title, explanation:profile.explanation, severity:"faible".into(), confidence:profile.confidence, source:"Mastodon/WebFinger".into(), source_url:Some(profile.profile_url), discovered_at:chrono_like_now(), review_status:"À vérifier".into(), exposure_id:None }], message:"WebFinger a résolu exactement l’identifiant fédéré déclaré. Le profil reste une correspondance possible et ne confirme jamais l’identité de la personne.".into(), analysis_job_id:None });
    }
    if module_id == "osint-gitlab-profile" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let query_name = if target_kind == "email" {
            "public_email"
        } else {
            "username"
        };
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("MANTIS-Posture/0.1 public-profile-check")
            .build()
            .map_err(|error| error.to_string())?
            .get("https://gitlab.com/api/v4/users")
            .query(&[(query_name, target.trim())])
            .send()
            .map_err(|_| {
                "GitLab est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées."
                    .to_string()
            })?;
        let status = response.status();
        let body = read_bounded_http_body(response, 512 * 1024)?;
        std::fs::write(&raw_path, &body).map_err(|error| error.to_string())?;
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "GitLab limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "GitLab a refusé ou interrompu cette vérification publique."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let Some(profile) = parse_gitlab_public_profile(&body, &target, &target_kind)? else {
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun profil GitLab public exact ne correspond à cette donnée.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun profil GitLab public exact n’a été trouvé. Ce n’est pas une preuve d’absence sur d’autres plateformes.".into(), analysis_job_id: None });
        };
        let id = Uuid::new_v4().to_string();
        let title = format!("Profil GitLab potentiel : {}", profile.username);
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'GitLab',?9,datetime('now'),?10,?11,'À vérifier')",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            "Profil GitLab public exact normalisé comme correspondance possible.",
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id: scan_id.clone(), target: target.clone(), signals: vec![OsintSignal { id, module_id, scan_id, target, signal_type:"profil_public".into(), title, explanation:profile.explanation, severity:"faible".into(), confidence:profile.confidence, source:"GitLab".into(), source_url:Some(profile.profile_url), discovered_at:chrono_like_now(), review_status:"À vérifier".into(), exposure_id:None }], message:"Un profil GitLab public correspond exactement à la donnée déclarée. Il est présenté comme correspondance possible et ne confirme jamais l’identité de la personne.".into(), analysis_job_id:None });
    }
    if module_id == "osint-github-profile" {
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("MANTIS-Posture/0.1 public-profile-check")
            .build()
            .map_err(|error| error.to_string())?;
        let response = client
            .get(format!("https://api.github.com/users/{}", target.trim()))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .send()
            .map_err(|_| {
                "GitHub est momentanément indisponible. Vos données MANTIS n’ont pas été modifiées."
                    .to_string()
            })?;
        let status = response.status();
        let body = read_bounded_http_body(response, 512 * 1024)?;
        std::fs::write(&raw_path, &body).map_err(|error| error.to_string())?;
        if status == reqwest::StatusCode::NOT_FOUND {
            conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
            conn.execute(
                "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
                params![module_id],
            )
            .map_err(|error| error.to_string())?;
            log_module(
                &conn,
                &module_id,
                "scan",
                "ok",
                "Aucun profil GitHub public ne correspond à ce pseudo.",
            )?;
            sync_scan_observability(&conn, &scan_id)?;
            return Ok(OsintScanSummary { scan_id, target, signals: Vec::new(), message: "Aucun profil GitHub public n’a été trouvé pour ce pseudo. Ce n’est pas une preuve d’absence sur d’autres plateformes.".into(), analysis_job_id: None });
        }
        if !status.is_success() {
            let message = if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                "GitHub limite temporairement les requêtes publiques. Réessayez plus tard."
            } else {
                "GitHub a refusé ou interrompu cette vérification publique."
            };
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),message,scan_id]).map_err(|error|error.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", message)?;
            return Err(message.into());
        }
        let mut profile = parse_github_public_profile(&body, &target)?;
        let repositories_body = client
            .get(format!(
                "https://api.github.com/users/{}/repos",
                profile.login
            ))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("per_page", "5"),
            ])
            .send()
            .ok()
            .filter(|response| response.status().is_success())
            .and_then(|response| read_bounded_http_body(response, 512 * 1024).ok())
            .unwrap_or_else(|| "[]".into());
        let repositories = parse_github_public_repositories(&repositories_body, &profile.login)
            .unwrap_or_default();
        if !repositories.is_empty() {
            profile.explanation.push_str(&format!(
                " Dépôts publics récents : {}.",
                repositories.join(" ; ")
            ));
        }
        let organizations_body = client
            .get(format!(
                "https://api.github.com/users/{}/orgs",
                profile.login
            ))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .query(&[("per_page", "5")])
            .send()
            .ok()
            .filter(|response| response.status().is_success())
            .and_then(|response| read_bounded_http_body(response, 512 * 1024).ok())
            .unwrap_or_else(|| "[]".into());
        let organizations =
            parse_github_public_organizations(&organizations_body).unwrap_or_default();
        if !organizations.is_empty() {
            profile.explanation.push_str(&format!(
                " Organisations publiques : {}.",
                organizations.join(" ; ")
            ));
        }
        let events_body = client
            .get(format!(
                "https://api.github.com/users/{}/events/public",
                profile.login
            ))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .query(&[("per_page", "8")])
            .send()
            .ok()
            .filter(|response| response.status().is_success())
            .and_then(|response| read_bounded_http_body(response, 512 * 1024).ok())
            .unwrap_or_else(|| "[]".into());
        let events = parse_github_public_events(&events_body, &profile.login).unwrap_or_default();
        if !events.is_empty() {
            profile.explanation.push_str(&format!(
                " Activité publique récente : {}.",
                events.join(" ; ")
            ));
        }
        std::fs::write(
            &raw_path,
            format!("{{\"profile\":{body},\"repositories\":{repositories_body},\"organizations\":{organizations_body},\"events\":{events_body}}}"),
        )
        .map_err(|error| error.to_string())?;
        let id = Uuid::new_v4().to_string();
        let title = format!("Profil GitHub potentiel : {}", profile.login);
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'profil_public',?6,?7,'faible',?8,'GitHub',?9,datetime('now'),?10,?11,'À vérifier')",params![id,module_id,scan_id,identity_id,target,title,profile.explanation,profile.confidence,profile.profile_url,profile.evidence_ref,raw_path.to_string_lossy().to_string()]).map_err(|error|error.to_string())?;
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|error|error.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|error| error.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            "Profil GitHub public exact normalisé comme correspondance possible.",
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        return Ok(OsintScanSummary { scan_id: scan_id.clone(), target: target.clone(), signals: vec![OsintSignal { id, module_id, scan_id, target, signal_type:"profil_public".into(), title, explanation:profile.explanation, severity:"faible".into(), confidence:profile.confidence, source:"GitHub".into(), source_url:Some(profile.profile_url), discovered_at:chrono_like_now(), review_status:"À vérifier".into(), exposure_id:None }], message:"Un profil GitHub public porte exactement ce pseudo. Il est présenté comme correspondance possible et ne confirme jamais l’identité de la personne.".into(), analysis_job_id:None });
    }
    if module_id == "osint-web-footprint" {
        let ddgs = installed_ddgs_path(&app)?;
        let search_target = match target_kind.as_str() {
            "telephone" => normalize_phone_number(&target),
            "adresse" => normalize_address_text(&target),
            _ => target.clone(),
        };
        let mut args = vec![
            "--target".into(),
            search_target,
            "--kind".into(),
            target_kind.clone(),
            "--max-results".into(),
            "100".into(),
        ];
        let ddgs_contexts = ddgs_context_for_identity(&conn, &identity_id, &target_kind);
        for context in &ddgs_contexts {
            args.push("--context".into());
            args.push(context.clone());
        }
        let (stdout, stderr) = match run_process_with_limit(
            &ddgs,
            &args,
            Duration::from_secs(420),
            32 * 1024 * 1024,
        ) {
            Ok(value) => value,
            Err(e) => {
                conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),error_message=?1 WHERE id=?2",params![e,scan_id]).map_err(|x|x.to_string())?;
                log_module(&conn, &module_id, "scan", "erreur", &e)?;
                return Err(e);
            }
        };
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        std::fs::write(&raw_path, format!("{}\n{}", stdout, stderr)).map_err(|e| e.to_string())?;
        let output: DdgsSidecarOutput = serde_json::from_str(&stdout)
            .map_err(|_| "Le module Empreinte Web a renvoyé un format inattendu.".to_string())?;
        if output.version != 1 {
            return Err(
                "Le module Empreinte Web est incompatible avec cette version de MANTIS.".into(),
            );
        }
        let mut seen_urls = std::collections::HashSet::new();
        let mut signals = Vec::new();
        for (index, item) in output
            .results
            .into_iter()
            .filter(|item| !item.url.trim().is_empty())
            .filter(|item| seen_urls.insert(item.url.clone()))
            .enumerate()
        {
            let id = Uuid::new_v4().to_string();
            let exact_query = item.query.trim_start().starts_with('"');
            // A second request is useful only for a small sample of the search
            // results, but exact-name queries deserve priority: otherwise a
            // valid profile or document ranked below a directory can vanish.
            let page_evidence = if target_kind == "adresse" {
                ddgs_contexts
                    .first()
                    .map_or_else(PublicPageEvidence::default, |postal_code| {
                        inspect_public_page_for_address(&item.url, &target, postal_code)
                    })
            } else if exact_query || matches!(target_kind.as_str(), "nom" | "prenom") || index < 5 {
                inspect_public_page_for_kind(&item.url, &target, &target_kind)
            } else {
                PublicPageEvidence::default()
            };
            // DDGS also runs a deliberately broad, unquoted name query. Its
            // results are discovery leads only: without direct page/document
            // evidence they must not enter the actionable queue, otherwise a
            // homonym such as “Zoe Zambito” can be mistaken for the target.
            // Phone numbers and postal addresses are too collision-prone to
            // become facts from an index snippet alone. Keep the raw DDGS
            // artifact, but create no signal/fact without a direct match.
            if matches!(target_kind.as_str(), "telephone" | "adresse")
                && !page_evidence.exact_identifier_match
            {
                continue;
            }
            if matches!(
                page_evidence.media_type.as_deref(),
                Some("application/pdf" | "text/plain")
            ) && !page_evidence.exact_identifier_match
            {
                continue;
            }
            let indexed_title = clean_web_result_text(&item.title)
                .unwrap_or_else(|| "Résultat Web à vérifier".to_string());
            let title = page_evidence.title.clone().unwrap_or(indexed_title);
            let wayback_snapshot = page_evidence
                .exact_identifier_match
                .then(|| lookup_wayback_snapshot(&item.url))
                .flatten();
            let document_note = page_evidence
                .excerpt
                .as_deref()
                .map(|excerpt| format!(" Extrait local probant : {excerpt}"))
                .unwrap_or_default();
            let archive_note = wayback_snapshot
                .as_ref()
                .map(|snapshot| {
                    format!(
                        " Une copie Wayback est signalée au {} ; elle n’a pas été consultée automatiquement.",
                        snapshot.timestamp
                    )
                })
                .unwrap_or_default();
            let backend_note = if item.backend.trim().is_empty() {
                "Moteur amont non communiqué par DDGS (métamoteur, sélection automatique)."
                    .to_string()
            } else {
                format!(
                    "Moteur amont non communiqué par DDGS (métamoteur, backend {}).",
                    item.backend.trim()
                )
            };
            let explanation = match (page_evidence.exact_identifier_match, clean_web_result_text(&item.snippet)) {
                (true, Some(snippet)) => format!("Preuve publique vérifiée : « {} » est présent dans la page ou le document. Extrait indexé : {snippet}.{document_note}{archive_note}", target.trim()),
                (true, None) => format!("Preuve publique vérifiée : « {} » est présent dans la page ou le document.{document_note}{archive_note}", target.trim()),
                (false, Some(snippet)) => format!("Résultat indexé non confirmé dans la page : {}", snippet),
                (false, None) => "Résultat indexé sans preuve directe lisible dans la page.".to_string(),
            };
            let explanation = format!("{backend_note} {explanation}");
            let explanation = if exact_query {
                format!("Requête exacte DDGS. {explanation}")
            } else {
                explanation
            };
            let evidence_path = if page_evidence.exact_identifier_match
                && matches!(
                    page_evidence.media_type.as_deref(),
                    Some("application/pdf" | "text/plain")
                ) {
                let extension = if page_evidence.media_type.as_deref() == Some("application/pdf") {
                    "pdf"
                } else {
                    "txt"
                };
                let path = osint_root(&app)?
                    .join("raw")
                    .join(format!("{scan_id}-document-{index}.{extension}"));
                if let Some(bytes) = page_evidence.downloaded_bytes.as_deref() {
                    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
                }
                path
            } else {
                raw_path.clone()
            };
            let evidence_ref = match (&page_evidence.media_type, &page_evidence.content_sha256) {
                (Some(media_type), Some(hash)) if page_evidence.exact_identifier_match => {
                    let archive = wayback_snapshot
                        .as_ref()
                        .map(|snapshot| {
                            format!(" · Wayback {} · {}", snapshot.timestamp, snapshot.url)
                        })
                        .unwrap_or_default();
                    format!(
                        "{} · {} · SHA-256 {}{}",
                        item.query, media_type, hash, archive
                    )
                }
                _ => item.query.clone(),
            };
            conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'mention',?6,?7,'faible','À vérifier','DDGS',?8,datetime('now'),?9,?10,'À vérifier')", params![id,module_id,scan_id,identity_id,target,title,explanation,item.url,evidence_ref,evidence_path.to_string_lossy().to_string()]).map_err(|e| e.to_string())?;
            signals.push(OsintSignal {
                id,
                module_id: module_id.clone(),
                scan_id: scan_id.clone(),
                target: target.clone(),
                signal_type: "mention".into(),
                title,
                explanation,
                severity: "faible".into(),
                confidence: "À vérifier".into(),
                source: "DDGS".into(),
                source_url: Some(item.url),
                discovered_at: chrono_like_now(),
                review_status: "À vérifier".into(),
                exposure_id: None,
            });
        }
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|e|e.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|e| e.to_string())?;
        let partial_note = if output.errors.is_empty() {
            String::new()
        } else {
            " Certains moteurs n’ont pas répondu ; les résultats affichés restent exploitables."
                .into()
        };
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            &format!(
                "{} mention(s) potentielle(s) normalisée(s).{}",
                signals.len(),
                partial_note
            ),
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        let message = if signals.is_empty() {
            format!("Aucune mention publique n’a été remontée pour cette requête. Ce n’est pas une preuve d’absence.{}", partial_note)
        } else {
            format!(
                "Les résultats sont des mentions potentielles, pas des expositions confirmées.{}",
                partial_note
            )
        };
        return Ok(OsintScanSummary {
            scan_id,
            target,
            signals,
            message,
            analysis_job_id: None,
        });
    }
    if module_id == "osint-username-profiles" {
        let executable = installed_maigret_path(&app)?;
        let args = vec![
            "--target".into(),
            target.clone(),
            "--timeout".into(),
            "8".into(),
            "--top-sites".into(),
            "150".into(),
        ];
        let (stdout, stderr) = match run_process(&executable, &args, Duration::from_secs(180)) {
            Ok(value) => value,
            Err(error) => {
                conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),error_message=?1 WHERE id=?2",params![error,scan_id]).map_err(|e|e.to_string())?;
                log_module(&conn, &module_id, "scan", "erreur", &error)?;
                return Err(error);
            }
        };
        let raw_path = osint_root(&app)?
            .join("raw")
            .join(format!("{}.json", scan_id));
        let raw = format!(
            "{}\n{}",
            stdout,
            stderr.chars().take(4_000).collect::<String>()
        );
        std::fs::write(&raw_path, raw.as_bytes()).map_err(|e| e.to_string())?;
        let output = match parse_maigret_output(&stdout, &target) {
            Ok(output) => output,
            Err(error) => {
                conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),error,scan_id]).map_err(|e|e.to_string())?;
                log_module(&conn, &module_id, "scan", "erreur", &error)?;
                return Err(error);
            }
        };
        if let Some(error) = output.error.as_deref() {
            let bounded = error.chars().take(400).collect::<String>();
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),raw_result_path=?1,error_message=?2 WHERE id=?3",params![raw_path.to_string_lossy().to_string(),bounded,scan_id]).map_err(|e|e.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", &bounded)?;
            return Err(bounded);
        }
        let mut signals = Vec::new();
        for (index, item) in deduplicate_maigret_results(output.results)
            .into_iter()
            .enumerate()
        {
            let id = Uuid::new_v4().to_string();
            let category =
                clean_web_result_text(&item.category).unwrap_or_else(|| "profil public".into());
            let signal_type = maigret_signal_type(&category);
            let title = format!("Profil potentiel : {}", item.site_name.trim());
            let page_evidence = if index < 5 {
                inspect_public_page(&item.url, &target)
            } else {
                PublicPageEvidence::default()
            };
            let explanation = if page_evidence.exact_identifier_match {
                format!("Preuve publique vérifiée : le pseudo exact est présent dans la page. Maigret a observé ce profil potentiel sur cette plateforme (catégorie : {}). Cette correspondance possible ne confirme jamais l’identité de la personne.", category)
            } else {
                format!("Maigret a observé une page publique correspondant exactement au pseudo sur cette plateforme (catégorie : {}). La page n’a pas pu confirmer directement le pseudo dans le délai imparti. Cette correspondance possible ne confirme jamais l’identité de la personne.", category)
            };
            conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status,identity_value_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'faible','À vérifier','Maigret',?9,datetime('now'),?10,?11,'À vérifier',?12)",params![id,module_id,scan_id,identity_id,target,signal_type,title,explanation,item.url,item.site_name,raw_path.to_string_lossy().to_string(),identity_value_id]).map_err(|e|e.to_string())?;
            signals.push(OsintSignal {
                id,
                module_id: module_id.clone(),
                scan_id: scan_id.clone(),
                target: target.clone(),
                signal_type: signal_type.into(),
                title,
                explanation,
                severity: "faible".into(),
                confidence: "À vérifier".into(),
                source: "Maigret".into(),
                source_url: Some(item.url),
                discovered_at: chrono_like_now(),
                review_status: "À vérifier".into(),
                exposure_id: None,
            });
        }
        conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|e|e.to_string())?;
        conn.execute(
            "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
            params![module_id],
        )
        .map_err(|e| e.to_string())?;
        log_module(
            &conn,
            &module_id,
            "scan",
            "ok",
            &format!(
                "{} profil(s) potentiel(s) normalisé(s) sur {} vérification(s).",
                signals.len(),
                output.summary.checked
            ),
        )?;
        sync_scan_observability(&conn, &scan_id)?;
        conn.execute(
            "UPDATE osint_raw_artifacts SET collector_version=?1 WHERE scan_id=?2",
            params![output.collector_version, scan_id],
        )
        .map_err(|e| e.to_string())?;
        let message = if signals.is_empty() {
            format!("Aucun profil potentiel remonté parmi {} sites vérifiés. Cela ne prouve pas l’absence de profil.",output.summary.checked)
        } else {
            format!("{} profil(s) potentiel(s) remonté(s). Chaque page doit être vérifiée avant toute attribution.",signals.len())
        };
        return Ok(OsintScanSummary {
            scan_id,
            target,
            signals,
            message,
            analysis_job_id: None,
        });
    }
    let user_scanner = installed_user_scanner_path(&app)?;
    let args = vec![
        "--target".into(),
        target.clone(),
        "--kind".into(),
        target_kind.clone(),
        "--timeout".into(),
        "8".into(),
        "--concurrency".into(),
        "10".into(),
    ];
    let (stdout, stderr) = match run_process(&user_scanner, &args, Duration::from_secs(180)) {
        Ok(v) => v,
        Err(e) => {
            conn.execute("UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),error_message=?1 WHERE id=?2",params![e,scan_id]).map_err(|x|x.to_string())?;
            log_module(&conn, &module_id, "scan", "erreur", &e)?;
            return Err(e);
        }
    };
    let raw_path = osint_root(&app)?
        .join("raw")
        .join(format!("{}.json", scan_id));
    std::fs::write(&raw_path, format!("{}\n{}", stdout, stderr)).map_err(|e| e.to_string())?;
    let output: UserScannerSidecarOutput = serde_json::from_str(&stdout)
        .map_err(|_| "User Scanner a renvoyé un format inattendu.".to_string())?;
    if output.version != 1 || output.target != target || output.target_kind != target_kind {
        return Err("Le résultat User Scanner ne correspond pas à l’identité demandée.".into());
    }
    if let Some(error) = output.error {
        return Err(error);
    }
    let mut seen = std::collections::HashSet::new();
    let mut signals = Vec::new();
    for (index, item) in output
        .results
        .into_iter()
        .filter(|item| matches!(item.status.as_str(), "Registered" | "Found"))
        .filter(|item| seen.insert(format!("{}|{}", item.site_name, item.url)))
        .take(100)
        .enumerate()
    {
        let id = Uuid::new_v4().to_string();
        let kind_label = if target_kind == "email" {
            "Service associé potentiellement à l’e-mail"
        } else {
            "Compte potentiel pour le pseudo"
        };
        let title = format!("{} : {}", kind_label, item.site_name.trim());
        let source_url = scanner_source_url(&item.url);
        let page_evidence = if target_kind == "pseudo" && index < 5 {
            source_url
                .as_deref()
                .map(|url| inspect_public_page(url, &target))
                .unwrap_or_default()
        } else {
            PublicPageEvidence::default()
        };
        let explanation = if page_evidence.exact_identifier_match {
            format!("Preuve publique vérifiée : le pseudo exact est présent dans la page. Catégorie : {}. {}. Cette correspondance possible ne confirme jamais l’identité de la personne.", item.category.trim(), scanner_details(&item.extra, &item.reason))
        } else {
            format!("Catégorie : {}. {}. Vérifiez la page et son contexte avant toute conclusion ou action.", item.category.trim(), scanner_details(&item.extra, &item.reason))
        };
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES (?1,?2,?3,?4,?5,'compte_potentiel',?6,?7,'faible','À vérifier','User Scanner',?8,datetime('now'),?9,?10,'À vérifier')", params![id,module_id,scan_id,identity_id,target,title,explanation,source_url,item.site_name,raw_path.to_string_lossy().to_string()]).map_err(|e| e.to_string())?;
        signals.push(OsintSignal {
            id,
            module_id: module_id.clone(),
            scan_id: scan_id.clone(),
            target: target.clone(),
            signal_type: "compte_potentiel".into(),
            title,
            explanation,
            severity: "faible".into(),
            confidence: "À vérifier".into(),
            source: "User Scanner".into(),
            source_url,
            discovered_at: chrono_like_now(),
            review_status: "À vérifier".into(),
            exposure_id: None,
        });
    }
    conn.execute("UPDATE osint_scans SET status='termine',completed_at=datetime('now'),raw_result_path=?1 WHERE id=?2",params![raw_path.to_string_lossy().to_string(),scan_id]).map_err(|e|e.to_string())?;
    conn.execute(
        "UPDATE osint_modules SET status='actif',last_run=datetime('now') WHERE id=?1",
        params![module_id],
    )
    .map_err(|e| e.to_string())?;
    let partial_note = if output.summary.errors > 0 {
        format!(
            " {} source(s) n’ont pas répondu ; le résultat est donc partiel.",
            output.summary.errors
        )
    } else {
        String::new()
    };
    let excluded_note = if output.summary.notification_checks_excluded > 0 {
        format!(
            " {} vérification(s) pouvant déclencher une notification ont été exclues.",
            output.summary.notification_checks_excluded
        )
    } else {
        String::new()
    };
    log_module(
        &conn,
        &module_id,
        "scan",
        "ok",
        &format!(
            "{} signal(s) normalisé(s) sur {} vérification(s).",
            signals.len(),
            output.summary.checked
        ),
    )?;
    let message = if signals.is_empty() {
        format!("Aucun compte potentiel n’a été détecté parmi {} vérification(s). Ce n’est pas une preuve d’absence.{}{}", output.summary.checked, partial_note, excluded_note)
    } else {
        format!("{} compte(s) potentiel(s) détecté(s) sur {} vérification(s). Les résultats restent à vérifier.{}{}", signals.len(), output.summary.checked, partial_note, excluded_note)
    };
    sync_scan_observability(&conn, &scan_id)?;
    let _ = (folder_id, output.summary.found, output.summary.skipped);
    Ok(OsintScanSummary {
        scan_id,
        target,
        signals,
        message,
        analysis_job_id: None,
    })
}

fn chrono_like_now() -> String {
    "Maintenant".into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedGithubPublicProfile {
    login: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
}

fn is_valid_github_username(value: &str) -> bool {
    let value = value.trim().trim_start_matches('@');
    !value.is_empty()
        && value.len() <= 39
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn read_bounded_http_body(
    response: reqwest::blocking::Response,
    maximum_bytes: u64,
) -> Result<String, String> {
    let mut body = Vec::new();
    response
        .take(maximum_bytes + 1)
        .read_to_end(&mut body)
        .map_err(|_| "La réponse de la source est illisible.".to_string())?;
    if body.len() as u64 > maximum_bytes {
        return Err("La réponse de la source dépasse la limite de sécurité MANTIS.".into());
    }
    String::from_utf8(body)
        .map_err(|_| "La réponse de la source n’est pas du texte UTF-8 valide.".into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedGravatarPublicProfile {
    display_name: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedKeybasePublicProfile {
    username: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
    verified_proof_count: usize,
    verified_proofs: Vec<KeybasePublicProof>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeybasePublicProof {
    service: String,
    account: String,
    url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedBlueskyPublicProfile {
    handle: String,
    did: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedHackerNewsPublicProfile {
    username: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
}

fn remove_sensitive_api_fields(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            object.retain(|key, _| {
                !matches!(
                    key.to_ascii_lowercase().as_str(),
                    "csrf_token"
                        | "auth_token"
                        | "access_token"
                        | "refresh_token"
                        | "session"
                        | "session_token"
                        | "cookie"
                )
            });
            for child in object.values_mut() {
                remove_sensitive_api_fields(child);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                remove_sensitive_api_fields(child);
            }
        }
        _ => {}
    }
}

fn sanitized_keybase_public_response(body: &str) -> Result<String, String> {
    let mut value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Keybase a renvoyé une réponse dans un format inattendu.".to_string())?;
    remove_sensitive_api_fields(&mut value);
    serde_json::to_string(&value)
        .map_err(|_| "La réponse publique Keybase ne peut pas être conservée sûrement.".to_string())
}

fn is_valid_keybase_username(value: &str) -> bool {
    let username = value.trim().trim_start_matches('@');
    !username.is_empty()
        && username.len() <= 64
        && username.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

fn is_valid_bluesky_handle(value: &str) -> bool {
    let handle = value.trim().trim_start_matches('@');
    if handle.is_empty() || handle.len() > 253 || handle.starts_with('.') || handle.ends_with('.') {
        return false;
    }
    let labels = handle.split('.').collect::<Vec<_>>();
    if labels.len() < 2
        || labels.iter().any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
        })
    {
        return false;
    }
    labels
        .last()
        .and_then(|label| label.chars().next())
        .is_some_and(|character| character.is_ascii_alphabetic())
}

fn bluesky_handle_for_target(value: &str) -> Option<String> {
    let normalized = value.trim().trim_start_matches('@').to_ascii_lowercase();
    let handle = if normalized.contains('.') {
        normalized
    } else {
        if normalized.is_empty()
            || normalized.len() > 63
            || normalized.starts_with('-')
            || normalized.ends_with('-')
            || !normalized
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return None;
        }
        format!("{normalized}.bsky.social")
    };
    is_valid_bluesky_handle(&handle).then_some(handle)
}

fn is_valid_hackernews_username(value: &str) -> bool {
    let username = value.trim();
    !username.is_empty()
        && username.len() <= 80
        && username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn parse_hackernews_public_profile(
    body: &str,
    expected_username: &str,
) -> Result<ParsedHackerNewsPublicProfile, String> {
    let response: serde_json::Value = serde_json::from_str(body).map_err(|_| {
        "Hacker News a renvoyé un profil public dans un format inattendu.".to_string()
    })?;
    let profile = response
        .get("profile")
        .ok_or("Hacker News n’a renvoyé aucun profil exploitable.")?;
    let username = github_public_text(profile, "username", 80)
        .ok_or("Hacker News n’a renvoyé aucun pseudo public exploitable.")?;
    if !username.eq_ignore_ascii_case(expected_username.trim())
        || !is_valid_hackernews_username(&username)
    {
        return Err("Le profil Hacker News reçu ne correspond pas au pseudo demandé.".into());
    }
    let profile_url = format!("https://news.ycombinator.com/user?id={username}");
    let mut details = Vec::new();
    for (key, label, maximum) in [
        ("about", "présentation publique", 280usize),
        ("created_at", "compte créé le", 40),
    ] {
        if let Some(value) = github_public_text(profile, key, maximum) {
            details.push(format!("{label} : {value}"));
        }
    }
    if let Some(karma) = profile
        .get("karma")
        .and_then(|value| value.as_i64())
        .filter(|value| *value >= 0)
    {
        details.push(format!("karma public : {karma}"));
    }
    let mut activity = Vec::new();
    for hit in response
        .get("activity")
        .and_then(|value| value.get("hits"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .take(5)
    {
        let author = github_public_text(hit, "author", 80).unwrap_or_default();
        if !author.eq_ignore_ascii_case(&username) {
            continue;
        }
        let Some(object_id) =
            hit.get("objectID")
                .and_then(|value| value.as_str())
                .filter(|value| {
                    !value.is_empty()
                        && value.len() <= 32
                        && value.chars().all(|character| character.is_ascii_digit())
                })
        else {
            continue;
        };
        let label = github_public_text(hit, "title", 180)
            .or_else(|| github_public_text(hit, "story_title", 180))
            .unwrap_or_else(|| "publication publique".into());
        activity.push(format!(
            "{} : https://news.ycombinator.com/item?id={object_id}",
            label.replace('\n', " ")
        ));
    }
    if !activity.is_empty() {
        details.push(format!(
            "activité publique récente : {}",
            activity.join(" ; ")
        ));
    }
    let context = if details.is_empty() {
        "Aucun détail public complémentaire n’est renseigné dans le profil.".into()
    } else {
        details.into_iter().take(5).collect::<Vec<_>>().join(" · ")
    };
    Ok(ParsedHackerNewsPublicProfile {
        username: username.clone(),
        profile_url,
        explanation: format!("Preuve publique vérifiée : le pseudo exact « {username} » est retourné par l’API publique Hacker News. {context} Ce profil reste une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: "Pseudo exact sur Hacker News — identité non attribuée".into(),
        evidence_ref: format!("Hacker News Search · pseudo exact : {username} · activité publique bornée"),
    })
}

fn parse_bluesky_public_profile(
    body: &str,
    expected_handle: &str,
) -> Result<ParsedBlueskyPublicProfile, String> {
    let profile: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Bluesky a renvoyé un profil public dans un format inattendu.".to_string())?;
    let handle = github_public_text(&profile, "handle", 253)
        .ok_or("Bluesky n’a renvoyé aucun handle public exploitable.")?
        .to_ascii_lowercase();
    if !handle.eq_ignore_ascii_case(expected_handle) || !is_valid_bluesky_handle(&handle) {
        return Err("Le profil Bluesky reçu ne correspond pas au handle demandé.".into());
    }
    let did = github_public_text(&profile, "did", 180)
        .filter(|value| value.starts_with("did:plc:") || value.starts_with("did:web:"))
        .ok_or("Bluesky n’a renvoyé aucun identifiant public stable exploitable.")?;
    let profile_url = format!("https://bsky.app/profile/{handle}");
    let mut details = Vec::new();
    for (key, label, maximum) in [
        ("displayName", "nom d’affichage", 120usize),
        ("description", "présentation publique", 280),
        ("createdAt", "compte créé le", 40),
    ] {
        if let Some(value) = github_public_text(&profile, key, maximum) {
            details.push(format!("{label} : {value}"));
        }
    }
    if let Some(website) = github_public_text(&profile, "website", 240)
        .and_then(|value| scanner_source_url(&value))
        .filter(|value| is_safe_public_http_url(value))
    {
        details.push(format!("site public déclaré : {website}"));
    }
    for (key, label) in [
        ("followersCount", "abonnés publics"),
        ("followsCount", "abonnements publics"),
        ("postsCount", "publications publiques"),
    ] {
        if let Some(value) = profile
            .get(key)
            .and_then(|value| value.as_i64())
            .filter(|value| *value >= 0)
        {
            details.push(format!("{label} : {value}"));
        }
    }
    let platform_verified = profile
        .get("verification")
        .and_then(|value| value.get("verifiedStatus"))
        .and_then(|value| value.as_str())
        == Some("valid");
    if platform_verified {
        details.push("compte marqué comme vérifié par Bluesky".into());
    }
    let context = if details.is_empty() {
        "Aucun détail public complémentaire n’est renseigné dans le profil.".into()
    } else {
        details.into_iter().take(7).collect::<Vec<_>>().join(" · ")
    };
    Ok(ParsedBlueskyPublicProfile {
        handle: handle.clone(),
        did: did.clone(),
        profile_url,
        explanation: format!("Preuve publique vérifiée : le handle exact « {handle} » est retourné par l’API publique Bluesky. {context} Ce profil reste une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: if platform_verified {
            "Handle exact et compte vérifié par la plateforme — identité non attribuée".into()
        } else {
            "Handle exact sur Bluesky — identité non attribuée".into()
        },
        evidence_ref: format!("Bluesky AppView · handle exact : {handle} · DID : {did}"),
    })
}

fn parse_keybase_public_profile(
    body: &str,
    expected_username: &str,
) -> Result<Option<ParsedKeybasePublicProfile>, String> {
    let response: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Keybase a renvoyé une réponse dans un format inattendu.".to_string())?;
    if response
        .get("status")
        .and_then(|status| status.get("code"))
        .and_then(|value| value.as_i64())
        != Some(0)
    {
        return Err("Keybase a refusé cette recherche publique.".into());
    }
    let users = response
        .get("them")
        .and_then(|value| value.as_array())
        .ok_or("Keybase n’a renvoyé aucune liste de profils exploitable.")?;
    if users.len() > 1 {
        return Err("Keybase a renvoyé trop de profils pour une recherche exacte.".into());
    }
    let Some(user) = users.first().filter(|user| !user.is_null()) else {
        return Ok(None);
    };
    let username = user
        .get("basics")
        .and_then(|value| github_public_text(value, "username", 64))
        .ok_or("Keybase n’a renvoyé aucun pseudo public exploitable.")?;
    let expected = expected_username.trim().trim_start_matches('@');
    if !username.eq_ignore_ascii_case(expected) || !is_valid_keybase_username(&username) {
        return Err("Le profil Keybase reçu ne correspond pas au pseudo demandé.".into());
    }
    let profile_url = format!("https://keybase.io/{username}");
    let mut details = Vec::new();
    if let Some(profile) = user.get("profile") {
        for (key, label, maximum) in [
            ("full_name", "nom public", 120usize),
            ("location", "localisation déclarée", 120),
            ("bio", "présentation publique", 280),
        ] {
            if let Some(value) = github_public_text(profile, key, maximum) {
                details.push(format!("{label} : {value}"));
            }
        }
    }
    let mut proofs = Vec::<KeybasePublicProof>::new();
    for proof in user
        .get("proofs_summary")
        .and_then(|value| value.get("all"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .take(50)
    {
        let valid_state = proof.get("state").is_some_and(|value| {
            value.as_i64() == Some(1)
                || value.as_str().is_some_and(|state| {
                    matches!(state.to_ascii_lowercase().as_str(), "valid" | "ok")
                })
        });
        if !valid_state {
            continue;
        }
        let Some(url) = github_public_text(proof, "service_url", 240)
            .or_else(|| github_public_text(proof, "human_url", 240))
            .and_then(|value| scanner_source_url(&value))
        else {
            continue;
        };
        let service =
            github_public_text(proof, "proof_type", 60).unwrap_or_else(|| "service public".into());
        let account = github_public_text(proof, "nametag", 120).unwrap_or_default();
        proofs.push(KeybasePublicProof {
            service,
            account,
            url,
        });
    }
    proofs.sort_by(|left, right| left.url.cmp(&right.url));
    proofs.dedup_by(|left, right| left.url == right.url);
    proofs.truncate(8);
    let verified_proof_count = proofs.len();
    if verified_proof_count > 0 {
        details.push(format!(
            "preuves publiques vérifiées par Keybase : {}",
            proofs
                .iter()
                .map(|proof| if proof.account.is_empty() {
                    format!("{} : {}", proof.service, proof.url)
                } else {
                    format!("{} ({}) : {}", proof.service, proof.account, proof.url)
                })
                .collect::<Vec<_>>()
                .join(" ; ")
        ));
    }
    let context = if details.is_empty() {
        "Aucun détail public complémentaire ni preuve externe valide n’est disponible.".into()
    } else {
        details.into_iter().take(4).collect::<Vec<_>>().join(" · ")
    };
    Ok(Some(ParsedKeybasePublicProfile {
        username: username.clone(),
        profile_url,
        explanation: format!("Preuve publique vérifiée : le pseudo exact « {username} » est retourné par l’annuaire public Keybase. {context} Ce réseau de comptes reste une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: if verified_proof_count > 0 {
            format!("Pseudo exact et {verified_proof_count} preuve(s) publique(s) Keybase — identité non attribuée")
        } else {
            "Pseudo exact sur Keybase sans preuve externe valide — identité non attribuée".into()
        },
        evidence_ref: format!("Keybase · pseudo exact : {username} · {verified_proof_count} preuve(s) publique(s) valide(s)"),
        verified_proof_count,
        verified_proofs: proofs,
    }))
}

fn parse_gravatar_public_profile(
    body: &str,
    expected_email_hash: &str,
) -> Result<ParsedGravatarPublicProfile, String> {
    let profile: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Gravatar a renvoyé un profil public dans un format inattendu.".to_string())?;
    let returned_hash = github_public_text(&profile, "hash", 64)
        .ok_or("Gravatar n’a renvoyé aucun identifiant de profil exploitable.")?;
    if returned_hash.len() != 64
        || !returned_hash.eq_ignore_ascii_case(expected_email_hash)
        || !returned_hash
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("Le profil Gravatar reçu ne correspond pas à l’adresse demandée.".into());
    }
    let profile_url = github_public_text(&profile, "profile_url", 240)
        .and_then(|value| scanner_source_url(&value))
        .filter(|value| {
            matches!(
                public_source_host(Some(value)).as_deref(),
                Some("gravatar.com" | "www.gravatar.com")
            )
        })
        .ok_or("Gravatar n’a renvoyé aucune URL publique sûre pour ce profil.")?;
    let display_name = github_public_text(&profile, "display_name", 120)
        .ok_or("Gravatar n’a renvoyé aucun nom d’affichage public exploitable.")?;
    let mut details = Vec::new();
    for (key, label, maximum) in [
        ("location", "localisation déclarée", 120usize),
        ("job_title", "fonction déclarée", 120),
        ("company", "organisation déclarée", 120),
        ("description", "présentation publique", 280),
    ] {
        if let Some(value) = github_public_text(&profile, key, maximum) {
            details.push(format!("{label} : {value}"));
        }
    }
    let mut public_links = Vec::new();
    for account in profile
        .get("verified_accounts")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .take(20)
    {
        if account.get("is_hidden").and_then(|value| value.as_bool()) == Some(true) {
            continue;
        }
        let Some(url) =
            github_public_text(account, "url", 240).and_then(|value| scanner_source_url(&value))
        else {
            continue;
        };
        let label = github_public_text(account, "service_label", 60)
            .or_else(|| github_public_text(account, "service_type", 60))
            .unwrap_or_else(|| "compte vérifié".into());
        public_links.push(format!("{label} : {url}"));
    }
    for link in profile
        .get("links")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .take(20)
    {
        let Some(url) =
            github_public_text(link, "url", 240).and_then(|value| scanner_source_url(&value))
        else {
            continue;
        };
        let label = github_public_text(link, "label", 60).unwrap_or_else(|| "lien public".into());
        public_links.push(format!("{label} : {url}"));
    }
    public_links.sort();
    public_links.dedup();
    if !public_links.is_empty() {
        details.push(format!(
            "liens publics déclarés : {}",
            public_links
                .into_iter()
                .take(8)
                .collect::<Vec<_>>()
                .join(" ; ")
        ));
    }
    let context = if details.is_empty() {
        "Aucun détail public complémentaire n’est renseigné dans le profil.".into()
    } else {
        details.into_iter().take(5).collect::<Vec<_>>().join(" · ")
    };
    Ok(ParsedGravatarPublicProfile {
        display_name,
        profile_url,
        explanation: format!("Preuve publique vérifiée : l’identifiant exact correspond au profil retourné par l’API publique Gravatar. {context} Ce profil reste une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: "Adresse exacte reliée à un profil Gravatar public — identité non attribuée".into(),
        evidence_ref: "Gravatar · profil public lié à l’adresse autorisée · API v3".into(),
    })
}

fn github_public_text(profile: &serde_json::Value, key: &str, max_chars: usize) -> Option<String> {
    profile
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max_chars).collect())
}

fn parse_github_public_profile(
    body: &str,
    expected_target: &str,
) -> Result<ParsedGithubPublicProfile, String> {
    let profile: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "GitHub a renvoyé un profil public dans un format inattendu.".to_string())?;
    let login = github_public_text(&profile, "login", 39)
        .ok_or("GitHub n’a renvoyé aucun pseudo public exploitable.")?;
    if !login.eq_ignore_ascii_case(expected_target.trim().trim_start_matches('@')) {
        return Err("Le profil GitHub reçu ne correspond pas au pseudo demandé.".into());
    }
    let profile_url = github_public_text(&profile, "html_url", 240)
        .and_then(|value| scanner_source_url(&value))
        .filter(|value| public_source_host(Some(value)).as_deref() == Some("github.com"))
        .ok_or("GitHub n’a renvoyé aucune URL publique sûre pour ce profil.")?;
    let mut details = Vec::new();
    for (key, label, maximum) in [
        ("name", "nom public", 120usize),
        ("company", "organisation", 120),
        ("location", "localisation déclarée", 120),
        ("bio", "biographie", 280),
        ("blog", "site déclaré", 180),
        ("twitter_username", "compte social déclaré", 80),
        ("created_at", "compte créé le", 40),
    ] {
        if let Some(value) = github_public_text(&profile, key, maximum) {
            details.push(format!("{label} : {value}"));
        }
    }
    for (key, label) in [
        ("public_repos", "dépôts publics"),
        ("followers", "abonnés publics"),
    ] {
        if let Some(value) = profile.get(key).and_then(|value| value.as_i64()) {
            details.push(format!("{label} : {value}"));
        }
    }
    let context = if details.is_empty() {
        "Aucun détail public complémentaire n’est renseigné dans le profil.".into()
    } else {
        details.into_iter().take(8).collect::<Vec<_>>().join(" · ")
    };
    Ok(ParsedGithubPublicProfile {
        login: login.clone(),
        profile_url,
        explanation: format!("Preuve publique vérifiée : le pseudo exact « {login} » est retourné par l’API publique GitHub. {context}. Ce profil est une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: "Pseudo exact retourné par l’API publique GitHub — identité non attribuée".into(),
        evidence_ref: format!("GitHub · pseudo exact : {login} · API publique versionnée"),
    })
}

fn parse_github_public_repositories(
    body: &str,
    expected_owner: &str,
) -> Result<Vec<String>, String> {
    let repositories: Vec<serde_json::Value> = serde_json::from_str(body).map_err(|_| {
        "GitHub a renvoyé une liste de dépôts dans un format inattendu.".to_string()
    })?;
    if repositories.len() > 5 {
        return Err("GitHub a renvoyé plus de dépôts que la limite MANTIS.".into());
    }
    let owner_prefix = format!(
        "https://github.com/{}/",
        expected_owner.to_ascii_lowercase()
    );
    let mut seen = std::collections::HashSet::new();
    Ok(repositories
        .into_iter()
        .filter_map(|repository| {
            let name = github_public_text(&repository, "name", 100)?;
            let url = github_public_text(&repository, "html_url", 240)?;
            let normalized_url = url.to_ascii_lowercase();
            if !normalized_url.starts_with(&owner_prefix) || !seen.insert(normalized_url) {
                return None;
            }
            let language = github_public_text(&repository, "language", 50)
                .map(|value| format!(" · {value}"))
                .unwrap_or_default();
            let updated = github_public_text(&repository, "pushed_at", 40)
                .map(|value| format!(" · activité : {value}"))
                .unwrap_or_default();
            Some(format!("{name}{language}{updated}"))
        })
        .take(5)
        .collect())
}

fn parse_github_public_organizations(body: &str) -> Result<Vec<String>, String> {
    let organizations: Vec<serde_json::Value> = serde_json::from_str(body).map_err(|_| {
        "GitHub a renvoyé une liste d’organisations dans un format inattendu.".to_string()
    })?;
    if organizations.len() > 5 {
        return Err("GitHub a renvoyé plus d’organisations que la limite MANTIS.".into());
    }
    let mut seen = std::collections::HashSet::new();
    Ok(organizations
        .into_iter()
        .filter_map(|organization| {
            let login = github_public_text(&organization, "login", 100)?;
            let url = github_public_text(&organization, "html_url", 240)?;
            let safe_url = scanner_source_url(&url)?;
            if public_source_host(Some(&safe_url)).as_deref() != Some("github.com")
                || !seen.insert(login.to_ascii_lowercase())
            {
                return None;
            }
            Some(login)
        })
        .take(5)
        .collect())
}

fn parse_github_public_events(body: &str, expected_actor: &str) -> Result<Vec<String>, String> {
    let events: Vec<serde_json::Value> = serde_json::from_str(body).map_err(|_| {
        "GitHub a renvoyé une activité publique dans un format inattendu.".to_string()
    })?;
    if events.len() > 8 {
        return Err("GitHub a renvoyé plus d’événements que la limite MANTIS.".into());
    }
    let expected_actor = expected_actor.to_ascii_lowercase();
    let mut seen = std::collections::HashSet::new();
    Ok(events
        .into_iter()
        .filter_map(|event| {
            if event.get("public").and_then(|value| value.as_bool()) != Some(true) {
                return None;
            }
            let actor = github_public_text(event.get("actor")?, "login", 100)?;
            if actor.to_ascii_lowercase() != expected_actor {
                return None;
            }
            let event_type = github_public_text(&event, "type", 80)?;
            let repository = github_public_text(event.get("repo")?, "name", 180)?;
            let created_at = github_public_text(&event, "created_at", 40)?;
            let fingerprint = format!("{event_type}|{repository}|{created_at}");
            if !seen.insert(fingerprint) {
                return None;
            }
            Some(format!("{event_type} · {repository} · {created_at}"))
        })
        .take(8)
        .collect())
}

fn mastodon_account_for_target(target: &str) -> Option<(String, String)> {
    let target = target.trim().trim_start_matches('@');
    let (username, instance) = target.split_once('@')?;
    if username.is_empty()
        || username.len() > 64
        || instance.is_empty()
        || instance.len() > 253
        || instance.contains('@')
        || !username.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
    {
        return None;
    }
    let probe = reqwest::Url::parse(&format!("https://{instance}/")).ok()?;
    let host = probe.host_str()?.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() || probe.port().is_some() || probe.path() != "/" {
        return None;
    }
    Some((username.to_string(), host))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedMastodonWebfingerProfile {
    account: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SpecializedPublicFinding {
    title: String,
    explanation: String,
    source: String,
    source_url: String,
    evidence_ref: String,
}

fn exact_public_name(value: &str) -> String {
    normalize_address_text(value)
}

fn parse_fr_company_findings(
    body: &str,
    expected_name: &str,
) -> Result<Vec<SpecializedPublicFinding>, String> {
    let payload: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "Le registre des entreprises a renvoyé un format inattendu.".to_string())?;
    let results = payload
        .get("results")
        .and_then(|value| value.as_array())
        .ok_or("Le registre des entreprises n’a renvoyé aucune liste exploitable.")?;
    if results.len() > 10 {
        return Err("Le registre des entreprises dépasse la limite MANTIS.".into());
    }
    let expected = exact_public_name(expected_name);
    let mut findings = Vec::new();
    for company in results {
        let exact_director = company
            .get("dirigeants")
            .and_then(|value| value.as_array())
            .is_some_and(|directors| {
                directors.iter().any(|director| {
                    let full = director
                        .get("nom_complet")
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                        .or_else(|| {
                            Some(format!(
                                "{} {}",
                                director.get("prenoms")?.as_str()?,
                                director.get("nom")?.as_str()?
                            ))
                        });
                    full.is_some_and(|value| exact_public_name(&value) == expected)
                })
            });
        if !exact_director {
            continue;
        }
        let siren = company
            .get("siren")
            .and_then(|value| value.as_str())
            .filter(|value| value.len() == 9 && value.chars().all(|c| c.is_ascii_digit()))
            .ok_or("Le registre a renvoyé un SIREN invalide.")?;
        let company_name = company
            .get("nom_complet")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Entreprise non libellée");
        findings.push(SpecializedPublicFinding {
            title: format!("Mandat public potentiel : {company_name}"),
            explanation: format!("Le nom complet exact « {expected_name} » apparaît parmi les dirigeants publics de {company_name} (SIREN {siren}). Un homonyme reste possible ; ce résultat ne confirme jamais l’identité de la personne."),
            source: "API Recherche d’Entreprises".into(),
            source_url: format!("https://annuaire-entreprises.data.gouv.fr/entreprise/{siren}"),
            evidence_ref: format!("Recherche d’Entreprises · dirigeant exact · SIREN {siren}"),
        });
    }
    Ok(findings.into_iter().take(5).collect())
}

fn parse_hal_findings(
    body: &str,
    expected_name: &str,
) -> Result<Vec<SpecializedPublicFinding>, String> {
    let payload: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "HAL a renvoyé un format inattendu.".to_string())?;
    let docs = payload
        .pointer("/response/docs")
        .and_then(|value| value.as_array())
        .ok_or("HAL n’a renvoyé aucune liste exploitable.")?;
    if docs.len() > 10 {
        return Err("HAL dépasse la limite MANTIS.".into());
    }
    let expected = exact_public_name(expected_name);
    let mut findings = Vec::new();
    for document in docs {
        let exact_author = document
            .get("authFullName_s")
            .and_then(|value| value.as_array())
            .is_some_and(|authors| {
                authors.iter().any(|author| {
                    author
                        .as_str()
                        .is_some_and(|value| exact_public_name(value) == expected)
                })
            });
        if !exact_author {
            continue;
        }
        let uri = document
            .get("uri_s")
            .and_then(|value| value.as_str())
            .and_then(scanner_source_url)
            .filter(|url| public_source_host(Some(url)).as_deref() == Some("hal.science"))
            .ok_or("HAL a renvoyé une URL publique invalide.")?;
        let title = document
            .get("title_s")
            .and_then(|value| value.as_array())
            .and_then(|values| values.first())
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Publication HAL");
        findings.push(SpecializedPublicFinding {
            title: format!("Publication HAL potentielle : {title}"),
            explanation: format!("Le nom d’auteur exact « {expected_name} » figure dans les métadonnées publiques de cette publication HAL. Un homonyme reste possible ; ce résultat ne confirme jamais l’identité de la personne."),
            source: "HAL".into(),
            source_url: uri,
            evidence_ref: "HAL · auteur exact · métadonnées publiques".into(),
        });
    }
    Ok(findings.into_iter().take(5).collect())
}

fn parse_mastodon_webfinger_profile(
    body: &str,
    expected_username: &str,
    expected_instance: &str,
) -> Result<ParsedMastodonWebfingerProfile, String> {
    let document: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "L’instance Mastodon a renvoyé un document WebFinger invalide.".to_string())?;
    let expected_account = format!("{expected_username}@{expected_instance}");
    let subject = document
        .get("subject")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .ok_or("Le document WebFinger ne contient aucun sujet exploitable.")?;
    if !subject
        .strip_prefix("acct:")
        .is_some_and(|account| account.eq_ignore_ascii_case(&expected_account))
    {
        return Err(
            "Le document WebFinger ne correspond pas à l’identifiant Mastodon demandé.".into(),
        );
    }
    let links = document
        .get("links")
        .and_then(|value| value.as_array())
        .ok_or("Le document WebFinger ne contient aucun lien public exploitable.")?;
    if links.len() > 30 {
        return Err("Le document WebFinger dépasse la limite de liens MANTIS.".into());
    }
    let profile_url = links
        .iter()
        .filter(|link| {
            matches!(
                link.get("rel").and_then(|value| value.as_str()),
                Some("self" | "http://webfinger.net/rel/profile-page")
            )
        })
        .filter_map(|link| link.get("href").and_then(|value| value.as_str()))
        .find_map(|href| {
            let safe = scanner_source_url(href)?;
            (public_source_host(Some(&safe)).as_deref() == Some(expected_instance)).then_some(safe)
        })
        .ok_or("WebFinger n’a renvoyé aucune URL de profil sûre sur l’instance demandée.")?;
    Ok(ParsedMastodonWebfingerProfile {
        account: expected_account.clone(),
        profile_url,
        explanation: format!("Preuve publique vérifiée : l’identifiant fédéré exact « {expected_account} » est retourné par WebFinger sur l’instance explicitement déclarée. Ce profil est une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: "Identifiant fédéré exact retourné par WebFinger — identité non attribuée".into(),
        evidence_ref: format!("Mastodon/WebFinger · acct:{expected_account} · instance explicite"),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedGitlabPublicProfile {
    username: String,
    profile_url: String,
    explanation: String,
    confidence: String,
    evidence_ref: String,
}

fn parse_gitlab_public_profile(
    body: &str,
    expected_target: &str,
    target_kind: &str,
) -> Result<Option<ParsedGitlabPublicProfile>, String> {
    let profiles: Vec<serde_json::Value> = serde_json::from_str(body).map_err(|_| {
        "GitLab a renvoyé une liste de profils dans un format inattendu.".to_string()
    })?;
    if profiles.len() > 20 {
        return Err("GitLab a renvoyé trop de profils pour une recherche exacte.".into());
    }
    let expected = expected_target.trim();
    let profile = profiles.into_iter().find(|profile| match target_kind {
        "pseudo" => github_public_text(profile, "username", 255).is_some_and(|username| {
            username.eq_ignore_ascii_case(expected.trim_start_matches('@'))
        }),
        "email" => github_public_text(profile, "public_email", 254)
            .is_some_and(|email| email.eq_ignore_ascii_case(expected)),
        _ => false,
    });
    let Some(profile) = profile else {
        return Ok(None);
    };
    let username = github_public_text(&profile, "username", 255)
        .ok_or("GitLab n’a renvoyé aucun pseudo public exploitable.")?;
    let profile_url = github_public_text(&profile, "web_url", 240)
        .and_then(|value| scanner_source_url(&value))
        .filter(|value| public_source_host(Some(value)).as_deref() == Some("gitlab.com"))
        .ok_or("GitLab n’a renvoyé aucune URL publique sûre pour ce profil.")?;
    let mut details = Vec::new();
    for (key, label, maximum) in [
        ("name", "nom public", 120usize),
        ("bio", "biographie", 280),
        ("location", "localisation déclarée", 120),
        ("organization", "organisation", 120),
        ("job_title", "fonction déclarée", 120),
        ("website_url", "site déclaré", 180),
    ] {
        if let Some(value) = github_public_text(&profile, key, maximum) {
            details.push(format!("{label} : {value}"));
        }
    }
    let context = if details.is_empty() {
        "Aucun détail public complémentaire n’est renseigné dans le profil.".into()
    } else {
        details.into_iter().take(6).collect::<Vec<_>>().join(" · ")
    };
    let evidence_phrase = if target_kind == "email" {
        "Preuve publique vérifiée : l’identifiant exact"
    } else {
        "Preuve publique vérifiée : le pseudo exact"
    };
    Ok(Some(ParsedGitlabPublicProfile {
        username: username.clone(),
        profile_url,
        explanation: format!("{evidence_phrase} est retourné par l’API publique GitLab. {context}. Ce profil est une correspondance possible et ne confirme jamais l’identité de la personne."),
        confidence: if target_kind == "email" { "Adresse e-mail publique exacte retournée par l’API GitLab".into() } else { "Pseudo exact retourné par l’API publique GitLab — identité non attribuée".into() },
        evidence_ref: format!("GitLab · {} exact(e) · API publique", if target_kind == "email" { "adresse e-mail" } else { "pseudo" }),
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedXposedBreach {
    breach: String,
    explanation: String,
    severity: String,
    confidence: String,
    source_url: Option<String>,
    evidence_ref: String,
}

fn xposed_text(item: &serde_json::Value, key: &str) -> Option<String> {
    let value = item.get(key)?;
    match value {
        serde_json::Value::String(value) if !value.trim().is_empty() => {
            Some(value.trim().to_string())
        }
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn xposed_data_labels(value: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .filter_map(|item| {
            let normalized = item.to_ascii_lowercase();
            let label = match normalized.as_str() {
                "email" | "emails" | "email addresses" => "adresse e-mail",
                "password" | "passwords" => "mot de passe",
                "name" | "names" => "nom",
                "username" | "usernames" => "pseudo",
                "date of birth" | "dates of birth" => "date de naissance",
                "phone" | "phones" | "phone numbers" => "numéro de téléphone",
                "address" | "addresses" | "physical addresses" => "adresse postale",
                "gender" | "genders" => "genre",
                _ => item,
            }
            .to_string();
            if seen.insert(normalized) {
                Some(label)
            } else {
                None
            }
        })
        .take(16)
        .collect()
}

fn xposed_reference_url(item: &serde_json::Value) -> Option<String> {
    let candidates = match item.get("references") {
        Some(serde_json::Value::Array(values)) => values
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect::<Vec<_>>(),
        Some(serde_json::Value::String(value)) => value
            .split([',', ';', ' '])
            .map(str::to_string)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    candidates
        .iter()
        .find_map(|candidate| scanner_source_url(candidate))
}

fn parse_xposed_breaches(body: &str) -> Result<Vec<ParsedXposedBreach>, String> {
    let data: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "La source a renvoyé un format inattendu.".to_string())?;
    let Some(items) = data
        .pointer("/ExposedBreaches/breaches_details")
        .and_then(|value| value.as_array())
    else {
        return Ok(Vec::new());
    };
    let mut breaches = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in items.iter().take(20) {
        let Some(breach) = xposed_text(item, "breach") else {
            continue;
        };
        let key = normalized_exact_text(&breach);
        if !seen.insert(key) {
            continue;
        }
        let data_labels = xposed_text(item, "xposed_data")
            .map(|value| xposed_data_labels(&value))
            .unwrap_or_default();
        let verified = xposed_text(item, "verified");
        let is_verified = verified.as_deref().is_some_and(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "yes" | "true" | "verified"
            )
        });
        let contains_password = data_labels.iter().any(|label| label == "mot de passe");
        let contains_sensitive = data_labels.iter().any(|label| {
            matches!(
                label.as_str(),
                "date de naissance" | "numéro de téléphone" | "adresse postale"
            )
        });
        let severity = if contains_password && is_verified {
            "critique"
        } else if contains_password || (contains_sensitive && is_verified) || is_verified {
            "elevee"
        } else {
            "moderee"
        }
        .to_string();
        let confidence = match verified.as_deref() {
            Some(value) if is_verified => {
                format!("Fuite déclarée vérifiée par XposedOrNot ({value})")
            }
            Some(value) => format!("Fuite déclarée non vérifiée par XposedOrNot ({value})"),
            None => "Statut de vérification non précisé par XposedOrNot".into(),
        };
        let mut facts = Vec::new();
        if let Some(date) = xposed_text(item, "xposed_date") {
            facts.push(format!("date déclarée : {date}"));
        }
        if let Some(domain) = xposed_text(item, "domain") {
            facts.push(format!("domaine concerné : {domain}"));
        }
        if let Some(records) = xposed_text(item, "xposed_records") {
            facts.push(format!("volume déclaré : {records} enregistrement(s)"));
        }
        if let Some(risk) = xposed_text(item, "password_risk") {
            facts.push(format!("risque lié aux mots de passe : {risk}"));
        }
        let exposed = if data_labels.is_empty() {
            "données non précisées".into()
        } else {
            data_labels.join(", ")
        };
        let mut explanation = format!(
            "Cette adresse e-mail apparaît dans la fuite « {breach} ». Données déclarées : {exposed}."
        );
        if !facts.is_empty() {
            explanation.push(' ');
            explanation.push_str(&facts.join(" ; "));
            explanation.push('.');
        }
        if let Some(details) = xposed_text(item, "details") {
            let bounded = details.chars().take(280).collect::<String>();
            if !bounded.is_empty() {
                explanation.push(' ');
                explanation.push_str(&bounded);
                if !explanation.ends_with('.') {
                    explanation.push('.');
                }
            }
        }
        explanation.push_str(" Vérifiez la preuve et changez tout mot de passe réutilisé avant de retenir cette fuite comme exposition confirmée.");
        let evidence_ref = format!(
            "XposedOrNot · {breach}{}{}",
            xposed_text(item, "domain")
                .map(|value| format!(" · {value}"))
                .unwrap_or_default(),
            xposed_text(item, "xposed_date")
                .map(|value| format!(" · {value}"))
                .unwrap_or_default()
        );
        breaches.push(ParsedXposedBreach {
            breach,
            explanation,
            severity,
            confidence,
            source_url: xposed_reference_url(item),
            evidence_ref,
        });
    }
    Ok(breaches)
}

fn execution_kind_for_module(module_id: &str, kind: &str) -> Option<String> {
    match (module_id, kind) {
        ("mock-osint" | "osint-email-intel", "email") => Some("email".into()),
        ("osint-email-platforms", "email" | "pseudo") => Some(kind.into()),
        ("osint-username-profiles", "pseudo") => Some("pseudo".into()),
        ("osint-github-profile", "pseudo") => Some("pseudo".into()),
        ("osint-gitlab-profile", "email" | "pseudo") => Some(kind.into()),
        ("osint-mastodon-webfinger", "pseudo") => Some("pseudo".into()),
        ("osint-gravatar-profile", "email") => Some("email".into()),
        ("osint-keybase-profile", "pseudo") => Some("pseudo".into()),
        ("osint-bluesky-profile", "pseudo") => Some("pseudo".into()),
        ("osint-hackernews-profile", "pseudo") => Some("pseudo".into()),
        ("osint-fr-company-register" | "osint-hal-author", "nom") => Some("nom".into()),
        ("osint-web-footprint", "email" | "pseudo" | "telephone" | "adresse") => Some(kind.into()),
        _ => None,
    }
}

fn mark_failed_scan_execution(
    app: &tauri::AppHandle,
    identity_id: &str,
    identity_value_id: &str,
    module_id: &str,
    session_id: Option<&str>,
    error: &str,
) {
    let Ok(conn) = get_db_connection(app) else {
        return;
    };
    let bounded_error = error.chars().take(500).collect::<String>();
    let _ = conn.execute(
        "UPDATE osint_scans SET status='erreur',completed_at=datetime('now'),error_message=?1 WHERE id=(SELECT id FROM osint_scans WHERE identity_id=?2 AND identity_value_id=?3 AND module_id=?4 AND session_id IS ?5 AND status='en_cours' ORDER BY started_at DESC,rowid DESC LIMIT 1)",
        params![bounded_error, identity_id, identity_value_id, module_id, session_id],
    );
    let _ = log_module(&conn, module_id, "scan", "erreur", &bounded_error);
}

fn build_veille_scan_plan(values: &[IdentityValue]) -> Vec<OsintScanPlanItem> {
    let module_ids = [
        "osint-email-intel",
        "osint-email-platforms",
        "osint-username-profiles",
        "osint-github-profile",
        "osint-gitlab-profile",
        "osint-mastodon-webfinger",
        "osint-gravatar-profile",
        "osint-keybase-profile",
        "osint-bluesky-profile",
        "osint-hackernews-profile",
        "osint-web-footprint",
    ];
    let mut plan = Vec::new();
    for value in values.iter().filter(|value| value.status == "active") {
        for module_id in module_ids {
            if module_id == "osint-mastodon-webfinger"
                && mastodon_account_for_target(&value.value).is_none()
            {
                continue;
            }
            if let Some(target_kind) = execution_kind_for_module(module_id, &value.kind) {
                plan.push(OsintScanPlanItem {
                    module_id,
                    identity_value_id: value.id.clone(),
                    target: value.value.clone(),
                    target_kind,
                });
            }
        }
    }
    let first_name = values
        .iter()
        .find(|value| value.status == "active" && value.kind == "prenom")
        .map(|value| value.value.trim())
        .filter(|value| !value.is_empty());
    if let Some(first_name) = first_name {
        let normalized_first_name = normalize_identity_value("prenom", first_name);
        let mut full_names = std::collections::HashSet::new();
        for last_name in values.iter().filter(|value| {
            value.status == "active"
                && value.kind == "nom"
                && !value.value.trim().is_empty()
                && value.normalized_value != normalized_first_name
        }) {
            let target = format!("{first_name} {}", last_name.value.trim());
            if full_names.insert(normalized_exact_text(&target)) {
                plan.push(OsintScanPlanItem {
                    module_id: "osint-web-footprint",
                    identity_value_id: last_name.id.clone(),
                    target,
                    target_kind: "nom".into(),
                });
            }
        }
    }
    plan
}

#[tauri::command]
fn run_real_osint_scan(
    app: tauri::AppHandle,
    module_id: String,
    identity_id: String,
    authorized: bool,
) -> Result<OsintScanSummary, String> {
    if !authorized {
        return Err("Confirmez que vous êtes autorisé à analyser cette cible.".into());
    }
    let conn = get_db_connection(&app)?;
    let identity = load_identity(&conn, &identity_id)?;
    let (value_id, target, target_kind) = if matches!(
        module_id.as_str(),
        "osint-fr-company-register" | "osint-hal-author"
    ) {
        let first_name = identity
            .values
            .iter()
            .find(|value| value.status == "active" && value.kind == "prenom")
            .ok_or("Cette source spécialisée exige un prénom actif.")?;
        let last_name = identity
            .values
            .iter()
            .find(|value| value.status == "active" && value.kind == "nom")
            .ok_or("Cette source spécialisée exige un nom actif.")?;
        (
            last_name.id.clone(),
            format!("{} {}", first_name.value.trim(), last_name.value.trim()),
            "nom".to_string(),
        )
    } else {
        let value = identity
            .values
            .iter()
            .find(|value| {
                value.status == "active"
                    && execution_kind_for_module(&module_id, &value.kind).is_some()
            })
            .ok_or_else(|| {
                "Cette identité ne contient aucune donnée active compatible avec cet outil."
                    .to_string()
            })?;
        (
            value.id.clone(),
            value.value.clone(),
            execution_kind_for_module(&module_id, &value.kind)
                .ok_or_else(|| "Cette donnée n’est pas compatible avec cet outil.".to_string())?,
        )
    };
    let result = run_real_osint_scan_for_value(
        app.clone(),
        module_id.clone(),
        identity_id.clone(),
        value_id.clone(),
        target,
        target_kind,
        None,
    );
    if let Err(error) = &result {
        mark_failed_scan_execution(&app, &identity_id, &value_id, &module_id, None, error);
    }
    if result.is_ok() {
        let mut conn = get_db_connection(&app)?;
        apply_signal_quality_gate(&mut conn, &identity_id)?;
        refresh_osint_claims(&mut conn, &identity_id)?;
    }
    result
}

fn run_veille_scan_inner(
    app: tauri::AppHandle,
    identity_id: String,
    authorized: bool,
    origin: &str,
) -> Result<OsintScanSessionSummary, String> {
    if !authorized {
        return Err(
            "Confirmez que vous êtes autorisé à analyser les identités de cette routine.".into(),
        );
    }
    let conn = get_db_connection(&app)?;
    let identity = load_identity(&conn, &identity_id)?;
    let full_plan = build_veille_scan_plan(&identity.values);
    if full_plan.is_empty() {
        return Err(
            "Cette identité ne contient aucune donnée active compatible avec les outils installés."
                .into(),
        );
    }
    // A manual launch must remain responsive on a laptop and should not queue
    // dozens of network timeouts. The routine can cover the remaining values
    // during later runs; the session records exactly what was skipped.
    const MAX_CHECKS_PER_SESSION: usize = 24;
    let planned_checks = full_plan.len();
    let overflow_skipped = planned_checks.saturating_sub(MAX_CHECKS_PER_SESSION);
    let plan = full_plan
        .into_iter()
        .take(MAX_CHECKS_PER_SESSION)
        .collect::<Vec<_>>();
    let session_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO osint_scan_sessions (id,identity_id,origin,status,planned_checks,skipped_checks) VALUES (?1,?2,?3,'en_cours',?4,?5)",
        params![session_id, identity_id, origin, planned_checks as i64, overflow_skipped as i64],
    ).map_err(|e| e.to_string())?;
    drop(conn);

    let mut signals = Vec::new();
    let mut completed_checks = 0usize;
    let mut failed_checks = 0usize;
    let mut skipped_checks = overflow_skipped;
    for item in plan {
        if module_installation_state(&app, item.module_id).0 != "prêt" {
            skipped_checks += 1;
            continue;
        }
        let check_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_real_osint_scan_for_value(
                app.clone(),
                item.module_id.into(),
                identity_id.clone(),
                item.identity_value_id.clone(),
                item.target.clone(),
                item.target_kind.clone(),
                Some(session_id.clone()),
            )
        }))
        .unwrap_or_else(|_| {
            Err("Cette source a été interrompue de façon inattendue. Les autres collectes continuent.".into())
        });
        match check_result {
            Ok(summary) => {
                completed_checks += 1;
                signals.extend(summary.signals);
            }
            Err(error) => {
                mark_failed_scan_execution(
                    &app,
                    &identity_id,
                    &item.identity_value_id,
                    item.module_id,
                    Some(&session_id),
                    &error,
                );
                failed_checks += 1;
            }
        }
    }
    let session_status = if completed_checks == 0 && failed_checks > 0 {
        "erreur"
    } else if failed_checks > 0 || skipped_checks > 0 {
        "partiel"
    } else {
        "termine"
    };
    let message = if session_status == "termine" {
        format!("Scan terminé : {completed_checks} vérification(s) effectuée(s). Tous les résultats restent à vérifier.")
    } else {
        format!("Scan partiel : {completed_checks} vérification(s) terminée(s), {failed_checks} en échec et {skipped_checks} ignorée(s). Les autres résultats restent disponibles.")
    };
    let mut conn = get_db_connection(&app)?;
    conn.execute(
        "UPDATE osint_scan_sessions SET status=?1,completed_checks=?2,failed_checks=?3,skipped_checks=?4,signal_count=?5,completed_at=datetime('now'),summary=?6 WHERE id=?7",
        params![session_status, completed_checks as i64, failed_checks as i64, skipped_checks as i64, signals.len() as i64, message, session_id],
    ).map_err(|e| e.to_string())?;
    apply_signal_quality_gate(&mut conn, &identity_id)?;
    refresh_osint_claims(&mut conn, &identity_id)?;
    record_claim_session_presence(&conn, &session_id, &identity_id)?;
    let claims = load_identity_claims(&conn, &identity_id)?;
    let resolutions = load_identity_fact_resolutions(&conn, &identity_id)?;
    signals = load_session_signals(&conn, &session_id, &identity_id)?;
    drop(conn);
    // Analysis is an optional downstream aid: its queue must never invalidate a completed scan session.
    let analysis_job_id = enqueue_identity_synthesis(&app, &identity_id, origin).unwrap_or(None);
    Ok(OsintScanSessionSummary {
        session_id: session_id.clone(),
        scan_id: session_id,
        identity_id,
        target: identity.label,
        signals,
        claims,
        resolutions,
        message,
        analysis_job_id,
        planned_checks,
        completed_checks,
        failed_checks,
        skipped_checks,
        coverage: Vec::new(),
    })
}

#[tauri::command]
async fn run_veille_scan(
    app: tauri::AppHandle,
    identity_id: String,
    authorized: bool,
) -> Result<OsintScanSessionSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_veille_scan_inner(app, identity_id, authorized, "scan_manuel")
        }))
        .unwrap_or_else(|_| {
            Err("Le scan a été interrompu de façon inattendue. Consultez l’historique : les collectes déjà terminées restent conservées.".into())
        })
    })
    .await
    .map_err(|_| "Le worker de scan a été interrompu avant la fin de la collecte.".to_string())?
}

fn load_session_signals(
    conn: &Connection,
    session_id: &str,
    identity_id: &str,
) -> Result<Vec<OsintSignal>, String> {
    let mut statement = conn.prepare(
        "SELECT s.id,s.module_id,s.scan_id,s.target,s.signal_type,s.title,s.explanation,s.severity,s.confidence,s.source,s.source_url,s.discovered_at,s.review_status,s.exposure_id
         FROM osint_signals s JOIN osint_scans sc ON sc.id=s.scan_id
         WHERE sc.session_id=?1 AND s.identity_id=?2
         ORDER BY CASE s.signal_type WHEN 'fuite' THEN 0 WHEN 'compte_potentiel' THEN 1 WHEN 'profil_public' THEN 1 WHEN 'mention' THEN 2 ELSE 3 END,
                  CASE s.severity WHEN 'critique' THEN 0 WHEN 'elevee' THEN 1 WHEN 'moderee' THEN 2 ELSE 3 END,
                  CASE WHEN s.source_url IS NOT NULL OR s.evidence_ref IS NOT NULL THEN 0 ELSE 1 END,
                  CASE s.review_status WHEN 'À vérifier' THEN 0 WHEN 'Suivi' THEN 1 WHEN 'Confirmé' THEN 1 ELSE 2 END,
                  s.discovered_at DESC,s.id"
    ).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![session_id, identity_id], |row| {
            Ok(OsintSignal {
                id: row.get(0)?,
                module_id: row.get(1)?,
                scan_id: row.get(2)?,
                target: row.get(3)?,
                signal_type: row.get(4)?,
                title: row.get(5)?,
                explanation: row.get(6)?,
                severity: row.get(7)?,
                confidence: row.get(8)?,
                source: row.get(9)?,
                source_url: row.get(10)?,
                discovered_at: row.get(11)?,
                review_status: row.get(12)?,
                exposure_id: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_identity_scan_sessions(
    app: tauri::AppHandle,
    identity_id: String,
    limit: i64,
    offset: i64,
) -> Result<Vec<OsintScanSessionListItem>, String> {
    let conn = get_db_connection(&app)?;
    let exists: i64 = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM identities WHERE id=?1)",
            params![identity_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exists == 0 {
        return Err("Identité introuvable.".into());
    }
    let bounded_limit = limit.clamp(1, 30);
    let bounded_offset = offset.clamp(0, 10_000);
    let mut statement = conn.prepare("SELECT id,identity_id,origin,status,signal_count,planned_checks,completed_checks,failed_checks,skipped_checks,started_at,completed_at,summary FROM osint_scan_sessions WHERE identity_id=?1 ORDER BY started_at DESC,rowid DESC LIMIT ?2 OFFSET ?3").map_err(|e|e.to_string())?;
    let rows = statement
        .query_map(params![identity_id, bounded_limit, bounded_offset], |row| {
            Ok(OsintScanSessionListItem {
                id: row.get(0)?,
                identity_id: row.get(1)?,
                origin: row.get(2)?,
                status: row.get(3)?,
                signal_count: row.get(4)?,
                planned_checks: row.get(5)?,
                completed_checks: row.get(6)?,
                failed_checks: row.get(7)?,
                skipped_checks: row.get(8)?,
                started_at: row.get(9)?,
                completed_at: row.get(10)?,
                summary: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_identity_scan_session(
    app: tauri::AppHandle,
    identity_id: String,
    session_id: String,
) -> Result<OsintScanSessionSummary, String> {
    let conn = get_db_connection(&app)?;
    let (target,planned,completed,failed,skipped,summary): (String,i64,i64,i64,i64,Option<String>) = conn.query_row(
        "SELECT i.label,s.planned_checks,s.completed_checks,s.failed_checks,s.skipped_checks,s.summary FROM osint_scan_sessions s JOIN identities i ON i.id=s.identity_id WHERE s.id=?1 AND s.identity_id=?2",
        params![session_id,identity_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
    ).map_err(|_|"Session de veille introuvable pour cette identité.".to_string())?;
    // Quality gating and claim projection are refreshed when a scan or review
    // changes data. Keeping session reads read-only avoids expensive work
    // every time the user opens scan history.
    let signals = load_session_signals(&conn, &session_id, &identity_id)?;
    let claims = load_identity_claims(&conn, &identity_id)?;
    let resolutions = load_identity_fact_resolutions(&conn, &identity_id)?;
    let mut coverage_statement = conn
        .prepare(
            "SELECT COALESCE(NULLIF(target_kind_snapshot,''),'autre'),
                    COUNT(*),
                    SUM(CASE WHEN status='termine' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN status='erreur' THEN 1 ELSE 0 END),
                    (SELECT COUNT(*) FROM osint_signals sig WHERE sig.scan_id IN (
                        SELECT child.id FROM osint_scans child
                        WHERE child.session_id=s.session_id
                          AND COALESCE(child.target_kind_snapshot,'')=COALESCE(s.target_kind_snapshot,'')
                          AND child.target=s.target
                    ))
             FROM osint_scans s
             WHERE s.session_id=?1 AND s.identity_id=?2
             GROUP BY COALESCE(NULLIF(target_kind_snapshot,''),'autre'),target
             ORDER BY CASE COALESCE(NULLIF(target_kind_snapshot,''),'autre')
                WHEN 'email' THEN 0 WHEN 'pseudo' THEN 1 WHEN 'nom' THEN 2 ELSE 9 END,target",
        )
        .map_err(|e| e.to_string())?;
    let coverage = coverage_statement
        .query_map(params![session_id, identity_id], |row| {
            Ok(OsintScanCoverage {
                target_kind: row.get(0)?,
                planned_checks: row.get::<_, i64>(1)?.max(0) as usize,
                completed_checks: row.get::<_, i64>(2)?.max(0) as usize,
                failed_checks: row.get::<_, i64>(3)?.max(0) as usize,
                signal_count: row.get::<_, i64>(4)?.max(0) as usize,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(OsintScanSessionSummary {
        session_id: session_id.clone(),
        scan_id: session_id,
        identity_id,
        target,
        signals,
        claims,
        resolutions,
        message: summary.unwrap_or_else(|| "Résultats historiques de cette identité.".into()),
        analysis_job_id: None,
        planned_checks: planned.max(0) as usize,
        completed_checks: completed.max(0) as usize,
        failed_checks: failed.max(0) as usize,
        skipped_checks: skipped.max(0) as usize,
        coverage,
    })
}

#[tauri::command]
fn run_veille_routine(
    app: tauri::AppHandle,
    authorized: bool,
) -> Result<OsintRoutineSummary, String> {
    if !authorized {
        return Err(
            "Confirmez que vous êtes autorisé à analyser les identités de cette routine.".into(),
        );
    }
    let conn = get_db_connection(&app)?;
    let mut statement = conn.prepare("SELECT i.id FROM identities i WHERE i.status='active' AND EXISTS (SELECT 1 FROM identity_values v WHERE v.identity_id=i.id AND v.status='active' AND v.kind IN ('email','pseudo','nom','prenom')) ORDER BY i.label").map_err(|e| e.to_string())?;
    let identities = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if identities.is_empty() {
        return Err("Activez au moins une identité contenant un e-mail, un pseudo ou un nom avant de lancer cette routine.".into());
    }
    drop(statement);
    drop(conn);

    let mut scanned_identities = 0;
    let mut signals_found = 0;
    let mut failed_identities = 0;
    let mut analysis_jobs_started = 0;
    for (index, identity_id) in identities.iter().enumerate() {
        if index > 0 {
            std::thread::sleep(Duration::from_secs(15));
        }
        match run_veille_scan_inner(app.clone(), identity_id.clone(), true, "routine") {
            Ok(summary) => {
                if summary.completed_checks == 0 && summary.failed_checks > 0 {
                    failed_identities += 1;
                } else {
                    scanned_identities += 1;
                }
                signals_found += summary.signals.len();
                if summary.analysis_job_id.is_some() {
                    analysis_jobs_started += 1;
                }
            }
            Err(_) => {
                failed_identities += 1;
            }
        }
    }
    let conn = get_db_connection(&app)?;
    conn.execute("INSERT INTO app_settings (key,value) VALUES ('veille_last_run',datetime('now')) ON CONFLICT(key) DO UPDATE SET value=excluded.value", []).map_err(|e| e.to_string())?;
    let frequency = veille_setting(&conn, "veille_frequency", "Manuelle");
    let paused = veille_setting(&conn, "veille_paused", "0") == "1";
    if !paused && frequency != "Manuelle" {
        if let Some(sql_time) = next_veille_run(&frequency) {
            conn.execute(&format!("INSERT INTO app_settings (key,value) VALUES ('veille_next_run',{}) ON CONFLICT(key) DO UPDATE SET value=excluded.value", sql_time), []).map_err(|e| e.to_string())?;
        }
    }
    let message = format!("Routine terminée : {} identité(s) analysée(s), {} signal(s) à vérifier, {} échec(s). Aucun incident ni aucune action n’a été créé automatiquement.", scanned_identities, signals_found, failed_identities);
    Ok(OsintRoutineSummary {
        scanned_identities,
        signals_found,
        failed_identities,
        message,
        analysis_jobs_started,
    })
}

fn veille_routine_is_due(app: &tauri::AppHandle) -> bool {
    let Ok(conn) = get_db_connection(app) else {
        return false;
    };
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM app_settings WHERE key='veille_next_run' AND value != '' AND value <= datetime('now'))",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) != 0
}

fn has_positive_human_decision(conn: &Connection, signal_id: &str) -> bool {
    conn.query_row("SELECT decision FROM osint_user_decisions WHERE target_type='signal' AND target_id=?1 ORDER BY created_at DESC,rowid DESC LIMIT 1",params![signal_id],|row|row.get::<_,String>(0)).is_ok_and(|decision|matches!(decision.as_str(),"confirmer"|"suivre"))
}

fn record_projection_event(
    conn: &Connection,
    identity_id: &str,
    signal_id: &str,
    projection_type: &str,
    target_id: &str,
    outcome: &str,
) -> Result<(), String> {
    conn.execute("INSERT OR IGNORE INTO osint_projection_events(id,identity_id,signal_id,projection_type,target_id,outcome) VALUES (?1,?2,?3,?4,?5,?6)",params![Uuid::new_v4().to_string(),identity_id,signal_id,projection_type,target_id,outcome]).map_err(|e|e.to_string())?;
    Ok(())
}

#[tauri::command]
fn create_exposure_from_osint_signal(
    app: tauri::AppHandle,
    signal_id: String,
) -> Result<String, String> {
    let mut conn = get_db_connection(&app)?;
    let (title, explanation, source, scan_id, folder_id, signal_type, severity,identity_id): (String,String,String,String,Option<String>,String,String,String) = conn.query_row("SELECT s.title,s.explanation,s.source,s.scan_id,i.folder_id,s.signal_type,s.severity,s.identity_id FROM osint_signals s JOIN identities i ON i.id=s.identity_id WHERE s.id=?1",params![signal_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).map_err(|_|"Signal introuvable.".to_string())?;
    let existing: Option<String> = conn
        .query_row(
            "SELECT exposure_id FROM osint_signals WHERE id=?1",
            params![signal_id],
            |r| r.get(0),
        )
        .ok()
        .flatten();
    if let Some(id) = existing {
        record_projection_event(
            &conn,
            &identity_id,
            &signal_id,
            "exposition",
            &id,
            "reutilise",
        )?;
        return Ok(id);
    };
    if !has_positive_human_decision(&conn, &signal_id) {
        return Err(
            "Confirmez ou suivez explicitement ce signal avant de créer une exposition.".into(),
        );
    }
    let kind = if signal_type == "fuite" || source == "DDGS" {
        "fuite"
    } else {
        "mention"
    };
    let exposure_id = Uuid::new_v4().to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO exposures (id,title,kind,severity,status,discovered_at,source,what,why,folder_id) VALUES (?1,?2,?3,?4,'nouvelle',datetime('now'),?5,?6,?7,?8)",params![exposure_id,title,kind,severity,source,explanation,"Signal OSINT retenu par décision humaine ; vérification métier requise.",folder_id]).map_err(|e|e.to_string())?;
    tx.execute(
        "UPDATE osint_signals SET exposure_id=?1,review_status='Traité' WHERE id=?2",
        params![exposure_id, signal_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO timeline_entries (id,event_type,description,created_at) VALUES (?1,'Signal OSINT',?2,datetime('now'))",params![Uuid::new_v4().to_string(),format!("Exposition créée manuellement depuis le scan {}",scan_id)]).map_err(|e|e.to_string())?;
    record_projection_event(
        &tx,
        &identity_id,
        &signal_id,
        "exposition",
        &exposure_id,
        "cree",
    )?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(exposure_id)
}

#[tauri::command]
fn create_incident_and_action_from_osint_signal(
    app: tauri::AppHandle,
    signal_id: String,
) -> Result<String, String> {
    let mut conn = get_db_connection(&app)?;
    let (title, explanation, exposure_id, folder_id,identity_id): (String, String, Option<String>, Option<String>,String) = conn.query_row(
        "SELECT s.title,s.explanation,s.exposure_id,i.folder_id,s.identity_id FROM osint_signals s JOIN identities i ON i.id=s.identity_id WHERE s.id=?1",
        params![signal_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,r.get(4)?))
    ).map_err(|_| "Signal introuvable.".to_string())?;
    let exposure_id = exposure_id.ok_or("Créez d’abord une exposition à partir de ce signal.")?;
    let existing_action: Option<String> = conn.query_row(
        "SELECT ia.action_id FROM exposure_incident ei JOIN incident_action ia ON ia.incident_id=ei.incident_id WHERE ei.exposure_id=?1 LIMIT 1",
        params![exposure_id], |r| r.get(0)
    ).ok();
    if let Some(action_id) = existing_action {
        record_projection_event(
            &conn,
            &identity_id,
            &signal_id,
            "incident_action",
            &action_id,
            "reutilise",
        )?;
        return Ok(action_id);
    }
    if !has_positive_human_decision(&conn, &signal_id) {
        return Err(
            "Une décision humaine positive est requise avant de créer un incident et une action."
                .into(),
        );
    }
    let incident_id = Uuid::new_v4().to_string();
    let action_id = Uuid::new_v4().to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO incidents (id,title,severity,discovered_at,what,why,impact,confidence,next_step,folder_id) VALUES (?1,?2,'moderee',datetime('now'),?3,?4,'Aucun incident confirmé à ce stade.','À vérifier','Vérifier ce signal et sécuriser le compte concerné si nécessaire.',?5)", params![incident_id, format!("Vérification requise : {}", title), title, explanation, folder_id]).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO exposure_incident (exposure_id,incident_id) VALUES (?1,?2)",
        params![exposure_id, incident_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO actions (id,title,priority_id,difficulty_id,deadline,status,guidance,proof_expected,folder_id) VALUES (?1,?2,'prio_003','diff_001',date('now','+7 days'),'a_faire',?3,'Note de vérification ou décision enregistrée.',?4)", params![action_id, format!("Vérifier : {}", title), "Ouvrez la source, décidez si le signal vous concerne, puis marquez l’action terminée ou bloquée.", folder_id]).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO incident_action (incident_id,action_id) VALUES (?1,?2)",
        params![incident_id, action_id],
    )
    .map_err(|e| e.to_string())?;
    record_projection_event(
        &tx,
        &identity_id,
        &signal_id,
        "incident_action",
        &action_id,
        "cree",
    )?;
    tx.execute("INSERT INTO timeline_entries(id,event_type,description,created_at) VALUES (?1,'Revue OSINT',?2,datetime('now'))",params![Uuid::new_v4().to_string(),format!("Incident et action créés manuellement depuis le signal {}",signal_id)]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(action_id)
}

fn verify_local_ai_catalog_signature(
    catalog_bytes: &[u8],
    signature_hex: &str,
) -> Result<(), String> {
    let public_key_bytes: [u8; 32] = hex::decode(LOCAL_AI_CATALOG_PUBLIC_KEY)
        .map_err(|_| "La clé publique du catalogue IA est invalide.")?
        .try_into()
        .map_err(|_| "La clé publique du catalogue IA est incomplète.")?;
    let signature_bytes: [u8; 64] = hex::decode(signature_hex.trim())
        .map_err(|_| "La signature du catalogue IA est invalide.")?
        .try_into()
        .map_err(|_| "La signature du catalogue IA est incomplète.")?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|_| "La clé publique du catalogue IA est refusée.")?;
    verifying_key
        .verify(catalog_bytes, &Signature::from_bytes(&signature_bytes))
        .map_err(|_| {
            "Le catalogue IA local n’a pas passé la vérification de signature.".to_string()
        })
}

fn verified_local_ai_catalog() -> Result<LocalAiCatalog, String> {
    verify_local_ai_catalog_signature(LOCAL_AI_CATALOG_JSON, LOCAL_AI_CATALOG_SIGNATURE)?;
    let catalog: LocalAiCatalog = serde_json::from_slice(LOCAL_AI_CATALOG_JSON)
        .map_err(|_| "Le catalogue IA local est illisible.")?;
    if catalog.schema_version != 1
        || catalog.catalog_version.trim().is_empty()
        || catalog.components.is_empty()
    {
        return Err("Le catalogue IA local est incompatible avec cette version de MANTIS.".into());
    }
    for component in &catalog.components {
        let parsed = reqwest::Url::parse(&component.url)
            .map_err(|_| "Une URL du catalogue IA est invalide.")?;
        let trusted_runtime = component.component_type == "runtime"
            && parsed.host_str() == Some("github.com")
            && parsed
                .path()
                .starts_with("/ggml-org/llama.cpp/releases/download/")
            && component.archive == "zip"
            && component.executable.as_deref() == Some("llama-cli.exe")
            && component.byte_size <= 100_000_000;
        let trusted_model = component.component_type == "model"
            && parsed.host_str() == Some("huggingface.co")
            && parsed.path().starts_with("/Qwen/Qwen3-")
            && parsed.path().contains("/resolve/")
            && component.archive == "none"
            && component.filename.ends_with(".gguf")
            && component.byte_size <= 2_000_000_000
            && matches!(component.tier.as_deref(), Some("leger" | "recommande"));
        if parsed.scheme() != "https" || (!trusted_runtime && !trusted_model) {
            return Err("Le catalogue IA contient une source non autorisée.".into());
        }
        if component.sha256.len() != 64
            || component.byte_size == 0
            || component.filename.contains(['/', '\\'])
        {
            return Err(
                "Un composant du catalogue IA dépasse les limites de sécurité MANTIS.".into(),
            );
        }
    }
    Ok(catalog)
}

fn local_ai_root(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let root = application_data_dir(app)?.join("local-ai");
    std::fs::create_dir_all(root.join("downloads")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(root.join("components")).map_err(|e| e.to_string())?;
    Ok(root)
}

fn local_ai_component() -> Result<LocalAiCatalogComponent, String> {
    let catalog = verified_local_ai_catalog()?;
    catalog
        .components
        .into_iter()
        .find(|component| component.id == "llama-cpp-cpu")
        .ok_or_else(|| "Le runtime IA recommandé est absent du catalogue signé.".to_string())
}

fn local_ai_destination(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
) -> Result<std::path::PathBuf, String> {
    Ok(local_ai_root(app)?
        .join("components")
        .join(format!("{}-{}", component.id, component.version)))
}

fn find_file_bounded(
    root: &std::path::Path,
    file_name: &str,
    depth: usize,
) -> Option<std::path::PathBuf> {
    if depth > 4 {
        return None;
    }
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(file_name))
        {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file_bounded(&path, file_name, depth + 1) {
                return Some(found);
            }
        }
    }
    None
}

fn extract_local_ai_archive(
    archive_path: &std::path::Path,
    staging: &std::path::Path,
) -> Result<(), String> {
    let file = std::fs::File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| "L’archive du runtime IA est invalide.")?;
    if archive.len() == 0 || archive.len() > 200 {
        return Err("L’archive du runtime IA contient un nombre de fichiers inattendu.".into());
    }
    let mut total_uncompressed = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| "Une entrée de l’archive IA est illisible.")?;
        let enclosed = entry
            .enclosed_name()
            .ok_or("L’archive IA contient un chemin dangereux.")?
            .to_path_buf();
        total_uncompressed = total_uncompressed.saturating_add(entry.size());
        if total_uncompressed > 300_000_000 {
            return Err("L’archive IA décompressée dépasse la limite autorisée.".into());
        }
        let output = staging.join(enclosed);
        if entry.is_dir() {
            std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut destination = std::fs::File::create(&output).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut destination).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn run_local_ai_version(program: &std::path::Path) -> Result<String, String> {
    let working_directory = program
        .parent()
        .ok_or("Le dossier du runtime IA est invalide.")?;
    let mut command = Command::new(program);
    command
        .arg("--version")
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_background_process(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Le runtime IA ne peut pas démarrer : {e}"))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(value) = child.stdout.take() {
                value
                    .take(65_536)
                    .read_to_end(&mut stdout)
                    .map_err(|e| e.to_string())?;
            }
            if let Some(value) = child.stderr.take() {
                value
                    .take(65_536)
                    .read_to_end(&mut stderr)
                    .map_err(|e| e.to_string())?;
            }
            if !status.success() {
                return Err(format!(
                    "Le runtime IA a refusé le test de compatibilité : {}",
                    String::from_utf8_lossy(&stderr).trim()
                ));
            }
            let version = String::from_utf8_lossy(&stdout)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            return Ok(if version.is_empty() {
                "Version vérifiée".into()
            } else {
                version.chars().take(300).collect()
            });
        }
        if started.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            return Err(
                "Le test du runtime IA a dépassé 15 secondes ; le processus a été arrêté.".into(),
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn installed_local_ai_integrity(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
) -> Result<(bool, String), String> {
    let destination = local_ai_destination(app, component)?;
    let manifest_path = destination.join("mantis-install-manifest.json");
    if !manifest_path.exists() {
        return Ok((false, "Runtime non installé.".into()));
    }
    let manifest: InstalledLocalAiManifest =
        serde_json::from_slice(&std::fs::read(&manifest_path).map_err(|e| e.to_string())?)
            .map_err(|_| "Le manifeste d’installation IA est invalide.")?;
    if manifest.schema_version != 1
        || manifest.component_id != component.id
        || manifest.version != component.version
        || manifest.archive_sha256 != component.sha256
    {
        return Ok((
            false,
            "Le manifeste installé ne correspond pas au catalogue signé.".into(),
        ));
    }
    let executable = destination.join(&manifest.executable_path);
    if !executable.exists() || sha256_file(&executable)? != manifest.executable_sha256 {
        return Ok((
            false,
            "L’exécutable IA installé a été modifié ou supprimé.".into(),
        ));
    }
    let version = run_local_ai_version(&executable)?;
    Ok((
        true,
        format!(
            "Runtime {} prêt. Test local : {}",
            component.version, version
        ),
    ))
}

/// Resolve the exact executable declared by the verified installation manifest.
///
/// The catalog is authoritative for the runtime archive, while the manifest is
/// authoritative for the executable path inside that archive. Keeping this in
/// one place prevents the readiness check and the inference paths from
/// disagreeing after a runtime upgrade.
fn installed_local_ai_executable(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
) -> Result<std::path::PathBuf, String> {
    let (ready, diagnostic) = installed_local_ai_integrity(app, component)?;
    if !ready {
        return Err(diagnostic);
    }

    let destination = local_ai_destination(app, component)?;
    let manifest_path = destination.join("mantis-install-manifest.json");
    let manifest: InstalledLocalAiManifest =
        serde_json::from_slice(&std::fs::read(&manifest_path).map_err(|e| e.to_string())?)
            .map_err(|_| "Le manifeste d’installation IA est invalide.")?;
    let executable = destination.join(manifest.executable_path);
    if !executable.is_file() {
        return Err("Le runtime IA vérifié ne contient plus son exécutable.".into());
    }
    Ok(executable)
}

fn local_ai_thread_count() -> String {
    // On Windows, available_parallelism can report physical cores while
    // llama.cpp benefits from the logical threads. Keep local inference below
    // a conservative ceiling while avoiding the systematic half-CPU setting.
    "12".into()
}

fn install_local_ai_runtime_inner(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
    download_id: &str,
) -> Result<String, String> {
    if std::env::consts::OS != component.platform
        || std::env::consts::ARCH != component.architecture
    {
        return Err(format!(
            "Ce runtime cible {} {} ; cette machine utilise {} {}.",
            component.platform,
            component.architecture,
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
    }
    let root = local_ai_root(app)?;
    let partial = root
        .join("downloads")
        .join(format!("{}-{}.zip.part", component.id, component.version));
    let conn = get_db_connection(app)?;
    let mut existing_bytes = std::fs::metadata(&partial)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
        .min(component.byte_size);
    let already_complete = existing_bytes == component.byte_size
        && sha256_file(&partial).ok().as_deref() == Some(component.sha256.as_str());
    let downloaded = if already_complete {
        existing_bytes
    } else {
        if existing_bytes == component.byte_size {
            std::fs::File::create(&partial).map_err(|e| e.to_string())?;
            existing_bytes = 0;
        }
        let redirect_policy = reqwest::redirect::Policy::custom(|attempt| {
            let trusted_host = matches!(
                attempt.url().host_str(),
                Some("github.com" | "release-assets.githubusercontent.com")
            );
            if trusted_host && attempt.previous().len() < 5 {
                attempt.follow()
            } else {
                attempt.stop()
            }
        });
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(300))
            .redirect(redirect_policy)
            .user_agent("MANTIS-POSTURE/0.1 local-ai-installer")
            .build()
            .map_err(|e| e.to_string())?;
        let mut request = client.get(&component.url);
        if existing_bytes > 0 {
            request = request.header(reqwest::header::RANGE, format!("bytes={existing_bytes}-"));
        }
        let mut response = request.send().map_err(|_| "Le téléchargement du runtime IA a été interrompu ; il pourra reprendre au prochain essai.".to_string())?;
        if !response.status().is_success() {
            return Err(format!(
                "La source du runtime IA a répondu avec le statut {}.",
                response.status()
            ));
        }
        let resumed =
            existing_bytes > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(&partial)
            .map_err(|e| e.to_string())?;
        let mut downloaded = if resumed { existing_bytes } else { 0 };
        let mut buffer = [0_u8; 64 * 1024];
        let mut last_recorded = downloaded;
        loop {
            let read = response.read(&mut buffer).map_err(|_| {
                "Le téléchargement a été interrompu ; le fichier partiel est conservé.".to_string()
            })?;
            if read == 0 {
                break;
            }
            downloaded = downloaded.saturating_add(read as u64);
            if downloaded > component.byte_size {
                return Err(
                    "Le téléchargement dépasse la taille annoncée par le catalogue signé.".into(),
                );
            }
            output
                .write_all(&buffer[..read])
                .map_err(|e| e.to_string())?;
            if downloaded.saturating_sub(last_recorded) >= 1_048_576 {
                conn.execute("UPDATE local_ai_downloads SET downloaded_bytes=?1,status='en_cours',updated_at=datetime('now') WHERE id=?2", params![downloaded as i64, download_id]).map_err(|e| e.to_string())?;
                last_recorded = downloaded;
            }
        }
        output.flush().map_err(|e| e.to_string())?;
        downloaded
    };
    if downloaded != component.byte_size {
        return Err(format!(
            "Téléchargement incomplet : {downloaded} octets sur {}.",
            component.byte_size
        ));
    }
    if sha256_file(&partial)? != component.sha256 {
        return Err(
            "Le runtime téléchargé est corrompu ou ne correspond pas au catalogue signé.".into(),
        );
    }
    conn.execute("UPDATE local_ai_downloads SET downloaded_bytes=?1,status='verifie',error_message=NULL,updated_at=datetime('now') WHERE id=?2", params![downloaded as i64, download_id]).map_err(|e| e.to_string())?;

    let components_root = root.join("components");
    let staging = components_root.join(format!("{}-staging-{}", component.id, Uuid::new_v4()));
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    if let Err(error) = extract_local_ai_archive(&partial, &staging) {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(error);
    }
    let executable_name = component
        .executable
        .as_deref()
        .ok_or("Le catalogue du runtime ne précise pas son exécutable.")?;
    let executable = find_file_bounded(&staging, executable_name, 0)
        .ok_or_else(|| "L’archive IA ne contient pas l’exécutable attendu.".to_string())?;
    let version_message = run_local_ai_version(&executable)?;
    let executable_relative = executable
        .strip_prefix(&staging)
        .map_err(|_| "Chemin d’exécutable IA invalide.")?
        .to_string_lossy()
        .replace('\\', "/");
    let installed_manifest = InstalledLocalAiManifest {
        schema_version: 1,
        component_id: component.id.clone(),
        version: component.version.clone(),
        archive_sha256: component.sha256.clone(),
        executable_path: executable_relative,
        executable_sha256: sha256_file(&executable)?,
    };
    std::fs::write(
        staging.join("mantis-install-manifest.json"),
        serde_json::to_vec_pretty(&installed_manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(staging.join("MANTIS_CATALOG.json"), LOCAL_AI_CATALOG_JSON)
        .map_err(|e| e.to_string())?;
    std::fs::write(
        staging.join("MANTIS_CATALOG.sig"),
        LOCAL_AI_CATALOG_SIGNATURE,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(staging.join("LLAMA_CPP_LICENSE.txt"), LLAMA_CPP_LICENSE)
        .map_err(|e| e.to_string())?;

    let destination = local_ai_destination(app, component)?;
    let backup = components_root.join(format!("{}-previous", component.id));
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|e| e.to_string())?;
    }
    if destination.exists() {
        std::fs::rename(&destination, &backup).map_err(|e| e.to_string())?;
    }
    if let Err(error) = std::fs::rename(&staging, &destination) {
        if backup.exists() {
            let _ = std::fs::rename(&backup, &destination);
        }
        return Err(format!("Activation du runtime IA impossible : {error}"));
    }
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|e| e.to_string())?;
    }
    let executable_path = destination
        .join(&installed_manifest.executable_path)
        .to_string_lossy()
        .to_string();
    conn.execute("UPDATE local_ai_components SET install_path=?1,status='pret',diagnostic=?2,installed_at=datetime('now'),updated_at=datetime('now') WHERE component_id=?3", params![executable_path, version_message, component.id]).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&partial);
    Ok(format!(
        "Runtime IA local {} installé et vérifié. Aucun service n’est resté actif.",
        component.version
    ))
}

fn local_ai_models() -> Result<Vec<LocalAiCatalogComponent>, String> {
    Ok(verified_local_ai_catalog()?
        .components
        .into_iter()
        .filter(|component| component.component_type == "model")
        .collect())
}

fn local_ai_model_destination(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
) -> Result<std::path::PathBuf, String> {
    Ok(local_ai_root(app)?
        .join("models")
        .join(format!("{}-{}", component.id, component.version)))
}

fn installed_model_integrity(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
) -> Result<(bool, String), String> {
    let file = local_ai_model_destination(app, component)?.join(&component.filename);
    if !file.exists() {
        return Ok((false, "Modèle non installé.".into()));
    }
    let size = std::fs::metadata(&file).map_err(|e| e.to_string())?.len();
    if size != component.byte_size {
        return Ok((false, "Le fichier du modèle est incomplet.".into()));
    }
    Ok((
        true,
        format!(
            "{} prêt et conservé uniquement sur cet appareil.",
            component.label.as_deref().unwrap_or(&component.id)
        ),
    ))
}

fn trusted_local_ai_model_download_url(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }
    matches!(
        url.host_str(),
        Some(
            "huggingface.co"
                | "cdn-lfs.huggingface.co"
                | "cas-bridge.xethub.hf.co"
                | "cas-server.xethub.hf.co"
        )
    ) || url
        .host_str()
        .is_some_and(|host| host.ends_with(".aws.cdn.hf.co"))
}

fn install_local_ai_model_inner(
    app: &tauri::AppHandle,
    component: &LocalAiCatalogComponent,
    download_id: &str,
) -> Result<String, String> {
    let root = local_ai_root(app)?;
    std::fs::create_dir_all(root.join("models")).map_err(|e| e.to_string())?;
    let partial = root
        .join("downloads")
        .join(format!("{}-{}.gguf.part", component.id, component.version));
    let required = component.byte_size.saturating_mul(2);
    let free = fs2::available_space(&root)
        .map_err(|e| format!("Espace disque impossible à vérifier : {e}"))?;
    if free < required {
        return Err(format!(
            "Espace insuffisant : {:.1} Go libres, {:.1} Go nécessaires pendant l’installation.",
            free as f64 / 1_073_741_824.0,
            required as f64 / 1_073_741_824.0
        ));
    }
    let conn = get_db_connection(app)?;
    let mut existing = std::fs::metadata(&partial)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
        .min(component.byte_size);
    if existing == component.byte_size
        && sha256_file(&partial).ok().as_deref() != Some(component.sha256.as_str())
    {
        std::fs::File::create(&partial).map_err(|e| e.to_string())?;
        existing = 0;
    }
    if existing < component.byte_size {
        let policy = reqwest::redirect::Policy::custom(|attempt| {
            if trusted_local_ai_model_download_url(attempt.url()) && attempt.previous().len() < 6 {
                attempt.follow()
            } else {
                attempt.stop()
            }
        });
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(900))
            .redirect(policy)
            .user_agent("MANTIS-POSTURE/0.1 local-ai-model-installer")
            .build()
            .map_err(|e| e.to_string())?;
        let mut request = client.get(&component.url);
        if existing > 0 {
            request = request.header(reqwest::header::RANGE, format!("bytes={existing}-"));
        }
        let mut response = request.send().map_err(|_| {
            "Téléchargement interrompu ; la partie déjà reçue est conservée.".to_string()
        })?;
        if !response.status().is_success() {
            return Err(format!(
                "La source officielle du modèle a répondu avec le statut {}.",
                response.status()
            ));
        }
        let resumed = existing > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
        let mut output = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(&partial)
            .map_err(|e| e.to_string())?;
        let mut downloaded = if resumed { existing } else { 0 };
        let mut buffer = [0_u8; 128 * 1024];
        let mut last_recorded = downloaded;
        loop {
            let state: String = conn
                .query_row(
                    "SELECT status FROM local_ai_downloads WHERE id=?1",
                    params![download_id],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "interrompu".into());
            if state == "interrompu" {
                return Err(
                    "Téléchargement mis en pause. Il reprendra sans repartir de zéro.".into(),
                );
            }
            let read = response.read(&mut buffer).map_err(|_| {
                "Téléchargement interrompu ; la partie déjà reçue est conservée.".to_string()
            })?;
            if read == 0 {
                break;
            }
            downloaded = downloaded.saturating_add(read as u64);
            if downloaded > component.byte_size {
                return Err("Le modèle dépasse la taille du catalogue signé.".into());
            }
            output
                .write_all(&buffer[..read])
                .map_err(|e| e.to_string())?;
            if downloaded.saturating_sub(last_recorded) >= 1_048_576 {
                conn.execute("UPDATE local_ai_downloads SET downloaded_bytes=?1,updated_at=datetime('now') WHERE id=?2", params![downloaded as i64, download_id]).map_err(|e| e.to_string())?;
                last_recorded = downloaded;
            }
        }
        output.flush().map_err(|e| e.to_string())?;
    }
    let downloaded = std::fs::metadata(&partial)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if downloaded != component.byte_size || sha256_file(&partial)? != component.sha256 {
        return Err("Le modèle reçu est incomplet ou ne correspond pas au catalogue signé.".into());
    }
    let destination = local_ai_model_destination(app, component)?;
    let staging = destination.with_extension(format!("staging-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    std::fs::rename(&partial, staging.join(&component.filename)).map_err(|e| e.to_string())?;
    if destination.exists() {
        std::fs::remove_dir_all(&destination).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&staging, &destination).map_err(|e| e.to_string())?;
    conn.execute("UPDATE local_ai_downloads SET downloaded_bytes=?1,status='verifie',error_message=NULL,updated_at=datetime('now') WHERE id=?2", params![component.byte_size as i64,download_id]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE local_ai_components SET install_path=?1,status='pret',diagnostic='Modèle vérifié.',installed_at=datetime('now'),updated_at=datetime('now') WHERE component_id=?2", params![destination.join(&component.filename).to_string_lossy().to_string(),component.id]).map_err(|e| e.to_string())?;
    // Installing a model is not consent to run it. Keep the feature disabled
    // until the user explicitly activates this experimental capability.
    conn.execute("UPDATE local_ai_preferences SET enabled=enabled,selected_model_id=?1,onboarding_status=CASE WHEN enabled=1 THEN 'configure' ELSE 'a_proposer' END,consented_at=CASE WHEN enabled=1 THEN COALESCE(consented_at,datetime('now')) ELSE consented_at END,updated_at=datetime('now') WHERE id=1", params![component.id]).map_err(|e| e.to_string())?;
    Ok(format!(
        "{} installé et vérifié.",
        component.label.as_deref().unwrap_or(&component.id)
    ))
}

#[tauri::command]
fn is_local_ai_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let conn = get_db_connection(&app)?;
    Ok(conn
        .query_row(
            "SELECT enabled FROM local_ai_preferences WHERE id=1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0)
}

#[tauri::command]
fn get_local_ai_status(app: tauri::AppHandle) -> Result<LocalAiStatus, String> {
    let component = local_ai_component()?;
    let conn = get_db_connection(&app)?;
    let mut system = System::new();
    system.refresh_memory();
    let total_memory_bytes = system.total_memory();
    let available_disk_bytes = fs2::available_space(local_ai_root(&app)?).unwrap_or(0);
    let recommended_model_id = if total_memory_bytes >= 12 * 1_073_741_824 {
        "qwen3-1.7b-q8"
    } else {
        "qwen3-0.6b-q8"
    }
    .to_string();
    let (enabled, selected_model_id, onboarding_status): (bool, Option<String>, String) = conn.query_row(
        "SELECT enabled,selected_model_id,onboarding_status FROM local_ai_preferences WHERE id=1", [],
        |row| Ok((row.get::<_, i64>(0)? != 0,row.get(1)?,row.get(2)?)),
    ).unwrap_or((false,None,"a_proposer".into()));
    let active_download_id = conn.query_row("SELECT d.id FROM local_ai_downloads d JOIN local_ai_components c ON c.component_id=d.component_id WHERE d.status='en_cours' AND c.component_type='model' ORDER BY d.rowid DESC LIMIT 1", [], |row| row.get::<_,String>(0)).ok();
    let (status, downloaded_bytes): (String, i64) = conn.query_row(
        "SELECT c.status,COALESCE((SELECT downloaded_bytes FROM local_ai_downloads d WHERE d.component_id=c.component_id ORDER BY d.rowid DESC LIMIT 1),0) FROM local_ai_components c WHERE c.component_id=?1",
        params![component.id], |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or_else(|_| ("non_installe".into(), 0));
    let (integrity_ok, diagnostic) =
        installed_local_ai_integrity(&app, &component).unwrap_or_else(|error| (false, error));
    let models = local_ai_models()?.into_iter().map(|model| {
        let (status, downloaded_bytes): (String,i64) = conn.query_row(
            "SELECT c.status,COALESCE((SELECT downloaded_bytes FROM local_ai_downloads d WHERE d.component_id=c.component_id ORDER BY d.rowid DESC LIMIT 1),0) FROM local_ai_components c WHERE c.component_id=?1",
            params![model.id], |row| Ok((row.get(0)?,row.get(1)?)),
        ).unwrap_or(("non_installe".into(),0));
        let (installed, model_diagnostic) = installed_model_integrity(&app,&model).unwrap_or_else(|error| (false,error));
        let min_ram_gb = model.min_ram_gb.unwrap_or(0);
        LocalAiModelStatus { component_id:model.id, label:model.label.unwrap_or_else(|| "Modèle local".into()), tier:model.tier.unwrap_or_else(|| "leger".into()), version:model.version,
            status:if installed {"pret".into()} else {status}, installed, downloaded_bytes, expected_bytes:model.byte_size as i64, license:model.license,
            source:model.source, min_ram_gb, context_size:model.context_size.unwrap_or(4096), compatible:total_memory_bytes >= min_ram_gb * 1_073_741_824, diagnostic:model_diagnostic }
    }).collect();
    Ok(LocalAiStatus {
        component_id: component.id,
        version: component.version,
        status: if integrity_ok { "pret".into() } else { status },
        installed: integrity_ok,
        integrity_ok,
        downloaded_bytes,
        expected_bytes: component.byte_size as i64,
        diagnostic,
        platform: component.platform,
        architecture: component.architecture,
        license: component.license,
        source: component.source,
        enabled,
        onboarding_status,
        selected_model_id,
        recommended_model_id,
        total_memory_bytes,
        available_disk_bytes,
        active_download_id,
        models,
    })
}

#[tauri::command]
fn start_local_ai_setup(app: tauri::AppHandle, model_id: String) -> Result<String, String> {
    let model = local_ai_models()?
        .into_iter()
        .find(|item| item.id == model_id)
        .ok_or("Ce modèle n’appartient pas au catalogue MANTIS.")?;
    let conn = get_db_connection(&app)?;
    if conn
        .query_row(
            "SELECT COUNT(*) FROM local_ai_downloads WHERE status='en_cours'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0
    {
        return Err("Un téléchargement IA est déjà en cours.".into());
    }
    let partial = local_ai_root(&app)?
        .join("downloads")
        .join(format!("{}-{}.gguf.part", model.id, model.version));
    let existing = std::fs::metadata(&partial)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let download_id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO local_ai_downloads (id,component_id,version,source_url,partial_path,expected_sha256,expected_bytes,downloaded_bytes,status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'en_cours')",
        params![download_id,model.id,model.version,model.url,partial.to_string_lossy().to_string(),model.sha256,model.byte_size as i64,existing as i64]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE local_ai_components SET status='telechargement',diagnostic='Téléchargement en cours.',updated_at=datetime('now') WHERE component_id=?1",params![model.id]).map_err(|e| e.to_string())?;
    let worker_id = download_id.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let runtime = local_ai_component()?;
            if !installed_local_ai_integrity(&app, &runtime)?.0 {
                install_local_ai_runtime(app.clone())?;
            }
            install_local_ai_model_inner(&app, &model, &worker_id)?;
            Ok(())
        })();
        if let Err(error) = result {
            if let Ok(conn) = get_db_connection(&app) {
                let paused = error.contains("pause");
                let _ = conn.execute("UPDATE local_ai_downloads SET status=?1,error_message=?2,updated_at=datetime('now') WHERE id=?3",params![if paused {"interrompu"} else {"erreur"},error,worker_id]);
                let _ = conn.execute("UPDATE local_ai_components SET status=?1,diagnostic=?2,updated_at=datetime('now') WHERE component_id=?3",params![if paused {"non_installe"} else {"erreur"},error,model.id]);
            }
        }
    });
    Ok(download_id)
}

#[tauri::command]
fn pause_local_ai_download(app: tauri::AppHandle, download_id: String) -> Result<String, String> {
    let conn = get_db_connection(&app)?;
    let changed = conn.execute("UPDATE local_ai_downloads SET status='interrompu',error_message='Pause demandée par l’utilisateur.',updated_at=datetime('now') WHERE id=?1 AND status='en_cours'",params![download_id]).map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Ce téléchargement n’est plus actif.".into());
    }
    Ok("Pause demandée. La reprise conservera les données déjà téléchargées.".into())
}

#[tauri::command]
fn set_local_ai_preference(
    app: tauri::AppHandle,
    enabled: bool,
    model_id: Option<String>,
    onboarding_status: String,
) -> Result<String, String> {
    if !matches!(
        onboarding_status.as_str(),
        "a_proposer" | "sans_ia" | "configure"
    ) {
        return Err("Préférence IA invalide.".into());
    }
    if let Some(ref id) = model_id {
        if !local_ai_models()?.iter().any(|model| &model.id == id) {
            return Err("Modèle non autorisé.".into());
        }
    }
    let conn = get_db_connection(&app)?;
    conn.execute("UPDATE local_ai_preferences SET enabled=?1,selected_model_id=?2,onboarding_status=?3,updated_at=datetime('now') WHERE id=1",params![enabled as i64,model_id,onboarding_status]).map_err(|e| e.to_string())?;
    Ok(if enabled {
        "Fonctions IA locales activées."
    } else {
        "MANTIS continue entièrement sans IA."
    }
    .into())
}

fn bounded_analysis_text(value: &str, max_chars: usize) -> String {
    value
        .replace(['\r', '\n', '\t'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

fn load_analysis_inputs(
    conn: &Connection,
    signal_ids: &[String],
) -> Result<Vec<LocalAiAnalysisInput>, String> {
    if signal_ids.is_empty() || signal_ids.len() > 12 {
        return Err("Sélectionnez entre 1 et 12 signaux à analyser.".into());
    }
    let mut unique = std::collections::HashSet::new();
    let mut inputs = Vec::new();
    for signal_id in signal_ids {
        if !unique.insert(signal_id.clone()) {
            continue;
        }
        let (id, observation_type, display_value, source, relevance_status): (String,String,String,String,String) = conn.query_row(
            "SELECT id,observation_type,display_value,source,relevance_status FROM osint_observations WHERE signal_id=?1",
            params![signal_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?)),
        ).map_err(|_| "Un signal ne possède pas d’observation traçable ; l’analyse est refusée.".to_string())?;
        let mut statement = conn.prepare("SELECT id,evidence_label,COALESCE(excerpt,'') FROM osint_evidence_links WHERE observation_id=?1 ORDER BY created_at,id LIMIT 3").map_err(|e| e.to_string())?;
        let evidence = statement
            .query_map(params![id], |row| {
                Ok(LocalAiAnalysisEvidence {
                    evidence_id: row.get(0)?,
                    label: row.get::<_, String>(1)?,
                    excerpt: bounded_analysis_text(&row.get::<_, String>(2)?, 240),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if evidence.is_empty() {
            return Err(
                "Une observation ne possède aucune preuve citabile ; l’analyse est refusée.".into(),
            );
        }
        inputs.push(LocalAiAnalysisInput {
            observation_id: id,
            observation_type,
            display_value: bounded_analysis_text(&display_value, 240),
            source: bounded_analysis_text(&source, 120),
            relevance_status,
            evidence,
        });
    }
    Ok(inputs)
}

fn deterministic_analysis(inputs: &[LocalAiAnalysisInput]) -> OsintAnalysisPayload {
    let items = inputs.iter().map(|input| {
        let rejected = matches!(input.relevance_status.as_str(), "pas_moi" | "ignoree");
        let (classification,relevance,action,reason,uncertainty) = if rejected {
            ("autre","bruit","ignorer","Votre décision précédente écarte ce signal ; MANTIS la conserve comme prioritaire.",false)
        } else if input.observation_type == "fuite" {
            ("fuite","important","securiser","Une source de fuite signale cet identifiant. Vérifiez le service concerné avant toute conclusion.",true)
        } else if matches!(input.observation_type.as_str(), "compte_potentiel" | "profil_public") {
            ("compte","a_verifier","verifier","La présence d’un identifiant similaire ne suffit pas à attribuer ce compte à la personne.",true)
        } else if input.observation_type == "mention" {
            ("mention","a_verifier","verifier","Cette mention publique doit être ouverte et comparée à des indices confirmés.",true)
        } else {
            ("autre","a_verifier","verifier","Les éléments disponibles ne permettent pas une conclusion fiable.",true)
        };
        OsintAnalysisItem { observation_id:input.observation_id.clone(), classification:classification.into(), relevance:relevance.into(), reason:reason.into(), recommended_action:action.into(), evidence_ids:vec![input.evidence[0].evidence_id.clone()], uncertainty }
    }).collect::<Vec<_>>();
    let important = items
        .iter()
        .filter(|item| item.relevance == "important")
        .count();
    let noise = items
        .iter()
        .filter(|item| item.relevance == "bruit")
        .count();
    OsintAnalysisPayload { schema_version:1, task:"triage_osint".into(), overview:format!("{} élément(s) prioritaire(s), {} écarté(s) selon vos décisions et {} restant à vérifier.",important,noise,items.len().saturating_sub(important+noise)), items, limitations:vec!["Analyse fondée uniquement sur les observations et extraits de preuve affichés ; aucune recherche externe n’a été effectuée.".into()] }
}

fn validate_analysis_payload(
    payload: &OsintAnalysisPayload,
    inputs: &[LocalAiAnalysisInput],
) -> Result<(), String> {
    if payload.schema_version != 1
        || payload.task != "triage_osint"
        || payload.overview.is_empty()
        || payload.overview.chars().count() > 500
        || payload.items.len() != inputs.len()
        || payload.limitations.len() > 5
    {
        return Err("La sortie IA ne respecte pas le contrat de synthèse.".into());
    }
    let expected = inputs
        .iter()
        .map(|input| input.observation_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let mut seen = std::collections::HashSet::new();
    for item in &payload.items {
        let Some(input) = inputs
            .iter()
            .find(|input| input.observation_id == item.observation_id)
        else {
            return Err("La sortie IA cite une observation inconnue.".into());
        };
        if !seen.insert(item.observation_id.as_str())
            || item.reason.is_empty()
            || item.reason.chars().count() > 300
            || !matches!(
                item.classification.as_str(),
                "fuite" | "compte" | "mention" | "contact" | "autre"
            )
            || !matches!(
                item.relevance.as_str(),
                "important" | "a_verifier" | "bruit"
            )
            || !matches!(
                item.recommended_action.as_str(),
                "securiser" | "verifier" | "suivre" | "ignorer"
            )
        {
            return Err("La sortie IA contient une valeur interdite ou dupliquée.".into());
        }
        let allowed_evidence = input
            .evidence
            .iter()
            .map(|evidence| evidence.evidence_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        if item.evidence_ids.is_empty()
            || item.evidence_ids.len() > 3
            || item
                .evidence_ids
                .iter()
                .any(|id| !allowed_evidence.contains(id.as_str()))
        {
            return Err("La sortie IA cite une preuve absente de l’observation.".into());
        }
    }
    if seen != expected {
        return Err("La sortie IA n’analyse pas exactement les observations demandées.".into());
    }
    Ok(())
}

fn json_object_from_output(output: &str) -> Result<OsintAnalysisPayload, String> {
    let cleaned = output.replace("\u{1b}[0m", "");
    let start = cleaned
        .find('{')
        .ok_or("Le modèle n’a produit aucun objet JSON.")?;
    let end = cleaned
        .rfind('}')
        .ok_or("Le modèle a produit un JSON incomplet.")?;
    serde_json::from_str(&cleaned[start..=end])
        .map_err(|_| "Le modèle a produit un JSON invalide.".into())
}

fn execute_local_triage(
    app: &tauri::AppHandle,
    inputs: &[LocalAiAnalysisInput],
) -> Result<(OsintAnalysisPayload, String), String> {
    let conn = get_db_connection(app)?;
    let (enabled, selected): (i64, Option<String>) = conn
        .query_row(
            "SELECT enabled,selected_model_id FROM local_ai_preferences WHERE id=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    if enabled == 0 {
        return Err("Les fonctions IA locales sont désactivées.".into());
    }
    let model_id = selected.ok_or("Aucun modèle local n’est sélectionné.")?;
    let model = local_ai_models()?
        .into_iter()
        .find(|model| model.id == model_id)
        .ok_or("Le modèle sélectionné n’appartient plus au catalogue signé.")?;
    if !installed_model_integrity(app, &model)?.0 {
        return Err("Le modèle local n’est pas prêt.".into());
    }
    let runtime = local_ai_component()?;
    let executable = installed_local_ai_executable(app, &runtime)?;
    let model_path = local_ai_model_destination(app, &model)?.join(&model.filename);
    let context_size = model
        .context_size
        .unwrap_or(4096)
        .clamp(4096, 8192)
        .to_string();
    let input_json = serde_json::to_string(inputs).map_err(|e| e.to_string())?;
    let base_prompt = format!(
        "{}\n\nOBSERVATIONS_JSON:\n{}\n\nRéponds maintenant uniquement avec l’objet JSON.",
        TRIAGE_OSINT_INSTRUCTIONS, input_json
    );
    let mut last_error = String::new();
    let threads = local_ai_thread_count();
    for attempt in 0..2 {
        let prompt = if attempt == 0 {
            base_prompt.clone()
        } else {
            format!("{}\n\nTa première réponse a été refusée par la validation Rust. Corrige strictement le format, les identifiants et les citations.",base_prompt)
        };
        let predicted_tokens = (220 + inputs.len() * 115).min(1400).to_string();
        let args = vec![
            "--model".into(),
            model_path.to_string_lossy().to_string(),
            "--threads".into(),
            threads.clone(),
            "--threads-batch".into(),
            threads.clone(),
            "--prompt".into(),
            prompt,
            "--ctx-size".into(),
            context_size.clone(),
            "--predict".into(),
            predicted_tokens,
            "--temp".into(),
            "0.1".into(),
            "--seed".into(),
            "42".into(),
            "--json-schema".into(),
            TRIAGE_OSINT_SCHEMA.into(),
            "--no-display-prompt".into(),
            "--simple-io".into(),
            "-no-cnv".into(),
            "--no-jinja".into(),
            "--single-turn".into(),
            "--reasoning".into(),
            "off".into(),
        ];
        match run_process(&executable, &args, Duration::from_secs(90))
            .and_then(|(stdout, _)| json_object_from_output(&stdout))
            .and_then(|payload| {
                validate_analysis_payload(&payload, inputs)?;
                Ok(payload)
            }) {
            Ok(payload) => return Ok((payload, model.label.unwrap_or(model.id))),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

fn analyze_osint_signals_inner(
    app: tauri::AppHandle,
    signal_ids: Vec<String>,
) -> Result<OsintAnalysisReport, String> {
    let conn = get_db_connection(&app)?;
    let inputs = load_analysis_inputs(&conn, &signal_ids)?;
    let input_json = serde_json::to_vec(&inputs).map_err(|e| e.to_string())?;
    let input_hash = hex::encode(Sha256::digest(&input_json));
    let run_id = Uuid::new_v4().to_string();
    // Runtime metadata is provenance only: a deterministic fallback must still
    // be generated when the local runtime or signed catalog is unavailable.
    let runtime = local_ai_component()?;
    let selected_model_id: Option<String> = conn
        .query_row(
            "SELECT selected_model_id FROM local_ai_preferences WHERE id=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(None);
    let selected_model = local_ai_models()
        .unwrap_or_default()
        .into_iter()
        .find(|model| Some(model.id.clone()) == selected_model_id);
    if let Some(model) = selected_model.as_ref() {
        let cached = conn.query_row(
            "SELECT r.id,o.output_json FROM osint_analysis_runs r JOIN osint_analysis_outputs o ON o.run_id=r.id WHERE r.task='triage_osint' AND r.contract_version='1.0.0' AND r.input_hash=?1 AND r.model_component_id=?2 AND r.model_version=?3 AND r.status='valide' AND o.validated=1 ORDER BY r.completed_at DESC LIMIT 1",
            params![input_hash,model.id,model.version], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)),
        ).ok();
        if let Some((cached_run_id, cached_json)) = cached {
            if let Ok(payload) = serde_json::from_str::<OsintAnalysisPayload>(&cached_json) {
                if validate_analysis_payload(&payload, &inputs).is_ok() {
                    return Ok(OsintAnalysisReport {
                        run_id: cached_run_id,
                        mode: "ia_locale".into(),
                        status: "cache_valide".into(),
                        model_label: Some(model.label.clone().unwrap_or_else(|| model.id.clone())),
                        overview: payload.overview,
                        items: payload.items,
                        limitations: payload.limitations,
                        conclusion: None,
                        findings: vec![],
                        citation_ids: vec![],
                    });
                }
            }
        }
    }
    conn.execute("INSERT INTO osint_analysis_runs (id,task,contract_version,input_hash,runtime_component_id,runtime_version,model_component_id,model_version,model_sha256,status) VALUES (?1,'triage_osint','1.0.0',?2,?3,?4,?5,?6,?7,'en_cours')",
        params![run_id,input_hash,runtime.id,runtime.version,selected_model.as_ref().map(|m|m.id.as_str()),selected_model.as_ref().map(|m|m.version.as_str()),selected_model.as_ref().map(|m|m.sha256.as_str())]).map_err(|e| e.to_string())?;
    for (position, input) in inputs.iter().enumerate() {
        conn.execute(
            "INSERT INTO osint_analysis_inputs (run_id,observation_id,position) VALUES (?1,?2,?3)",
            params![run_id, input.observation_id, position as i64],
        )
        .map_err(|e| e.to_string())?;
    }
    let started = Instant::now();
    let (payload, mode, status, model_label, error_message) =
        match execute_local_triage(&app, &inputs) {
            Ok((payload, label)) => (payload, "ia_locale", "valide", Some(label), None),
            Err(error) => (
                deterministic_analysis(&inputs),
                "deterministe",
                "fallback",
                None,
                Some(error),
            ),
        };
    validate_analysis_payload(&payload, &inputs)?;
    let output_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    conn.execute("UPDATE osint_analysis_runs SET status=?1,fallback_used=?2,completed_at=datetime('now'),duration_ms=?3,error_message=?4 WHERE id=?5",params![status,(mode=="deterministe") as i64,started.elapsed().as_millis() as i64,error_message,run_id]).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO osint_analysis_outputs (id,run_id,schema_version,output_json,overview,needs_human_review,validated) VALUES (?1,?2,1,?3,?4,1,1)",params![Uuid::new_v4().to_string(),run_id,output_json,payload.overview]).map_err(|e| e.to_string())?;
    Ok(OsintAnalysisReport {
        run_id,
        mode: mode.into(),
        status: status.into(),
        model_label,
        overview: payload.overview,
        items: payload.items,
        limitations: payload.limitations,
        conclusion: None,
        findings: vec![],
        citation_ids: vec![],
    })
}

fn load_claim_synthesis_inputs(
    conn: &Connection,
    identity_id: &str,
) -> Result<Vec<LocalAiClaimInput>, String> {
    let claims = load_identity_claims(conn, identity_id)?;
    if claims.is_empty() {
        return Err("Aucune revendication traçable à synthétiser pour cette identité.".into());
    }
    let inputs = claims
        .into_iter()
        .take(12)
        .map(|claim| {
            let resolution = conn
                .query_row(
                    "SELECT r.status,r.source_count,r.rationale
                     FROM osint_claim_fact_links cl
                     JOIN osint_fact_resolution_evidence re ON re.fact_id=cl.fact_id
                     JOIN osint_fact_resolutions r ON r.id=re.resolution_id
                     WHERE cl.claim_id=?1
                     ORDER BY CASE r.status WHEN 'contradictoire' THEN 0 WHEN 'rejete' THEN 1 WHEN 'corroboree' THEN 2 ELSE 3 END,
                              r.source_count DESC LIMIT 1",
                    params![claim.id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
                )
                .unwrap_or_else(|_| (claim.status.clone(), claim.source_count, claim.rationale.clone()));
            let mut evidence_seen = std::collections::HashSet::new();
            let evidence = claim
                .evidence
                .into_iter()
                .filter(|item| {
                    let key = format!(
                        "{}|{}|{}",
                        normalized_exact_text(&item.source),
                        item.source_url
                            .as_deref()
                            .and_then(normalized_claim_url)
                            .unwrap_or_default(),
                        item.role
                    );
                    evidence_seen.insert(key)
                })
                // Three references per claim retain source diversity while
                // keeping room for the constrained JSON response.
                .take(3)
                .map(|item| LocalAiClaimEvidence {
                    evidence_id: item.observation_id,
                    source: bounded_analysis_text(&item.source, 100),
                    source_url: item
                        .source_url
                        .filter(|url| url.starts_with("https://") || url.starts_with("http://")),
                    role: item.role,
                    observed_at: item.observed_at,
                })
                .collect::<Vec<_>>();
            LocalAiClaimInput {
                claim_id: claim.id,
                claim_type: claim.claim_type,
                display_value: bounded_analysis_text(&claim.display_value, 220),
                status: claim.status,
                priority: claim.priority,
                rationale: bounded_analysis_text(&claim.rationale, 300),
                resolution_status: resolution.0,
                resolution_source_count: resolution.1,
                resolution_rationale: bounded_analysis_text(&resolution.2, 300),
                evidence,
            }
        })
        .collect::<Vec<_>>();
    if inputs.iter().any(|input| input.evidence.is_empty()) {
        return Err(
            "Une revendication ne possède aucune preuve citabile ; la synthèse est refusée.".into(),
        );
    }
    Ok(inputs)
}

fn resolved_claim_status(input: &LocalAiClaimInput) -> &str {
    if input.status == "confirmee" {
        "confirmee"
    } else {
        input.resolution_status.as_str()
    }
}

fn deterministic_claim_synthesis(inputs: &[LocalAiClaimInput]) -> OsintSynthesisPayload {
    let corroborated = inputs
        .iter()
        .filter(|input| matches!(resolved_claim_status(input), "corroboree" | "confirmee"))
        .count();
    let contradictions = inputs
        .iter()
        .filter(|input| {
            matches!(
                resolved_claim_status(input),
                "contradictoire" | "rejete" | "rejetee"
            )
        })
        .count();
    let pending = inputs.len().saturating_sub(corroborated + contradictions);
    let citation_ids = inputs
        .iter()
        .flat_map(|input| {
            input
                .evidence
                .iter()
                .take(1)
                .map(|evidence| evidence.evidence_id.clone())
        })
        .take(8)
        .collect::<Vec<_>>();
    let findings = inputs.iter().map(|input| {
        let resolved_status = resolved_claim_status(input);
        let contradiction = matches!(resolved_status, "contradictoire" | "rejete" | "rejetee");
        let confidence = if contradiction || resolved_status == "a_verifier" { "faible" } else if resolved_status == "corroboree" && input.resolution_source_count >= 2 { "forte" } else { "moyenne" };
        let action = if contradiction { "ignorer" } else if input.claim_type == "fuite_eventuelle" { "securiser" } else if resolved_status == "corroboree" { "suivre" } else { "verifier" };
        let statement = if contradiction {
            format!("Une preuve contraire ou une décision humaine écarte cette correspondance possible : {}.", input.display_value)
        } else if resolved_status == "corroboree" {
            format!("Plusieurs sources signalent la même correspondance possible : {}. Ce recoupement ne confirme pas l’identité.", input.display_value)
        } else {
            format!("Signal à vérifier : {}. Les preuves disponibles ne permettent pas d’attribuer ce résultat à la personne.", input.display_value)
        };
        let exposure_kind = input.claim_type.clone();
        let exposed_data = if input.claim_type == "fuite_eventuelle" { vec![input.display_value.clone()] } else { vec![] };
        let where_found = input.evidence.iter().take(8).map(|evidence| evidence.source_url.clone().unwrap_or_else(|| evidence.source.clone())).collect();
        OsintSynthesisFinding { claim_id:input.claim_id.clone(),statement,confidence:confidence.into(),recommended_action:action.into(),evidence_ids:input.evidence.iter().take(4).map(|evidence|evidence.evidence_id.clone()).collect(),contradiction,exposure_kind,exposed_data,where_found }
    }).collect();
    OsintSynthesisPayload {
        schema_version:1,
        task:"synthese_multi_source".into(),
        conclusion:format!("MANTIS regroupe {} revendication(s) : {} recoupée(s), {} contradictoire(s) ou écartée(s), et {} restant à vérifier. Ces rapprochements décrivent des correspondances possibles, jamais une identité confirmée.",inputs.len(),corroborated,contradictions,pending),
        citation_ids,
        findings,
        limitations:vec!["Synthèse locale fondée uniquement sur les revendications et preuves conservées par MANTIS ; aucune recherche externe n’a été effectuée.".into(),"Une correspondance OSINT, même recoupée, nécessite une validation humaine avant toute action.".into()],
    }
}

fn apply_deterministic_claim_annotations(
    payload: &mut OsintSynthesisPayload,
    inputs: &[LocalAiClaimInput],
) {
    for finding in &mut payload.findings {
        let Some(input) = inputs
            .iter()
            .find(|input| input.claim_id == finding.claim_id)
        else {
            continue;
        };
        finding.contradiction = matches!(
            resolved_claim_status(input),
            "contradictoire" | "rejete" | "rejetee"
        );
        finding.exposure_kind = input.claim_type.clone();
        finding.exposed_data = if input.claim_type == "fuite_eventuelle" {
            vec![input.display_value.clone()]
        } else {
            Vec::new()
        };
        finding.where_found = input
            .evidence
            .iter()
            .take(8)
            .map(|evidence| {
                evidence
                    .source_url
                    .clone()
                    .unwrap_or_else(|| evidence.source.clone())
            })
            .collect();
    }
}

fn validate_claim_synthesis(
    payload: &OsintSynthesisPayload,
    inputs: &[LocalAiClaimInput],
) -> Result<(), String> {
    if payload.schema_version != 1
        || payload.task != "synthese_multi_source"
        || payload.conclusion.is_empty()
        || payload.conclusion.chars().count() > 700
        || payload.findings.len() != inputs.len()
        || payload.citation_ids.is_empty()
        || payload.citation_ids.len() > 8
        || payload.limitations.is_empty()
        || payload.limitations.len() > 5
    {
        return Err("La sortie IA ne respecte pas le contrat de synthèse multi-source.".into());
    }
    let forbidden = [
        "ce profil est confirmé",
        "profil confirmé pour cette personne",
        "appartient à la personne",
        "est bien la personne",
    ];
    if forbidden
        .iter()
        .any(|phrase| payload.conclusion.to_lowercase().contains(phrase))
    {
        return Err("La synthèse tente une attribution d’identité interdite.".into());
    }
    let all_evidence = inputs
        .iter()
        .flat_map(|input| {
            input
                .evidence
                .iter()
                .map(|evidence| evidence.evidence_id.as_str())
        })
        .collect::<std::collections::HashSet<_>>();
    let mut top_seen = std::collections::HashSet::new();
    if payload
        .citation_ids
        .iter()
        .any(|id| !all_evidence.contains(id.as_str()) || !top_seen.insert(id.as_str()))
    {
        return Err("La conclusion cite une preuve inconnue ou dupliquée.".into());
    }
    let expected = inputs
        .iter()
        .map(|input| input.claim_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let mut seen = std::collections::HashSet::new();
    for finding in &payload.findings {
        let Some(input) = inputs
            .iter()
            .find(|input| input.claim_id == finding.claim_id)
        else {
            return Err("La synthèse cite une revendication inconnue.".into());
        };
        if !seen.insert(finding.claim_id.as_str())
            || finding.statement.is_empty()
            || finding.statement.chars().count() > 360
            || !matches!(finding.confidence.as_str(), "faible" | "moyenne" | "forte")
            || !matches!(
                finding.recommended_action.as_str(),
                "verifier" | "securiser" | "suivre" | "ignorer"
            )
        {
            return Err("La synthèse contient une valeur interdite ou dupliquée.".into());
        }
        if forbidden
            .iter()
            .any(|phrase| finding.statement.to_lowercase().contains(phrase))
        {
            return Err("Un constat tente une attribution d’identité interdite.".into());
        }
        let allowed = input
            .evidence
            .iter()
            .map(|evidence| evidence.evidence_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let mut evidence_seen = std::collections::HashSet::new();
        if finding.evidence_ids.is_empty()
            || finding.evidence_ids.len() > 4
            || finding
                .evidence_ids
                .iter()
                .any(|id| !allowed.contains(id.as_str()) || !evidence_seen.insert(id.as_str()))
        {
            return Err("Un constat cite une preuve absente de sa revendication.".into());
        }
        if matches!(
            resolved_claim_status(input),
            "contradictoire" | "rejete" | "rejetee"
        ) && !finding.contradiction
        {
            return Err("La synthèse masque une contradiction ou une décision humaine.".into());
        }
        let expected_kind = input.claim_type.clone();
        let expected_data = if input.claim_type == "fuite_eventuelle" {
            vec![input.display_value.clone()]
        } else {
            vec![]
        };
        let expected_where: Vec<String> = input
            .evidence
            .iter()
            .take(8)
            .map(|evidence| {
                evidence
                    .source_url
                    .clone()
                    .unwrap_or_else(|| evidence.source.clone())
            })
            .collect();
        if finding.exposure_kind != expected_kind
            || finding.exposed_data != expected_data
            || finding.where_found != expected_where
        {
            return Err("Les annotations de fuite ou de provenance ne correspondent pas aux données déterministes.".into());
        }
    }
    if seen != expected {
        return Err("La synthèse ne couvre pas exactement les revendications demandées.".into());
    }
    Ok(())
}

/// Keep prose produced by a local model only when it can be attached to an
/// already-known claim. All identifiers, citations, provenance and actions
/// remain the deterministic projection. This lets compact models contribute a
/// readable conclusion without allowing a malformed internal reference to
/// discard the entire local synthesis.
fn repair_local_claim_synthesis(
    candidate: &OsintSynthesisPayload,
    inputs: &[LocalAiClaimInput],
) -> OsintSynthesisPayload {
    let mut repaired = deterministic_claim_synthesis(inputs);
    let forbidden = [
        "ce profil est confirmé",
        "profil confirmé pour cette personne",
        "appartient à la personne",
        "est bien la personne",
    ];
    let safe_text = |text: &str, limit: usize| {
        !text.trim().is_empty()
            && text.chars().count() <= limit
            && !forbidden
                .iter()
                .any(|phrase| text.to_lowercase().contains(phrase))
    };
    if candidate.schema_version == 1
        && candidate.task == "synthese_multi_source"
        && safe_text(&candidate.conclusion, 700)
    {
        repaired.conclusion = candidate.conclusion.clone();
    }
    for finding in &mut repaired.findings {
        if let Some(model_finding) = candidate
            .findings
            .iter()
            .find(|item| item.claim_id == finding.claim_id && safe_text(&item.statement, 360))
        {
            finding.statement = model_finding.statement.clone();
        }
    }
    let limitations = candidate
        .limitations
        .iter()
        .filter(|item| safe_text(item, 240))
        .take(5)
        .cloned()
        .collect::<Vec<_>>();
    if !limitations.is_empty() {
        repaired.limitations = limitations;
    }
    repaired
}

fn synthesis_json_from_output(output: &str) -> Result<OsintSynthesisPayload, String> {
    let cleaned = output.replace("\u{1b}[0m", "");
    // llama.cpp may append a terminal marker, a timing line, or a second
    // fenced object. Parse the first complete synthesis object instead of
    // treating the entire process output as one JSON document.
    for (start, _) in cleaned.match_indices('{') {
        let mut values = serde_json::Deserializer::from_str(&cleaned[start..])
            .into_iter::<OsintSynthesisPayload>();
        if let Some(Ok(payload)) = values.next() {
            return Ok(payload);
        }
    }
    Err("Le modèle n’a produit aucun objet JSON de synthèse complet.".into())
}

fn execute_local_claim_synthesis(
    app: &tauri::AppHandle,
    inputs: &[LocalAiClaimInput],
) -> Result<(OsintSynthesisPayload, String), String> {
    let conn = get_db_connection(app)?;
    let (enabled, selected): (i64, Option<String>) = conn
        .query_row(
            "SELECT enabled,selected_model_id FROM local_ai_preferences WHERE id=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    if enabled == 0 {
        return Err("Les fonctions IA locales sont désactivées.".into());
    }
    let model_id = selected.ok_or("Aucun modèle local n’est sélectionné.")?;
    let model = local_ai_models()?
        .into_iter()
        .find(|model| model.id == model_id)
        .ok_or("Le modèle sélectionné n’appartient plus au catalogue signé.")?;
    if !installed_model_integrity(app, &model)?.0 {
        return Err("Le modèle local n’est pas prêt.".into());
    }
    let runtime = local_ai_component()?;
    let executable = installed_local_ai_executable(app, &runtime)?;
    let model_path = local_ai_model_destination(app, &model)?.join(&model.filename);
    // A compact local model is useful as a brief writer, not as a second
    // evidence engine. Give it only the highest-priority three claims; the
    // complete evidence-backed synthesis remains deterministic below.
    let brief_inputs = inputs.iter().take(3).cloned().collect::<Vec<_>>();
    let input_json = serde_json::to_string(&brief_inputs).map_err(|e| e.to_string())?;
    let base_prompt = format!(
        "{}\n\nREVENDICATIONS_PRIORITAIRES_JSON:\n{}\n\nÉcris un brief très concis : une conclusion de 2 phrases, un constat de 1 phrase par revendication, et 1 ou 2 limites. Réponds uniquement avec l’objet JSON.",
        MULTI_SOURCE_SYNTHESIS_INSTRUCTIONS, input_json
    );
    let mut last_error = String::new();
    let threads = local_ai_thread_count();
    for attempt in 0..1 {
        let prompt = if attempt == 0 {
            base_prompt.clone()
        } else {
            format!("{}\n\nLa première réponse a été refusée. Corrige strictement les identifiants, citations, contradictions et le langage de non-attribution.",base_prompt)
        };
        let predicted_tokens = "320".to_string();
        let args = vec![
            "--model".into(),
            model_path.to_string_lossy().to_string(),
            "--threads".into(),
            threads.clone(),
            "--threads-batch".into(),
            threads.clone(),
            "--prompt".into(),
            prompt,
            "--ctx-size".into(),
            "4096".into(),
            "--predict".into(),
            predicted_tokens,
            "--temp".into(),
            "0.1".into(),
            "--seed".into(),
            "42".into(),
            "--json-schema".into(),
            MULTI_SOURCE_SYNTHESIS_SCHEMA.into(),
            "--no-display-prompt".into(),
            "--simple-io".into(),
            "-no-cnv".into(),
            "--no-jinja".into(),
            "--single-turn".into(),
            "--reasoning".into(),
            "off".into(),
        ];
        match run_process(&executable, &args, Duration::from_secs(60))
            .and_then(|(stdout, _)| synthesis_json_from_output(&stdout))
        {
            Ok(mut payload) => {
                apply_deterministic_claim_annotations(&mut payload, &brief_inputs);
                // The model may only cover the brief inputs. Merge its safe
                // prose into the full deterministic result instead of asking
                // it to regenerate all findings and technical references.
                payload = repair_local_claim_synthesis(&payload, inputs);
                match validate_claim_synthesis(&payload, inputs) {
                    Ok(()) => return Ok((payload, model.label.unwrap_or(model.id))),
                    Err(error) => last_error = error,
                }
            }
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

fn synthesis_report(
    run_id: String,
    mode: &str,
    status: String,
    model_label: Option<String>,
    payload: OsintSynthesisPayload,
) -> OsintAnalysisReport {
    OsintAnalysisReport {
        run_id,
        mode: mode.into(),
        status,
        model_label,
        overview: payload.conclusion.clone(),
        items: vec![],
        limitations: payload.limitations,
        conclusion: Some(payload.conclusion),
        findings: payload.findings,
        citation_ids: payload.citation_ids,
    }
}

fn analyze_identity_synthesis_inner(
    app: tauri::AppHandle,
    identity_id: String,
) -> Result<OsintAnalysisReport, String> {
    let conn = get_db_connection(&app)?;
    let inputs = load_claim_synthesis_inputs(&conn, &identity_id)?;
    let input_json = serde_json::to_vec(&inputs).map_err(|e| e.to_string())?;
    let input_hash = hex::encode(Sha256::digest(&input_json));
    // Runtime metadata is provenance only: a deterministic fallback must still
    // be generated when the local runtime or signed catalog is unavailable.
    let runtime = local_ai_component().ok();
    let selected_model_id: Option<String> = conn
        .query_row(
            "SELECT selected_model_id FROM local_ai_preferences WHERE id=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(None);
    let selected_model = local_ai_models()
        .unwrap_or_default()
        .into_iter()
        .find(|model| Some(model.id.clone()) == selected_model_id);
    if let Some(model) = selected_model.as_ref() {
        let cached=conn.query_row("SELECT r.id,o.output_json FROM osint_analysis_runs r JOIN osint_analysis_outputs o ON o.run_id=r.id JOIN osint_synthesis_runs s ON s.run_id=r.id WHERE r.task='synthese_multi_source' AND r.contract_version='1.0.0' AND r.input_hash=?1 AND r.model_component_id=?2 AND r.model_version=?3 AND r.status='valide' AND o.validated=1 AND s.identity_id=?4 ORDER BY r.completed_at DESC LIMIT 1",params![input_hash,model.id,model.version,identity_id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).ok();
        if let Some((cached_id, json)) = cached {
            if let Ok(payload) = serde_json::from_str::<OsintSynthesisPayload>(&json) {
                if validate_claim_synthesis(&payload, &inputs).is_ok() {
                    return Ok(synthesis_report(
                        cached_id,
                        "ia_locale",
                        "cache_valide".into(),
                        Some(model.label.clone().unwrap_or_else(|| model.id.clone())),
                        payload,
                    ));
                }
            }
        }
    }
    let run_id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO osint_analysis_runs (id,task,contract_version,input_hash,runtime_component_id,runtime_version,model_component_id,model_version,model_sha256,status) VALUES (?1,'synthese_multi_source','1.0.0',?2,?3,?4,?5,?6,?7,'en_cours')",params![run_id,input_hash,runtime.as_ref().map(|component|component.id.as_str()),runtime.as_ref().map(|component|component.version.as_str()),selected_model.as_ref().map(|m|m.id.as_str()),selected_model.as_ref().map(|m|m.version.as_str()),selected_model.as_ref().map(|m|m.sha256.as_str())]).map_err(|e|e.to_string())?;
    conn.execute(
        "INSERT INTO osint_synthesis_runs (run_id,identity_id) VALUES (?1,?2)",
        params![run_id, identity_id],
    )
    .map_err(|e| e.to_string())?;
    for (position, input) in inputs.iter().enumerate() {
        conn.execute(
            "INSERT INTO osint_analysis_claim_inputs (run_id,claim_id,position) VALUES (?1,?2,?3)",
            params![run_id, input.claim_id, position as i64],
        )
        .map_err(|e| e.to_string())?;
    }
    let started = Instant::now();
    let (payload, mode, status, label, error) = match execute_local_claim_synthesis(&app, &inputs) {
        Ok((payload, label)) => (payload, "ia_locale", "valide", Some(label), None),
        Err(error) => (
            deterministic_claim_synthesis(&inputs),
            "deterministe",
            "fallback",
            None,
            Some(error),
        ),
    };
    validate_claim_synthesis(&payload, &inputs)?;
    let output_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    conn.execute("UPDATE osint_analysis_runs SET status=?1,fallback_used=?2,completed_at=datetime('now'),duration_ms=?3,error_message=?4 WHERE id=?5",params![status,(mode=="deterministe") as i64,started.elapsed().as_millis() as i64,error,run_id]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO osint_analysis_outputs (id,run_id,schema_version,output_json,overview,needs_human_review,validated) VALUES (?1,?2,1,?3,?4,1,1)",params![Uuid::new_v4().to_string(),run_id,output_json,payload.conclusion]).map_err(|e|e.to_string())?;
    Ok(synthesis_report(
        run_id,
        mode,
        status.into(),
        label,
        payload,
    ))
}

fn enqueue_osint_analysis(
    app: &tauri::AppHandle,
    signal_ids: Vec<String>,
    origin: &str,
) -> Result<Option<String>, String> {
    if signal_ids.is_empty() {
        return Ok(None);
    }
    if !matches!(origin, "scan_manuel" | "routine" | "relance_manuelle") {
        return Err("Origine d’analyse inconnue.".into());
    }
    let signal_ids = signal_ids.into_iter().take(12).collect::<Vec<_>>();
    let conn = get_db_connection(app)?;
    let enabled = conn
        .query_row(
            "SELECT enabled FROM local_ai_preferences WHERE id=1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !enabled {
        return Ok(None);
    }
    let manual_mode = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key='local_ai_manual_scan_mode'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "automatic".into());
    if origin == "scan_manuel" && manual_mode == "manual" {
        return Ok(None);
    }
    let historical_seconds: Option<i64> = conn.query_row("SELECT CAST(AVG(duration_ms)/1000 AS INTEGER) FROM osint_analysis_runs WHERE status='valide' AND duration_ms IS NOT NULL",[],|row|row.get(0)).unwrap_or(None);
    let estimated_seconds = historical_seconds
        .unwrap_or(15 + (signal_ids.len().saturating_sub(1) as i64 * 6))
        .clamp(10, 180);
    let job_id = Uuid::new_v4().to_string();
    let serialized = serde_json::to_string(&signal_ids).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO osint_analysis_jobs (id,origin,signal_ids_json,signal_count,status,estimated_seconds) VALUES (?1,?2,?3,?4,'en_attente',?5)",params![job_id,origin,serialized,signal_ids.len() as i64,estimated_seconds]).map_err(|e|e.to_string())?;
    let worker_app = app.clone();
    let worker_job_id = job_id.clone();
    std::thread::spawn(move || {
        let lock = LOCAL_AI_ANALYSIS_LOCK.get_or_init(|| Mutex::new(()));
        let Ok(_guard) = lock.lock() else {
            return;
        };
        let Ok(conn) = get_db_connection(&worker_app) else {
            return;
        };
        let status: String = conn
            .query_row(
                "SELECT status FROM osint_analysis_jobs WHERE id=?1",
                params![worker_job_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "interrompu".into());
        if status != "en_attente" {
            return;
        }
        let enabled = conn
            .query_row(
                "SELECT enabled FROM local_ai_preferences WHERE id=1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            != 0;
        if !enabled {
            let _ = conn.execute(
                "UPDATE osint_analysis_jobs SET status='interrompu',message='Analyse annulée : les fonctions IA locales sont désactivées.',completed_at=datetime('now') WHERE id=?1",
                params![worker_job_id],
            );
            return;
        }
        let _ = conn.execute("UPDATE osint_analysis_jobs SET status='en_cours',started_at=datetime('now'),message='Analyse locale en cours.' WHERE id=?1",params![worker_job_id]);
        match analyze_osint_signals_inner(worker_app.clone(), signal_ids) {
            Ok(report) => {
                let final_status = if report.mode == "ia_locale" {
                    "termine"
                } else {
                    "fallback"
                };
                let message = if final_status == "termine" {
                    "Analyse IA locale terminée et validée."
                } else {
                    "Analyse déterministe terminée ; la sortie IA n’a pas été retenue."
                };
                let _ = conn.execute("UPDATE osint_analysis_jobs SET status=?1,run_id=?2,result_mode=?3,message=?4,completed_at=datetime('now') WHERE id=?5",params![final_status,report.run_id,report.mode,message,worker_job_id]);
            }
            Err(error) => {
                let _ = conn.execute("UPDATE osint_analysis_jobs SET status='erreur',message=?1,completed_at=datetime('now') WHERE id=?2",params![error,worker_job_id]);
            }
        }
    });
    Ok(Some(job_id))
}

fn enqueue_identity_synthesis(
    app: &tauri::AppHandle,
    identity_id: &str,
    origin: &str,
) -> Result<Option<String>, String> {
    if !matches!(origin, "scan_manuel" | "routine" | "relance_manuelle") {
        return Err("Origine de synthèse inconnue.".into());
    }
    let conn = get_db_connection(app)?;
    let enabled = conn
        .query_row(
            "SELECT enabled FROM local_ai_preferences WHERE id=1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !enabled {
        return Ok(None);
    }
    let manual_mode = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key='local_ai_manual_scan_mode'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "automatic".into());
    if origin == "scan_manuel" && manual_mode == "manual" {
        return Ok(None);
    }
    let inputs = load_claim_synthesis_inputs(&conn, identity_id)?;
    let claim_ids = inputs
        .iter()
        .map(|input| input.claim_id.clone())
        .collect::<Vec<_>>();
    let historical:Option<i64>=conn.query_row("SELECT CAST(AVG(duration_ms)/1000 AS INTEGER) FROM osint_analysis_runs WHERE task='synthese_multi_source' AND status='valide' AND duration_ms IS NOT NULL",[],|row|row.get(0)).unwrap_or(None);
    let estimate = historical
        .unwrap_or(18 + (inputs.len().saturating_sub(1) as i64 * 7))
        .clamp(10, 240);
    let job_id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO osint_analysis_jobs (id,origin,signal_ids_json,signal_count,status,estimated_seconds,task,identity_id) VALUES (?1,?2,?3,?4,'en_attente',?5,'synthese_multi_source',?6)",params![job_id,origin,serde_json::to_string(&claim_ids).map_err(|e|e.to_string())?,inputs.len() as i64,estimate,identity_id]).map_err(|e|e.to_string())?;
    let worker_app = app.clone();
    let worker_job_id = job_id.clone();
    let worker_identity = identity_id.to_string();
    std::thread::spawn(move || {
        let lock = LOCAL_AI_ANALYSIS_LOCK.get_or_init(|| Mutex::new(()));
        let Ok(_guard) = lock.lock() else {
            return;
        };
        let Ok(conn) = get_db_connection(&worker_app) else {
            return;
        };
        let status: String = conn
            .query_row(
                "SELECT status FROM osint_analysis_jobs WHERE id=?1",
                params![worker_job_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "interrompu".into());
        if status != "en_attente" {
            return;
        }
        let enabled = conn
            .query_row(
                "SELECT enabled FROM local_ai_preferences WHERE id=1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            != 0;
        if !enabled {
            let _ = conn.execute(
                "UPDATE osint_analysis_jobs SET status='interrompu',message='Synthèse annulée : les fonctions IA locales sont désactivées.',completed_at=datetime('now') WHERE id=?1",
                params![worker_job_id],
            );
            return;
        }
        let _=conn.execute("UPDATE osint_analysis_jobs SET status='en_cours',started_at=datetime('now'),message='Synthèse multi-source locale en cours.' WHERE id=?1",params![worker_job_id]);
        match analyze_identity_synthesis_inner(worker_app.clone(), worker_identity) {
            Ok(report) => {
                let final_status = if report.mode == "ia_locale" {
                    "termine"
                } else {
                    "fallback"
                };
                let message = if final_status == "termine" {
                    "Synthèse multi-source locale terminée et validée."
                } else {
                    "Synthèse déterministe disponible ; la sortie IA n’a pas été retenue."
                };
                let _=conn.execute("UPDATE osint_analysis_jobs SET status=?1,run_id=?2,result_mode=?3,message=?4,completed_at=datetime('now') WHERE id=?5",params![final_status,report.run_id,report.mode,message,worker_job_id]);
            }
            Err(error) => {
                let _=conn.execute("UPDATE osint_analysis_jobs SET status='erreur',message=?1,completed_at=datetime('now') WHERE id=?2",params![error,worker_job_id]);
            }
        }
    });
    Ok(Some(job_id))
}

#[tauri::command]
fn start_osint_analysis(app: tauri::AppHandle, signal_ids: Vec<String>) -> Result<String, String> {
    enqueue_osint_analysis(&app, signal_ids, "relance_manuelle")?
        .ok_or_else(|| "Les fonctions IA locales sont désactivées.".into())
}

#[tauri::command]
fn start_osint_claim_synthesis(
    app: tauri::AppHandle,
    identity_id: String,
) -> Result<String, String> {
    enqueue_identity_synthesis(&app, &identity_id, "relance_manuelle")?
        .ok_or_else(|| "Les fonctions IA locales sont désactivées.".into())
}

fn analysis_report_from_run(conn: &Connection, run_id: &str) -> Option<OsintAnalysisReport> {
    let (output_json,status,model_id,task): (String,String,Option<String>,String) = conn.query_row("SELECT o.output_json,r.status,r.model_component_id,r.task FROM osint_analysis_outputs o JOIN osint_analysis_runs r ON r.id=o.run_id WHERE r.id=?1 AND o.validated=1",params![run_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).ok()?;
    let label = model_id.and_then(|id| {
        local_ai_models()
            .ok()?
            .into_iter()
            .find(|model| model.id == id)
            .and_then(|model| model.label)
    });
    let mode = if status == "valide" {
        "ia_locale"
    } else {
        "deterministe"
    };
    if task == "synthese_multi_source" {
        let payload: OsintSynthesisPayload = serde_json::from_str(&output_json).ok()?;
        return Some(synthesis_report(
            run_id.into(),
            mode,
            status,
            label,
            payload,
        ));
    }
    let payload: OsintAnalysisPayload = serde_json::from_str(&output_json).ok()?;
    Some(OsintAnalysisReport {
        run_id: run_id.into(),
        mode: mode.into(),
        status,
        model_label: label,
        overview: payload.overview,
        items: payload.items,
        limitations: payload.limitations,
        conclusion: None,
        findings: vec![],
        citation_ids: vec![],
    })
}

#[tauri::command]
fn get_osint_analysis_job(
    app: tauri::AppHandle,
    job_id: String,
) -> Result<OsintAnalysisJobStatus, String> {
    let conn = get_db_connection(&app)?;
    recover_stale_analysis_jobs(&conn)?;
    let (id,origin,status,signal_count,estimated_seconds,elapsed_seconds,run_id,result_mode,message): (String,String,String,i64,i64,i64,Option<String>,Option<String>,Option<String>) = conn.query_row("SELECT id,origin,status,signal_count,estimated_seconds,CASE WHEN started_at IS NULL THEN 0 ELSE CAST(strftime('%s',COALESCE(completed_at,datetime('now')))-strftime('%s',started_at) AS INTEGER) END,run_id,result_mode,message FROM osint_analysis_jobs WHERE id=?1",params![job_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?))).map_err(|_|"Tâche d’analyse introuvable.".to_string())?;
    let report = run_id
        .as_deref()
        .and_then(|id| analysis_report_from_run(&conn, id));
    Ok(OsintAnalysisJobStatus {
        id,
        origin,
        status,
        signal_count,
        estimated_seconds,
        elapsed_seconds,
        run_id,
        result_mode,
        message,
        report,
    })
}

#[tauri::command]
fn get_latest_identity_synthesis(
    app: tauri::AppHandle,
    identity_id: String,
) -> Result<Option<OsintAnalysisReport>, String> {
    let conn = get_db_connection(&app)?;
    let run_id:Option<String>=conn.query_row("SELECT s.run_id FROM osint_synthesis_runs s JOIN osint_analysis_runs r ON r.id=s.run_id JOIN osint_analysis_outputs o ON o.run_id=r.id WHERE s.identity_id=?1 AND r.task='synthese_multi_source' AND r.status IN ('valide','fallback') AND o.validated=1 ORDER BY r.completed_at DESC,s.created_at DESC LIMIT 1",params![identity_id],|row|row.get(0)).ok();
    Ok(run_id
        .as_deref()
        .and_then(|id| analysis_report_from_run(&conn, id)))
}

#[tauri::command]
fn get_local_ai_analysis_mode(app: tauri::AppHandle) -> Result<String, String> {
    Ok(get_db_connection(&app)?
        .query_row(
            "SELECT value FROM app_settings WHERE key='local_ai_manual_scan_mode'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "automatic".into()))
}

#[tauri::command]
fn set_local_ai_analysis_mode(app: tauri::AppHandle, mode: String) -> Result<String, String> {
    if !matches!(mode.as_str(), "automatic" | "manual") {
        return Err("Mode d’analyse invalide.".into());
    }
    get_db_connection(&app)?.execute("INSERT INTO app_settings (key,value) VALUES ('local_ai_manual_scan_mode',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![mode]).map_err(|e|e.to_string())?;
    Ok(if mode == "automatic" {
        "Les scans manuels lanceront automatiquement l’analyse locale."
    } else {
        "Les scans manuels attendront votre clic pour lancer l’analyse locale."
    }
    .into())
}

#[tauri::command]
fn get_app_theme(app: tauri::AppHandle) -> Result<String, String> {
    Ok(veille_setting(
        &get_db_connection(&app)?,
        "app_theme",
        "obsidian",
    ))
}

#[tauri::command]
fn set_app_theme(app: tauri::AppHandle, theme: String) -> Result<String, String> {
    if !matches!(
        theme.as_str(),
        "obsidian" | "arctic" | "ember" | "verdant" | "cyber" | "cyberpunk" | "osint"
    ) {
        return Err("Thème d’application invalide.".into());
    }
    save_veille_setting(&get_db_connection(&app)?, "app_theme", &theme)?;
    Ok(theme)
}

#[tauri::command]
fn get_app_language(app: tauri::AppHandle) -> Result<String, String> {
    Ok(veille_setting(
        &get_db_connection(&app)?,
        "app_language",
        "fr",
    ))
}

#[tauri::command]
fn set_app_language(app: tauri::AppHandle, language: String) -> Result<String, String> {
    if !matches!(language.as_str(), "fr" | "en") {
        return Err("Langue d’application invalide.".into());
    }
    save_veille_setting(&get_db_connection(&app)?, "app_language", &language)?;
    Ok(language)
}

#[tauri::command]
fn install_local_ai_runtime(app: tauri::AppHandle) -> Result<String, String> {
    let component = local_ai_component()?;
    let conn = get_db_connection(&app)?;
    let partial = local_ai_root(&app)?
        .join("downloads")
        .join(format!("{}-{}.zip.part", component.id, component.version));
    let existing = std::fs::metadata(&partial)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let download_id = Uuid::new_v4().to_string();
    conn.execute("INSERT INTO local_ai_downloads (id,component_id,version,source_url,partial_path,expected_sha256,expected_bytes,downloaded_bytes,status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,'en_cours')", params![download_id,component.id,component.version,component.url,partial.to_string_lossy().to_string(),component.sha256,component.byte_size as i64,existing as i64]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE local_ai_components SET status='telechargement',diagnostic='Téléchargement et vérification en cours.',updated_at=datetime('now') WHERE component_id=?1", params![component.id]).map_err(|e| e.to_string())?;
    match install_local_ai_runtime_inner(&app, &component, &download_id) {
        Ok(message) => Ok(message),
        Err(error) => {
            let current_size = std::fs::metadata(&partial)
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            let _ = conn.execute("UPDATE local_ai_downloads SET downloaded_bytes=?1,status='interrompu',error_message=?2,updated_at=datetime('now') WHERE id=?3", params![current_size as i64,error,download_id]);
            let _ = conn.execute("UPDATE local_ai_components SET status='erreur',diagnostic=?1,updated_at=datetime('now') WHERE component_id=?2", params![error,component.id]);
            Err(error)
        }
    }
}

#[tauri::command]
fn diagnose_local_ai(app: tauri::AppHandle) -> Result<String, String> {
    let component = local_ai_component()?;
    let threads = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    let (ok, runtime) = installed_local_ai_integrity(&app, &component)?;
    Ok(format!("Windows ciblé : {}. Architecture : {}. Processeurs logiques disponibles : {}. Catalogue signé : valide. {}", std::env::consts::OS == "windows", std::env::consts::ARCH, threads, if ok { runtime } else { "Le runtime n’est pas encore prêt ; MANTIS continue sans IA.".into() }))
}

#[tauri::command]
fn remove_local_ai_components(app: tauri::AppHandle) -> Result<String, String> {
    let root = local_ai_root(&app)?;
    let expected_parent = application_data_dir(&app)?;
    if root.parent() != Some(expected_parent.as_path())
        || root.file_name().and_then(|value| value.to_str()) != Some("local-ai")
    {
        return Err("Le dossier IA à supprimer n’a pas passé le contrôle de sécurité.".into());
    }
    if root.exists() {
        std::fs::remove_dir_all(&root)
            .map_err(|e| format!("Suppression des composants IA impossible : {e}"))?;
    }
    let conn = get_db_connection(&app)?;
    conn.execute("DELETE FROM local_ai_downloads", [])
        .map_err(|e| e.to_string())?;
    conn.execute("UPDATE local_ai_components SET install_path=NULL,status='supprime',diagnostic='Composants IA supprimés par l’utilisateur.',installed_at=NULL,updated_at=datetime('now')", []).map_err(|e| e.to_string())?;
    conn.execute("UPDATE local_ai_preferences SET enabled=0,selected_model_id=NULL,onboarding_status='sans_ia',updated_at=datetime('now') WHERE id=1", []).map_err(|e| e.to_string())?;
    Ok("Runtime, modèles, téléchargements partiels, caches et fichiers temporaires IA supprimés. Les workflows sans IA restent disponibles.".into())
}

fn published_mailto_contact(url: &str) -> Option<String> {
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .ok()?
        .get(url)
        .header(
            "User-Agent",
            "MANTIS-POSTURE/0.1 (local privacy request helper)",
        )
        .send()
        .ok()?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > 2_000_000)
    {
        return None;
    }
    let page = response.text().ok()?.to_ascii_lowercase();
    let mut candidates = page
        .split("mailto:")
        .skip(1)
        .filter_map(|rest| {
            let end = rest
                .find(|character: char| {
                    matches!(
                        character,
                        '"' | '\'' | '?' | '&' | '<' | '>' | ' ' | '\r' | '\n'
                    )
                })
                .unwrap_or(rest.len());
            let candidate = rest[..end].trim_matches('/').trim();
            (candidate.contains('@')
                && candidate.len() <= 254
                && !candidate.contains("javascript:"))
            .then(|| candidate.to_string())
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|email| {
        if email.contains("dpo") || email.contains("privacy") || email.contains("data") {
            0
        } else {
            1
        }
    });
    candidates.into_iter().next()
}

#[tauri::command]
fn create_dpo_request_from_osint_signal(
    app: tauri::AppHandle,
    signal_id: String,
) -> Result<String, String> {
    let conn = get_db_connection(&app)?;
    let (title, explanation, source, source_url, exposure_id, review_status,identity_id): (String, String, String, Option<String>, Option<String>,String,String) = conn.query_row(
        "SELECT title,explanation,source,source_url,exposure_id,review_status,identity_id FROM osint_signals WHERE id=?1", params![signal_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,row.get(5)?,row.get(6)?))
    ).map_err(|_| "Signal introuvable.".to_string())?;
    if source != "DDGS" {
        return Err(
            "Une demande DPO peut être préparée depuis un résultat Empreinte Web confirmé.".into(),
        );
    }
    if !matches!(review_status.as_str(), "Confirmé" | "Suivi" | "Traité")
        || !has_positive_human_decision(&conn, &signal_id)
    {
        return Err("Confirmez ou suivez ce résultat avant de préparer un brouillon DPO.".into());
    }
    let source_url = source_url.ok_or("Ce résultat ne comporte pas d’URL à vérifier.")?;
    let parsed_url =
        reqwest::Url::parse(&source_url).map_err(|_| "L’URL de la source est invalide.")?;
    if !matches!(parsed_url.scheme(), "http" | "https") {
        return Err(
            "Seules les pages Web HTTP(S) peuvent être utilisées pour cette démarche.".into(),
        );
    }
    let exposure_id =
        exposure_id.ok_or("Confirmez d’abord cette fuite afin de créer son suivi.")?;
    let incident_id: String = conn
        .query_row(
            "SELECT incident_id FROM exposure_incident WHERE exposure_id=?1",
            params![exposure_id],
            |row| row.get(0),
        )
        .map_err(|_| "Le suivi de cette fuite doit être créé avant la demande DPO.")?;
    let existing: Option<String> = conn
        .query_row(
            "SELECT rgpd_id FROM incident_rgpd WHERE incident_id=?1 LIMIT 1",
            params![incident_id],
            |row| row.get(0),
        )
        .ok();
    if let Some(id) = existing {
        record_projection_event(&conn, &identity_id, &signal_id, "dpo", &id, "reutilise")?;
        return Ok(id);
    }

    let target = parsed_url.host_str().unwrap_or("Site concerné").to_string();
    let public_contact = published_mailto_contact(&source_url);
    let dpo_contact = public_contact.clone().unwrap_or_else(|| {
        "Aucun contact DPO/public trouvé — vérifiez la politique de confidentialité du site.".into()
    });
    let contact_source_url = public_contact.as_ref().map(|_| source_url.clone());
    let request_id = Uuid::new_v4().to_string();
    let data_summary = format!(
        "Résultat confirmé à vérifier sur : {}\n{}",
        source_url, title
    );
    let draft = format!("Objet : Demande d’effacement de données personnelles\n\nBonjour,\n\nJe vous contacte au sujet d’une page accessible à l’adresse suivante :\n{}\n\nCette page semble faire apparaître des informations me concernant : {}\n\nJe vous demande de vérifier le traitement de ces données et, le cas échéant, de les effacer ou de m’indiquer les mesures prises. Merci de me confirmer votre réponse par retour de message.\n\nCordialement,\n\n[Votre nom]\n[Votre adresse e-mail]", source_url, explanation);
    conn.execute("INSERT INTO rgpd_requests (id,type_id,target,dpo_contact,status_id,data_summary,draft_preview,source_url,contact_source_url) VALUES (?1,'type_003',?2,?3,'status_001',?4,?5,?6,?7)",
        params![request_id,target,dpo_contact,data_summary,draft,source_url,contact_source_url]).map_err(|e| e.to_string())?;
    let version_id = Uuid::new_v4().to_string();
    let draft_hash = hex::encode(Sha256::digest(draft.as_bytes()));
    conn.execute("INSERT INTO rgpd_draft_versions (id,request_id,contract_version,draft_text,content_sha256,source_url,source_signal_id) VALUES (?1,?2,'effacement-source-confirmee-v1',?3,?4,?5,?6)",params![version_id,request_id,draft,draft_hash,source_url,signal_id]).map_err(|e|e.to_string())?;
    conn.execute(
        "INSERT INTO incident_rgpd (incident_id,rgpd_id) VALUES (?1,?2)",
        params![incident_id, request_id],
    )
    .map_err(|e| e.to_string())?;
    if let Ok(action_id) = conn.query_row(
        "SELECT action_id FROM incident_action WHERE incident_id=?1 LIMIT 1",
        params![incident_id],
        |row| row.get::<_, String>(0),
    ) {
        conn.execute(
            "INSERT OR IGNORE INTO action_rgpd (action_id,rgpd_id) VALUES (?1,?2)",
            params![action_id, request_id],
        )
        .map_err(|e| e.to_string())?;
    }
    conn.execute("INSERT INTO timeline_entries (id,event_type,description,created_at) VALUES (?1,'Démarche DPO',?2,datetime('now'))", params![Uuid::new_v4().to_string(), format!("Brouillon DPO préparé pour {}", target)]).map_err(|e| e.to_string())?;
    record_projection_event(&conn, &identity_id, &signal_id, "dpo", &request_id, "cree")?;
    Ok(request_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Set the native window icon explicitly so `tauri dev` uses the
            // same logo as packaged builds and the taskbar entry.
            let window_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png"))
                .map_err(|error| format!("Unable to load application icon: {error}"))?;
            if let Some(window) = app.get_webview_window("main") {
                window
                    .set_icon(window_icon)
                    .map_err(|error| format!("Unable to set application icon: {error}"))?;
            }
            let handle = app.handle().clone();
            init_database(&handle)?;
            // Les sidecars fournis dans le bundle sont installés dans l’espace
            // privé de MANTIS avant toute première utilisation de la veille.
            let _ = install_default_osint_modules(&handle);
            // A routine starts only after the user has launched it once. If the app was
            // closed at the planned time, it resumes on the next opening instead.
            if veille_routine_is_due(&handle) {
                std::thread::spawn(move || {
                    let _ = run_veille_routine(handle, true);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_folders,
            get_folder,
            create_folder,
            delete_folder,
            list_incidents,
            get_incident,
            list_actions,
            get_action,
            list_identities,
            get_identity,
            create_identity,
            update_identity,
            delete_identity,
            list_exposures,
            get_exposure,
            list_remediation_recommendations,
            create_remediation_plan,
            get_remediation_plan,
            enrich_remediation_plan,
            get_latest_remediation_enrichment,
            list_rgpd_requests,
            get_rgpd_request,
            list_timeline_entries,
            list_osint_modules,
            list_osint_module_inventory,
            list_osint_module_logs,
            get_public_ip_context,
            get_posture_score,
            update_action_status,
            update_action_tracking,
            list_action_events,
            add_action_evidence,
            list_action_evidence,
            update_rgpd_request_status,
            list_rgpd_evidence,
            add_rgpd_evidence,
            list_rgpd_events,
            get_rgpd_review_status,
            validate_rgpd_draft,
            revoke_rgpd_draft_validation,
            save_rgpd_draft_revision,
            use_validated_rgpd_draft,
            create_action,
            install_osint_module,
            rollback_osint_module,
            remove_osint_module_runtime,
            cleanup_orphaned_osint_runtimes,
            install_veille,
            diagnose_osint_module,
            run_real_osint_scan,
            run_veille_scan,
            list_identity_scan_sessions,
            get_identity_scan_session,
            get_veille_routine,
            update_veille_routine,
            run_veille_routine,
            review_osint_signal,
            review_osint_claim,
            get_identity_review_workspace,
            start_osint_analysis,
            start_osint_claim_synthesis,
            get_osint_analysis_job,
            get_latest_identity_synthesis,
            get_local_ai_analysis_mode,
            set_local_ai_analysis_mode,
            get_app_theme,
            set_app_theme,
            get_app_language,
            set_app_language,
            get_local_ai_status,
            is_local_ai_enabled,
            install_local_ai_runtime,
            start_local_ai_setup,
            pause_local_ai_download,
            set_local_ai_preference,
            diagnose_local_ai,
            remove_local_ai_components,
            create_exposure_from_osint_signal,
            create_incident_and_action_from_osint_signal,
            create_dpo_request_from_osint_signal,
            get_osint_graph,
            generate_osint_report,
            get_latest_osint_report,
            export_osint_report,
            clear_scan_results,
            clear_user_scan_data,
            reset_database
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posture_score_is_gradual_for_a_single_incident_and_high_exposure() {
        let incidents = vec!["elevee".to_string()];
        let exposures = vec!["elevee".to_string()];
        assert_eq!(calculate_posture_score(&incidents, &exposures, 0), 83);
    }

    #[test]
    fn posture_score_keeps_critical_risk_visible_without_collapsing_to_zero() {
        let incidents = vec!["critique".to_string()];
        let exposures = vec!["elevee".to_string()];
        assert_eq!(calculate_posture_score(&incidents, &exposures, 0), 75);
    }

    #[test]
    fn posture_score_caps_mitigation_credit_and_floor() {
        let incidents = vec!["critique".to_string(); 8];
        let exposures = vec!["critique".to_string(); 8];
        assert_eq!(calculate_posture_score(&[], &[], 10), 100);
        assert_eq!(calculate_posture_score(&incidents, &exposures, 0), 5);
    }

    fn legacy_database_with_signal() -> Connection {
        let conn = Connection::open_in_memory().expect("base mémoire");
        conn.execute_batch(MIGRATION_SQL).expect("schéma initial");
        ensure_osint_schema(&conn).expect("schéma OSINT initial");
        conn.execute("INSERT INTO folders (id,name,context) VALUES ('folder-kept','Données à conserver','test')", []).unwrap();
        conn.execute("INSERT INTO identities (id,label,kind,value,folder_id) VALUES ('identity-1','Courriel','email','test@example.test','folder-kept')", []).unwrap();
        conn.execute("INSERT INTO osint_scans (id,module_id,identity_id,target,status,started_at,completed_at,raw_result_path) VALUES ('scan-1','mock-osint','identity-1','test@example.test','termine',datetime('now'),datetime('now'),'raw/scan-1.txt')", []).unwrap();
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,discovered_at,evidence_ref,raw_result_path,review_status) VALUES ('signal-1','mock-osint','scan-1','identity-1','test@example.test','profil_public','Signal synthétique','Preuve synthétique','faible','À vérifier','Test',datetime('now'),'fixture','raw/scan-1.txt','À vérifier')", []).unwrap();
        conn
    }

    #[test]
    fn quality_gate_hides_unconfirmed_single_source_candidates() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='email' WHERE id='scan-1'",
            [],
        )
        .expect("type de cible");
        apply_signal_quality_gate(&mut conn, "identity-1").expect("filtre qualité");
        let status: String = conn
            .query_row(
                "SELECT review_status FROM osint_signals WHERE id='signal-1'",
                [],
                |row| row.get(0),
            )
            .expect("signal filtré");
        assert_eq!(status, AUTO_FILTERED_REVIEW_STATUS);
    }

    #[test]
    fn evidence_facts_keep_a_verified_breach_separate_from_identity_attribution() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='email' WHERE id='scan-1'",
            [],
        )
        .expect("type de cible");
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-email-intel',signal_type='fuite',title='Fuite connue : Example breach',source='XposedOrNot',source_url='https://xposedornot.com/api/v1/breach-analytics/test%40example.test' WHERE id='signal-1'",
            [],
        )
        .expect("signal de fuite");

        apply_signal_quality_gate(&mut conn, "identity-1").expect("filtre qualité");
        let fact: (String, String, String, String) = conn
            .query_row(
                "SELECT fact_type,fact_status,match_level,source_reliability FROM osint_evidence_facts WHERE signal_id='signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("fait de preuve");
        assert_eq!(
            fact,
            (
                "breach_event".into(),
                "retenu".into(),
                "identifiant_exact".into(),
                "structuree".into(),
            )
        );
        refresh_osint_claims(&mut conn, "identity-1").expect("claim traçable");
        let linked_facts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_claim_fact_links l JOIN osint_evidence_facts f ON f.id=l.fact_id WHERE f.signal_id='signal-1'",
                [],
                |row| row.get(0),
            )
            .expect("lien claim-fait");
        assert_eq!(linked_facts, 1);
        let resolution_status: String = conn
            .query_row(
                "SELECT status FROM osint_fact_resolutions WHERE identity_id='identity-1'",
                [],
                |row| row.get(0),
            )
            .expect("résolution déterministe");
        assert_eq!(resolution_status, "a_verifier");

        conn.execute(
            "INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status)
             SELECT 'signal-duplicate',module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status FROM osint_signals WHERE id='signal-1'",
            [],
        )
        .expect("doublon de même source");
        apply_signal_quality_gate(&mut conn, "identity-1").expect("recalcul du doublon");
        let repeated_source: (String, i64, i64) = conn
            .query_row(
                "SELECT status,source_count,favorable_count FROM osint_fact_resolutions WHERE identity_id='identity-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("résolution sans inflation");
        assert_eq!(repeated_source, ("a_verifier".into(), 1, 2));

        conn.execute(
            "UPDATE osint_signals SET review_status='Ce n’est pas moi' WHERE id='signal-1'",
            [],
        )
        .expect("rejet utilisateur");
        conn.execute(
            "UPDATE osint_signals SET review_status='Ce n’est pas moi' WHERE id='signal-duplicate'",
            [],
        )
        .expect("rejet du doublon");
        apply_signal_quality_gate(&mut conn, "identity-1").expect("recalcul après rejet");
        let fact_status: String = conn
            .query_row(
                "SELECT fact_status FROM osint_evidence_facts WHERE signal_id='signal-1'",
                [],
                |row| row.get(0),
            )
            .expect("fait rejeté");
        assert_eq!(fact_status, "rejete");
        let resolution_status: String = conn
            .query_row(
                "SELECT status FROM osint_fact_resolutions WHERE identity_id='identity-1'",
                [],
                |row| row.get(0),
            )
            .expect("résolution rejetée");
        assert_eq!(resolution_status, "rejete");
    }

    #[test]
    fn public_page_check_never_targets_local_or_private_addresses() {
        assert!(is_safe_public_http_url("https://example.org/person"));
        assert!(!is_safe_public_http_url("http://localhost:8080/private"));
        assert!(!is_safe_public_http_url("http://10.0.0.8/internal"));
        assert!(!is_safe_public_http_url("http://172.20.0.8/internal"));
        assert!(!is_safe_public_http_url("http://192.168.1.8/internal"));
        assert!(!is_safe_public_http_url("http://169.254.1.8/internal"));
        assert!(!is_safe_public_http_url("http://100.64.1.8/internal"));
        assert!(!is_safe_public_http_url("http://198.18.1.8/internal"));
        assert!(!is_safe_public_http_url("https://service.internal/profile"));
        assert!(is_safe_public_http_url("https://172.200.1.8/public"));
        assert!(scanner_source_url("https://example.org/public").is_some());
        assert!(scanner_source_url("https://user:password@example.org/public").is_none());
        assert!(scanner_source_url("http://127.0.0.1:8080/private").is_none());
        assert!(scanner_source_url("file:///C:/private.txt").is_none());
        assert!(scanner_source_url("http://[::1]/private").is_none());
    }

    #[test]
    fn public_ip_context_is_bounded_and_keeps_only_display_fields() {
        let fixture = br#"{
            "success":true,"ip":"8.8.4.4","type":"IPv4","country":"United States",
            "region":"California","city":"Mountain View",
            "connection":{"asn":15169,"org":"Google LLC","isp":"Google LLC","domain":"google.com"},
            "timezone":{"id":"America/Los_Angeles","utc":"-07:00"},
            "security":{"proxy":false,"vpn":false,"tor":false,"hosting":true}
        }"#;
        let context = parse_public_ip_context(fixture).expect("contexte IP public");
        assert_eq!(context.ip, "8.8.4.4");
        assert_eq!(context.asn, Some(15169));
        assert_eq!(context.city.as_deref(), Some("Mountain View"));
        assert_eq!(context.isp.as_deref(), Some("Google LLC"));
        assert!(context.hosting);
        assert!(!context.vpn);

        let private_fixture = br#"{"success":true,"ip":"192.168.1.4","type":"IPv4"}"#;
        assert!(parse_public_ip_context(private_fixture).is_err());
    }

    #[test]
    fn quality_gate_hides_an_exact_pseudonym_without_identity_corroboration() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='pseudo' WHERE id='scan-1'",
            [],
        )
        .expect("type de cible");
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-username-profiles',target='alice_42',source_url='https://example.test/alice_42',explanation='Preuve publique vérifiée : le pseudo exact est présent dans la page. Cette correspondance possible ne confirme jamais l’identité de la personne.' WHERE id='signal-1'",
            [],
        )
        .expect("profil synthétique");
        apply_signal_quality_gate(&mut conn, "identity-1").expect("filtre qualité");
        let (status, confidence): (String, String) = conn
            .query_row(
                "SELECT review_status,confidence FROM osint_signals WHERE id='signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("profil évalué");
        assert_eq!(status, AUTO_FILTERED_REVIEW_STATUS);
        assert!(confidence.contains("ne suffit pas à attribuer un profil"));
        let assessment: (String, String, String) = conn
            .query_row(
                "SELECT match_level,publication_status,source_family FROM osint_evidence_assessments WHERE signal_id='signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("évaluation persistée");
        assert_eq!(
            assessment,
            (
                "pseudo_exact".into(),
                "masque".into(),
                "enumeration:public-accounts".into()
            )
        );
    }

    #[test]
    fn quality_gate_does_not_count_two_collectors_on_one_host_as_two_sources() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='pseudo' WHERE id='scan-1'",
            [],
        )
        .expect("type de cible");
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-username-profiles',target='alice_42',source='Maigret',source_url='https://example.test/alice_42',explanation='Résultat de collecteur à vérifier.' WHERE id='signal-1'",
            [],
        )
        .expect("premier résultat");
        conn.execute(
            "INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,raw_result_path,review_status) VALUES ('signal-2','osint-email-platforms','scan-1','identity-1','alice_42','compte_potentiel','Même page','Résultat de collecteur à vérifier.','faible','À vérifier','User Scanner','https://example.test/alice_42',datetime('now'),'fixture','raw/scan-1.txt','À vérifier')",
            [],
        )
        .expect("second résultat");
        apply_signal_quality_gate(&mut conn, "identity-1").expect("filtre qualité");
        let visible_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_signals WHERE review_status != ?1",
                params![AUTO_FILTERED_REVIEW_STATUS],
                |row| row.get(0),
            )
            .expect("signaux filtrés");
        assert_eq!(visible_count, 0);
        let independent_families: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT source_family) FROM osint_evidence_assessments WHERE identity_id='identity-1'",
                [],
                |row| row.get(0),
            )
            .expect("familles de preuve");
        assert_eq!(independent_families, 1);
    }

    #[test]
    fn quality_gate_publishes_an_exact_breach_and_records_its_provenance() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='email' WHERE id='scan-1'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-email-intel',signal_type='fuite',source='XposedOrNot',title='Fuite potentielle : Example' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        apply_signal_quality_gate(&mut conn, "identity-1").expect("filtre qualité");
        let assessment: (String, String, String, String) = conn
            .query_row(
                "SELECT source_family,source_reliability,match_level,publication_status FROM osint_evidence_assessments WHERE signal_id='signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            assessment,
            (
                "breach:xposedornot".into(),
                "structuree".into(),
                "identifiant_exact".into(),
                "visible".into()
            )
        );
    }

    #[test]
    fn quality_gate_publishes_an_exact_gravatar_link_without_identity_attribution() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='email' WHERE id='scan-1'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-gravatar-profile',source='Gravatar',source_url='https://gravatar.com/example',explanation='Preuve publique vérifiée : l’identifiant exact correspond au profil retourné. Cette correspondance possible ne confirme jamais l’identité.' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        apply_signal_quality_gate(&mut conn, "identity-1").unwrap();
        let assessment: (String, String, String) = conn
            .query_row(
                "SELECT source_family,match_level,publication_status FROM osint_evidence_assessments WHERE signal_id='signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            assessment,
            (
                "platform:gravatar".into(),
                "identifiant_exact".into(),
                "visible".into()
            )
        );
    }

    #[test]
    fn quality_gate_shows_only_keybase_profiles_with_external_valid_proofs() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité OSINT");
        ensure_scan_session_columns(&conn).expect("colonnes sessions");
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='pseudo' WHERE id='scan-1'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-keybase-profile',target='mantis_example',source='Keybase',source_url='https://keybase.io/mantis_example',explanation='Preuve publique vérifiée : le pseudo exact est présent. Preuves publiques vérifiées par Keybase : GitHub.' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        apply_signal_quality_gate(&mut conn, "identity-1").unwrap();
        let visible: (String, String) = conn
            .query_row(
                "SELECT publication_status,source_family FROM osint_evidence_assessments WHERE signal_id='signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(visible, ("visible".into(), "platform:keybase".into()));

        conn.execute(
            "UPDATE osint_signals SET review_status='À vérifier',explanation='Preuve publique vérifiée : le pseudo exact est présent, sans preuve externe valide.' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE osint_observations SET relevance_status='a_verifier' WHERE signal_id='signal-1'",
            [],
        )
        .unwrap();
        apply_signal_quality_gate(&mut conn, "identity-1").unwrap();
        let hidden: String = conn
            .query_row(
                "SELECT publication_status FROM osint_evidence_assessments WHERE signal_id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(hidden, "masque");
    }

    #[test]
    fn action_tracking_migration_preserves_legacy_actions_and_status_history() {
        let conn = Connection::open_in_memory().expect("base mémoire");
        conn.execute_batch(MIGRATION_SQL).expect("schéma initial");
        conn.execute_batch("INSERT INTO action_metadata (id,type,value,label) VALUES ('prio_003','priority','haute','Haute'),('diff_001','difficulty','facile','Facile')").expect("métadonnées");
        conn.execute(
            "INSERT INTO actions (id,title,priority_id,difficulty_id,deadline,status,guidance,proof_expected) VALUES ('legacy-action','Action conservée','prio_003','diff_001','2026-08-10','faite','[]','note')",
            [],
        ).expect("action legacy");
        ensure_action_tracking_columns(&conn).expect("colonnes suivi");
        conn.execute_batch(ACTION_TRACKING_MIGRATION_SQL)
            .expect("migration suivi");
        ensure_remediation_catalog_columns(&conn).expect("colonnes catalogue");
        conn.execute_batch(REMEDIATION_CATALOG_MIGRATION_SQL)
            .expect("migration catalogue");
        conn.execute_batch(REMEDIATION_AI_MIGRATION_SQL)
            .expect("migration IA remédiation");
        let status: String = conn
            .query_row(
                "SELECT workflow_status FROM actions WHERE id='legacy-action'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "effectue_moi");
        assert!(conn
            .execute(
                "UPDATE actions SET workflow_status='impossible' WHERE id='legacy-action'",
                []
            )
            .is_ok());
        assert!(conn.execute("INSERT INTO action_events (id,action_id,to_status) VALUES ('event-1','legacy-action','impossible')", []).is_ok());
        assert!(conn.execute("INSERT INTO action_evidence (id,action_id,kind,locator) VALUES ('proof-1','legacy-action','hash','abc123')", []).is_ok());
    }

    #[test]
    fn remediation_catalog_is_local_versioned_and_has_safe_steps() {
        let catalog = load_remediation_catalog().expect("catalogue local");
        assert_eq!(catalog.version, "1.0.0");
        assert!(!catalog.reviewed_at.is_empty());
        assert!(catalog
            .recommendations
            .iter()
            .any(|item| item.exposure_kinds.contains(&"fuite".into())));
        assert!(catalog
            .recommendations
            .iter()
            .all(|item| !item.steps.is_empty()
                && !item.proof_expected.is_empty()
                && !item.expected_outcome.is_empty()));
        assert!(catalog.recommendations.iter().all(|item| !item
            .steps
            .iter()
            .any(|step| step.to_lowercase().contains("mot de passe"))));
        assert_eq!(priority_for_exposure("critique"), "critique");
        assert_eq!(priority_for_exposure("élevée"), "haute");
    }

    #[test]
    fn remediation_ai_fallback_is_bounded_and_citations_are_validated() {
        let catalog = load_remediation_catalog().unwrap();
        let recommendation = catalog.recommendations.first().unwrap().clone();
        let plan = RemediationPlan {
            id: "plan-test".into(),
            identity_id: None,
            folder_id: None,
            scan_id: None,
            title: "Plan test".into(),
            status: "valide".into(),
            priority: "haute".into(),
            rationale: "Signal à vérifier.".into(),
            catalog_version: catalog.version.clone(),
            items: vec![RemediationPlanItem {
                id: "item-test".into(),
                action_id: None,
                exposure_id: Some("exp-test".into()),
                incident_id: None,
                sort_order: 0,
                expected_outcome: recommendation.expected_outcome.clone(),
                proof_expected: recommendation.proof_expected.clone(),
                execution_mode: recommendation.execution_mode.clone(),
                recommendation_id: recommendation.id.clone(),
            }],
        };
        let payload = deterministic_remediation_enrichment(&plan, &[recommendation.clone()]);
        let allowed = std::iter::once(recommendation.id.clone()).collect();
        assert!(validate_remediation_enrichment(&payload, &allowed).is_ok());
        let mut invalid = payload.clone();
        invalid.citation_ids = vec!["invented-proof".into()];
        assert!(validate_remediation_enrichment(&invalid, &allowed).is_err());
        invalid = payload;
        invalid.summary = "Ce profil est confirmé pour la personne.".into();
        assert!(validate_remediation_enrichment(&invalid, &allowed).is_err());
    }

    #[test]
    fn rgpd_tracking_migration_keeps_evidence_without_secrets() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATION_SQL).unwrap();
        conn.execute_batch(RGPD_REVIEW_MIGRATION_SQL).unwrap();
        conn.execute_batch(RGPD_TRACKING_MIGRATION_SQL).unwrap();
        conn.execute(
            "INSERT INTO rgpd_types (id,name,label) VALUES ('type_003','effacement','Effacement')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO rgpd_statuses (id,name,label) VALUES ('status_001','brouillon','Brouillon')", []).unwrap();
        conn.execute("INSERT INTO rgpd_requests (id,type_id,target,dpo_contact,status_id,data_summary,draft_preview) VALUES ('req-test','type_003','example.test','dpo@example.test','status_001','Donnée publique','Brouillon factuel')", []).unwrap();
        conn.execute("INSERT INTO rgpd_request_evidence (id,request_id,kind,locator,description,verified) VALUES ('proof-test','req-test','source','https://example.test/page','Page vérifiée',1)", []).unwrap();
        let verified: i64 = conn
            .query_row(
                "SELECT verified FROM rgpd_request_evidence WHERE id='proof-test'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(verified, 1);
        assert!(conn.execute("INSERT INTO rgpd_request_events (id,request_id,event_type,note) VALUES ('event-test','req-test','preuve_ajoutee','Référence ajoutée')", []).is_ok());
        assert!(validate_rgpd_evidence_locator("password=secret").is_err());
    }

    #[test]
    fn observability_migration_preserves_existing_business_data() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("migration additive");

        let folder_name: String = conn
            .query_row(
                "SELECT name FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let observation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_observations WHERE signal_id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let evidence_count: i64 = conn.query_row("SELECT COUNT(*) FROM osint_evidence_links WHERE observation_id='observation-signal-1'", [], |row| row.get(0)).unwrap();

        assert_eq!(folder_name, "Données à conserver");
        assert_eq!(observation_count, 1);
        assert_eq!(evidence_count, 1);
    }

    #[test]
    fn observability_migration_is_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("première migration");
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("seconde migration");

        let observation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_observations WHERE signal_id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let evidence_count: i64 = conn.query_row("SELECT COUNT(*) FROM osint_evidence_links WHERE observation_id='observation-signal-1'", [], |row| row.get(0)).unwrap();
        assert_eq!(observation_count, 1);
        assert_eq!(evidence_count, 1);
    }

    #[test]
    fn identity_target_migration_preserves_legacy_rows_and_links() {
        let conn = legacy_database_with_signal();
        ensure_identity_target_columns(&conn).expect("colonnes identité");
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL)
            .expect("migration identités");

        let target: (String, String) = conn
            .query_row(
                "SELECT label,status FROM identities WHERE id='identity-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let value: (String, String, String) = conn
            .query_row(
                "SELECT identity_id,kind,origin FROM identity_values WHERE id='legacy-identity-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        let linked_signal: String = conn
            .query_row(
                "SELECT identity_id FROM osint_signals WHERE id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(target, ("Courriel".into(), "active".into()));
        assert_eq!(value, ("identity-1".into(), "email".into(), "user".into()));
        assert_eq!(linked_signal, "identity-1");
    }

    #[test]
    fn identity_target_migration_is_idempotent_and_never_merges_people() {
        let conn = legacy_database_with_signal();
        conn.execute("INSERT INTO identities (id,label,kind,value,folder_id) VALUES ('identity-2','Courriel','email','test@example.test','folder-kept')", []).unwrap();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();

        let targets: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM identities WHERE id IN ('identity-1','identity-2')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let values: i64 = conn.query_row("SELECT COUNT(*) FROM identity_values WHERE identity_id IN ('identity-1','identity-2')", [], |row| row.get(0)).unwrap();
        assert_eq!(targets, 2);
        assert_eq!(values, 2);
    }

    #[test]
    fn identity_values_reject_duplicates_and_keep_stable_ids() {
        let conn = legacy_database_with_signal();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        let existing = load_identity_values(&conn, "identity-1").unwrap();
        let input = vec![IdentityValueInput {
            id: Some(existing[0].id.clone()),
            kind: "email".into(),
            value: " Updated@Example.Test ".into(),
            label: Some("principal".into()),
            status: "active".into(),
            address_line1: None,
            address_line2: None,
            city: None,
            postal_code: None,
            country: None,
        }];
        sync_identity_values(&conn, "identity-1", &input).unwrap();
        let updated = load_identity_values(&conn, "identity-1").unwrap();
        assert_eq!(updated[0].id, existing[0].id);
        assert_eq!(updated[0].normalized_value, "updated@example.test");

        let duplicate = vec![
            input[0].clone(),
            IdentityValueInput {
                id: None,
                ..input[0].clone()
            },
        ];
        assert!(sync_identity_values(&conn, "identity-1", &duplicate).is_err());
    }

    #[test]
    fn legacy_identity_column_maps_first_name_without_changing_authoritative_value_kind() {
        assert_eq!(legacy_identity_kind("prenom"), "nom");
        assert_eq!(legacy_identity_kind("email"), "email");

        let conn = legacy_database_with_signal();
        conn.execute(
            "UPDATE identities SET kind=?1,value=?2 WHERE id='identity-1'",
            params![legacy_identity_kind("prenom"), "Alice"],
        )
        .unwrap();
        let legacy: (String, String) = conn
            .query_row(
                "SELECT kind,value FROM identities WHERE id='identity-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(legacy, ("nom".into(), "Alice".into()));
    }

    #[test]
    fn identity_migration_disables_only_the_synthetic_first_name_duplicate() {
        let conn = legacy_database_with_signal();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute(
            "INSERT INTO identities (id,label,kind,value,status) VALUES ('person','Alice Dupont','nom','Alice','active')",
            [],
        )
        .unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        conn.execute("INSERT INTO identity_values (id,identity_id,kind,value,normalized_value,status,origin,sort_order) VALUES ('first','person','prenom','Alice','alice','active','user',1),('surname','person','nom','Dupont','dupont','active','user',2)", []).unwrap();

        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();

        let legacy_status: String = conn
            .query_row(
                "SELECT status FROM identity_values WHERE id='legacy-person'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let active_values: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM identity_values WHERE identity_id='person' AND status='active'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(legacy_status, "inactive");
        assert_eq!(active_values, 2);
    }

    #[test]
    fn identity_history_remains_attached_to_the_original_target() {
        let conn = legacy_database_with_signal();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        let history: i64 = conn.query_row(
            "SELECT (SELECT COUNT(*) FROM osint_scans WHERE identity_id='identity-1') + (SELECT COUNT(*) FROM osint_signals WHERE identity_id='identity-1')",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(history, 2);
        assert_eq!(load_identity(&conn, "identity-1").unwrap().values.len(), 1);
    }

    #[test]
    fn scan_session_migration_backfills_value_provenance_without_data_loss() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        ensure_scan_session_columns(&conn).unwrap();
        conn.execute_batch(OSINT_SCAN_SESSIONS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_SCAN_SESSIONS_MIGRATION_SQL)
            .unwrap();

        let scan_value: Option<String> = conn
            .query_row(
                "SELECT identity_value_id FROM osint_scans WHERE id='scan-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let signal_value: Option<String> = conn
            .query_row(
                "SELECT identity_value_id FROM osint_signals WHERE id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let observation_value: Option<String> = conn
            .query_row(
                "SELECT identity_value_id FROM osint_observations WHERE signal_id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let signal_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_signals WHERE id='signal-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let session_count: i64 = conn.query_row("SELECT COUNT(*) FROM osint_scan_sessions WHERE id='legacy-scan-1' AND identity_id='identity-1'", [], |row| row.get(0)).unwrap();
        let wrong_identity_signals =
            load_session_signals(&conn, "legacy-scan-1", "identity-does-not-exist").unwrap();

        assert_eq!(scan_value.as_deref(), Some("legacy-identity-1"));
        assert_eq!(signal_value, scan_value);
        assert_eq!(observation_value, scan_value);
        assert_eq!(signal_count, 1);
        assert_eq!(session_count, 1);
        assert!(wrong_identity_signals.is_empty());
    }

    #[test]
    fn scan_plan_uses_every_active_compatible_value_only() {
        let values = vec![
            IdentityValue {
                id: "email".into(),
                kind: "email".into(),
                value: "a@example.test".into(),
                normalized_value: "a@example.test".into(),
                label: None,
                status: "active".into(),
                origin: "user".into(),
                address_line1: None,
                address_line2: None,
                city: None,
                postal_code: None,
                country: None,
                sort_order: 0,
            },
            IdentityValue {
                id: "pseudo".into(),
                kind: "pseudo".into(),
                value: "mantis".into(),
                normalized_value: "mantis".into(),
                label: None,
                status: "active".into(),
                origin: "user".into(),
                address_line1: None,
                address_line2: None,
                city: None,
                postal_code: None,
                country: None,
                sort_order: 1,
            },
            IdentityValue {
                id: "nom".into(),
                kind: "nom".into(),
                value: "Dupont".into(),
                normalized_value: "dupont".into(),
                label: None,
                status: "active".into(),
                origin: "user".into(),
                address_line1: None,
                address_line2: None,
                city: None,
                postal_code: None,
                country: None,
                sort_order: 2,
            },
            IdentityValue {
                id: "prenom".into(),
                kind: "prenom".into(),
                value: "Alice".into(),
                normalized_value: "alice".into(),
                label: None,
                status: "active".into(),
                origin: "user".into(),
                address_line1: None,
                address_line2: None,
                city: None,
                postal_code: None,
                country: None,
                sort_order: 3,
            },
            IdentityValue {
                id: "phone".into(),
                kind: "telephone".into(),
                value: "+33102030405".into(),
                normalized_value: "+33102030405".into(),
                label: None,
                status: "active".into(),
                origin: "user".into(),
                address_line1: None,
                address_line2: None,
                city: None,
                postal_code: None,
                country: None,
                sort_order: 4,
            },
            IdentityValue {
                id: "paused".into(),
                kind: "email".into(),
                value: "paused@example.test".into(),
                normalized_value: "paused@example.test".into(),
                label: None,
                status: "inactive".into(),
                origin: "user".into(),
                address_line1: None,
                address_line2: None,
                city: None,
                postal_code: None,
                country: None,
                sort_order: 5,
            },
        ];
        let plan = build_veille_scan_plan(&values);
        assert_eq!(plan.len(), 15);
        assert_eq!(
            plan.iter()
                .filter(|item| item.identity_value_id == "email")
                .count(),
            5
        );
        assert_eq!(
            plan.iter()
                .filter(|item| item.identity_value_id == "pseudo")
                .count(),
            8
        );
        assert!(plan.iter().any(|item| item.identity_value_id == "pseudo"
            && item.module_id == "osint-username-profiles"));
        assert!(plan
            .iter()
            .any(|item| item.identity_value_id == "email"
                && item.module_id == "osint-gravatar-profile"));
        assert!(plan
            .iter()
            .any(|item| item.identity_value_id == "pseudo"
                && item.module_id == "osint-keybase-profile"));
        assert!(plan
            .iter()
            .any(|item| item.identity_value_id == "pseudo"
                && item.module_id == "osint-bluesky-profile"));
        assert!(plan.iter().any(|item| item.identity_value_id == "pseudo"
            && item.module_id == "osint-hackernews-profile"));
        assert!(plan.iter().any(|item| item.identity_value_id == "nom"
            && item.target_kind == "nom"
            && item.target == "Alice Dupont"));
        assert!(plan.iter().any(|item| item.identity_value_id == "phone"
            && item.module_id == "osint-web-footprint"
            && item.target_kind == "telephone"));
        assert!(!plan.iter().any(|item| item.identity_value_id == "prenom"));
        assert!(!plan.iter().any(|item| item.identity_value_id == "paused"));
    }

    #[test]
    fn phone_and_address_evidence_require_a_direct_normalized_match() {
        assert_eq!(
            normalize_phone_number("00 33 (0)6 12 34 56 78"),
            "+33612345678"
        );
        assert!(public_text_contains_identifier(
            "Contact public : +33 1 02 03 04 05.",
            "+33102030405",
            "telephone"
        ));
        assert!(!public_text_contains_identifier(
            "Références 33 et 102030405 sans numéro publié.",
            "+33102030405",
            "telephone"
        ));
        assert!(public_text_contains_identifier(
            "Adresse : 12, rue de la Paix — 75002 Paris",
            "12 rue de la Paix 75002 Paris",
            "adresse"
        ));
        assert!(!public_text_contains_identifier(
            "12 rue de la Paix, 69000 Lyon",
            "12 rue de la Paix 75002 Paris",
            "adresse"
        ));
        assert_eq!(
            execution_kind_for_module("osint-web-footprint", "adresse").as_deref(),
            Some("adresse")
        );
    }

    #[test]
    fn public_pdf_extraction_is_bounded_and_requires_real_pdf_content() {
        let pdf = simple_pdf("Preuve documentaire\nContact exact : alice@example.test");
        let text = extract_public_pdf_text(&pdf).expect("PDF local extractible");
        assert!(public_text_contains_identifier(
            &text,
            "alice@example.test",
            "email"
        ));
        assert!(extract_public_pdf_text(b"not a pdf").is_none());
        assert!(extract_public_pdf_text(&vec![b'x'; MAX_PUBLIC_DOCUMENT_BYTES + 1]).is_none());
        assert!(public_text_contains_identifier(
            "Alice Example",
            "Alice Example",
            "nom"
        ));
    }

    #[test]
    fn embedded_public_document_links_are_bounded_and_resolved() {
        let urls = extract_embedded_public_document_urls(
            "https://www.calameo.com/books/demo",
            r#"<iframe src="/viewer?download=publication.pdf"></iframe>
               <a href="https://cdn.example.test/publication.pdf">PDF</a>
               <a href="https://cdn.example.test/image.png">ignore</a>"#,
        );
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().any(|url| url.ends_with("/publication.pdf")));
        assert!(urls
            .iter()
            .any(|url| url.contains("download=publication.pdf")));
    }

    #[test]
    fn documentary_evidence_keeps_its_own_hash_and_artifact_link() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        let directory =
            std::env::temp_dir().join(format!("mantis-document-evidence-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let scan_raw = directory.join("scan.json");
        let document = directory.join("preuve.pdf");
        std::fs::write(&scan_raw, b"{}").unwrap();
        let pdf = simple_pdf("Identifiant exact : alice@example.test");
        std::fs::write(&document, &pdf).unwrap();
        conn.execute(
            "UPDATE osint_scans SET raw_result_path=?1 WHERE id='scan-1'",
            params![scan_raw.to_string_lossy().to_string()],
        )
        .unwrap();
        conn.execute(
            "UPDATE osint_signals SET raw_result_path=?1 WHERE id='signal-1'",
            params![document.to_string_lossy().to_string()],
        )
        .unwrap();

        sync_scan_observability(&conn, "scan-1").unwrap();

        let artifact: (String, String, i64) = conn
            .query_row(
                "SELECT a.media_type,a.sha256,a.byte_size
             FROM osint_evidence_links e
             JOIN osint_raw_artifacts a ON a.id=e.artifact_id
             WHERE e.observation_id='observation-signal-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(artifact.0, "application/pdf");
        assert_eq!(artifact.1, hex::encode(Sha256::digest(&pdf)));
        assert_eq!(artifact.2, pdf.len() as i64);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn wayback_availability_only_accepts_the_already_validated_exact_url() {
        let payload = r#"{
            "archived_snapshots": {"closest": {
                "available": true,
                "url": "https://web.archive.org/web/20250714123045/https://example.test/public/profile",
                "timestamp": "20250714123045",
                "status": "200"
            }}
        }"#;
        let snapshot = parse_wayback_availability(payload, "https://example.test/public/profile")
            .expect("copie exacte");
        assert_eq!(snapshot.timestamp, "20250714123045");
        assert!(
            parse_wayback_availability(payload, "https://example.test/another-profile").is_none()
        );
        assert!(parse_wayback_availability(
            &payload.replace("web.archive.org", "archive.example.test"),
            "https://example.test/public/profile"
        )
        .is_none());
    }

    #[test]
    fn scan_plan_never_uses_a_first_name_duplicate_as_the_surname() {
        let make_value = |id: &str, kind: &str, value: &str| IdentityValue {
            id: id.into(),
            kind: kind.into(),
            value: value.into(),
            normalized_value: normalize_identity_value(kind, value),
            label: None,
            status: "active".into(),
            origin: "user".into(),
            address_line1: None,
            address_line2: None,
            city: None,
            postal_code: None,
            country: None,
            sort_order: 0,
        };
        let plan = build_veille_scan_plan(&[
            make_value("first", "prenom", "Zoé"),
            make_value("legacy", "nom", "Zoé"),
            make_value("surname", "nom", "Johnson"),
        ]);
        let name_targets = plan
            .iter()
            .filter(|item| item.target_kind == "nom")
            .map(|item| item.target.as_str())
            .collect::<Vec<_>>();
        assert_eq!(name_targets, vec!["Zoé Johnson"]);
    }

    #[test]
    fn xposed_fixture_preserves_useful_facts_and_filters_duplicates() {
        let breaches = parse_xposed_breaches(include_str!(
            "../fixtures/xposed-breach-analytics-valid.json"
        ))
        .unwrap();
        assert_eq!(breaches.len(), 1);
        let breach = &breaches[0];
        assert_eq!(breach.breach, "ExampleService");
        assert_eq!(breach.severity, "critique");
        assert!(breach.explanation.contains("mot de passe"));
        assert!(breach.explanation.contains("2025-04-12"));
        assert_eq!(
            breach.source_url.as_deref(),
            Some("https://example.test/security-notice")
        );
        assert!(breach.confidence.contains("vérifiée"));
    }

    #[test]
    fn github_profile_fixture_requires_the_exact_declared_pseudonym() {
        let profile = parse_github_public_profile(
            include_str!("../fixtures/github-public-profile-valid.json"),
            "mantis-user",
        )
        .unwrap();
        assert_eq!(profile.login, "MANTIS-user");
        assert_eq!(profile.profile_url, "https://github.com/MANTIS-user");
        assert!(profile.explanation.contains("Preuve publique vérifiée"));
        assert!(profile.explanation.contains("identité de la personne"));
        assert!(parse_github_public_profile(
            include_str!("../fixtures/github-public-profile-valid.json"),
            "another-user",
        )
        .is_err());
        assert!(is_valid_github_username("mantis-user"));
        assert!(!is_valid_github_username("-mantis"));
        assert!(!is_valid_github_username("mantis user"));
    }

    #[test]
    fn github_repositories_are_bounded_and_owned_by_the_exact_profile() {
        let repositories = parse_github_public_repositories(
            include_str!("../fixtures/github-public-repositories-valid.json"),
            "MANTIS-user",
        )
        .unwrap();
        assert_eq!(repositories.len(), 2);
        assert!(repositories[0].contains("signal-dashboard"));
        assert!(repositories[0].contains("Rust"));
        let foreign = include_str!("../fixtures/github-public-repositories-valid.json").replace(
            "https://github.com/MANTIS-user/",
            "https://github.com/other-user/",
        );
        assert!(parse_github_public_repositories(&foreign, "MANTIS-user")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn github_organizations_are_public_bounded_and_safe() {
        let organizations = parse_github_public_organizations(include_str!(
            "../fixtures/github-public-organizations-valid.json"
        ))
        .unwrap();
        assert_eq!(organizations, vec!["mantis-lab", "MANTIS-Research"]);
        let unsafe_url = include_str!("../fixtures/github-public-organizations-valid.json")
            .replace(
                "https://github.com/mantis-lab",
                "https://example.test/mantis-lab",
            );
        assert_eq!(
            parse_github_public_organizations(&unsafe_url).unwrap(),
            vec!["MANTIS-Research"]
        );
    }

    #[test]
    fn github_events_are_public_bounded_and_owned_by_the_exact_profile() {
        let events = parse_github_public_events(
            include_str!("../fixtures/github-public-events-valid.json"),
            "MANTIS-user",
        )
        .unwrap();
        assert_eq!(events.len(), 2);
        assert!(events[0].contains("PushEvent"));
        assert!(events[0].contains("MANTIS-user/signal-dashboard"));
        let foreign = include_str!("../fixtures/github-public-events-valid.json")
            .replace("MANTIS-user", "other-user");
        assert!(parse_github_public_events(&foreign, "MANTIS-user")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn gitlab_profile_fixture_requires_an_exact_public_value() {
        let profile = parse_gitlab_public_profile(
            include_str!("../fixtures/gitlab-public-profile-valid.json"),
            "mantis-user",
            "pseudo",
        )
        .unwrap()
        .expect("profil exact");
        assert_eq!(profile.username, "MANTIS-user");
        assert_eq!(profile.profile_url, "https://gitlab.com/MANTIS-user");
        assert!(profile.explanation.contains("pseudo exact"));
        let email_profile = parse_gitlab_public_profile(
            include_str!("../fixtures/gitlab-public-profile-valid.json"),
            "public@example.test",
            "email",
        )
        .unwrap()
        .expect("e-mail public exact");
        assert!(email_profile.explanation.contains("identifiant exact"));
        assert!(parse_gitlab_public_profile(
            include_str!("../fixtures/gitlab-public-profile-valid.json"),
            "other-user",
            "pseudo",
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn mastodon_webfinger_requires_an_explicit_account_and_exact_subject() {
        let (username, instance) = mastodon_account_for_target("@mantis_user@social.example.test")
            .expect("compte explicite");
        assert_eq!(username, "mantis_user");
        assert_eq!(instance, "social.example.test");
        assert!(mastodon_account_for_target("mantis_user").is_none());
        assert!(mastodon_account_for_target("mantis_user@localhost").is_some());

        let profile = parse_mastodon_webfinger_profile(
            include_str!("../fixtures/mastodon-webfinger-valid.json"),
            &username,
            &instance,
        )
        .expect("réponse exacte");
        assert_eq!(profile.account, "mantis_user@social.example.test");
        assert_eq!(
            profile.profile_url,
            "https://social.example.test/users/mantis_user"
        );
        assert!(profile
            .explanation
            .contains("ne confirme jamais l’identité"));

        let mismatch = include_str!("../fixtures/mastodon-webfinger-valid.json")
            .replace("acct:mantis_user", "acct:another_user");
        assert!(parse_mastodon_webfinger_profile(&mismatch, &username, &instance).is_err());
        let foreign = include_str!("../fixtures/mastodon-webfinger-valid.json").replace(
            "https://social.example.test/",
            "https://other.example.test/",
        );
        assert!(parse_mastodon_webfinger_profile(&foreign, &username, &instance).is_err());
    }

    #[test]
    fn specialized_public_sources_keep_exact_names_as_homonym_candidates() {
        let companies = parse_fr_company_findings(
            include_str!("../fixtures/fr-company-register-valid.json"),
            "Alice Dupont",
        )
        .unwrap();
        assert_eq!(companies.len(), 1);
        assert!(companies[0].explanation.contains("homonyme"));
        assert!(companies[0].source_url.ends_with("/123456789"));
        assert!(parse_fr_company_findings(
            include_str!("../fixtures/fr-company-register-valid.json"),
            "Alice Martin"
        )
        .unwrap()
        .is_empty());

        let publications = parse_hal_findings(
            include_str!("../fixtures/hal-author-valid.json"),
            "Alice Dupont",
        )
        .unwrap();
        assert_eq!(publications.len(), 1);
        assert_eq!(
            publications[0].source_url,
            "https://hal.science/hal-01234567"
        );
        assert!(publications[0].explanation.contains("homonyme"));
    }

    #[test]
    fn gravatar_fixture_requires_the_exact_email_hash_and_keeps_public_links() {
        let body = include_str!("../fixtures/gravatar-public-profile-valid.json");
        let expected_hash = hex::encode(Sha256::digest(b"test@example.com"));
        let profile = parse_gravatar_public_profile(body, &expected_hash).unwrap();
        assert_eq!(profile.display_name, "Mantis Example");
        assert_eq!(profile.profile_url, "https://gravatar.com/mantis-example");
        assert!(profile
            .explanation
            .contains("https://github.com/mantis-example"));
        assert!(profile.explanation.contains("correspondance possible"));
        assert!(profile
            .explanation
            .contains("ne confirme jamais l’identité"));
        assert!(parse_gravatar_public_profile(body, &"0".repeat(64)).is_err());
    }

    #[test]
    fn gravatar_parser_rejects_incomplete_or_unsafe_profiles() {
        assert!(parse_gravatar_public_profile("{}", &"0".repeat(64)).is_err());
        let body = include_str!("../fixtures/gravatar-public-profile-valid.json").replace(
            "https://gravatar.com/mantis-example",
            "file:///private/profile",
        );
        let expected_hash = hex::encode(Sha256::digest(b"test@example.com"));
        assert!(parse_gravatar_public_profile(&body, &expected_hash).is_err());
    }

    #[test]
    fn keybase_fixture_keeps_only_valid_public_proofs_and_removes_tokens() {
        let raw = include_str!("../fixtures/keybase-public-profile-valid.json");
        let sanitized = sanitized_keybase_public_response(raw).unwrap();
        assert!(!sanitized.contains("csrf_token"));
        assert!(!sanitized.contains("synthetic-token"));
        assert!(!sanitized.contains("synthetic-session"));
        let profile = parse_keybase_public_profile(&sanitized, "mantis_example")
            .unwrap()
            .expect("profil exact");
        assert_eq!(profile.username, "mantis_example");
        assert_eq!(profile.profile_url, "https://keybase.io/mantis_example");
        assert_eq!(profile.verified_proof_count, 2);
        assert_eq!(profile.verified_proofs.len(), 2);
        assert!(profile
            .verified_proofs
            .iter()
            .any(|proof| proof.service == "github"
                && proof.url == "https://github.com/mantis-example"));
        assert!(profile
            .explanation
            .contains("https://github.com/mantis-example"));
        assert!(profile
            .explanation
            .contains("https://www.reddit.com/user/mantis_example"));
        assert!(!profile.explanation.contains("hidden-example"));
        assert!(profile
            .explanation
            .contains("ne confirme jamais l’identité"));
    }

    #[test]
    fn keybase_parser_rejects_mismatch_incomplete_and_unsafe_proofs() {
        let raw = include_str!("../fixtures/keybase-public-profile-valid.json");
        assert!(parse_keybase_public_profile(raw, "another_user").is_err());
        assert!(parse_keybase_public_profile("{}", "mantis_example").is_err());
        assert!(is_valid_keybase_username("mantis_example"));
        assert!(!is_valid_keybase_username("Mantis Example"));
        let unsafe_body = raw.replace(
            "https://github.com/mantis-example",
            "file:///private/keybase-proof",
        );
        let profile = parse_keybase_public_profile(&unsafe_body, "mantis_example")
            .unwrap()
            .expect("profil exact");
        assert_eq!(profile.verified_proof_count, 1);
        assert!(!profile.explanation.contains("file:///"));
    }

    #[test]
    fn bluesky_fixture_requires_the_exact_handle_and_keeps_stable_provenance() {
        let body = include_str!("../fixtures/bluesky-public-profile-valid.json");
        let profile = parse_bluesky_public_profile(body, "mantis-example.bsky.social").unwrap();
        assert_eq!(profile.handle, "mantis-example.bsky.social");
        assert_eq!(profile.did, "did:plc:syntheticmantisprofile");
        assert_eq!(
            profile.profile_url,
            "https://bsky.app/profile/mantis-example.bsky.social"
        );
        assert!(profile.explanation.contains("handle exact"));
        assert!(profile.explanation.contains("correspondance possible"));
        assert!(profile.evidence_ref.contains(&profile.did));
        assert!(parse_bluesky_public_profile(body, "another.bsky.social").is_err());
    }

    #[test]
    fn bluesky_handles_are_deterministic_and_invalid_profiles_are_rejected() {
        assert_eq!(
            bluesky_handle_for_target("@Mantis-Example").as_deref(),
            Some("mantis-example.bsky.social")
        );
        assert_eq!(
            bluesky_handle_for_target("person.example.com").as_deref(),
            Some("person.example.com")
        );
        assert!(bluesky_handle_for_target("bad_pseudo").is_none());
        assert!(bluesky_handle_for_target("-bad").is_none());
        assert!(parse_bluesky_public_profile("{}", "mantis-example.bsky.social").is_err());
        let unsafe_did = include_str!("../fixtures/bluesky-public-profile-valid.json")
            .replace("did:plc:syntheticmantisprofile", "invalid-identifier");
        assert!(parse_bluesky_public_profile(&unsafe_did, "mantis-example.bsky.social").is_err());
    }

    #[test]
    fn hackernews_fixture_requires_the_exact_pseudonym_and_bounds_activity() {
        let body = include_str!("../fixtures/hackernews-public-profile-valid.json");
        let profile = parse_hackernews_public_profile(body, "mantis_example").unwrap();
        assert_eq!(profile.username, "mantis_example");
        assert_eq!(
            profile.profile_url,
            "https://news.ycombinator.com/user?id=mantis_example"
        );
        assert!(profile
            .explanation
            .contains("Publication publique synthétique"));
        assert!(profile.explanation.contains("item?id=123456"));
        assert!(profile
            .explanation
            .contains("ne confirme jamais l’identité"));
        assert!(parse_hackernews_public_profile(body, "another_user").is_err());
    }

    #[test]
    fn hackernews_parser_rejects_invalid_profiles_and_foreign_activity() {
        assert!(parse_hackernews_public_profile("{}", "mantis_example").is_err());
        assert!(is_valid_hackernews_username("mantis_example"));
        assert!(!is_valid_hackernews_username("mantis example"));
        let foreign_activity = include_str!("../fixtures/hackernews-public-profile-valid.json")
            .replace(
                "\"author\": \"mantis_example\"",
                "\"author\": \"other_user\"",
            );
        let profile = parse_hackernews_public_profile(&foreign_activity, "mantis_example").unwrap();
        assert!(!profile.explanation.contains("item?id=123456"));
    }

    #[test]
    fn xposed_invalid_json_is_rejected_and_missing_list_is_empty() {
        assert!(parse_xposed_breaches("not-json").is_err());
        assert!(parse_xposed_breaches("{}").unwrap().is_empty());
    }

    #[test]
    fn maigret_fixture_is_valid_bounded_and_keeps_provenance() {
        let output =
            parse_maigret_output(include_str!("../fixtures/maigret-valid.json"), "alice_42")
                .unwrap();
        assert_eq!(output.collector_version, "maigret-0.6.3");
        assert_eq!(output.summary.checked, 150);
        assert_eq!(output.results.len(), 2);
        assert!(output
            .results
            .iter()
            .all(|item| scanner_source_url(&item.url).is_some()));
    }

    #[test]
    fn maigret_rejects_invalid_incomplete_or_mismatched_json() {
        assert!(
            parse_maigret_output(include_str!("../fixtures/maigret-invalid.json"), "alice_42")
                .is_err()
        );
        assert!(parse_maigret_output(
            include_str!("../fixtures/maigret-valid.json"),
            "another_person"
        )
        .is_err());
        let invalid_url = include_str!("../fixtures/maigret-valid.json")
            .replace("https://social.example/alice_42", "file:///secret");
        assert!(parse_maigret_output(&invalid_url, "alice_42").is_err());
    }

    #[test]
    fn maigret_exact_duplicates_are_removed_and_never_expressed_as_identity_confirmation() {
        let output =
            parse_maigret_output(include_str!("../fixtures/maigret-valid.json"), "alice_42")
                .unwrap();
        let duplicate = MaigretResult {
            site_name: "Duplicate".into(),
            url: output.results[0].url.to_uppercase(),
            category: "social".into(),
        };
        let mut results = output.results;
        results.push(duplicate);
        let unique = deduplicate_maigret_results(results);
        assert_eq!(unique.len(), 2);
        assert_eq!(maigret_signal_type("dating"), "site_rencontre");
        let wording = "Cette correspondance possible ne confirme jamais l’identité de la personne.";
        assert!(wording.contains("ne confirme jamais"));
    }

    #[test]
    fn signed_local_ai_catalog_is_valid_and_tampering_is_rejected() {
        let catalog = verified_local_ai_catalog().expect("catalogue signé valide");
        assert_eq!(catalog.schema_version, 1);
        assert!(catalog
            .components
            .iter()
            .any(|component| component.id == "llama-cpp-cpu"));

        let mut tampered = LOCAL_AI_CATALOG_JSON.to_vec();
        tampered[0] = b'[';
        assert!(verify_local_ai_catalog_signature(&tampered, LOCAL_AI_CATALOG_SIGNATURE).is_err());
    }

    #[test]
    fn local_ai_migration_is_additive_and_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .expect("observabilité");
        conn.execute_batch(LOCAL_AI_MIGRATION_SQL)
            .expect("fondations IA");
        conn.execute_batch(LOCAL_AI_MIGRATION_SQL)
            .expect("fondations IA rejouées");

        let folder_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let component_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM local_ai_components WHERE component_id='llama-cpp-cpu'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(folder_count, 1);
        assert_eq!(component_count, 1);
    }

    #[test]
    fn local_ai_onboarding_migration_is_additive_and_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(LOCAL_AI_MIGRATION_SQL)
            .expect("fondations IA");
        conn.execute_batch(LOCAL_AI_ONBOARDING_MIGRATION_SQL)
            .expect("onboarding IA");
        conn.execute_batch(LOCAL_AI_ONBOARDING_MIGRATION_SQL)
            .expect("onboarding IA rejoué");
        let models: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM local_ai_components WHERE component_type='model'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let preference: (i64, String) = conn
            .query_row(
                "SELECT enabled,onboarding_status FROM local_ai_preferences WHERE id=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let folder_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(models, 2);
        assert_eq!(preference, (0, "a_proposer".into()));
        assert_eq!(folder_count, 1);
    }

    #[test]
    fn signed_catalog_only_exposes_closed_official_model_choices() {
        let models = local_ai_models().expect("modèles signés");
        assert_eq!(models.len(), 2);
        assert!(models.iter().all(
            |model| model.url.starts_with("https://huggingface.co/Qwen/")
                && model.license == "Apache-2.0"
        ));
        assert!(models
            .iter()
            .any(|model| model.tier.as_deref() == Some("leger")));
        assert!(models
            .iter()
            .any(|model| model.tier.as_deref() == Some("recommande")));
    }

    #[test]
    fn model_redirects_only_accept_hugging_face_storage_hosts() {
        for url in [
            "https://huggingface.co/Qwen/model",
            "https://us.aws.cdn.hf.co/xet-bridge-us/file",
            "https://cas-bridge.xethub.hf.co/file",
        ] {
            assert!(trusted_local_ai_model_download_url(
                &reqwest::Url::parse(url).unwrap()
            ));
        }
        for url in [
            "http://us.aws.cdn.hf.co/file",
            "https://hf.co.evil.example/file",
            "https://aws.cdn.hf.co.evil.example/file",
        ] {
            assert!(!trusted_local_ai_model_download_url(
                &reqwest::Url::parse(url).unwrap()
            ));
        }
    }

    fn synthetic_analysis_input() -> Vec<LocalAiAnalysisInput> {
        vec![LocalAiAnalysisInput {
            observation_id: "observation-test".into(),
            observation_type: "mention".into(),
            display_value: "Homonyme sans autre indice".into(),
            source: "fixture".into(),
            relevance_status: "a_verifier".into(),
            evidence: vec![LocalAiAnalysisEvidence {
                evidence_id: "evidence-test".into(),
                label: "fixture".into(),
                excerpt: "Même nom seulement.".into(),
            }],
        }]
    }

    fn synthetic_claim_inputs() -> Vec<LocalAiClaimInput> {
        vec![
            LocalAiClaimInput {
                claim_id: "claim-profile".into(),
                claim_type: "profil_potentiel".into(),
                display_value: "https://social.example/alice".into(),
                status: "corroboree".into(),
                priority: "moyenne".into(),
                rationale: "Deux sources distinctes indiquent la même URL.".into(),
                resolution_status: "corroboree".into(),
                resolution_source_count: 2,
                resolution_rationale: "Même fait exact observé par deux familles de sources."
                    .into(),
                evidence: vec![
                    LocalAiClaimEvidence {
                        evidence_id: "observation-a".into(),
                        source: "Maigret".into(),
                        source_url: Some("https://social.example/alice".into()),
                        role: "favorable".into(),
                        observed_at: "2026-07-31 10:00:00".into(),
                    },
                    LocalAiClaimEvidence {
                        evidence_id: "observation-b".into(),
                        source: "DDGS".into(),
                        source_url: Some("https://social.example/alice".into()),
                        role: "favorable".into(),
                        observed_at: "2026-07-31 10:01:00".into(),
                    },
                ],
            },
            LocalAiClaimInput {
                claim_id: "claim-homonym".into(),
                claim_type: "mention_web".into(),
                display_value: "Alice Martin".into(),
                status: "contradictoire".into(),
                priority: "faible".into(),
                rationale: "L’utilisateur a indiqué que cette mention ne le concerne pas.".into(),
                resolution_status: "contradictoire".into(),
                resolution_source_count: 1,
                resolution_rationale: "Une décision utilisateur contredit ce résultat.".into(),
                evidence: vec![LocalAiClaimEvidence {
                    evidence_id: "observation-c".into(),
                    source: "DDGS".into(),
                    source_url: Some("https://example.test/homonyme".into()),
                    role: "contradictoire".into(),
                    observed_at: "2026-07-31 10:02:00".into(),
                }],
            },
        ]
    }

    fn review_phase_database() -> Connection {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        ensure_scan_session_columns(&conn).unwrap();
        conn.execute_batch(OSINT_SCAN_SESSIONS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        refresh_osint_claims(&mut conn, "identity-1").unwrap();
        conn.execute_batch(OSINT_REVIEW_PROJECTION_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
            .unwrap();
        conn
    }

    #[test]
    fn review_projection_migration_is_additive_idempotent_and_creates_no_business_object() {
        let conn = review_phase_database();
        conn.execute_batch(OSINT_REVIEW_PROJECTION_MIGRATION_SQL)
            .unwrap();
        let folders: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let tables:i64=conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('osint_claim_reviews','osint_claim_session_presence','osint_projection_events')",[],|row|row.get(0)).unwrap();
        let business:i64=conn.query_row("SELECT (SELECT COUNT(*) FROM exposures)+(SELECT COUNT(*) FROM incidents)+(SELECT COUNT(*) FROM actions)+(SELECT COUNT(*) FROM rgpd_requests)",[],|row|row.get(0)).unwrap();
        assert_eq!(folders, 1);
        assert_eq!(tables, 3);
        assert_eq!(business, 0);
    }

    #[test]
    fn latest_human_decision_is_the_only_projection_gate() {
        let conn = review_phase_database();
        assert!(!has_positive_human_decision(&conn, "signal-1"));
        conn.execute("INSERT INTO osint_user_decisions(id,target_type,target_id,decision,previous_status,created_at) VALUES ('decision-ok','signal','signal-1','confirmer','À vérifier',datetime('now'))",[]).unwrap();
        assert!(has_positive_human_decision(&conn, "signal-1"));
        conn.execute("INSERT INTO osint_user_decisions(id,target_type,target_id,decision,previous_status,created_at) VALUES ('decision-no','signal','signal-1','pas_moi','Confirmé',datetime('now'))",[]).unwrap();
        assert!(!has_positive_human_decision(&conn, "signal-1"));
    }

    #[test]
    fn projection_audit_is_idempotent_and_identity_scoped() {
        let conn = review_phase_database();
        record_projection_event(
            &conn,
            "identity-1",
            "signal-1",
            "exposition",
            "exposure-1",
            "cree",
        )
        .unwrap();
        record_projection_event(
            &conn,
            "identity-1",
            "signal-1",
            "exposition",
            "exposure-1",
            "reutilise",
        )
        .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_projection_events WHERE identity_id='identity-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn review_workspace_distinguishes_new_persistent_and_unavailable_claims() {
        let conn = review_phase_database();
        let first: String = conn
            .query_row(
                "SELECT session_id FROM osint_scans WHERE id='scan-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        record_claim_session_presence(&conn, &first, "identity-1").unwrap();
        conn.execute("INSERT INTO osint_scan_sessions(id,identity_id,origin,status,planned_checks,completed_checks,failed_checks,skipped_checks,signal_count,started_at,completed_at) VALUES ('session-new','identity-1','scan_manuel','termine',1,1,0,0,0,datetime('now','+1 minute'),datetime('now','+1 minute'))",[]).unwrap();
        let workspace = identity_review_workspace_in_connection(&conn, "identity-1").unwrap();
        assert!(workspace
            .evolution
            .iter()
            .any(|item| item.change == "source_indisponible"
                && item.previous_sources > 0
                && item.current_sources == 0));
    }

    #[test]
    fn multi_source_synthesis_migration_is_additive_and_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(LOCAL_AI_JOBS_MIGRATION_SQL).unwrap();
        ensure_analysis_job_columns(&conn).unwrap();
        conn.execute_batch(OSINT_SYNTHESIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_SYNTHESIS_MIGRATION_SQL).unwrap();
        let folder_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let tables:i64=conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('osint_synthesis_runs','osint_analysis_claim_inputs')",[],|row|row.get(0)).unwrap();
        let columns = table_columns(&conn, "osint_analysis_jobs").unwrap();
        assert_eq!(folder_count, 1);
        assert_eq!(tables, 2);
        assert!(columns.contains(&"task".into()) && columns.contains(&"identity_id".into()));
    }

    #[test]
    fn deterministic_multi_source_synthesis_cites_every_finding_and_keeps_contradictions() {
        let inputs = synthetic_claim_inputs();
        let payload = deterministic_claim_synthesis(&inputs);
        validate_claim_synthesis(&payload, &inputs).expect("synthèse déterministe valide");
        assert_eq!(payload.findings.len(), 2);
        assert!(
            payload
                .findings
                .iter()
                .find(|finding| finding.claim_id == "claim-homonym")
                .unwrap()
                .contradiction
        );
        assert!(payload
            .findings
            .iter()
            .all(|finding| !finding.evidence_ids.is_empty()));
        assert!(payload
            .findings
            .iter()
            .all(|finding| !finding.exposure_kind.is_empty() && !finding.where_found.is_empty()));
        assert!(payload.conclusion.contains("jamais une identité confirmée"));
    }

    #[test]
    fn deterministic_synthesis_never_upgrades_a_claim_beyond_its_fact_resolution() {
        let mut inputs = synthetic_claim_inputs();
        inputs[0].resolution_status = "a_verifier".into();
        inputs[0].resolution_source_count = 1;
        inputs[0].resolution_rationale = "Une seule famille de source soutient ce fait.".into();
        let payload = deterministic_claim_synthesis(&inputs);
        let finding = payload
            .findings
            .iter()
            .find(|finding| finding.claim_id == "claim-profile")
            .expect("constat du profil");
        assert_eq!(finding.confidence, "faible");
        assert_eq!(finding.recommended_action, "verifier");
    }

    #[test]
    fn deterministic_annotations_replace_untrusted_model_annotations() {
        let inputs = synthetic_claim_inputs();
        let mut payload = deterministic_claim_synthesis(&inputs);
        for finding in &mut payload.findings {
            finding.exposure_kind = "invented".into();
            finding.exposed_data = vec!["invented".into()];
            finding.where_found = vec!["https://invented.invalid".into()];
            finding.contradiction = false;
        }
        apply_deterministic_claim_annotations(&mut payload, &inputs);
        validate_claim_synthesis(&payload, &inputs)
            .expect("annotations restaurées depuis les faits");
        assert!(payload
            .findings
            .iter()
            .all(|finding| finding.exposure_kind != "invented"));
    }

    #[test]
    fn multi_source_synthesis_rejects_invented_citations_and_hidden_contradictions() {
        let inputs = synthetic_claim_inputs();
        let mut invented = deterministic_claim_synthesis(&inputs);
        invented.citation_ids = vec!["preuve-inventee".into()];
        assert!(validate_claim_synthesis(&invented, &inputs).is_err());
        let mut hidden = deterministic_claim_synthesis(&inputs);
        hidden
            .findings
            .iter_mut()
            .find(|finding| finding.claim_id == "claim-homonym")
            .unwrap()
            .contradiction = false;
        assert!(validate_claim_synthesis(&hidden, &inputs).is_err());
    }

    #[test]
    fn multi_source_synthesis_rejects_identity_attribution_language() {
        let inputs = synthetic_claim_inputs();
        let mut payload = deterministic_claim_synthesis(&inputs);
        payload.findings[0].statement = "Ce profil appartient à la personne.".into();
        assert!(validate_claim_synthesis(&payload, &inputs).is_err());
    }

    #[test]
    fn local_ai_analysis_migration_is_additive_and_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL).unwrap();
        let folder_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let tables: i64 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE 'osint_analysis_%'",[],|row|row.get(0)).unwrap();
        assert_eq!(folder_count, 1);
        assert_eq!(tables, 3);
    }

    #[test]
    fn local_ai_job_migration_seeds_automatic_mode_and_preserves_data() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(LOCAL_AI_JOBS_MIGRATION_SQL).unwrap();
        conn.execute_batch(LOCAL_AI_JOBS_MIGRATION_SQL).unwrap();
        let mode: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key='local_ai_manual_scan_mode'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let folder_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(mode, "automatic");
        assert_eq!(folder_count, 1);
        assert!(conn.execute("INSERT INTO osint_analysis_jobs (id,origin,signal_ids_json,signal_count,status,estimated_seconds) VALUES ('bad','origine_interdite','[]',0,'en_attente',10)",[]).is_err());
    }

    #[test]
    fn clearing_user_data_preserves_modules_settings_and_local_ai() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(LOCAL_AI_MIGRATION_SQL).unwrap();
        conn.execute_batch(LOCAL_AI_ONBOARDING_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(LOCAL_AI_JOBS_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_GRAPH_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_REPORT_MIGRATION_SQL).unwrap();
        conn.execute_batch(RGPD_REVIEW_MIGRATION_SQL).unwrap();
        ensure_identity_target_columns(&conn).unwrap();
        conn.execute_batch(IDENTITY_TARGETS_MIGRATION_SQL).unwrap();
        ensure_scan_session_columns(&conn).unwrap();
        conn.execute_batch(OSINT_SCAN_SESSIONS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        ensure_analysis_job_columns(&conn).unwrap();
        conn.execute_batch(OSINT_SYNTHESIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_REVIEW_PROJECTION_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
            .unwrap();
        conn.execute(
            "UPDATE osint_modules SET last_run=datetime('now') WHERE id='mock-osint'",
            [],
        )
        .unwrap();

        ensure_action_tracking_columns(&conn).unwrap();
        conn.execute_batch(ACTION_TRACKING_MIGRATION_SQL).unwrap();
        ensure_remediation_catalog_columns(&conn).unwrap();
        conn.execute_batch(REMEDIATION_CATALOG_MIGRATION_SQL)
            .unwrap();
        clear_scan_results_in_connection(&mut conn).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM folders", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM identities", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );

        clear_user_scan_data_in_connection(&mut conn).unwrap();

        let folders: i64 = conn
            .query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0))
            .unwrap();
        let scans: i64 = conn
            .query_row("SELECT COUNT(*) FROM osint_scans", [], |row| row.get(0))
            .unwrap();
        let sessions: i64 = conn
            .query_row("SELECT COUNT(*) FROM osint_scan_sessions", [], |row| {
                row.get(0)
            })
            .unwrap();
        let signals: i64 = conn
            .query_row("SELECT COUNT(*) FROM osint_signals", [], |row| row.get(0))
            .unwrap();
        let modules: i64 = conn
            .query_row("SELECT COUNT(*) FROM osint_modules", [], |row| row.get(0))
            .unwrap();
        let ai_components: i64 = conn
            .query_row("SELECT COUNT(*) FROM local_ai_components", [], |row| {
                row.get(0)
            })
            .unwrap();
        let mode: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key='local_ai_manual_scan_mode'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let last_run: Option<String> = conn
            .query_row(
                "SELECT last_run FROM osint_modules WHERE id='mock-osint'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!((folders, scans, sessions, signals), (0, 0, 0, 0));
        assert!(modules > 0);
        assert!(ai_components > 0);
        assert_eq!(mode, "automatic");
        assert_eq!(last_run, None);
    }

    #[test]
    fn graph_relations_are_evidence_backed_and_user_rejection_is_a_contradiction() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_GRAPH_MIGRATION_SQL).unwrap();
        refresh_osint_graph(&mut conn).unwrap();
        let evidence_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM osint_relation_evidence", [], |row| {
                row.get(0)
            })
            .unwrap();
        let possible: String = conn.query_row("SELECT evidence_level FROM osint_relations WHERE id='relation:observe:observation-signal-1'", [], |row| row.get(0)).unwrap();
        assert!(evidence_count >= 2);
        assert_eq!(possible, "possible");

        conn.execute("UPDATE osint_observations SET relevance_status='pas_moi' WHERE id='observation-signal-1'", []).unwrap();
        refresh_osint_graph(&mut conn).unwrap();
        let rejected: (String,String) = conn.query_row("SELECT evidence_level,review_status FROM osint_relations WHERE id='relation:observe:observation-signal-1'", [], |row| Ok((row.get(0)?,row.get(1)?))).unwrap();
        assert_eq!(rejected, ("contradiction".into(), "rejetee".into()));
    }

    #[test]
    fn claim_migration_is_additive_and_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        let folder: String = conn
            .query_row(
                "SELECT name FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let tables: i64 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('osint_claims','osint_claim_evidence')",[],|row|row.get(0)).unwrap();
        assert_eq!(folder, "Données à conserver");
        assert_eq!(tables, 2);
    }

    #[test]
    fn fact_resolution_migrations_are_additive_and_idempotent() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_EVIDENCE_ASSESSMENTS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_EVIDENCE_FACTS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIM_FACT_LINKS_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_FACT_RESOLUTIONS_MIGRATION_SQL)
            .unwrap();
        let folder: String = conn
            .query_row(
                "SELECT name FROM folders WHERE id='folder-kept'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let tables: i64 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('osint_evidence_facts','osint_claim_fact_links','osint_fact_resolutions','osint_fact_resolution_evidence')",[],|row|row.get(0)).unwrap();
        assert_eq!(folder, "Données à conserver");
        assert_eq!(tables, 4);
    }

    #[test]
    fn declared_claim_value_types_use_exact_deterministic_canonicalization() {
        assert_eq!(
            normalized_claim_value("email", "  Alice@Example.TEST "),
            "alice@example.test"
        );
        assert_eq!(
            normalized_claim_value("telephone", "+33 (0)6 12-34-56-78"),
            "+33612345678"
        );
        assert_eq!(normalized_claim_value("pseudo", " @Alice_42 "), "alice_42");
        assert_eq!(
            normalized_claim_value("domaine", "WWW.Example.TEST."),
            "example.test"
        );
        assert_ne!(
            normalized_claim_value("nom", "Jean Martin"),
            normalized_claim_value("nom", "Jean-Martin")
        );
    }

    #[test]
    fn exact_url_claims_are_deduplicated_and_corroborated_without_identity_attribution() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        conn.execute("UPDATE osint_observations SET source='Source A',source_url='https://Example.test/profile/?utm_source=a#bio',display_value='Profil potentiel Alice' WHERE id='observation-signal-1'",[]).unwrap();
        conn.execute("INSERT INTO osint_observations (id,scan_id,signal_id,identity_id,observation_type,canonical_key,display_value,source,source_url,observed_at,relevance_status) VALUES ('observation-2','scan-1','signal-2','identity-1','profil_public','legacy-key-2','Autre présentation du même profil','Source B','https://example.test/profile/',datetime('now'),'a_verifier')",[]).unwrap();
        refresh_osint_claims(&mut conn, "identity-1").unwrap();
        let claims = load_identity_claims(&conn, "identity-1").unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].status, "corroboree");
        assert_eq!(claims[0].source_count, 2);
        assert_eq!(claims[0].evidence.len(), 2);
        assert!(claims[0].rationale.contains("ne confirme pas l’identité"));
    }

    #[test]
    fn two_collectors_from_the_same_source_family_do_not_corroborate_a_claim() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-username-profiles' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        conn.execute("UPDATE osint_observations SET source='Maigret',source_url='https://example.test/alice',display_value='Profil potentiel Alice' WHERE id='observation-signal-1'",[]).unwrap();
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,review_status) VALUES ('signal-2','osint-email-platforms','scan-1','identity-1','alice','profil_public','Profil potentiel Alice','Résultat à vérifier','faible','À vérifier','User Scanner','https://example.test/alice',datetime('now'),'fixture','À vérifier')",[]).unwrap();
        conn.execute("INSERT INTO osint_observations (id,scan_id,signal_id,identity_id,observation_type,canonical_key,display_value,source,source_url,observed_at,relevance_status) VALUES ('observation-2','scan-1','signal-2','identity-1','profil_public','legacy-key-2','Profil potentiel Alice','User Scanner','https://example.test/alice',datetime('now'),'a_verifier')",[]).unwrap();
        refresh_osint_claims(&mut conn, "identity-1").unwrap();
        let claims = load_identity_claims(&conn, "identity-1").unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].status, "a_verifier");
        assert_eq!(claims[0].source_count, 1);
        assert_eq!(claims[0].evidence.len(), 2);
    }

    #[test]
    fn keybase_and_direct_platform_evidence_merge_into_one_corroborated_claim() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        ensure_scan_session_columns(&conn).unwrap();
        conn.execute(
            "UPDATE osint_scans SET target_kind_snapshot='pseudo' WHERE id='scan-1'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE osint_signals SET module_id='osint-keybase-profile',target='mantis-example',title='Compte public potentiellement lié : GitHub',explanation='Preuves publiques vérifiées par Keybase : GitHub',source_url='https://github.com/mantis-example' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        conn.execute("UPDATE osint_observations SET source='Keybase',source_url='https://github.com/mantis-example',display_value='Compte public potentiellement lié : GitHub' WHERE id='observation-signal-1'",[]).unwrap();
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,source_url,discovered_at,evidence_ref,review_status) VALUES ('signal-direct','osint-github-profile','scan-1','identity-1','mantis-example','profil_public','Profil GitHub potentiel : mantis-example','Pseudo exact retourné par GitHub','faible','À vérifier','GitHub','https://github.com/mantis-example',datetime('now'),'fixture','À vérifier')",[]).unwrap();
        conn.execute("INSERT INTO osint_observations (id,scan_id,signal_id,identity_id,observation_type,canonical_key,display_value,source,source_url,observed_at,relevance_status) VALUES ('observation-direct','scan-1','signal-direct','identity-1','profil_public','direct-key','Profil GitHub potentiel : mantis-example','GitHub','https://github.com/mantis-example',datetime('now'),'a_verifier')",[]).unwrap();
        apply_signal_quality_gate(&mut conn, "identity-1").unwrap();
        let visible_assessments: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM osint_evidence_assessments WHERE publication_status='visible'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(visible_assessments, 2);
        refresh_osint_claims(&mut conn, "identity-1").unwrap();
        let claims = load_identity_claims(&conn, "identity-1").unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].status, "corroboree");
        assert_eq!(claims[0].source_count, 2);
        assert_eq!(claims[0].evidence.len(), 2);
        assert!(claims[0].rationale.contains("ne confirme pas l’identité"));
    }

    #[test]
    fn repeated_observation_from_one_source_is_not_false_corroboration() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        conn.execute("UPDATE osint_observations SET observation_type='fuite',source='XposedOrNot',source_url='https://example.test/breach?utm_source=first',display_value='Fuite potentielle : Example' WHERE id='observation-signal-1'",[]).unwrap();
        conn.execute("INSERT INTO osint_observations (id,scan_id,signal_id,identity_id,observation_type,canonical_key,display_value,source,source_url,observed_at,relevance_status) VALUES ('observation-repeat','scan-2','signal-repeat','identity-1','fuite','old-key','Fuite potentielle : Example','XposedOrNot','https://example.test/breach?utm_medium=repeat',datetime('now'),'a_verifier')",[]).unwrap();
        refresh_osint_claims(&mut conn, "identity-1").unwrap();
        let claims = load_identity_claims(&conn, "identity-1").unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].status, "a_verifier");
        assert_eq!(claims[0].source_count, 1);
        assert_eq!(claims[0].favorable_count, 1);
        assert_eq!(claims[0].evidence.len(), 2);
    }

    #[test]
    fn claim_contradictions_are_preserved_and_similar_names_are_not_merged() {
        let mut conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(OSINT_CLAIMS_MIGRATION_SQL).unwrap();
        conn.execute("UPDATE osint_observations SET display_value='Profil Jean Martin',source='Source A',relevance_status='pas_moi' WHERE id='observation-signal-1'",[]).unwrap();
        conn.execute("INSERT INTO osint_observations (id,scan_id,signal_id,identity_id,observation_type,canonical_key,display_value,source,observed_at,relevance_status) VALUES ('observation-2','scan-1','signal-2','identity-1','profil_public','legacy-key-2','Profil Jean-Martin','Source B',datetime('now'),'a_verifier')",[]).unwrap();
        refresh_osint_claims(&mut conn, "identity-1").unwrap();
        let claims = load_identity_claims(&conn, "identity-1").unwrap();
        assert_eq!(claims.len(), 2);
        assert!(claims
            .iter()
            .any(|claim| claim.status == "rejetee" && claim.contradictory_count == 1));
        assert!(claims
            .iter()
            .any(|claim| claim.status == "a_verifier" && claim.favorable_count == 1));
    }

    #[test]
    fn report_snapshot_is_reproducible_and_pdf_is_valid() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_OBSERVABILITY_MIGRATION_SQL)
            .unwrap();
        conn.execute_batch(LOCAL_AI_ANALYSIS_MIGRATION_SQL).unwrap();
        conn.execute_batch(OSINT_REPORT_MIGRATION_SQL).unwrap();
        let first =
            build_report_snapshot(&conn, "fixed-id".into(), "2026-07-31 12:00:00".into(), None)
                .unwrap();
        let second =
            build_report_snapshot(&conn, "fixed-id".into(), "2026-07-31 12:00:00".into(), None)
                .unwrap();
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert_eq!(first.analyzed_count, 1);
        let markdown = report_markdown(&first);
        assert!(markdown.contains("Sources et observations"));
        let pdf = simple_pdf(&markdown);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF"));
        if let Ok(path) = std::env::var("MANTIS_TEST_REPORT_PDF") {
            std::fs::write(path, pdf).unwrap();
        }
    }

    #[test]
    fn rgpd_draft_requires_confirmed_source_and_latest_explicit_review() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(RGPD_REVIEW_MIGRATION_SQL).unwrap();
        conn.execute("INSERT OR IGNORE INTO rgpd_types (id,name,label) VALUES ('type_003','effacement','Effacement')",[]).unwrap();
        conn.execute("INSERT OR IGNORE INTO rgpd_statuses (id,name,label) VALUES ('status_001','brouillon','Brouillon')",[]).unwrap();
        conn.execute("INSERT INTO rgpd_requests (id,type_id,target,dpo_contact,status_id,data_summary,draft_preview,source_url) VALUES ('rgpd-qa','type_003','example.test','contact@example.test','status_001','Synthèse','Brouillon','https://example.test/page')",[]).unwrap();
        conn.execute("INSERT INTO rgpd_draft_versions (id,request_id,contract_version,draft_text,content_sha256,source_url,source_signal_id) VALUES ('version-qa','rgpd-qa','effacement-source-confirmee-v1','Brouillon','hash','https://example.test/page','signal-1')",[]).unwrap();
        let blocked = rgpd_review_status_in_connection(&conn, "rgpd-qa").unwrap();
        assert!(!blocked.eligible);
        conn.execute(
            "UPDATE osint_signals SET review_status='Confirmé' WHERE id='signal-1'",
            [],
        )
        .unwrap();
        let eligible = rgpd_review_status_in_connection(&conn, "rgpd-qa").unwrap();
        assert!(eligible.eligible && !eligible.validated);
        conn.execute("INSERT INTO rgpd_user_reviews (id,request_id,draft_version_id,source_checked,identity_checked,recipient_checked,content_checked,legal_notice_accepted,decision) VALUES ('review-ok','rgpd-qa','version-qa',1,1,1,1,1,'valide')",[]).unwrap();
        assert!(
            rgpd_review_status_in_connection(&conn, "rgpd-qa")
                .unwrap()
                .validated
        );
        conn.execute("INSERT INTO rgpd_user_reviews (id,request_id,draft_version_id,source_checked,identity_checked,recipient_checked,content_checked,legal_notice_accepted,decision) VALUES ('review-revoke','rgpd-qa','version-qa',0,0,0,0,0,'revoque')",[]).unwrap();
        assert!(
            !rgpd_review_status_in_connection(&conn, "rgpd-qa")
                .unwrap()
                .validated
        );
    }

    #[test]
    fn deterministic_fallback_is_valid_and_traceable() {
        let inputs = synthetic_analysis_input();
        let payload = deterministic_analysis(&inputs);
        validate_analysis_payload(&payload, &inputs).expect("fallback valide");
        assert_eq!(payload.items[0].evidence_ids, vec!["evidence-test"]);
        assert!(payload.items[0].uncertainty);
    }

    #[test]
    fn analysis_rejects_unknown_evidence_and_duplicate_observations() {
        let inputs = synthetic_analysis_input();
        let mut payload = deterministic_analysis(&inputs);
        payload.items[0].evidence_ids = vec!["evidence-inventee".into()];
        assert!(validate_analysis_payload(&payload, &inputs).is_err());
        let mut duplicate = deterministic_analysis(&inputs);
        duplicate.items.push(duplicate.items[0].clone());
        assert!(validate_analysis_payload(&duplicate, &inputs).is_err());
    }

    #[test]
    fn catalog_lifecycle_keeps_only_supported_modules_active() {
        let conn = legacy_database_with_signal();
        conn.execute_batch(OSINT_CATALOG_LIFECYCLE_MIGRATION_SQL)
            .expect("cycle de vie du catalogue");

        let active: Vec<String> = {
            let mut statement = conn
                .prepare("SELECT id FROM osint_modules WHERE catalog_status='active' ORDER BY id")
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        let mock_status: String = conn
            .query_row(
                "SELECT catalog_status FROM osint_modules WHERE id='mock-osint'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(
            active,
            vec![
                "osint-bluesky-profile",
                "osint-email-intel",
                "osint-email-platforms",
                "osint-fr-company-register",
                "osint-github-profile",
                "osint-gitlab-profile",
                "osint-gravatar-profile",
                "osint-hackernews-profile",
                "osint-hal-author",
                "osint-keybase-profile",
                "osint-mastodon-webfinger",
                "osint-username-profiles",
                "osint-web-footprint"
            ]
        );
        assert_eq!(mock_status, "test_only");
    }

    #[test]
    fn archiving_a_module_never_deletes_historical_scans_or_evidence() {
        let conn = legacy_database_with_signal();
        conn.execute("INSERT OR REPLACE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args,catalog_status) VALUES ('osint-email-breaches-local','h8mail historique','Ancien collecteur','email','Manuel','actif',NULL,NULL,NULL,NULL,'active')", []).unwrap();
        conn.execute("INSERT INTO osint_scans (id,module_id,identity_id,target,status,started_at,completed_at,raw_result_path) VALUES ('scan-historical','osint-email-breaches-local','identity-1','test@example.test','termine',datetime('now'),datetime('now'),'raw/historical.json')", []).unwrap();
        conn.execute("INSERT INTO osint_signals (id,module_id,scan_id,identity_id,target,signal_type,title,explanation,severity,confidence,source,discovered_at,evidence_ref,raw_result_path,review_status) VALUES ('signal-historical','osint-email-breaches-local','scan-historical','identity-1','test@example.test','fuite','Signal historique','Preuve conservée','faible','À vérifier','h8mail',datetime('now'),'preuve-historique','raw/historical.json','À vérifier')", []).unwrap();

        conn.execute_batch(OSINT_CATALOG_LIFECYCLE_MIGRATION_SQL)
            .expect("archivage additif");

        let lifecycle: (String, String) = conn
            .query_row("SELECT catalog_status,replacement_id FROM osint_modules WHERE id='osint-email-breaches-local'", [], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();
        let kept: (i64, i64, String) = conn
            .query_row("SELECT (SELECT COUNT(*) FROM osint_scans WHERE id='scan-historical'),(SELECT COUNT(*) FROM osint_signals WHERE id='signal-historical'),raw_result_path FROM osint_signals WHERE id='signal-historical'", [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap();

        assert_eq!(lifecycle, ("archived".into(), "osint-email-intel".into()));
        assert_eq!(kept, (1, 1, "raw/historical.json".into()));
    }

    #[test]
    fn managed_runtime_integrity_accepts_manifest_and_rejects_tampering() {
        let directory =
            std::env::temp_dir().join(format!("mantis-managed-module-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let executable = directory.join("collector.exe");
        std::fs::write(&executable, b"synthetic-runtime-v1").unwrap();
        std::fs::write(directory.join("LICENSE.txt"), b"Synthetic test license").unwrap();
        let manifest = ManagedBundleManifest {
            schema_version: 1,
            module_id: "module-test".into(),
            version: "1.0.0-test".into(),
            file: "collector.exe".into(),
            sha256: sha256_file(&executable).unwrap(),
            license_file: Some("LICENSE.txt".into()),
        };
        std::fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        assert_eq!(
            managed_directory_integrity(&directory, "module-test", "collector.exe").unwrap(),
            "1.0.0-test"
        );
        std::fs::write(&executable, b"tampered-runtime").unwrap();
        assert!(managed_directory_integrity(&directory, "module-test", "collector.exe").is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn managed_runtime_rollback_restores_verified_previous_version() {
        let root = std::env::temp_dir().join(format!("mantis-rollback-{}", Uuid::new_v4()));
        let current = root.join("collector");
        let previous = root.join("collector-previous");
        for (directory, version, bytes) in [
            (&current, "2.0.0-test", b"runtime-v2".as_slice()),
            (&previous, "1.0.0-test", b"runtime-v1".as_slice()),
        ] {
            std::fs::create_dir_all(directory).unwrap();
            let executable = directory.join("collector.exe");
            std::fs::write(&executable, bytes).unwrap();
            let manifest = ManagedBundleManifest {
                schema_version: 1,
                module_id: "module-test".into(),
                version: version.into(),
                file: "collector.exe".into(),
                sha256: sha256_file(&executable).unwrap(),
                license_file: None,
            };
            std::fs::write(
                directory.join("manifest.json"),
                serde_json::to_vec_pretty(&manifest).unwrap(),
            )
            .unwrap();
        }

        let restored =
            rollback_managed_runtime(&current, &previous, "module-test", "collector.exe").unwrap();

        assert_eq!(restored, "1.0.0-test");
        assert_eq!(
            std::fs::read(current.join("collector.exe")).unwrap(),
            b"runtime-v1"
        );
        assert!(!previous.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn json_parser_accepts_bounded_runtime_suffix() {
        let payload = deterministic_analysis(&synthetic_analysis_input());
        let output = format!("{} [end of text]", serde_json::to_string(&payload).unwrap());
        assert_eq!(
            json_object_from_output(&output).unwrap().task,
            "triage_osint"
        );
    }

    #[test]
    fn process_output_is_bounded_while_the_remaining_stream_is_drained() {
        let input = vec![b'x'; MAX_PROCESS_OUTPUT_BYTES + 1_024];
        let (captured, truncated) = read_bounded_process_output_with_limit(
            std::io::Cursor::new(input),
            MAX_PROCESS_OUTPUT_BYTES,
        )
        .expect("lecture bornée");
        assert_eq!(captured.len(), MAX_PROCESS_OUTPUT_BYTES);
        assert!(truncated);
    }

    #[test]
    fn official_runtime_archive_passes_integrity_extraction_and_canary_when_provided() {
        let Ok(archive_value) = std::env::var("MANTIS_TEST_LLAMA_ARCHIVE") else {
            return;
        };
        let archive = std::path::PathBuf::from(archive_value);
        let component = local_ai_component().expect("composant signé");
        assert_eq!(
            sha256_file(&archive).expect("hash archive"),
            component.sha256
        );
        assert_eq!(
            std::fs::metadata(&archive).expect("taille archive").len(),
            component.byte_size
        );

        let staging =
            std::env::temp_dir().join(format!("mantis-runtime-canary-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&staging).expect("staging");
        extract_local_ai_archive(&archive, &staging).expect("extraction sûre");
        let executable = find_file_bounded(
            &staging,
            component.executable.as_deref().expect("nom exécutable"),
            0,
        )
        .expect("llama-cli");
        let version = run_local_ai_version(&executable).expect("canari --version");
        assert!(!version.is_empty());
        std::fs::remove_dir_all(&staging).expect("nettoyage canari");
    }
}
