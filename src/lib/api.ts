import { invoke } from '@tauri-apps/api/core';

export interface Folder {
  id: string;
  name: string;
  context: string;
}

export interface Incident {
  id: string;
  title: string;
  severity: string;
  discovered_at: string;
  what: string;
  why: string;
  impact: string;
  confidence: string;
  next_step: string;
  folder_id: string | null;
}

export interface Action {
  id: string;
  title: string;
  priority_id: string;
  difficulty_id: string;
  deadline: string;
  status: string;
  guidance: string;
  proof_expected: string;
  folder_id: string | null;
  incident_id: string | null;
  tracking_note: string | null;
  actor: 'moi' | 'site' | null;
  completed_at: string | null;
  blocked_reason: string | null;
}

export interface ActionEvent {
  id: string; action_id: string; from_status: string | null; to_status: string;
  actor: 'moi' | 'site' | null; note: string | null; created_at: string;
}

export interface ActionEvidence {
  id: string; action_id: string; kind: 'url' | 'fichier' | 'note' | 'hash';
  locator: string; description: string | null; created_at: string;
}

export interface RemediationRecommendation {
  id: string; title: string; exposure_kinds: string[]; priority: string; why: string;
  steps: string[]; proof_expected: string; expected_outcome: string;
  guide_id: string | null; execution_mode: string;
}

export interface RemediationPlanItem {
  id: string; action_id: string | null; exposure_id: string | null; incident_id: string | null;
  sort_order: number; expected_outcome: string; proof_expected: string;
  execution_mode: string; recommendation_id: string;
}

export interface RemediationPlan {
  id: string; identity_id: string | null; folder_id: string | null; scan_id: string | null;
  title: string; status: string; priority: string; rationale: string;
  catalog_version: string; items: RemediationPlanItem[];
}

export interface RemediationAiEnrichment {
  id: string; plan_id: string; contract_version: string; mode: string; status: string;
  model_label: string | null; summary: string; education: string[];
  priority_rationale: string; cautions: string[]; citation_ids: string[];
  error_message: string | null; created_at: string;
}

export interface Identity {
  id: string;
  label: string;
  kind: string;
  value: string;
  folder_id: string | null;
  notes: string | null;
  address_line1: string | null;
  address_line2: string | null;
  city: string | null;
  postal_code: string | null;
  country: string | null;
  status: 'active' | 'inactive';
  values: IdentityValue[];
}

export type IdentityValueKind = 'prenom' | 'nom' | 'pseudo' | 'email' | 'telephone' | 'adresse' | 'domaine' | 'url';

export interface IdentityValue {
  id: string;
  kind: IdentityValueKind;
  value: string;
  normalized_value: string;
  label: string | null;
  status: 'active' | 'inactive';
  origin: 'user';
  address_line1: string | null;
  address_line2: string | null;
  city: string | null;
  postal_code: string | null;
  country: string | null;
  sort_order: number;
}

export type IdentityValueInput = Omit<IdentityValue, 'id' | 'normalized_value' | 'origin' | 'sort_order'> & { id?: string };
export interface IdentityInput {
  label: string;
  status: 'active' | 'inactive';
  folder_id: string | null;
  notes: string | null;
  values: IdentityValueInput[];
}

export interface Exposure {
  id: string;
  title: string;
  kind: string;
  severity: string;
  status: string;
  discovered_at: string;
  source: string;
  what: string;
  why: string;
  folder_id: string | null;
}

export interface RgpdRequest {
  id: string;
  type_id: string;
  target: string;
  dpo_contact: string;
  status_id: string;
  data_summary: string;
  draft_preview: string;
  source_url: string | null;
  contact_source_url: string | null;
}

export interface RgpdEvidence { id: string; request_id: string; kind: string; locator: string; description: string | null; verified: boolean; created_at: string; }
export interface RgpdEvent { id: string; request_id: string; from_status: string | null; to_status: string | null; event_type: string; note: string | null; created_at: string; }

export interface TimelineEntry {
  id: string;
  event_type: string;
  description: string;
  created_at: string;
}

export interface OsintModule {
  id: string;
  name: string;
  description: string;
  target_kind: string;
  frequency: string;
  status: string;
  last_run: string | null;
  next_run: string | null;
  script_path: string | null;
  script_args: string | null;
  installation_status: string;
  diagnostic: string | null;
  catalog_status: 'active' | 'archived' | 'test_only';
  replacement_id: string | null;
  archived_reason: string | null;
}
export interface RgpdReviewStatus { request_id:string; draft_version_id:string; contract_version:string; validated:boolean; eligible:boolean; reason:string; source_signal_id:string|null; reviewed_at:string|null; }
export interface RgpdDraftUse { text:string; path:string|null; draft_version_id:string; }

export interface OsintSignal {
  id: string; module_id: string; scan_id: string; target: string; signal_type: string; title: string; explanation: string; severity: string; confidence: string; source: string;
  discovered_at: string;
  source_url: string | null; review_status: string; exposure_id: string | null;
}

export interface OsintScanSummary { scan_id: string; target: string; signals: OsintSignal[]; message: string; analysis_job_id: string | null; }
export interface OsintClaimEvidence { observation_id:string; signal_id:string|null; source:string; source_url:string|null; observed_at:string; role:'favorable'|'contradictoire'; }
export interface OsintClaim {
  id:string; identity_id:string; claim_type:string; display_value:string; status:'a_verifier'|'corroboree'|'confirmee'|'contradictoire'|'rejetee';
  priority:'faible'|'moyenne'|'haute'|'critique'; favorable_count:number; contradictory_count:number; source_count:number;
  first_observed_at:string; last_observed_at:string; rationale:string; evidence:OsintClaimEvidence[];
}
export interface OsintFactResolution {
  id:string; fact_type:string; status:'a_verifier'|'corroboree'|'contradictoire'|'rejete';
  source_count:number; favorable_count:number; contradictory_count:number; rationale:string;
}
export interface OsintReviewEvent { id:string; event_type:'decision'|'projection'; target_label:string; decision:string; reason:string|null; created_at:string; }
export interface OsintEvolutionItem { claim_id:string; display_value:string; change:'nouveau'|'toujours_present'|'source_indisponible'; current_sources:number; previous_sources:number; }
export interface OsintReviewWorkspace { events:OsintReviewEvent[]; evolution:OsintEvolutionItem[]; }
export interface OsintScanCoverage { target_kind:string; planned_checks:number; completed_checks:number; failed_checks:number; signal_count:number; }
export interface OsintScanSessionSummary extends OsintScanSummary {
  session_id: string;
  identity_id: string;
  planned_checks: number;
  completed_checks: number;
  failed_checks: number;
  skipped_checks: number;
  coverage: OsintScanCoverage[];
  claims: OsintClaim[];
  resolutions: OsintFactResolution[];
}
export interface OsintScanSessionListItem {
  id: string; identity_id: string; origin: 'scan_manuel'|'routine'; status: 'en_cours'|'termine'|'partiel'|'erreur';
  signal_count: number; planned_checks: number; completed_checks: number; failed_checks: number; skipped_checks: number;
  started_at: string; completed_at: string|null; summary: string|null;
}
export interface OsintAnalysisItem { observation_id: string; classification: string; relevance: 'important' | 'a_verifier' | 'bruit'; reason: string; recommended_action: string; evidence_ids: string[]; uncertainty: boolean; }
export interface OsintSynthesisFinding { claim_id:string; statement:string; confidence:'faible'|'moyenne'|'forte'; recommended_action:'verifier'|'securiser'|'suivre'|'ignorer'; evidence_ids:string[]; contradiction:boolean; exposure_kind?:string; exposed_data?:string[]; where_found?:string[]; }
export interface OsintAnalysisReport { run_id: string; mode: 'ia_locale' | 'deterministe'; status: string; model_label: string | null; overview: string; items: OsintAnalysisItem[]; limitations: string[]; conclusion?:string; findings?:OsintSynthesisFinding[]; citation_ids?:string[]; }
export interface OsintAnalysisJobStatus { id: string; origin: string; status: 'en_attente'|'en_cours'|'termine'|'fallback'|'erreur'|'interrompu'; signal_count: number; estimated_seconds: number; elapsed_seconds: number; run_id: string|null; result_mode: string|null; message: string|null; report: OsintAnalysisReport|null; }
export interface OsintRoutineSummary { scanned_identities: number; signals_found: number; failed_identities: number; message: string; analysis_jobs_started: number; }
export interface OsintGraphEvidence { observation_id: string; label: string; excerpt: string|null; source_url: string|null; role: 'favorable'|'contradictoire'; }
export interface OsintGraphNode { id: string; node_type: 'identite'|'observation'|'source'; label: string; detail: string; }
export interface OsintGraphEdge { id: string; from: string; to: string; relation_type: string; level: 'observe'|'possible'|'probable'|'corroboree'|'contradiction'; justification: string; review_status: string; evidence: OsintGraphEvidence[]; }
export interface OsintGraphTimelineItem { id: string; date: string; date_kind: 'observation'|'decision'; label: string; source: string; level: string; }
export interface OsintGraph { nodes: OsintGraphNode[]; edges: OsintGraphEdge[]; timeline: OsintGraphTimelineItem[]; truncated: boolean; }
export interface OsintReportSource { signal_id:string; title:string; severity:string; source:string; source_url:string|null; observed_at:string; review_status:string; explanation:string; }
export interface OsintReportDecision { decision:string; reason:string|null; created_at:string; target_id:string; }
export interface OsintReportAction { id:string; title:string; priority:string; status:string; deadline:string; }
export interface OsintReportSnapshot { id:string; created_at:string; analyzed_count:number; discarded_count:number; attention_count:number; uncertain_count:number; contradiction_count:number; actions_now_count:number; overview:string; priorities:OsintReportSource[]; sources:OsintReportSource[]; decisions:OsintReportDecision[]; actions:OsintReportAction[]; guide_ids:string[]; limitations:string[]; has_local_ai_analysis:boolean; }
export interface OsintReportExport { path:string; format:'markdown'|'pdf'; snapshot_id:string; }
export interface OsintModuleLog { operation: string; status: string; message: string; created_at: string; }
export interface VeilleRoutine { frequency: string; paused: boolean; last_run: string | null; next_run: string | null; status: string; }

export interface LocalAiStatus {
  component_id: string;
  version: string;
  status: string;
  installed: boolean;
  integrity_ok: boolean;
  downloaded_bytes: number;
  expected_bytes: number;
  diagnostic: string;
  platform: string;
  architecture: string;
  license: string;
  source: string;
  enabled: boolean;
  onboarding_status: 'a_proposer' | 'sans_ia' | 'configure';
  selected_model_id: string | null;
  recommended_model_id: string;
  total_memory_bytes: number;
  available_disk_bytes: number;
  active_download_id: string | null;
  models: LocalAiModelStatus[];
}

export interface LocalAiModelStatus {
  component_id: string; label: string; tier: string; version: string; status: string; installed: boolean;
  downloaded_bytes: number; expected_bytes: number; license: string; source: string; min_ram_gb: number;
  context_size: number; compatible: boolean; diagnostic: string;
}

export interface PostureScore {
  score: number | null;
  open_incidents: number;
  high_exposures: number;
  completed_actions: number;
}

export interface PublicIpContext {
  ip: string;
  ip_type: string | null;
  city: string | null;
  region: string | null;
  country: string | null;
  asn: number | null;
  organization: string | null;
  isp: string | null;
  network_domain: string | null;
  timezone: string | null;
  utc_offset: string | null;
  proxy: boolean;
  vpn: boolean;
  tor: boolean;
  hosting: boolean;
}

export async function listFolders(): Promise<Folder[]> {
  return await invoke<Folder[]>('list_folders');
}

export async function getFolder(id: string): Promise<Folder> { return await invoke<Folder>('get_folder', { id }); }
export async function createFolder(name: string, context: string): Promise<Folder> { return await invoke<Folder>('create_folder', { name, context }); }
export async function deleteFolder(id: string): Promise<void> { await invoke('delete_folder', { id }); }

export async function listIncidents(identityId: string | null = null): Promise<Incident[]> {
  return await invoke<Incident[]>('list_incidents', { identityId });
}

export async function getIncident(id: string): Promise<Incident> { return await invoke<Incident>('get_incident', { id }); }

export async function listActions(identityId: string | null = null): Promise<Action[]> {
  return await invoke<Action[]>('list_actions', { identityId });
}

export async function listIdentities(): Promise<Identity[]> {
  return await invoke<Identity[]>('list_identities');
}

export async function getIdentity(id: string): Promise<Identity> { return await invoke<Identity>('get_identity', { id }); }

export async function listExposures(identityId: string | null = null): Promise<Exposure[]> {
  return await invoke<Exposure[]>('list_exposures', { identityId });
}

export async function listRemediationRecommendations(exposureKind: string): Promise<RemediationRecommendation[]> {
  return await invoke<RemediationRecommendation[]>('list_remediation_recommendations', { exposureKind });
}

export async function createRemediationPlan(exposureId: string, identityId: string | null = null): Promise<RemediationPlan> {
  return await invoke<RemediationPlan>('create_remediation_plan', { exposureId, identityId });
}

export async function getRemediationPlan(planId: string): Promise<RemediationPlan> {
  return await invoke<RemediationPlan>('get_remediation_plan', { planId });
}

export async function enrichRemediationPlan(planId: string): Promise<RemediationAiEnrichment> {
  return await invoke<RemediationAiEnrichment>('enrich_remediation_plan', { planId });
}

export async function getLatestRemediationEnrichment(planId: string): Promise<RemediationAiEnrichment> {
  return await invoke<RemediationAiEnrichment>('get_latest_remediation_enrichment', { planId });
}

export async function getExposure(id: string): Promise<Exposure> { return await invoke<Exposure>('get_exposure', { id }); }

export async function listRgpdRequests(identityId: string | null = null): Promise<RgpdRequest[]> {
  return await invoke<RgpdRequest[]>('list_rgpd_requests', { identityId });
}

export async function listTimelineEntries(): Promise<TimelineEntry[]> {
  return await invoke<TimelineEntry[]>('list_timeline_entries');
}

export async function listOsintModules(): Promise<OsintModule[]> {
  return await invoke<OsintModule[]>('list_osint_modules');
}

export async function listOsintModuleInventory(): Promise<OsintModule[]> {
  return await invoke<OsintModule[]>('list_osint_module_inventory');
}

export async function getPostureScore(identityId: string | null = null): Promise<PostureScore> {
  return await invoke<PostureScore>('get_posture_score', { identityId });
}

export async function getPublicIpContext(): Promise<PublicIpContext> {
  return await invoke<PublicIpContext>('get_public_ip_context');
}

export async function updateActionStatus(id: string, status: string): Promise<void> {
  await invoke('update_action_status', { actionId: id, status });
}

export async function updateActionTracking(id: string, status: string, note: string | null, actor: 'moi' | 'site' | null, blocked_reason: string | null): Promise<void> {
  await invoke('update_action_tracking', { actionId: id, status, note, actor, blockedReason: blocked_reason });
}

export async function listActionEvents(actionId: string): Promise<ActionEvent[]> {
  return await invoke<ActionEvent[]>('list_action_events', { actionId });
}

export async function listActionEvidence(actionId: string): Promise<ActionEvidence[]> {
  return await invoke<ActionEvidence[]>('list_action_evidence', { actionId });
}

export async function addActionEvidence(actionId: string, kind: ActionEvidence['kind'], locator: string, description: string | null): Promise<ActionEvidence> {
  return await invoke<ActionEvidence>('add_action_evidence', { actionId, kind, locator, description });
}

export async function updateRgpdRequestStatus(id: string, statusId: string): Promise<void> {
  await invoke('update_rgpd_request_status', { requestId: id, statusId });
}

export async function listRgpdEvidence(requestId: string): Promise<RgpdEvidence[]> { return await invoke<RgpdEvidence[]>('list_rgpd_evidence', { requestId }); }
export async function addRgpdEvidence(requestId: string, kind: string, locator: string, description: string | null, verified: boolean): Promise<RgpdEvidence> { return await invoke<RgpdEvidence>('add_rgpd_evidence', { requestId, kind, locator, description, verified }); }
export async function listRgpdEvents(requestId: string): Promise<RgpdEvent[]> { return await invoke<RgpdEvent[]>('list_rgpd_events', { requestId }); }

export async function createAction(
  title: string,
  priority_id: string,
  difficulty_id: string,
  deadline: string,
  guidance: string,
  proof_expected: string,
  folder_id: string | null = null,
  incident_id: string | null = null
): Promise<Action> {
  return await invoke<Action>('create_action', {
    title,
    priority_id,
    difficulty_id,
    deadline,
    guidance,
    proof_expected,
    folder_id,
    incident_id
  });
}

export async function createIdentity(input: IdentityInput): Promise<Identity> {
  return await invoke<Identity>('create_identity', {
    label: input.label,
    status: input.status,
    folderId: input.folder_id,
    notes: input.notes,
    values: input.values
  });
}

export async function updateIdentity(id: string, input: IdentityInput): Promise<Identity> {
  return await invoke<Identity>('update_identity', {
    id,
    label: input.label,
    status: input.status,
    folderId: input.folder_id,
    notes: input.notes,
    values: input.values
  });
}

export async function deleteIdentity(id: string): Promise<void> {
  await invoke('delete_identity', { id });
}

export async function listOsintModuleLogs(moduleId: string): Promise<OsintModuleLog[]> { return await invoke<OsintModuleLog[]>('list_osint_module_logs', { moduleId }); }
export async function getOsintGraph(identityId: string | null = null): Promise<OsintGraph> { return await invoke<OsintGraph>('get_osint_graph', { identityId }); }
export async function generateOsintReport(identityId: string | null = null): Promise<OsintReportSnapshot> { return await invoke<OsintReportSnapshot>('generate_osint_report', { identityId }); }
export async function getLatestOsintReport(): Promise<OsintReportSnapshot|null> { return await invoke<OsintReportSnapshot|null>('get_latest_osint_report'); }
export async function exportOsintReport(snapshotId:string, format:'markdown'|'pdf'): Promise<OsintReportExport> { return await invoke<OsintReportExport>('export_osint_report',{snapshotId,format}); }

export async function installOsintModule(moduleId: string): Promise<void> {
  await invoke('install_osint_module', { moduleId });
}
export async function rollbackOsintModule(moduleId: string): Promise<string> {
  return await invoke<string>('rollback_osint_module', { moduleId });
}
export async function removeOsintModuleRuntime(moduleId: string): Promise<string> {
  return await invoke<string>('remove_osint_module_runtime', { moduleId });
}
export async function cleanupOrphanedOsintRuntimes(): Promise<string> {
  return await invoke<string>('cleanup_orphaned_osint_runtimes');
}
export async function getRgpdReviewStatus(requestId:string):Promise<RgpdReviewStatus>{return await invoke<RgpdReviewStatus>('get_rgpd_review_status',{requestId});}
export async function validateRgpdDraft(requestId:string,checks:{sourceChecked:boolean;identityChecked:boolean;recipientChecked:boolean;contentChecked:boolean;legalNoticeAccepted:boolean}):Promise<RgpdReviewStatus>{return await invoke<RgpdReviewStatus>('validate_rgpd_draft',{requestId,...checks});}
export async function revokeRgpdDraftValidation(requestId:string):Promise<RgpdReviewStatus>{return await invoke<RgpdReviewStatus>('revoke_rgpd_draft_validation',{requestId});}
export async function saveRgpdDraftRevision(requestId:string,draftText:string):Promise<RgpdReviewStatus>{return await invoke<RgpdReviewStatus>('save_rgpd_draft_revision',{requestId,draftText});}
export async function useValidatedRgpdDraft(requestId:string,useType:'copie'|'export_texte'):Promise<RgpdDraftUse>{return await invoke<RgpdDraftUse>('use_validated_rgpd_draft',{requestId,useType});}
export async function installVeille(): Promise<string> { return await invoke<string>('install_veille'); }
export async function diagnoseOsintModule(moduleId: string): Promise<string> {
  return await invoke<string>('diagnose_osint_module', { moduleId });
}
export async function runRealOsintScan(moduleId: string, identityId: string, authorized: boolean): Promise<OsintScanSummary> {
  return await invoke<OsintScanSummary>('run_real_osint_scan', { moduleId, identityId, authorized });
}
export async function runVeilleScan(identityId: string, authorized: boolean): Promise<OsintScanSessionSummary> {
  return await invoke<OsintScanSessionSummary>('run_veille_scan', { identityId, authorized });
}
export async function listIdentityScanSessions(identityId: string, limit = 10, offset = 0): Promise<OsintScanSessionListItem[]> {
  return await invoke<OsintScanSessionListItem[]>('list_identity_scan_sessions', { identityId, limit, offset });
}
export async function getIdentityScanSession(identityId: string, sessionId: string): Promise<OsintScanSessionSummary> {
  return await invoke<OsintScanSessionSummary>('get_identity_scan_session', { identityId, sessionId });
}
export async function getVeilleRoutine(): Promise<VeilleRoutine> { return await invoke<VeilleRoutine>('get_veille_routine'); }
export async function updateVeilleRoutine(frequency: string, paused: boolean): Promise<VeilleRoutine> {
  return await invoke<VeilleRoutine>('update_veille_routine', { frequency, paused });
}
export async function runVeilleRoutine(authorized: boolean): Promise<OsintRoutineSummary> {
  return await invoke<OsintRoutineSummary>('run_veille_routine', { authorized });
}
export async function reviewOsintSignal(signalId: string, decision: 'confirmer' | 'pas_moi' | 'ignorer' | 'suivre', reason: string | null = null): Promise<string> {
  return await invoke<string>('review_osint_signal', { signalId, decision, reason });
}
export async function reviewOsintClaim(claimId:string,decision:'confirmer'|'pas_moi'|'ignorer'|'suivre',reason:string|null=null):Promise<string>{return await invoke<string>('review_osint_claim',{claimId,decision,reason});}
export async function getIdentityReviewWorkspace(identityId:string):Promise<OsintReviewWorkspace>{return await invoke<OsintReviewWorkspace>('get_identity_review_workspace',{identityId});}
export async function startOsintAnalysis(signalIds: string[]): Promise<string> { return await invoke<string>('start_osint_analysis', { signalIds }); }
export async function startOsintClaimSynthesis(identityId: string): Promise<string> { return await invoke<string>('start_osint_claim_synthesis', { identityId }); }
export async function getOsintAnalysisJob(jobId: string): Promise<OsintAnalysisJobStatus> { return await invoke<OsintAnalysisJobStatus>('get_osint_analysis_job', { jobId }); }
export async function getLatestIdentitySynthesis(identityId: string): Promise<OsintAnalysisReport|null> { return await invoke<OsintAnalysisReport|null>('get_latest_identity_synthesis', { identityId }); }
export async function getLocalAiAnalysisMode(): Promise<'automatic'|'manual'> { return await invoke<'automatic'|'manual'>('get_local_ai_analysis_mode'); }
export async function setLocalAiAnalysisMode(mode: 'automatic'|'manual'): Promise<string> { return await invoke<string>('set_local_ai_analysis_mode', { mode }); }
export type AppTheme = 'obsidian' | 'arctic' | 'ember' | 'verdant' | 'cyber' | 'cyberpunk' | 'osint';
export async function getAppTheme(): Promise<AppTheme> { return await invoke<AppTheme>('get_app_theme'); }
export async function setAppTheme(theme: AppTheme): Promise<AppTheme> { return await invoke<AppTheme>('set_app_theme', { theme }); }
export type AppLanguage = 'fr' | 'en';
export async function getAppLanguage(): Promise<AppLanguage> { return await invoke<AppLanguage>('get_app_language'); }
export async function setAppLanguage(language: AppLanguage): Promise<AppLanguage> { return await invoke<AppLanguage>('set_app_language', { language }); }
export async function createExposureFromOsintSignal(signalId: string): Promise<string> {
  return await invoke<string>('create_exposure_from_osint_signal', { signalId });
}
export async function createIncidentAndActionFromOsintSignal(signalId: string): Promise<string> {
  return await invoke<string>('create_incident_and_action_from_osint_signal', { signalId });
}
export async function createDpoRequestFromOsintSignal(signalId: string): Promise<string> {
  return await invoke<string>('create_dpo_request_from_osint_signal', { signalId });
}
export async function getLocalAiStatus(): Promise<LocalAiStatus> { return await invoke<LocalAiStatus>('get_local_ai_status'); }
export async function isLocalAiEnabled(): Promise<boolean> { return await invoke<boolean>('is_local_ai_enabled'); }
export async function installLocalAiRuntime(): Promise<string> { return await invoke<string>('install_local_ai_runtime'); }
export async function startLocalAiSetup(modelId: string): Promise<string> { return await invoke<string>('start_local_ai_setup', { modelId }); }
export async function pauseLocalAiDownload(downloadId: string): Promise<string> { return await invoke<string>('pause_local_ai_download', { downloadId }); }
export async function setLocalAiPreference(enabled: boolean, modelId: string | null, onboardingStatus: LocalAiStatus['onboarding_status']): Promise<string> {
  return await invoke<string>('set_local_ai_preference', { enabled, modelId, onboardingStatus });
}
export async function diagnoseLocalAi(): Promise<string> { return await invoke<string>('diagnose_local_ai'); }
export async function removeLocalAiComponents(): Promise<string> { return await invoke<string>('remove_local_ai_components'); }
