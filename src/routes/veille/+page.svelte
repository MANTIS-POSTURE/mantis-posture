<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-shell';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import NextStepBar from '$lib/NextStepBar.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import {
    listOsintModules, listOsintModuleInventory, listOsintModuleLogs, listIdentities, installVeille, installOsintModule,
    diagnoseOsintModule, rollbackOsintModule, removeOsintModuleRuntime, cleanupOrphanedOsintRuntimes,
    runVeilleScan, runVeilleRoutine, runRealOsintScan, getVeilleRoutine,
    listIdentityScanSessions, getIdentityScanSession,
    updateVeilleRoutine, createExposureFromOsintSignal, createIncidentAndActionFromOsintSignal,
    createDpoRequestFromOsintSignal, reviewOsintSignal, reviewOsintClaim, getIdentityReviewWorkspace, startOsintClaimSynthesis, getOsintAnalysisJob, getLatestIdentitySynthesis, getLocalAiAnalysisMode, getLocalAiStatus, isLocalAiEnabled,
    type Identity, type OsintModule, type OsintModuleLog, type OsintScanSummary, type OsintScanSessionSummary, type OsintScanSessionListItem,
    type OsintSignal, type OsintClaim, type OsintFactResolution, type OsintReviewWorkspace, type VeilleRoutine, type OsintAnalysisReport, type OsintAnalysisItem, type OsintAnalysisJobStatus, type LocalAiStatus
  } from '$lib/api';
  import { activeIdentityId, setActiveIdentityId } from '$lib/active-identity';
  import { getGuide } from '$lib/guides';
  import { t } from '$lib/i18n';
  import { getVeilleViewCache, setVeilleViewCache } from '$lib/veille-cache';

  type ResultGroup = { title: string; description: string; signals: OsintSignal[] };
  const groupOrder = [
    ['fuites', 'Fuites et compromissions', 'Données potentiellement présentes dans une fuite connue.'],
    ['comptes', 'Comptes et profils potentiels', 'Présences possibles sur des services ou plateformes.'],
    ['mentions', 'Mentions web publiques', 'Pages publiques qui semblent faire référence à l’identité.'],
    ['contacts', 'Données de contact visibles', 'Coordonnées ou moyens de contact possiblement visibles publiquement.'],
    ['relations', 'Relations et recoupements', 'Éléments qui méritent un rapprochement avec d’autres informations.'],
    ['verifier', 'Éléments à vérifier', 'Signaux à examiner avant de les retenir dans votre suivi.']
  ] as const;
  const preventiveGuides = ['guide-email-aliases', 'guide-proton-pass', 'guide-compartmentalization']
    .map(getGuide).filter((guide) => guide !== undefined);

  let modules = $state<OsintModule[]>([]);
  let moduleInventory = $state<OsintModule[]>([]);
  let identities = $state<Identity[]>([]);
  let routine = $state<VeilleRoutine | null>(null);
  let routineOpen = $state(false);
  let selectedIdentityId = $state('');
  let authorized = $state(false);
  let routineAuthorized = $state(false);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  let result = $state<OsintScanSummary | OsintScanSessionSummary | null>(null);
  let advanced = $state(false);
  let advancedModuleId = $state('');
  let diagnostic = $state<string | null>(null);
  let maintenanceNotice = $state<string | null>(null);
  let logs = $state<OsintModuleLog[]>([]);
  let incidentFor = $state<string | null>(null);
  let dpoFor = $state<string | null>(null);
  let analysis = $state<OsintAnalysisReport | null>(null);
  let analyzing = $state(false);
  let analysisJob = $state<OsintAnalysisJobStatus | null>(null);
  let analysisMode = $state<'automatic' | 'manual'>('automatic');
  let aiStatus = $state<LocalAiStatus | null>(null);
  let noAiAlertVisible = $state(false);
  let analysisPoll: ReturnType<typeof setInterval> | null = null;
  let advancedWorkspaceLoaded = $state(false);
  let sessions = $state<OsintScanSessionListItem[]>([]);
  let reviewWorkspace = $state<OsintReviewWorkspace>({ events:[], evolution:[] });
  let reviewWorkspaceLoaded = $state(false);
  let historyHasMore = $state(false);
  let historyLoading = $state(false);
  let scanPhase = $state<'idle' | 'tools' | 'ai' | 'done'>('idle');
  let zeroIdentityPromptOpen = $state(false);
  let zeroIdentityPromptShown = false;
  let projectedActionIds = $state<Record<string, string>>({});
  let postScanExpanded = $state(false);
  const HISTORY_PAGE_SIZE = 8;

  const scanIdentities = $derived(identities.filter((identity) => identity.values.some((value) => value.status === 'active' && ['email', 'pseudo', 'nom', 'prenom', 'telephone', 'adresse'].includes(value.kind))));
  const selectedModule = $derived(modules.find((module) => module.id === advancedModuleId) ?? null);
  const selectedModuleManaged = $derived(selectedModule !== null && selectedModule.id !== 'osint-email-intel');
  const archivedModules = $derived(moduleInventory.filter((module) => module.catalog_status !== 'active'));
  const advancedTargets = $derived.by(() => {
    if (!selectedModule) return [];
    if (selectedModule.id === 'osint-email-intel' || selectedModule.id === 'mock-osint') return identities.filter((identity) => identity.values.some((value) => value.status === 'active' && value.kind === 'email'));
    if (selectedModule.id === 'osint-email-platforms') return identities.filter((identity) => identity.values.some((value) => value.status === 'active' && ['email', 'pseudo'].includes(value.kind)));
    if (selectedModule.id === 'osint-username-profiles') return identities.filter((identity) => identity.values.some((value) => value.status === 'active' && value.kind === 'pseudo'));
    return scanIdentities;
  });
  const rawSignals = $derived(result?.signals ?? []);
  const visibleSignals = $derived(rawSignals.filter((signal) => signal.review_status !== 'Filtré automatiquement'));
  const hiddenDdgsSignals = $derived(rawSignals.filter((signal) => signal.source === 'DDGS' && signal.review_status === 'Filtré automatiquement'));
  const hiddenCandidateCount = $derived(rawSignals.length - visibleSignals.length - hiddenDdgsSignals.length);
  const resultGroups = $derived(groupResults(visibleSignals));
  const resultSummary = $derived.by(() => {
    const signals = visibleSignals;
    const ignored = signals.filter((signal) => ['Ignoré', 'Ce n’est pas moi'].includes(signal.review_status)).length;
    const followed = signals.filter((signal) => ['Confirmé', 'Suivi', 'Traité'].includes(signal.review_status)).length;
    return { total: signals.length, ignored, followed, uncertain: signals.length - ignored - followed };
  });
  const postScanQueue = $derived.by(() => {
    const severityRank: Record<string, number> = { critique: 0, elevee: 1, élevée: 1, moderee: 2, modérée: 2, faible: 3 };
    // An exact DDGS result is often a useful lead even when the automatic
    // quality gate keeps it out of the ordinary result list. It belongs in
    // the explicit human-decision area, never in an automatic projection.
    const candidates = rawSignals
      .filter((signal) => !['Ignoré', 'Ce n’est pas moi'].includes(signal.review_status))
      .filter((signal) => signal.source !== 'DDGS'
        || ['Confirmé', 'Suivi', 'Traité'].includes(signal.review_status)
        || signal.explanation.toLowerCase().includes('preuve publique vérifiée')
        || (signal.explanation.toLowerCase().includes('requête exacte ddgs') && targetMatchCoverage(signal) >= 0.67)
        || exactTargetAppears(signal))
      .sort((left, right) => {
        const sourceRank = Number(right.source === 'DDGS') - Number(left.source === 'DDGS');
        return sourceRank || (severityRank[left.severity] ?? 9) - (severityRank[right.severity] ?? 9);
      });
    const bestPerWebsite = new Map<string, OsintSignal>();
    for (const signal of candidates) {
      const key = websiteKey(signal);
      const current = bestPerWebsite.get(key);
      if (!current || findingValue(signal) > findingValue(current)) bestPerWebsite.set(key, signal);
    }
    const distinctCandidates = [...bestPerWebsite.values()];
    const exactPublicFootprints = distinctCandidates.filter((signal) => signal.source === 'DDGS' && signal.source_url && exactTargetAppears(signal) && (signal.review_status !== 'Filtré automatiquement' || claimBackedSignal(signal)));
    const important = distinctCandidates.filter((signal) => signal.signal_type === 'fuite' || (severityRank[signal.severity] ?? 9) < 3 || claimBackedSignal(signal));
    // A directly accessible public profile or portfolio gives the user an
    // immediate, reversible decision to make. It is more actionable than an
    // old breach with no concrete remediation path, while remaining a
    // potential match until the user reviews it.
    const priority = [...exactPublicFootprints, ...important, ...distinctCandidates]
      .filter((signal, index, items) => items.findIndex((item) => item.id === signal.id) === index);
    return priority;
  });
  const xposedOrNotSignals = $derived(postScanQueue.filter((signal) => signal.source === 'XposedOrNot'));
  const userScannerSignals = $derived(postScanQueue.filter((signal) => signal.source === 'User Scanner'));
  const postScanDisplayQueue = $derived.by(() => {
    const firstXposed = postScanQueue.find((signal) => signal.source === 'XposedOrNot');
    const firstUserScanner = postScanQueue.find((signal) => signal.source === 'User Scanner');
    const withoutGrouped = postScanQueue.filter((signal) => signal.source !== 'XposedOrNot' && signal.source !== 'User Scanner');
    return [firstXposed, firstUserScanner, ...withoutGrouped].filter((signal): signal is OsintSignal => Boolean(signal));
  });
  const postScanVisible = $derived(postScanExpanded ? postScanDisplayQueue : postScanDisplayQueue.slice(0, 9));
  const veilleReady = $derived(modules.filter((m) => ['osint-email-platforms', 'osint-username-profiles', 'osint-web-footprint'].includes(m.id)).every((m) => m.installation_status === 'prêt'));
  const sessionResult = $derived(result && 'session_id' in result ? result : null);
  const claims = $derived<OsintClaim[]>(sessionResult?.claims ?? []);
  const resolutions = $derived<OsintFactResolution[]>(sessionResult?.resolutions ?? []);
  const resolutionSummary = $derived.by(() => ({
    corroborated: resolutions.filter((item) => item.status === 'corroboree').length,
    toVerify: resolutions.filter((item) => item.status === 'a_verifier').length,
    contradictory: resolutions.filter((item) => item.status === 'contradictoire').length,
    rejected: resolutions.filter((item) => item.status === 'rejete').length
  }));
  const localAiReady = $derived(Boolean(aiStatus?.enabled && aiStatus.installed && aiStatus.integrity_ok && aiStatus.models.some((model) => model.installed && model.compatible)));
  const aiGuidedView = $derived(localAiReady && analysis?.mode === 'ia_locale' && !advanced);
  const aiAnalysisPending = $derived(localAiReady && !advanced && Boolean(result) && !analysis);
  const presentedClaims = $derived.by(() => {
    if (!aiGuidedView || !analysis) return claims;
    const rank: Record<string, number> = { securiser: 0, verifier: 1, suivre: 2, ignorer: 3 };
    return claims.filter((claim) => synthesisFor(claim)?.recommended_action !== 'ignorer')
      .sort((left, right) => (rank[synthesisFor(left)?.recommended_action ?? 'verifier'] ?? 9) - (rank[synthesisFor(right)?.recommended_action ?? 'verifier'] ?? 9));
  });
  const aiFilteredClaims = $derived(Math.max(0, claims.length - presentedClaims.length));

  async function loadAiState(identityId: string, session: OsintScanSessionSummary | null) {
    if (!localAiReady || !session || !session.claims.length) return;
    const latestAnalysis = await getLatestIdentitySynthesis(identityId);
    if (identityId !== selectedIdentityId) return;
    analysis = latestAnalysis;
    if (!analysis) void ensureGuidedAnalysis();
  }

  async function loadReviewWorkspace() {
    if (!advanced || !selectedIdentityId || reviewWorkspaceLoaded) return;
    reviewWorkspaceLoaded = true;
    try {
      reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId);
      setVeilleViewCache(selectedIdentityId, { sessions, result: sessionResult, reviewWorkspace, analysis, historyHasMore });
    } catch (cause) {
      reviewWorkspaceLoaded = false;
      error ??= String(cause);
    }
  }

  async function loadAdvancedWorkspace() {
    if (!advanced || advancedWorkspaceLoaded) return;
    advancedWorkspaceLoaded = true;
    try {
      [modules, moduleInventory] = await Promise.all([listOsintModules(), listOsintModuleInventory()]);
      if (!advancedModuleId) advancedModuleId = modules[0]?.id ?? '';
      if (advancedModuleId) logs = await listOsintModuleLogs(advancedModuleId);
    } catch (cause) {
      advancedWorkspaceLoaded = false;
      error ??= String(cause);
    }
  }

  function identitySummary(identity: Identity): string {
    const active = identity.values.filter((value) => value.status === 'active');
    const kinds = [...new Set(active.map((value) => value.kind))];
    return `${active.length} donnée${active.length > 1 ? 's' : ''} · ${kinds.join(', ')}`;
  }

  function websiteKey(signal: OsintSignal): string {
    if (!signal.source_url) return `signal:${signal.id}`;
    try { return `site:${new URL(signal.source_url).hostname.replace(/^www\./, '').toLowerCase()}`; }
    catch { return `signal:${signal.id}`; }
  }

  function findingValue(signal: OsintSignal): number {
    if (!signal.source_url) return 0;
    let path = '';
    try { path = decodeURIComponent(new URL(signal.source_url).pathname).toLowerCase(); } catch { /* baseline only */ }
    const text = `${signal.title} ${signal.explanation}`.toLowerCase();
    let score = 20;
    if (text.includes('preuve publique vérifiée')) score += 30;
    if (/\/(?:in|profile|people|users)\//.test(path) || /instagram\.com\/(?!stories)/.test(signal.source_url)) score += 35;
    if (['/directory/', '/search/', '/stories/'].some((part) => path.includes(part))) score -= 45;
    if (path.includes(signal.target.toLowerCase().replaceAll(' ', '-').replaceAll('é', 'e'))) score += 15;
    return score;
  }

  function normalizedMatchText(value: string): string {
    return value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '');
  }

  function targetMatchCoverage(signal: OsintSignal): number {
    const rawTarget = signal.target.trim();
    const tokens = rawTarget.split(/\s+/).map(normalizedMatchText).filter((token) => token.length >= 2);
    if (!tokens.length) return 0;
    const haystack = normalizedMatchText(`${signal.title} ${signal.explanation} ${signal.source_url ?? ''}`);
    return tokens.filter((token) => haystack.includes(token)).length / tokens.length;
  }

  function exactTargetAppears(signal: OsintSignal): boolean {
    const target = normalizedMatchText(signal.target);
    if (target.length < 4) return false;
    return [signal.title, signal.explanation, signal.source_url ?? '']
      .some((value) => normalizedMatchText(value).includes(target));
  }

  function claimBackedSignal(signal: OsintSignal): boolean {
    if (!signal.source_url || !result || !('claims' in result) || !exactTargetAppears(signal)) return false;
    return result.claims.some((claim) => claim.status !== 'rejetee'
      && claim.evidence.some((evidence) => evidence.source_url === signal.source_url));
  }

  onMount(() => {
    routineOpen = window.location.hash === '#routine';
    void load();
  });
  onDestroy(() => { if (analysisPoll) clearInterval(analysisPoll); });

  async function load() {
    loading = true;
    const requestedIdentityId = new URL(window.location.href).searchParams.get('identity');
    const cachedIdentityId = selectedIdentityId || requestedIdentityId || $activeIdentityId || '';
    const cachedView = getVeilleViewCache(cachedIdentityId);
    if (cachedView) {
      selectedIdentityId = cachedIdentityId;
      sessions = cachedView.sessions;
      result = cachedView.result;
      reviewWorkspace = cachedView.reviewWorkspace;
      analysis = cachedView.analysis;
      historyHasMore = cachedView.historyHasMore;
    }
    try {
      [identities, routine] = await Promise.all([listIdentities(), getVeilleRoutine()]);
      if (identities.length === 0 && !zeroIdentityPromptShown) {
        zeroIdentityPromptOpen = true;
        zeroIdentityPromptShown = true;
      }
      if (!selectedIdentityId) {
        const requested = requestedIdentityId;
        selectedIdentityId = scanIdentities.find((identity) => identity.id === requested)?.id
          ?? scanIdentities.find((identity) => identity.id === $activeIdentityId)?.id
          ?? scanIdentities[0]?.id ?? '';
		setActiveIdentityId(selectedIdentityId || null);
		const cachedView = getVeilleViewCache(selectedIdentityId);
		if (cachedView) {
			sessions = cachedView.sessions;
			result = cachedView.result;
			reviewWorkspace = cachedView.reviewWorkspace;
			analysis = cachedView.analysis;
			historyHasMore = cachedView.historyHasMore;
		}
      }
    } catch (e) { error = String(e); }
    finally { loading = false; }

    // The normal view loads only the identity, routine and compact history.
    // Tool diagnostics/logs are advanced-only; the model state is queried only
    // after the persisted local-AI preference explicitly enables it.
    void Promise.all([
      isLocalAiEnabled().then(async (enabled) => {
        if (!enabled) { aiStatus = null; return; }
        const [status, mode] = await Promise.all([getLocalAiStatus(), getLocalAiAnalysisMode()]);
        aiStatus = status;
        analysisMode = mode;
        if (localAiReady && sessionResult) void loadAiState(selectedIdentityId, sessionResult);
      }),
      selectedIdentityId ? loadIdentityHistory(true) : Promise.resolve()
    ]).catch((cause) => { error ??= String(cause); });
  }

	$effect(() => {
		const globalIdentityId = $activeIdentityId;
		if (!loading && globalIdentityId && globalIdentityId !== selectedIdentityId && scanIdentities.some((identity) => identity.id === globalIdentityId)) {
			selectedIdentityId = globalIdentityId;
			void loadIdentityHistory(true);
		}
	});

  $effect(() => {
    if (advanced) {
      void loadReviewWorkspace();
      void loadAdvancedWorkspace();
    }
  });

  function groupResults(signals: OsintSignal[]): ResultGroup[] {
    const buckets: Record<string, OsintSignal[]> = Object.fromEntries(groupOrder.map(([key]) => [key, []]));
    for (const signal of signals) {
      const text = `${signal.title} ${signal.explanation}`.toLowerCase();
      const key = signal.signal_type === 'fuite' ? 'fuites'
        : ['compte_potentiel', 'profil_public'].includes(signal.signal_type) ? 'comptes'
        : /(?:contact|dpo|mailto|courriel|e-mail|email|téléphone|adresse)/.test(text) ? 'contacts'
        : /(?:relation|recoup|associé|lié)/.test(text) ? 'relations'
        : signal.signal_type === 'mention' ? 'mentions' : 'verifier';
      buckets[key].push(signal);
    }
    const severityRank: Record<string, number> = { critique:0, elevee:1, élevée:1, moderee:2, modérée:2, faible:3 };
    const statusRank: Record<string, number> = { 'À vérifier':0, Suivi:1, Confirmé:1, Traité:2, Ignoré:3, 'Ce n’est pas moi':3 };
    const sortSignals = (left: OsintSignal, right: OsintSignal) =>
      (severityRank[left.severity] ?? 9) - (severityRank[right.severity] ?? 9)
      || Number(!left.source_url) - Number(!right.source_url)
      || (statusRank[left.review_status] ?? 9) - (statusRank[right.review_status] ?? 9)
      || left.title.localeCompare(right.title, 'fr');
    return groupOrder.map(([key, title, description]) => ({ title, description, signals: buckets[key].sort(sortSignals) })).filter((group) => group.signals.length > 0);
  }

  async function loadIdentityHistory(reset = false) {
    if (!selectedIdentityId) { sessions = []; result = null; return; }
    historyLoading = true;
    try {
      const offset = reset ? 0 : sessions.length;
      const page = await listIdentityScanSessions(selectedIdentityId, HISTORY_PAGE_SIZE + 1, offset);
      historyHasMore = page.length > HISTORY_PAGE_SIZE;
      const visible = page.slice(0, HISTORY_PAGE_SIZE);
      sessions = reset ? visible : [...sessions, ...visible];
      if (reset) {
        const session = sessions[0]
          ? await getIdentityScanSession(selectedIdentityId, sessions[0].id)
          : null;
        result = session;
        noAiAlertVisible = Boolean(result && !localAiReady);
        analysis = null; analysisJob = null;
        reviewWorkspace = { events: [], evolution: [] };
        reviewWorkspaceLoaded = false;
        if (session) void loadAiState(selectedIdentityId, session);
        setVeilleViewCache(selectedIdentityId, { sessions, result: session, reviewWorkspace, analysis, historyHasMore });
      }
    } catch (cause) { error = String(cause); }
    finally { historyLoading = false; }
  }

  async function selectSession(sessionId: string) {
    historyLoading = true; error = null;
    try {
      const session = await getIdentityScanSession(selectedIdentityId, sessionId);
      result = session; analysis = null; analysisJob = null;
      reviewWorkspace = { events: [], evolution: [] };
      reviewWorkspaceLoaded = false;
      void loadAiState(selectedIdentityId, session);
      setVeilleViewCache(selectedIdentityId, { sessions, result: session, reviewWorkspace, analysis, historyHasMore });
    }
    catch (cause) { error = String(cause); }
    finally { historyLoading = false; }
  }

  function displayText(text: string) { return text.replaceAll('�', '…').replace(/\s+/g, ' ').trim(); }
  function sourceDomain(url: string | null) { try { return url ? new URL(url).hostname.replace(/^www\./, '') : null; } catch { return null; } }
  function formatDate(value: string | null) { return value || 'Pas encore exécuté'; }
  function targetKindLabel(kind: string) { return ({ email:'E-mail', pseudo:'Pseudo', nom:'Nom complet', prenom:'Prénom', telephone:'Téléphone', adresse:'Adresse' } as Record<string,string>)[kind] ?? 'Autre donnée'; }
  function claimTypeLabel(value: string) { return t(({ fuite_eventuelle:'Fuite potentielle', profil_potentiel:'Profil potentiel', annuaire:'Annuaire', site_de_rencontre:'Site de rencontre', mention_web:'Mention web', source_indisponible:'Source indisponible' } as Record<string,string>)[value] ?? 'Résultat à vérifier'); }
  function claimStatusLabel(value: string) { return ({ a_verifier:'À vérifier', corroboree:'Sources croisées', confirmee:'Confirmé par vous', contradictoire:'Contradiction', rejetee:'Écarté par vous' } as Record<string,string>)[value] ?? value; }
  function resolutionStatusLabel(value: OsintFactResolution['status']) { return ({ a_verifier:'À vérifier', corroboree:'Sources croisées', contradictoire:'À clarifier', rejete:'Écarté' } as Record<string,string>)[value]; }
  function exposedDataLabels(signal: OsintSignal): string[] {
    const sourceText = `${signal.title} ${signal.explanation}`;
    const declaredData = sourceText.match(/données déclarées\s*:\s*([^.]*)/i)?.[1]
      ?.split(/[,;·]/).map((item) => item.trim()).filter(Boolean);
    if (declaredData?.length) return declaredData.slice(0, 4);
    const text = sourceText.toLocaleLowerCase('fr');
    const labels = [
      ['mot de passe', 'Mot de passe'], ['password', 'Mot de passe'], ['e-mail', 'E-mail'], ['email', 'E-mail'], ['courriel', 'E-mail'],
      ['téléphone', 'Téléphone'], ['telephone', 'Téléphone'], ['adresse', 'Adresse'], ['nom', 'Nom'], ['date de naissance', 'Date de naissance'],
      ['ip', 'Adresse IP'], ['hash', 'Hash']
    ].filter(([needle]) => text.includes(needle)).map(([, label]) => label);
    return [...new Set(labels)].slice(0, 4).length ? [...new Set(labels)].slice(0, 4) : signal.signal_type === 'fuite' ? ['Données de compte non détaillées'] : ['Donnée publique à vérifier'];
  }
  function severityLabel(value: string) { return ({ critique: 'Critique', elevee: 'Élevée', élevée: 'Élevée', moderee: 'Modérée', modérée: 'Modérée', faible: 'Faible' } as Record<string,string>)[value] ?? value; }

  async function installAll() {
    busy = true; error = null; notice = null;
    try { notice = await installVeille(); await load(); }
    catch (e) { error = String(e); }
    finally { busy = false; }
  }
  async function launchScan() {
    busy = true; error = null; notice = null; result = null; analysis = null; incidentFor = null; dpoFor = null; analysisJob = null; scanPhase = 'tools';
    try {
      result = await runVeilleScan(selectedIdentityId, authorized);
      noAiAlertVisible = !localAiReady;
      if (localAiReady && result.analysis_job_id) void monitorAnalysis(result.analysis_job_id);
      else if (localAiReady && 'claims' in result && result.claims.length) void ensureGuidedAnalysis();
      scanPhase = 'done';
      void loadIdentityHistory(true);
    }
    catch (e) { error = String(e); scanPhase = 'idle'; }
    finally { busy = false; if (scanPhase === 'done') window.setTimeout(() => { scanPhase = 'idle'; }, 700); }
  }
  async function analyzeResult() {
    if (!localAiReady || !selectedIdentityId || claims.length === 0) return;
    analyzing = true; error = null; analysis = null;
    try { const jobId = await startOsintClaimSynthesis(selectedIdentityId); await monitorAnalysis(jobId); }
    catch (e) { error = String(e); }
  }
  async function ensureGuidedAnalysis() {
    if (!localAiReady || analysisMode !== 'automatic' || !selectedIdentityId || !result || !('claims' in result) || !result.claims.length || analysis || analyzing) return;
    try { scanPhase = 'ai'; await monitorAnalysis(await startOsintClaimSynthesis(selectedIdentityId)); }
    catch (e) { error = String(e); }
  }
  function dismissNoAiAlert() { noAiAlertVisible = false; }
  async function monitorAnalysis(jobId: string) {
    if (!localAiReady) return;
    if (analysisPoll) clearInterval(analysisPoll);
    analyzing = true;
    scanPhase = 'ai';
    const refresh = async () => {
      try {
        analysisJob = await getOsintAnalysisJob(jobId);
        if (['termine','fallback','erreur','interrompu'].includes(analysisJob.status)) {
          if (analysisPoll) clearInterval(analysisPoll); analysisPoll = null; analyzing = false;
          if (analysisJob.report) analysis = analysisJob.report;
          if (analysisJob.status === 'erreur') error = analysisJob.message ?? 'L’analyse locale a échoué.';
          else notice = analysisJob.message;
          scanPhase = 'done';
          window.setTimeout(() => { if (!busy) scanPhase = 'idle'; }, 700);
        }
      } catch (e) { if (analysisPoll) clearInterval(analysisPoll); analysisPoll = null; analyzing = false; error = String(e); }
    };
    await refresh();
    if (analyzing) analysisPoll = setInterval(() => { void refresh(); }, 1500);
  }
  function analysisFor(signal: OsintSignal): OsintAnalysisItem | null { return analysis?.items.find((item) => item.observation_id === `observation-${signal.id}`) ?? null; }
  function synthesisFor(claim: OsintClaim) { return analysis?.findings?.find((finding) => finding.claim_id === claim.id) ?? null; }
  function relevanceLabel(value: OsintAnalysisItem['relevance']) { return value === 'important' ? 'Important' : value === 'bruit' ? 'Bruit probable' : 'À vérifier'; }
  async function saveRoutine(frequency = routine?.frequency ?? 'Manuelle', paused = routine?.paused ?? false) {
    busy = true; error = null;
    try { routine = await updateVeilleRoutine(frequency, paused); }
    catch (e) { error = String(e); }
    finally { busy = false; }
  }
  async function launchRoutine() {
    busy = true; error = null; notice = null;
    try { const summary = await runVeilleRoutine(routineAuthorized); notice = `${summary.message} ${summary.analysis_jobs_started} analyse(s) locale(s) lancée(s) en arrière-plan.`; routine = await getVeilleRoutine(); await load(); }
    catch (e) { error = String(e); }
    finally { busy = false; }
  }
  async function openExternal(event: MouseEvent, url: string) {
    event.preventDefault();
    try { await open(url); } catch (e) { error = `Impossible d’ouvrir ce lien : ${String(e)}`; }
  }
  async function createExposure(signalId: string) {
    busy = true; error = null;
    try {
      const exposureId = await createExposureFromOsintSignal(signalId);
      if (result) result = { ...result, signals: result.signals.map((s) => s.id === signalId ? { ...s, exposure_id: exposureId } : s) };
      reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId);
    } catch (e) { error = String(e); } finally { busy = false; }
  }
  async function reviewSignal(signalId: string, decision: 'confirmer' | 'pas_moi' | 'ignorer' | 'suivre') {
    busy = true; error = null;
    try {
      const reviewStatus = await reviewOsintSignal(signalId, decision);
      if (result) result = { ...result, signals: result.signals.map((signal) => signal.id === signalId ? { ...signal, review_status: reviewStatus } : signal) };
      if (sessionResult) result = await getIdentityScanSession(selectedIdentityId, sessionResult.session_id);
      reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId);
    } catch (e) { error = String(e); } finally { busy = false; }
  }
  async function reviewClaim(claimId: string, decision: 'confirmer' | 'pas_moi' | 'ignorer' | 'suivre') {
    busy = true; error = null;
    try { await reviewOsintClaim(claimId, decision); if (sessionResult) { result = await getIdentityScanSession(selectedIdentityId, sessionResult.session_id); void loadAiState(selectedIdentityId, sessionResult); } reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId); }
    catch (e) { error = String(e); } finally { busy = false; }
  }
  async function createIncident(signal: OsintSignal) {
    busy = true; error = null;
    try { await createIncidentAndActionFromOsintSignal(signal.id); incidentFor = signal.id; reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId); }
    catch (e) { error = String(e); } finally { busy = false; }
  }
  async function retainAndCreatePlan(signal: OsintSignal) {
    busy = true; error = null; notice = null;
    try {
      if (!['Confirmé', 'Suivi', 'Traité'].includes(signal.review_status)) await reviewOsintSignal(signal.id, 'confirmer');
      const exposureId = await createExposureFromOsintSignal(signal.id);
      const actionId = await createIncidentAndActionFromOsintSignal(signal.id);
      incidentFor = signal.id;
      projectedActionIds = { ...projectedActionIds, [signal.id]: actionId };
      if (sessionResult) result = await getIdentityScanSession(selectedIdentityId, sessionResult.session_id);
      reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId);
      notice = `Exposition, incident et action créés à partir de « ${displayText(signal.title)} ».`;
      if (result) result = { ...result, signals: result.signals.map((item) => item.id === signal.id ? { ...item, exposure_id: exposureId, review_status: 'Traité' } : item) };
    } catch (e) { error = String(e); } finally { busy = false; }
  }
  async function prepareDpo(signal: OsintSignal) {
    busy = true; error = null;
    try { dpoFor = await createDpoRequestFromOsintSignal(signal.id); reviewWorkspace = await getIdentityReviewWorkspace(selectedIdentityId); }
    catch (e) { error = String(e); } finally { busy = false; }
  }
  async function selectAdvancedModule(id: string) {
    advancedModuleId = id; diagnostic = null; logs = await listOsintModuleLogs(id);
  }
  async function installAdvanced() { if (!selectedModule) return; busy = true; error = null; try { await installOsintModule(selectedModule.id); await load(); } catch (e) { error = String(e); } finally { busy = false; } }
  async function diagnoseAdvanced() { if (!selectedModule) return; busy = true; try { diagnostic = await diagnoseOsintModule(selectedModule.id); logs = await listOsintModuleLogs(selectedModule.id); } catch (e) { error = String(e); } finally { busy = false; } }
  async function rollbackAdvanced() {
    if (!selectedModule || !selectedModuleManaged || !confirm(`Restaurer la version précédente de ${selectedModule.name} ?`)) return;
    busy = true; error = null;
    try { maintenanceNotice = await rollbackOsintModule(selectedModule.id); await load(); }
    catch (e) { error = String(e); } finally { busy = false; }
  }
  async function removeAdvancedRuntime() {
    if (!selectedModule || !selectedModuleManaged || !confirm(`Désinstaller le moteur ${selectedModule.name} ? Les scans et preuves déjà enregistrés seront conservés.`)) return;
    busy = true; error = null;
    try { maintenanceNotice = await removeOsintModuleRuntime(selectedModule.id); await load(); }
    catch (e) { error = String(e); } finally { busy = false; }
  }
  async function cleanupOrphans() {
    if (!confirm('Nettoyer uniquement les anciens moteurs et dossiers temporaires ? Les scans, preuves, résultats bruts et données IA seront conservés.')) return;
    busy = true; error = null;
    try { maintenanceNotice = await cleanupOrphanedOsintRuntimes(); moduleInventory = await listOsintModuleInventory(); }
    catch (e) { error = String(e); } finally { busy = false; }
  }
  async function runAdvanced() { if (!selectedModule) return; busy = true; error = null; result = null; try { result = await runRealOsintScan(selectedModule.id, selectedIdentityId, authorized); if (localAiReady && analysisMode === 'automatic' && result.signals.length) await monitorAnalysis(await startOsintClaimSynthesis(selectedIdentityId)); logs = await listOsintModuleLogs(selectedModule.id); } catch (e) { error = String(e); } finally { busy = false; } }
</script>

<section class="wf-view" class:has-results={Boolean(result)}>
  <div class="scan-page-header">
    <GuideHeader title="Scanner" question="Qu’est-ce qui est publiquement visible à propos de cette identité ?" intro="Préparez les sources, lancez un scan autorisé, puis examinez chaque résultat avant d’en tirer une conclusion." />
    <details class="history-panel">
      <summary><span><strong>{t('Historique')}</strong><small>{sessions.length || '—'}</small></span><b aria-hidden="true">⌄</b></summary>
      <div class="history-strip" aria-label={t('Historique des scans')}>
        {#each sessions as session (session.id)}
          <button class:active={sessionResult?.session_id === session.id} onclick={() => selectSession(session.id)} disabled={historyLoading}>
            <strong>{formatDate(session.started_at)}</strong><span>{session.signal_count} signal{session.signal_count > 1 ? 's' : ''} · {session.status}</span>
          </button>
        {:else}<p class="muted">{t('Aucune vérification enregistrée pour cette identité.')}</p>{/each}
        {#if historyHasMore}<button class="more" onclick={() => loadIdentityHistory(false)} disabled={historyLoading}>{t('Afficher plus')}</button>{/if}
      </div>
    </details>
  </div>

  <Modal open={zeroIdentityPromptOpen} title="Aucune identité configurée" description="Ajoutez d’abord les informations que vous souhaitez surveiller. MANTIS ne recherchera que les identifiants que vous aurez vous-même renseignés." onClose={() => zeroIdentityPromptOpen = false}>
    {#snippet actions()}
      <button class="wf-btn" type="button" onclick={() => zeroIdentityPromptOpen = false}>Plus tard</button>
      <a class="wf-btn primary" href="/identites">Configurer une identité</a>
    {/snippet}
  </Modal>

  {#if loading && !result}
    <div class="glass-card"><p class="muted">{t('Préparation de votre veille…')}</p></div>
  {:else}
    {#if error}<p class="error">Erreur : {error}</p>{/if}
    {#if notice}<p class="notice">{notice}</p>{/if}

    <div class="veille-workspace">
    <article class="glass-card identity-command">
      <div class="identity-selector"><div><p class="eyebrow">{t('Préparer le scan')}</p><h2>{scanIdentities.find((identity) => identity.id === selectedIdentityId)?.label ?? t('Choisissez une identité')}</h2><p class="muted">{t('Seules les données actives et compatibles seront utilisées. Les résultats ne compléteront jamais cette identité automatiquement.')}</p></div></div>
      <div class="scan-command">
        <label class="authorization"><input type="checkbox" bind:checked={authorized} /> {t('Je suis autorisé à analyser cette identité.')}</label>
        <button class="wf-btn primary scan-primary" onclick={launchScan} disabled={busy || !veilleReady || !selectedIdentityId || !authorized}>{t(busy ? 'Scan en cours…' : 'Démarrer le scan →')}</button>
      </div>
    </article>

    {#if scanPhase !== 'idle'}
      <article class:ai-phase={scanPhase === 'ai'} class="glass-card scan-progress" role="status" aria-live="polite">
        <div class="scan-progress-heading"><div><p class="eyebrow">{t('Traitement en cours')}</p><h2>{t(scanPhase === 'tools' ? 'Recherche dans les sources utiles' : scanPhase === 'ai' ? 'Mise en contexte de vos résultats' : 'Veille terminée')}</h2></div><span class="progress-pulse" aria-hidden="true"></span></div>
        <div class="scan-stages">
          <div class:active={scanPhase === 'tools'} class:complete={['ai','done'].includes(scanPhase)} class="scan-stage tools-stage"><span class="stage-visual tools-visual" aria-hidden="true"><i></i><i></i><i></i><b></b></span><div><strong>{t('Outils de veille')}</strong><small>{t(scanPhase === 'tools' ? 'Interrogation des sources compatibles…' : 'Collecte terminée')}</small></div></div>
          <div class:active={scanPhase === 'ai'} class:complete={scanPhase === 'done'} class="scan-stage ai-stage"><span class="stage-visual ai-stage-visual" aria-hidden="true"><i></i><i></i><i></i><b></b></span><div><strong>{t('IA locale')}</strong><small>{t(scanPhase === 'ai' ? 'Regroupement, tri et recoupement des signaux…' : scanPhase === 'done' ? 'Présentation prête' : 'En attente des résultats')}</small></div></div>
        </div>
        <div class="scan-progress-track"><div class:ai-fill={scanPhase === 'ai'} class="scan-progress-fill" style={`width:${scanPhase === 'tools' ? '38%' : scanPhase === 'ai' ? `${analysisJob ? Math.min(96, Math.max(42, Math.round((analysisJob.elapsed_seconds / Math.max(1, analysisJob.estimated_seconds)) * 100))) : 58}%` : '100%'}`}></div></div>
        <p class="scan-progress-detail">{t(scanPhase === 'tools' ? 'Vérification des sources compatibles en cours…' : scanPhase === 'ai' && analysisJob ? `${analysisJob.signal_count} élément(s) à relire · environ ${Math.max(1, Math.ceil((analysisJob.estimated_seconds - analysisJob.elapsed_seconds) / 10) * 10)} seconde(s) restantes` : scanPhase === 'ai' ? 'Préparation d’une lecture claire…' : 'Vos résultats sont prêts.')}</p>
      </article>
    {/if}
    <div class="main-grid">
      {#if !veilleReady}<article class="glass-card step-card">
        <p class="eyebrow">{t('Première étape')}</p><h2>{t('Installer la veille')}</h2>
        <p>{t('Prépare les sources disponibles sur cet appareil. Cette étape ne consulte encore aucune information.')}</p>
        <button class="wf-btn primary" onclick={installAll} disabled={busy}>{t(busy ? 'Préparation en cours…' : 'Préparer la veille')}</button>
      </article>{:else}<article class="glass-card step-card ready-card"><p class="eyebrow">{t('Tout est prêt')}</p><h2>{t('Votre veille est opérationnelle')}</h2><p class="muted">{t('Les sources compatibles sont prêtes. Lancez une vérification ou programmez une routine.')}</p><span class="ready">✓ {t('Sources prêtes')}</span></article>{/if}
    </div>

    <details id="routine" bind:open={routineOpen} class="secondary-panel routine-panel">
      <summary><span><strong>{t('Routine de scan')}</strong><small>{t('Surveiller progressivement vos identités dans le temps.')}</small></span><b>{t('Configurer')}</b></summary>
      <article class="routine-card">
        <div><p class="eyebrow">{t('Routine de veille')}</p><h2>{t('Surveiller dans le temps')}</h2><p class="muted">{t('La routine traite vos identités progressivement. Les résultats restent à vérifier : aucun incident n’est créé automatiquement.')}</p></div>
        <div class="routine-controls">
          <label for="frequency">Fréquence</label><select id="frequency" value={routine?.frequency ?? 'Manuelle'} onchange={(event) => saveRoutine((event.currentTarget as HTMLSelectElement).value)} disabled={busy}><option>Manuelle</option><option>Quotidienne</option><option>Hebdomadaire</option><option>Mensuelle</option></select>
          <p><strong>État :</strong> {routine?.status ?? 'Manuelle'}<br /><strong>Dernier scan :</strong> {formatDate(routine?.last_run ?? null)}<br /><strong>Prochain scan :</strong> {routine?.next_run || 'Non planifié'}</p>
          <label class="authorization"><input type="checkbox" bind:checked={routineAuthorized} /> Je suis autorisé à analyser toutes les identités de cette routine.</label>
          <div class="button-row"><button class="wf-btn" onclick={() => saveRoutine(routine?.frequency, !(routine?.paused ?? false))} disabled={busy}>{routine?.paused ? 'Reprendre' : 'Mettre en pause'}</button><button class="wf-btn primary" onclick={launchRoutine} disabled={busy || !routineAuthorized}>{busy ? 'Routine en cours…' : 'Lancer maintenant'}</button></div>
        </div>
      </article>
    </details>

    {#if result}
      <article class="glass-card results"><p class="eyebrow">{t('Examiner · résultats du scan')}</p><h2>{t(aiGuidedView ? 'Synthèse de votre exposition' : 'Résultats pour')} {result.target}</h2><p class="muted">{aiGuidedView ? t('MANTIS a trié les signaux, regroupé les sources et prépare les prochaines actions. Les détails techniques restent disponibles en mode avancé.') : displayText(result.message)}</p>
        {#if aiGuidedView && analysis}
          <div class="ai-view-banner"><span class="ai-status-dot" aria-hidden="true"></span><div><strong>{t('Présentation guidée par l’IA locale')}</strong><p>{t('Les preuves brutes, journaux et détails de collecte sont conservés mais masqués. La synthèse est toujours à vérifier avant toute action.')}</p></div><button class="wf-btn" onclick={() => advanced = true}>{t('Voir les preuves')}</button></div>
        {/if}
        {#if aiAnalysisPending}
          <div class="ai-gate premium-ai-gate" role="status" aria-live="polite"><span class="ai-orb" aria-hidden="true"><i></i><i></i><i></i></span><div><strong>{t('L’IA locale prépare votre lecture')}</strong><p>{t('Les résultats sont disponibles pendant le regroupement des fuites, la vérification des contradictions et la préparation des prochaines actions.')}</p>{#if analysisJob}<small>{t(`${analysisJob.signal_count} signal(s) · environ ${Math.max(1, Math.ceil((analysisJob.estimated_seconds - analysisJob.elapsed_seconds) / 10) * 10)} seconde(s) restantes`)}</small>{/if}</div></div>
        {/if}
        {#if postScanQueue.length > 0}
          <section class="post-scan-workspace" aria-labelledby="post-scan-title">
            <div class="post-scan-heading"><div><p class="eyebrow">{t('À décider maintenant')}</p><h3 id="post-scan-title">{t('Résultats immédiatement exploitables')}</h3><p>{t('Chaque carte présente le signal, l’identifiant concerné, les données signalées et la mesure qui sera créée si vous le retenez.')}</p></div><div class="post-scan-count"><strong>{postScanQueue.length}</strong><span>{t(`priorité${postScanQueue.length > 1 ? 's' : ''} à examiner`)}</span></div></div>
            <div class="post-scan-grid">
              {#each postScanVisible as signal, index (signal.id)}
                {@const actionId = projectedActionIds[signal.id]}
                {#if (signal.source !== 'XposedOrNot' && signal.source !== 'User Scanner') || postScanVisible.findIndex((item) => item.source === signal.source) === index}
                <article class="post-scan-item severity-{signal.severity} {signal.source === 'XposedOrNot' ? 'post-scan-grouped-breaches' : signal.source === 'User Scanner' ? 'post-scan-grouped-services' : ''}">
                  {#if signal.source === 'XposedOrNot'}
                    <div class="post-scan-item-head"><span class="signal-kind">{t('Fuites détectées')}</span><span class="severity-pill">{t(`${xposedOrNotSignals.length} site${xposedOrNotSignals.length > 1 ? 's' : ''}`)}</span></div>
                    <h4>{t('Fuites XposedOrNot')}</h4>
                    <div class="breach-sites">
                      {#each xposedOrNotSignals as breach (breach.id)}
                        {@const breachActionId = projectedActionIds[breach.id]}
                        <details class="breach-site">
                          <summary><strong>{displayText(breach.title)}</strong><span>{severityLabel(breach.severity)} · {sourceDomain(breach.source_url)}</span></summary>
                          <div class="breach-site-detail"><p>{displayText(breach.explanation)}</p>
                            {#if breach.source_url}<a class="source-link verification-link" href={breach.source_url} onclick={(event) => openExternal(event, breach.source_url!)}>{t('Vérifier la source')} · {sourceDomain(breach.source_url)}</a>{/if}
                            {#if breachActionId}<div class="post-scan-done"><span>{t('Plan créé')}</span><a href={`/actions?id=${breachActionId}`}>{t('Ouvrir l’action →')}</a></div>{:else}<div class="post-scan-actions"><button class="wf-btn primary" onclick={() => retainAndCreatePlan(breach)} disabled={busy}>{t('Me concerne — créer le plan')}</button><button class="wf-btn" onclick={() => reviewSignal(breach.id, 'pas_moi')} disabled={busy}>{t('Pas moi')}</button><button class="wf-btn" onclick={() => reviewSignal(breach.id, 'suivre')} disabled={busy}>{t('Suivre')}</button></div>{/if}
                          </div>
                        </details>
                      {/each}
                    </div>
                  {:else if signal.source === 'User Scanner'}
                    <div class="post-scan-item-head"><span class="signal-kind">{t('Services associés')}</span><span class="severity-pill">{userScannerSignals.length} {t(userScannerSignals.length > 1 ? 'résultats' : 'résultat')}</span></div>
                    <h4>{t('Services associés potentiellement')}</h4>
                    <p class="muted small">{t('Les services détectés sont regroupés ici. Ouvrez chaque ligne pour vérifier le compte avant toute décision.')}</p>
                    <div class="breach-sites">
                      {#each userScannerSignals as service (service.id)}
                        {@const serviceActionId = projectedActionIds[service.id]}
                        <details class="breach-site">
                          <summary><strong>{displayText(service.title.replace('Service associé potentiellement à l’e-mail : ', '').replace('Compte potentiel pour le pseudo : ', ''))}</strong><span>{severityLabel(service.severity)} · {sourceDomain(service.source_url)}</span></summary>
                          <div class="breach-site-detail"><p>{displayText(service.explanation)}</p>
                            {#if service.source_url}<a class="source-link verification-link" href={service.source_url} onclick={(event) => openExternal(event, service.source_url!)}>{t('Vérifier la source')} · {sourceDomain(service.source_url)}</a>{/if}
                            {#if serviceActionId}<div class="post-scan-done"><span>Plan créé</span><a href={`/actions?id=${serviceActionId}`}>Ouvrir l’action →</a></div>{:else}<div class="post-scan-actions"><button class="wf-btn primary" onclick={() => retainAndCreatePlan(service)} disabled={busy}>Me concerne — créer le plan</button><button class="wf-btn" onclick={() => reviewSignal(service.id, 'pas_moi')} disabled={busy}>Pas moi</button><button class="wf-btn" onclick={() => reviewSignal(service.id, 'suivre')} disabled={busy}>Suivre</button></div>{/if}
                          </div>
                        </details>
                      {/each}
                    </div>
                  {:else}
                  <div class="post-scan-item-head"><span class="signal-kind">{signal.signal_type === 'fuite' ? 'Fuite détectée' : signal.signal_type === 'profil_public' ? 'Profil public' : 'Signal public'}</span><span class="severity-pill">{severityLabel(signal.severity)}</span></div>
                  <h4>{displayText(signal.title)}</h4>
                  <dl>
                    <div><dt>Identifiant concerné</dt><dd>{signal.target}</dd></div>
                    <div><dt>Données signalées</dt><dd>{exposedDataLabels(signal).join(' · ')}</dd></div>
                    <div><dt>Source</dt><dd>{signal.source}</dd></div>
                  </dl>
                  <details class="post-scan-evidence">
                    <summary>Voir le détail de l’indexation</summary>
                    <p>{displayText(signal.explanation)}</p>
                  </details>
                  {#if signal.source_url}
                    <a class="source-link verification-link" href={signal.source_url} onclick={(event) => openExternal(event, signal.source_url!)}>Vérifier la source · {sourceDomain(signal.source_url)}</a>
                  {:else}
                    <p class="muted small">Aucun lien public précis n’est disponible pour cette source ; MANTIS conserve seulement sa référence locale.</p>
                  {/if}
                  {#if actionId}
                    <div class="post-scan-done"><span>Plan créé</span><a href={`/actions?id=${actionId}`}>Ouvrir l’action →</a></div>
                  {:else}
                    <div class="post-scan-actions">
                      <button class="wf-btn primary" onclick={() => retainAndCreatePlan(signal)} disabled={busy}>Me concerne — créer le plan</button>
                      <button class="wf-btn" onclick={() => reviewSignal(signal.id, 'pas_moi')} disabled={busy}>Pas moi</button>
                      <button class="wf-btn" onclick={() => reviewSignal(signal.id, 'suivre')} disabled={busy}>Suivre</button>
                    </div>
                  {/if}
                  {/if}
                </article>
                {/if}
              {/each}
            </div>
            {#if postScanDisplayQueue.length > 9 && !postScanExpanded}
              <button class="wf-btn post-scan-more" onclick={() => postScanExpanded = true}>Voir {postScanDisplayQueue.length - 9} résultat{postScanDisplayQueue.length - 9 > 1 ? 's' : ''} de plus</button>
            {/if}
            <p class="post-scan-disclaimer">{t('« Me concerne — créer le plan » enregistre votre décision puis crée une Exposition, un Incident et une Action. Rien n’est créé à la fin d’un scan sans ce choix explicite.')}</p>
          </section>
        {/if}
        {#if sessionResult}
          <div class="session-summary" aria-label={t('Bilan des vérifications de la session')}>
            <div><strong>{sessionResult.completed_checks}/{sessionResult.planned_checks}</strong><span>{t('vérifications terminées')}</span></div>
            <div class:has-error={sessionResult.failed_checks > 0}><strong>{sessionResult.failed_checks}</strong><span>{t('échecs isolés')}</span></div>
            <div><strong>{sessionResult.skipped_checks}</strong><span>{t('vérifications ignorées')}</span></div>
          </div>
          {#if sessionResult.coverage.length}
            <section class="coverage-panel" aria-label="Couverture de la collecte">
              <div><p class="eyebrow">{t('Couverture du scan')}</p><h3>{t('Ce qui a réellement été vérifié')}</h3><p class="muted">{t('Les données non compatibles avec un outil ne sont pas envoyées. Une absence de résultat ne prouve pas une absence sur le Web.')}</p></div>
              <div class="coverage-list">
                {#each sessionResult.coverage as coverage}
                  <div class:has-error={coverage.failed_checks > 0} class="coverage-item">
                    <strong>{targetKindLabel(coverage.target_kind)}</strong>
                    <span>{coverage.completed_checks}/{coverage.planned_checks} source{coverage.planned_checks > 1 ? 's' : ''} terminée{coverage.planned_checks > 1 ? 's' : ''} · {coverage.signal_count} signal{coverage.signal_count > 1 ? 's' : ''} collecté{coverage.signal_count > 1 ? 's' : ''}</span>
                    {#if coverage.failed_checks > 0}<small>{coverage.failed_checks} source{coverage.failed_checks > 1 ? 's' : ''} indisponible{coverage.failed_checks > 1 ? 's' : ''}, sans bloquer le reste du scan.</small>{/if}
                  </div>
                {/each}
              </div>
            </section>
          {/if}
        {/if}
        <div class="result-summary" aria-label="Résumé déterministe du scan">
          <div><strong>{resultSummary.total}</strong><span>{t('signaux analysés')}</span></div>
          <div><strong>{resultSummary.ignored}</strong><span>{t('écartés par votre décision')}</span></div>
          <div><strong>{resultSummary.followed}</strong><span>{t('retenus ou suivis')}</span></div>
          <div><strong>{resultSummary.uncertain}</strong><span>{t('encore incertains')}</span></div>
        </div>
        <section class="prevention-path" aria-labelledby="prevention-title">
          <div class="prevention-heading"><div><p class="eyebrow">{t('Prévenir · après la vérification')}</p><h3 id="prevention-title">{t('Réduire l’impact d’une prochaine fuite')}</h3></div><p>{t('Ces mesures ne changent pas le résultat du scan. Elles limitent la réutilisation, la corrélation et la portée d’une exposition future.')}</p></div>
          <div class="prevention-guides">
            {#each preventiveGuides as guide}
              <a href={`/guides?id=${guide.id}`} style={`--prevent-accent:${guide.accent}`}>
                <img src={guide.image} alt="" /><span><small>{guide.minutes} min</small><strong>{guide.shortTitle}</strong><p>{guide.outcome}</p></span><b>→</b>
              </a>
            {/each}
          </div>
        </section>
        {#if resolutions.length > 0}
          <section class="resolution-panel" aria-label={t('Lecture des preuves')}>
            <div><p class="eyebrow">{t('Lecture des preuves')}</p><h3>{t('Ce que les sources permettent réellement de conclure')}</h3><p>{t('Un recoupement demande des sources indépendantes. Une répétition du même fournisseur ne renforce pas automatiquement un résultat.')}</p></div>
            <div class="resolution-summary">
              {#if resolutionSummary.corroborated}<span class="resolution-chip corroborated">{resolutionSummary.corroborated} source{resolutionSummary.corroborated > 1 ? 's' : ''} croisée{resolutionSummary.corroborated > 1 ? 's' : ''}</span>{/if}
              {#if resolutionSummary.toVerify}<span class="resolution-chip">{resolutionSummary.toVerify} {t('à vérifier')}</span>{/if}
              {#if resolutionSummary.contradictory}<span class="resolution-chip contradictory">{resolutionSummary.contradictory} à clarifier</span>{/if}
              {#if advanced && resolutionSummary.rejected}<span class="resolution-chip muted">{resolutionSummary.rejected} écarté{resolutionSummary.rejected > 1 ? 's' : ''}</span>{/if}
            </div>
            {#if advanced}
              <div class="resolution-list">
                {#each resolutions as resolution (resolution.id)}
                  <div class:contradictory={resolution.status === 'contradictoire'} class:muted={resolution.status === 'rejete'}>
                    <strong>{resolutionStatusLabel(resolution.status)}</strong><span>{resolution.source_count} famille{resolution.source_count > 1 ? 's' : ''} de source{resolution.source_count > 1 ? 's' : ''} · {resolution.favorable_count} preuve{resolution.favorable_count > 1 ? 's' : ''}</span><p>{resolution.rationale}</p>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        {/if}
        {#if advanced && presentedClaims.length > 0}
          <section class="claims-panel" aria-label={t('Signaux à examiner')}>
            <div class="claims-intro"><p>{aiGuidedView && analysis ? `L’IA locale a retenu ${presentedClaims.length} élément(s) sur ${claims.length} et écarté ${aiFilteredClaims} signal(aux) faible(s) ou contradictoire(s).` : `MANTIS présente ${presentedClaims.length} élément(s) classé(s) par ses règles déterministes.`} Les rapprochements ne confirment jamais une identité.</p></div>
            <div class="claims-grid">
              {#each presentedClaims as claim (claim.id)}
                {@const synthesis = synthesisFor(claim)}
                <article class="claim-card" class:claim-contradiction={claim.status === 'contradictoire'} class:claim-muted={claim.status === 'rejetee'}>
                  <div class="claim-heading"><span class="claim-type">{claimTypeLabel(claim.claim_type)}</span><span class={`claim-priority ${claim.priority}`}>Priorité {claim.priority}</span></div>
                  <h4>{displayText(claim.display_value)}</h4>
                  <div class="claim-status"><strong>{claimStatusLabel(claim.status)}</strong><span>{claim.source_count} source{claim.source_count > 1 ? 's' : ''} distincte{claim.source_count > 1 ? 's' : ''} · {claim.favorable_count} preuve{claim.favorable_count > 1 ? 's' : ''} favorable{claim.favorable_count > 1 ? 's' : ''}{#if claim.contradictory_count} · {claim.contradictory_count} contraire{claim.contradictory_count > 1 ? 's' : ''}{/if}</span></div>
                  <p>{claim.rationale}</p>
                  {#if synthesis}
                    <div class="claim-synthesis" class:contrary={synthesis.contradiction}>
                      <div><strong>{synthesis.confidence === 'forte' ? 'Recoupement fort' : synthesis.confidence === 'moyenne' ? 'Recoupement modéré' : 'Hypothèse faible'}</strong><span>Action suggérée : {synthesis.recommended_action}</span></div>
                      <p>{synthesis.statement}</p>
                      {#if synthesis.exposure_kind === 'fuite_eventuelle'}<div class="leak-annotation"><strong>Fuite annotée</strong>{#if synthesis.exposed_data?.length}<span>Donnée signalée : {synthesis.exposed_data.join(', ')}</span>{/if}{#if synthesis.where_found?.length}<span>Observée dans : {synthesis.where_found.join(' · ')}</span>{/if}</div>{/if}
                      <small>Preuves citées : {synthesis.evidence_ids.join(', ')}</small>
                    </div>
                  {/if}
                  <div class="claim-evidence">
                    {#each claim.evidence.slice(0, advanced ? 8 : 3) as evidence (evidence.observation_id + evidence.role)}
                      <div class:contrary={evidence.role === 'contradictoire'}>
                        <span>{evidence.role === 'contradictoire' ? 'Preuve contraire' : 'Preuve conservée'} · {formatDate(evidence.observed_at)}{#if advanced} · {evidence.source}{/if}</span>
                        {#if evidence.source_url}<a href={evidence.source_url} onclick={(event) => openExternal(event, evidence.source_url!)}>{advanced ? evidence.source_url : sourceDomain(evidence.source_url)}</a>{:else}<em>Référence brute locale</em>{/if}
                      </div>
                    {/each}
                  </div>
                  <div class="claim-review-actions" aria-label="Décision sur cette revendication">
                    <button class="wf-btn" onclick={() => reviewClaim(claim.id,'confirmer')} disabled={busy}>Me concerne</button>
                    <button class="wf-btn" onclick={() => reviewClaim(claim.id,'pas_moi')} disabled={busy}>Pas moi</button>
                    <button class="wf-btn" onclick={() => reviewClaim(claim.id,'suivre')} disabled={busy}>Suivre</button>
                    <button class="wf-btn" onclick={() => reviewClaim(claim.id,'ignorer')} disabled={busy}>Ignorer</button>
                  </div>
                  {#if advanced}<p class="tech-meta">Revendication {claim.id} · première observation {claim.first_observed_at} · dernière {claim.last_observed_at}</p>{/if}
                </article>
              {/each}
            </div>
          </section>
        {/if}
        {#if advanced && (reviewWorkspace.evolution.length || reviewWorkspace.events.length)}
          <section class="review-workspace" aria-labelledby="review-title">
            <div class="claims-intro"><div><p class="eyebrow">Évolution et décisions</p><h3 id="review-title">Ce qui a changé depuis le scan précédent</h3></div><p>Les disparitions indiquent seulement qu’une source n’a pas été retrouvée pendant le dernier scan. Elles ne prouvent pas que la donnée a disparu du Web.</p></div>
            {#if reviewWorkspace.evolution.length}
              <div class="evolution-grid">{#each reviewWorkspace.evolution as item (item.claim_id)}<div class:unavailable={item.change === 'source_indisponible'}><strong>{item.change === 'nouveau' ? 'Nouveau' : item.change === 'source_indisponible' ? 'Non retrouvé' : 'Toujours présent'}</strong><span>{displayText(item.display_value)}</span><small>{item.previous_sources} → {item.current_sources} source(s)</small></div>{/each}</div>
            {/if}
            {#if reviewWorkspace.events.length}<details><summary>Historique complet des décisions et projections ({reviewWorkspace.events.length})</summary><div class="review-history">{#each reviewWorkspace.events as event (event.id)}<div><span>{formatDate(event.created_at)}</span><strong>{event.event_type === 'projection' ? 'Projection manuelle' : 'Décision humaine'} · {event.decision}</strong><p>{displayText(event.target_label)}{#if event.reason} — {displayText(event.reason)}{/if}</p></div>{/each}</div></details>{/if}
          </section>
        {/if}
        <section class="analysis-panel" aria-labelledby="analysis-title">
          <div><p class="eyebrow">{t('Valorisation des résultats')}</p><h3 id="analysis-title">{analysis?.mode === 'ia_locale' ? t('Conclusion locale avec sources croisées') : t('Lecture déterministe des résultats')}</h3><p class="muted">{analysis?.mode === 'ia_locale' ? t('Fonction expérimentale : l’IA locale est lente et peut échouer. MANTIS transforme les revendications, preuves et contradictions en conclusion, priorités et prochaines actions ; l’IA ne recherche rien, ne lance aucun outil et ne confirme jamais une identité.') : localAiReady ? t('Le modèle local n’a pas produit de sortie validée. MANTIS affiche son classement déterministe sans attribuer ce résultat à l’IA.') : t('Fonction IA expérimentale désactivée par défaut et non fiable aujourd’hui. MANTIS conserve les preuves et classe les résultats avec ses règles déterministes.')}</p></div>
          {#if localAiReady}<button class="wf-btn primary" onclick={analyzeResult} disabled={analyzing || claims.length === 0}>{analyzing ? t('Synthèse locale en cours…') : analysis ? t('Actualiser la synthèse') : t('Lancer la synthèse guidée')}</button>{:else}<a class="wf-btn primary" href="/parametres">{t('Configurer l’IA locale')}</a>{/if}
          {#if analysisJob && ['en_attente','en_cours'].includes(analysisJob.status) && !aiAnalysisPending}
            <div class="analysis-progress" role="status" aria-live="polite"><span class="analysis-spinner" aria-hidden="true"></span><div><strong>{analysisJob.status === 'en_attente' ? 'Analyse locale en attente' : 'L’IA locale traite les résultats'}</strong><p>{analysisJob.signal_count} signal(s) · durée estimée : environ {Math.max(1, Math.ceil((analysisJob.estimated_seconds - analysisJob.elapsed_seconds) / 10) * 10)} seconde(s). Vous pouvez continuer à utiliser MANTIS.</p></div></div>
          {/if}
          {#if analysis}
            <div class="analysis-result" class:fallback={analysis.mode === 'deterministe'}>
              <div class="analysis-heading"><strong>{analysis.mode === 'ia_locale' ? 'Synthèse IA locale validée' : 'Synthèse déterministe de secours'}</strong><span>{analysis.model_label ?? 'Règles MANTIS'} · revue humaine requise</span></div>
              <p>{analysis.conclusion ?? analysis.overview}</p>
              {#if analysis.citation_ids?.length}<small>Citations internes : {analysis.citation_ids.join(', ')}</small>{/if}
              {#each analysis.limitations as limitation}<small>{limitation}</small>{/each}
            </div>
          {/if}
        </section>
        {#if resultGroups.length === 0}<div class="empty"><strong>Vérification terminée</strong><p>Aucun élément suffisamment étayé n’a été retenu. Cela ne prouve pas l’absence d’information publique.</p></div>{/if}
        {#if hiddenDdgsSignals.length > 0 && !advanced}
          <details class="ddgs-leads">
            <summary><span><strong>{t('Résultats Web potentiels')}</strong><small>{hiddenDdgsSignals.length} {t(hiddenDdgsSignals.length > 1 ? 'résultats DDGS' : 'résultat DDGS')}</small></span><b aria-hidden="true">⌄</b></summary>
            <p>Ces résultats ont été trouvés par recherche Web, mais ne comportent pas encore assez de preuves pour être présentés comme vous concernant.</p>
            <div>
              {#each hiddenDdgsSignals as signal (signal.id)}
                <article><span>{sourceDomain(signal.source_url ?? '') || 'Résultat Web'}</span><strong>{displayText(signal.title)}</strong>{#if signal.source_url}<a href={signal.source_url} onclick={(event) => openExternal(event, signal.source_url!)}>{sourceDomain(signal.source_url)}</a>{/if}</article>
              {/each}
            </div>
          </details>
        {/if}
        {#if hiddenCandidateCount > 0 && !advanced}<div class="filtered-candidates"><strong>{hiddenCandidateCount} {t(hiddenCandidateCount > 1 ? 'candidats faibles masqués' : 'candidat faible masqué')}</strong><p>{t('Ces résultats n’ont pas une preuve ou un recoupement suffisant pour être présentés comme vous concernant.')}</p><button class="wf-btn" onclick={() => advanced = true}>{t('Voir les candidats et leurs raisons')}</button></div>{/if}
        {#if !aiGuidedView}
        {#each resultGroups as group (group.title)}
          <section class="result-group"><h3>{group.title}</h3><p class="muted">{group.description}</p>
            {#each group.signals as signal (signal.id)}
              {@const signalAnalysis = analysisFor(signal)}
              <div class="signal-card" class:signal-muted={['Ignoré', 'Ce n’est pas moi'].includes(signal.review_status)}><div class="signal-heading"><h4>{displayText(signal.title)}</h4><span class="badge" class:decided={signal.review_status !== 'À vérifier'}>{signal.review_status}</span></div>
                {#if signalAnalysis}<div class="analysis-item"><div><span class:important={signalAnalysis.relevance === 'important'}>{relevanceLabel(signalAnalysis.relevance)}</span>{#if signalAnalysis.uncertainty}<em>incertain</em>{/if}</div><p>{signalAnalysis.reason}</p><small>Preuve citée : {signalAnalysis.evidence_ids.join(', ')} · recommandation : {signalAnalysis.recommended_action}</small></div>{/if}
                <div class="evidence"><span>Preuve disponible{#if advanced} · {signal.source}{:else if sourceDomain(signal.source_url)} · {sourceDomain(signal.source_url)}{/if}</span>{#if signal.source_url}<a href={signal.source_url} onclick={(event) => openExternal(event, signal.source_url!)}>{signal.source_url}</a>{:else}<p>Référence conservée localement avec le résultat brut du scan.</p>{/if}</div>
                {#if advanced}<p class="tech-meta">Outil : {signal.source} · module {signal.module_id} · valeur analysée : {signal.target} · scan {signal.scan_id}</p>{/if}
                <p>{displayText(signal.explanation)}</p>
                {#if signal.source_url}<a class="source-link" href={signal.source_url} onclick={(event) => openExternal(event, signal.source_url!)}>Ouvrir la page</a>{/if}
                <div class="review-actions" aria-label="Décision sur ce signal">
                  <button class="wf-btn" onclick={() => reviewSignal(signal.id, 'confirmer')} disabled={busy}>Cela me concerne</button>
                  <button class="wf-btn" onclick={() => reviewSignal(signal.id, 'pas_moi')} disabled={busy}>Ce n’est pas moi</button>
                  <button class="wf-btn" onclick={() => reviewSignal(signal.id, 'ignorer')} disabled={busy}>Ignorer</button>
                  <button class="wf-btn" onclick={() => reviewSignal(signal.id, 'suivre')} disabled={busy}>Suivre</button>
                </div>
                {#if !signal.exposure_id && ['Confirmé', 'Suivi'].includes(signal.review_status)}<p class="muted small">Vous avez retenu ce signal. Vous pouvez maintenant l’intégrer au suivi MANTIS.</p><button class="wf-btn primary" onclick={() => createExposure(signal.id)} disabled={busy}>Créer une exposition</button>
                {:else if !signal.exposure_id}<p class="muted small">Décidez d’abord si ce signal vous concerne ou doit être suivi. Une hypothèse ignorée ne crée rien automatiquement.</p>
                {:else if incidentFor !== signal.id}<p class="muted small">Exposition créée. Créez un incident uniquement si la situation nécessite un suivi et une action.</p><button class="wf-btn" onclick={() => createIncident(signal)} disabled={busy}>Créer un incident si nécessaire</button>
                {:else}<p class="ready">Incident et action créés manuellement.</p><div class="button-row"><a class="wf-btn" href="/incidents">Voir l’incident</a><a class="wf-btn" href="/actions">Voir l’action</a>{#if signal.source_url && !dpoFor}<button class="wf-btn primary" onclick={() => prepareDpo(signal)} disabled={busy}>Préparer une demande DPO</button>{:else if dpoFor}<a class="wf-btn primary" href={`/dpo?id=${dpoFor}`}>Ouvrir le brouillon DPO</a>{/if}</div>{/if}
              </div>
            {/each}
          </section>
        {/each}
        {:else if analysis}
          <div class="guided-details-note"><strong>Les détails de collecte sont masqués.</strong><p>Retrouvez les signaux bruts, URLs, modules et journaux en activant le mode avancé.</p><button class="wf-btn" onclick={() => advanced = true}>Ouvrir le mode avancé</button></div>
        {/if}
      </article>
    {:else}
      <article class="glass-card results empty-results"><p class="eyebrow">Examiner</p><h2>Aucun scan pour cette identité</h2><p class="muted">Lancez les sources compatibles depuis la barre ci-dessus. Les résultats apparaîtront ici et resteront filtrés sur cette identité.</p></article>
    {/if}

    <section class="advanced-toggle"><div><strong>Outils et preuves techniques</strong><p class="muted">Diagnostics, journaux et détails de collecte, à ouvrir seulement en cas de besoin.</p></div><label><input type="checkbox" bind:checked={advanced} /> Afficher</label></section>
    {#if noAiAlertVisible}
      <div class="no-ai-note"><span aria-hidden="true">○</span><p><strong>{t('IA locale expérimentale indisponible.')}</strong> {t('Les résultats restent classés de façon déterministe.')}</p><a href="/parametres">{t('Réglages')}</a><button onclick={dismissNoAiAlert} aria-label={t('Masquer la note IA locale')}>×</button></div>
    {/if}
    {#if advanced}
      {#if hiddenCandidateCount + hiddenDdgsSignals.length > 0}<article class="glass-card candidate-audit"><p class="eyebrow">Arrière-boutique de collecte</p><h2>Candidats masqués automatiquement</h2><p class="muted">Ils sont conservés pour audit, mais exclus des conclusions IA et du parcours utilisateur tant qu’une preuve plus forte n’existe pas.</p>{#each rawSignals.filter((signal) => signal.review_status === 'Filtré automatiquement') as signal (signal.id)}<div><strong>{signal.title}</strong><span>{signal.confidence}</span><span>{#if signal.source_url}<a href={signal.source_url} onclick={(event) => openExternal(event, signal.source_url!)}>{sourceDomain(signal.source_url)}</a>{/if}<button class="wf-btn" onclick={() => reviewSignal(signal.id, 'suivre')} disabled={busy}>Je reconnais ce résultat</button></span></div>{/each}</article>{/if}
      <article class="glass-card advanced"><h2>Outils et diagnostic</h2><div class="advanced-grid"><div class="tool-list">{#each modules as module (module.id)}<button class:active={module.id === advancedModuleId} onclick={() => selectAdvancedModule(module.id)}><strong>{module.name}</strong><span>{module.installation_status}</span></button>{/each}</div>
        {#if selectedModule}<div><h3>{selectedModule.name}</h3><p class="muted">{selectedModule.description}</p><p><strong>État :</strong> {selectedModule.installation_status}</p><div class="button-row">{#if selectedModuleManaged}<button class="wf-btn" onclick={installAdvanced} disabled={busy}>Installer / réparer</button>{:else}<span class="builtin-badge">Intégré à MANTIS</span>{/if}<button class="wf-btn" onclick={diagnoseAdvanced} disabled={busy}>Diagnostiquer</button>{#if selectedModuleManaged}<button class="wf-btn" onclick={rollbackAdvanced} disabled={busy}>Restaurer la version précédente</button><button class="wf-btn danger" onclick={removeAdvancedRuntime} disabled={busy}>Désinstaller le moteur</button>{/if}</div>{#if diagnostic}<p class="notice">{diagnostic}</p>{/if}
          <label for="advanced-target">Cible compatible</label><select id="advanced-target" bind:value={selectedIdentityId}>{#each advancedTargets as identity (identity.id)}<option value={identity.id}>{identity.label} · {identitySummary(identity)}</option>{/each}</select><button class="wf-btn primary" onclick={runAdvanced} disabled={busy || !selectedIdentityId || !authorized || selectedModule.installation_status !== 'prêt'}>Lancer cet outil</button>
          <h4>Journal technique</h4>{#if logs.length === 0}<p class="muted small">Aucune entrée.</p>{/if}{#each logs as log}<p class="log">{log.created_at} · {log.operation} · {log.status}<br />{log.message}</p>{/each}
        </div>{/if}</div></article>
      <article class="glass-card catalog-maintenance">
        <div><p class="eyebrow">Maintenance contrôlée</p><h2>Inventaire des moteurs</h2><p class="muted">{modules.length} moteur{modules.length > 1 ? 's' : ''} actif{modules.length > 1 ? 's' : ''} · {archivedModules.length} référence{archivedModules.length > 1 ? 's' : ''} historique{archivedModules.length > 1 ? 's' : ''}. Le nettoyage ne supprime jamais les scans ni leurs preuves.</p></div>
        <button class="wf-btn" onclick={cleanupOrphans} disabled={busy}>Nettoyer les anciens moteurs</button>
        {#if maintenanceNotice}<p class="notice maintenance-notice">{maintenanceNotice}</p>{/if}
        <div class="archive-grid">{#each archivedModules as module (module.id)}<div><span>{module.catalog_status === 'test_only' ? 'Test uniquement' : 'Archivé'}</span><strong>{module.name}</strong><p>{module.archived_reason ?? 'Conservé dans le catalogue pour la traçabilité historique.'}</p>{#if module.replacement_id}<small>Remplacement : {moduleInventory.find((candidate) => candidate.id === module.replacement_id)?.name ?? module.replacement_id}</small>{/if}</div>{/each}</div>
      </article>
    {/if}
    </div>
    <NextStepBar hint="Les scans donnent des signaux : vous décidez toujours ce qui devient une exposition, un incident ou une action." primaryHref="/expositions" primaryLabel="Voir les expositions"><a class="wf-btn" href="/identites">Gérer mes identités</a></NextStepBar>
  {/if}
</section>

<style>
  .muted { color: var(--mantis-text-muted); font-size: .87rem; line-height: 1.5; } .small { font-size: .78rem; } .error { color: var(--mantis-danger); } .notice { color: var(--mantis-accent); } .ready { color: var(--mantis-ok); font-size: .86rem; } .no-ai-note { display:flex; align-items:center; gap:.45rem; min-height:28px; margin:.45rem .25rem 0; color:var(--mantis-text-muted); font-size:.68rem; } .no-ai-note>span { color:var(--mantis-warn); font-size:.9rem; } .no-ai-note p { margin:0; } .no-ai-note strong { color:var(--mantis-text-secondary); font-weight:650; } .no-ai-note a { color:var(--mantis-accent); text-decoration:none; font-weight:650; } .no-ai-note button { width:20px; height:20px; margin-left:auto; border:0; border-radius:50%; color:var(--mantis-text-muted); background:transparent; cursor:pointer; font-size:1rem; line-height:1; } .no-ai-note button:hover { color:var(--mantis-text); background:rgba(255,255,255,.06); } .ai-view-banner { display:flex; align-items:center; gap:.7rem; margin:1rem 0; padding:.75rem .85rem; border:1px solid color-mix(in srgb,var(--mantis-accent) 42%,var(--mantis-border)); border-radius:9px; background:color-mix(in srgb,var(--mantis-accent) 7%,transparent); } .ai-view-banner>div { flex:1; } .ai-view-banner p { margin:.2rem 0 0; color:var(--mantis-text-muted); font-size:.76rem; line-height:1.4; } .ai-status-dot { width:.55rem; height:.55rem; flex:0 0 auto; border-radius:50%; background:var(--mantis-accent); box-shadow:0 0 12px var(--mantis-accent); } .guided-details-note { margin:1rem 0; padding:.8rem; border:1px dashed var(--mantis-border); border-radius:8px; color:var(--mantis-text-muted); } .guided-details-note strong { color:var(--mantis-text); } .guided-details-note p { margin:.25rem 0; font-size:.78rem; }
  .scan-page-header { display:flex; align-items:flex-start; justify-content:space-between; gap:1.25rem; } .scan-page-header :global(.guide-header) { flex:1; } .veille-workspace { display:flex; flex-direction:column; } .identity-command { order:1; } .scan-progress { order:1.7; } .results { order:2; } .main-grid { order:4; } .routine-panel { order:5; } .advanced-toggle { order:6; } .advanced { order:7; } .has-results { gap:.8rem; padding-top:1rem; } .has-results .scan-page-header :global(.guide-header) { gap:0; } .has-results .scan-page-header :global(.guide-kicker),.has-results .scan-page-header :global(.guide-question),.has-results .scan-page-header :global(.guide-intro) { display:none; } .has-results .scan-page-header :global(.guide-heading h1) { font-size:1.25rem; } .has-results .scan-page-header :global(.guide-heading span) { width:18px; } .has-results .identity-command { min-height:0; padding:.7rem .9rem; margin-top:0; } .has-results .identity-selector { grid-template-columns:1.8rem minmax(180px,1fr); gap:.65rem; } .has-results .identity-selector::before { width:1.8rem; height:1.8rem; font-size:.8rem; box-shadow:none; } .has-results .identity-selector .eyebrow,.has-results .identity-selector .muted { display:none; } .has-results .identity-selector h2 { margin:0; font-size:.92rem; } .has-results .scan-primary { min-height:2.35rem; } .has-results .history-panel { margin-top:0; }
  .identity-command { display:grid; grid-template-columns:minmax(280px,1.2fr) minmax(280px,1fr); gap:.8rem 1.2rem; align-items:end; margin-top:.65rem; border-color:color-mix(in srgb,var(--mantis-warn) 40%,var(--mantis-border)); background:linear-gradient(110deg,color-mix(in srgb,var(--mantis-warn) 7%,var(--ui-surface-2)),var(--ui-surface-1)); } .identity-selector { display:grid; grid-template-columns:auto minmax(220px,1fr); align-items:center; gap:1rem; } .identity-selector::before { content:'◈'; display:grid; place-items:center; width:3.1rem; height:3.1rem; border:1px solid color-mix(in srgb,var(--mantis-warn) 55%,var(--mantis-border)); border-radius:50%; color:var(--mantis-warn); font-size:1.2rem; background:color-mix(in srgb,var(--mantis-warn) 8%,transparent); box-shadow:0 0 1.5rem color-mix(in srgb,var(--mantis-warn) 14%,transparent); animation:identity-beacon 3.8s ease-in-out infinite; } .identity-selector h2 { margin:.15rem 0 0; font-size:1.14rem; letter-spacing:-.02em; } .scan-command { display:flex; justify-content:flex-end; align-items:center; gap:.8rem; } .scan-command .authorization,.scan-command .wf-btn { margin:0; } .scan-primary { min-height:2.85rem; padding-inline:1.1rem; } @keyframes identity-beacon { 50% { transform:scale(1.05); box-shadow:0 0 2.1rem color-mix(in srgb,var(--mantis-warn) 22%,transparent); } }
  .secondary-panel { margin-top:.7rem; padding:.8rem .9rem; border:1px solid var(--mantis-border); border-radius:12px; background:var(--ui-surface-1); } .secondary-panel>summary { display:flex; align-items:center; justify-content:space-between; gap:1rem; cursor:pointer; list-style:none; } .secondary-panel>summary::-webkit-details-marker { display:none; } .secondary-panel summary span,.secondary-panel summary strong,.secondary-panel summary small { display:block; } .secondary-panel summary strong { font-size:.82rem; } .secondary-panel summary small { margin-top:.18rem; color:var(--mantis-text-muted); font-size:.7rem; } .secondary-panel summary b { color:var(--mantis-accent); font-size:.7rem; font-weight:700; text-transform:uppercase; letter-spacing:.06em; } .history-panel { align-self:flex-end; width:min(100%,260px); margin-top:.5rem; padding:0; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-sm); background:var(--ui-canvas-elevated); } .history-panel>summary { display:flex; align-items:center; justify-content:space-between; min-height:34px; padding:.35rem .55rem .35rem .7rem; cursor:pointer; list-style:none; } .history-panel>summary::-webkit-details-marker { display:none; } .history-panel summary span { display:flex; align-items:center; gap:.45rem; } .history-panel summary strong { font:700 .67rem/1 var(--font-meta); color:var(--ui-text-secondary); letter-spacing:.07em; text-transform:uppercase; } .history-panel summary small { display:grid; place-items:center; min-width:1.25rem; height:1.25rem; padding:0 .25rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-pill); color:var(--ui-text-primary); background:var(--ui-surface-2); font:700 .64rem/1 var(--font-meta); } .history-panel summary b { color:var(--ui-text-tertiary); font-size:1rem; line-height:1; transition:transform var(--duration-fast); } .history-panel[open] summary b { transform:rotate(180deg); }
  .history-strip { display:flex; flex-direction:column; gap:.35rem; margin:.4rem; padding:.4rem; border-top:1px solid var(--mantis-border); } .history-strip button { width:100%; padding:.55rem .65rem; text-align:left; border:1px solid var(--mantis-border); border-radius:7px; background:var(--ui-surface-2); color:var(--mantis-text); transition:border-color var(--duration-fast),background var(--duration-fast); } .history-strip button:hover { border-color:var(--mantis-border-strong); } .history-strip button.active { border-color:var(--mantis-accent); background:color-mix(in srgb,var(--mantis-accent) 8%,var(--ui-surface-2)); } .history-strip strong,.history-strip span { display:block; } .history-strip strong { font-size:.75rem; } .history-strip span { margin-top:.2rem; color:var(--mantis-text-muted); font-size:.68rem; } .history-strip button.more { min-width:auto; text-align:center; }
  .main-grid { display:grid; grid-template-columns:1fr; gap:1rem; } .advanced-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:1rem; } .step-card h2, .routine-card h2, .results h2, .advanced h2 { margin:.15rem 0 .55rem; font-size:1.08rem; } .eyebrow { color:var(--mantis-accent); font-size:.68rem; font-weight:700; letter-spacing:.09em; text-transform:uppercase; margin:0; }
  label { display:block; margin:.7rem 0 .35rem; font-size:.84rem; } select { width:100%; padding:.55rem; border-radius:6px; border:1px solid var(--mantis-border); background:var(--mantis-bg); color:var(--mantis-text); } .authorization { display:flex; align-items:center; gap:.45rem; color:var(--mantis-text-muted); } .authorization input { width:auto; } .wf-btn { margin-top:.6rem; } .button-row { display:flex; flex-wrap:wrap; gap:.45rem; }
  .routine-card { margin-top:1rem; display:grid; grid-template-columns:1.2fr 1fr; gap:1.5rem; } .routine-controls p { font-size:.83rem; line-height:1.6; }
  .results { display:flex; flex-direction:column; margin-top:.8rem; background:radial-gradient(circle at 88% 9%,rgba(78,152,232,.11),transparent 27%),linear-gradient(145deg,rgba(255,255,255,.07),rgba(255,255,255,.018)); } .results>h2 { max-width:38rem; font-size:clamp(1.22rem,2vw,1.65rem); letter-spacing:-.035em; } .results>.muted { max-width:54rem; } .empty-results { min-height:180px; display:flex; flex-direction:column; justify-content:center; } .result-summary { order:3; display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); gap:.65rem; margin:1rem 0; } .result-summary div { position:relative; overflow:hidden; padding:.85rem; border:1px solid rgba(255,255,255,.11); border-radius:13px; background:linear-gradient(145deg,rgba(255,255,255,.085),rgba(0,0,0,.16)); box-shadow:inset 0 1px 0 rgba(255,255,255,.09); } .result-summary div::after { content:''; position:absolute; inset:auto -20% -80% 30%; height:120%; background:radial-gradient(circle,rgba(45,212,191,.17),transparent 65%); } .result-summary strong,.result-summary span { position:relative; z-index:1; display:block; } .result-summary strong { font-size:1.5rem; letter-spacing:-.06em; } .result-summary span { color:var(--mantis-text-muted); font-size:.68rem; text-transform:uppercase; letter-spacing:.06em; } .result-group { order:7; margin-top:1.25rem; border-top:1px solid rgba(255,255,255,.1); padding-top:1rem; } .result-group h3 { margin:0; font-size:1.08rem; } .signal-card { position:relative; margin-top:.75rem; padding:1rem; border:1px solid rgba(255,255,255,.1); border-radius:14px; background:linear-gradient(145deg,rgba(255,255,255,.065),rgba(3,8,10,.24)); box-shadow:inset 0 1px 0 rgba(255,255,255,.08); transition:transform var(--duration-normal),border-color var(--duration-normal),box-shadow var(--duration-normal); } .signal-card::before { content:''; position:absolute; left:0; top:1rem; bottom:1rem; width:2px; border-radius:99px; background:linear-gradient(var(--mantis-accent),transparent); opacity:.8; } .signal-card:hover { transform:translateX(3px); border-color:rgba(45,212,191,.32); box-shadow:inset 0 1px 0 rgba(255,255,255,.1),0 14px 30px rgba(0,0,0,.22); } .signal-card.signal-muted { opacity:.58; } .signal-heading { display:flex; justify-content:space-between; gap:.75rem; align-items:flex-start; } .signal-heading h4 { margin:0; font-size:.98rem; } .badge { white-space:nowrap; border:1px solid var(--mantis-warn); color:var(--mantis-warn); border-radius:999px; padding:.18rem .45rem; font-size:.7rem; } .badge.decided { border-color:var(--mantis-accent); color:var(--mantis-accent); } .evidence { margin:.75rem 0; padding:.65rem; border-left:3px solid var(--mantis-accent); background:color-mix(in srgb,var(--mantis-accent) 7%,transparent); } .evidence span { display:block; font-size:.7rem; color:var(--mantis-text-muted); text-transform:uppercase; } .evidence a { overflow-wrap:anywhere; color:var(--mantis-accent); font-size:.78rem; } .evidence p { margin:.25rem 0 0; color:var(--mantis-text-muted); font-size:.78rem; } .source-link { color:var(--mantis-accent); font-size:.84rem; } .tech-meta { color:var(--mantis-text-muted); font:.7rem/1.45 monospace; overflow-wrap:anywhere; } .review-actions { display:flex; flex-wrap:wrap; gap:.4rem; margin-top:.7rem; } .empty { order:8; padding:1rem; border:1px dashed var(--mantis-border); }
  .session-summary { order:2; display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:.6rem; margin:1rem 0; } .session-summary div { padding:.72rem; border:1px solid rgba(255,255,255,.1); border-radius:11px; background:linear-gradient(145deg,rgba(45,212,191,.09),rgba(255,255,255,.025)); } .session-summary strong,.session-summary span { display:block; } .session-summary strong { font-size:1.15rem; letter-spacing:-.04em; } .session-summary span { margin-top:.15rem; color:var(--mantis-text-muted); font-size:.68rem; } .session-summary .has-error { border-color:color-mix(in srgb,var(--mantis-danger) 45%,var(--mantis-border)); }
  .coverage-panel { margin:1rem 0; padding:1rem; border:1px solid color-mix(in srgb,var(--mantis-accent) 28%,var(--mantis-border)); border-radius:10px; background:linear-gradient(120deg,color-mix(in srgb,var(--mantis-accent) 8%,transparent),transparent); } .coverage-panel h3 { margin:.15rem 0 .25rem; } .coverage-list { display:grid; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); gap:.6rem; margin-top:.8rem; } .coverage-item { padding:.7rem; border:1px solid var(--mantis-border); border-radius:8px; background:color-mix(in srgb,var(--mantis-bg) 62%,transparent); } .coverage-item strong,.coverage-item span,.coverage-item small { display:block; } .coverage-item span { margin-top:.22rem; color:var(--mantis-text-muted); font-size:.78rem; } .coverage-item small { margin-top:.35rem; color:var(--mantis-warn); font-size:.72rem; } .coverage-item.has-error { border-color:color-mix(in srgb,var(--mantis-danger) 45%,var(--mantis-border)); }
  .resolution-panel { margin:1rem 0; padding:1rem; border:1px solid rgba(255,255,255,.11); border-radius:15px; background:linear-gradient(145deg,rgba(255,255,255,.055),rgba(5,10,12,.28)); } .resolution-panel>div:first-child { display:flex; justify-content:space-between; gap:1rem; align-items:flex-end; } .resolution-panel h3 { margin:.18rem 0; font-size:1.02rem; } .resolution-panel p { margin:.35rem 0 0; color:var(--mantis-text-muted); font-size:.76rem; line-height:1.45; } .resolution-summary { display:flex; flex-wrap:wrap; gap:.45rem; margin-top:.7rem; } .resolution-chip { border:1px solid rgba(255,255,255,.15); border-radius:999px; padding:.3rem .55rem; color:var(--mantis-text-muted); font-size:.69rem; } .resolution-chip.corroborated { border-color:color-mix(in srgb,var(--mantis-accent) 68%,white); color:var(--mantis-accent); } .resolution-chip.contradictory { border-color:var(--mantis-warn); color:var(--mantis-warn); } .resolution-chip.muted { opacity:.62; } .resolution-list { display:grid; grid-template-columns:repeat(auto-fit,minmax(245px,1fr)); gap:.5rem; margin-top:.75rem; } .resolution-list>div { padding:.65rem; border:1px solid rgba(255,255,255,.09); border-radius:10px; background:rgba(0,0,0,.12); } .resolution-list>div.contradictory { border-left:2px solid var(--mantis-warn); } .resolution-list>div.muted { opacity:.58; } .resolution-list strong { display:block; font-size:.74rem; } .resolution-list span { color:var(--mantis-text-muted); font-size:.68rem; } .resolution-list p { font-size:.7rem; }
  .claims-panel { order:4; margin:1.15rem 0; padding:1rem; border:1px solid color-mix(in srgb,var(--mantis-accent) 42%,var(--mantis-border)); border-radius:16px; background:radial-gradient(circle at 94% 5%,rgba(45,212,191,.12),transparent 31%),linear-gradient(135deg,color-mix(in srgb,var(--mantis-accent) 9%,transparent),rgba(255,255,255,.025)); } .claims-intro { display:flex; justify-content:space-between; gap:1rem; align-items:flex-start; } .claims-intro h3 { margin:.18rem 0 0; font-size:1.18rem; letter-spacing:-.025em; } .claims-intro>p { max-width:520px; margin:0; color:var(--mantis-text-muted); font-size:.76rem; line-height:1.45; } .claims-grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(290px,1fr)); gap:.75rem; margin-top:.9rem; } .claim-card { position:relative; overflow:hidden; padding:1rem; border:1px solid rgba(255,255,255,.11); border-left:3px solid var(--mantis-accent); border-radius:13px; background:linear-gradient(145deg,rgba(255,255,255,.07),rgba(0,0,0,.16)); box-shadow:inset 0 1px 0 rgba(255,255,255,.08); } .claim-card::after { content:''; position:absolute; z-index:0; width:12rem; height:12rem; right:-7rem; top:-8rem; border-radius:50%; background:radial-gradient(circle,rgba(45,212,191,.14),transparent 67%); } .claim-card>* { position:relative; z-index:1; } .claim-card.claim-contradiction { border-left-color:var(--mantis-warn); } .claim-card.claim-muted { opacity:.65; } .claim-heading,.claim-status { display:flex; justify-content:space-between; gap:.6rem; align-items:center; } .claim-type { color:var(--mantis-accent); font-size:.68rem; font-weight:750; letter-spacing:.05em; text-transform:uppercase; } .claim-priority { border:1px solid var(--mantis-border); border-radius:999px; padding:.15rem .4rem; color:var(--mantis-text-muted); font-size:.65rem; } .claim-priority.haute,.claim-priority.critique { border-color:var(--mantis-warn); color:var(--mantis-warn); } .claim-card h4 { margin:.55rem 0; font-size:1rem; } .claim-card>p { color:var(--mantis-text-muted); font-size:.78rem; line-height:1.45; } .claim-status strong { font-size:.78rem; } .claim-status span { color:var(--mantis-text-muted); font-size:.68rem; text-align:right; } .claim-evidence { display:grid; gap:.35rem; margin-top:.65rem; } .claim-evidence>div { display:flex; justify-content:space-between; gap:.6rem; padding:.42rem .5rem; border-radius:7px; background:rgba(255,255,255,.045); } .claim-evidence>div.contrary { border-left:2px solid var(--mantis-warn); } .claim-evidence span,.claim-evidence a,.claim-evidence em { font-size:.68rem; overflow-wrap:anywhere; } .claim-evidence span,.claim-evidence em { color:var(--mantis-text-muted); } .claim-evidence a { color:var(--mantis-accent); text-align:right; }
  .claim-synthesis { margin:.7rem 0; padding:.65rem; border:1px solid color-mix(in srgb,var(--mantis-ok) 38%,var(--mantis-border)); border-radius:7px; background:color-mix(in srgb,var(--mantis-ok) 6%,transparent); } .claim-synthesis.contrary { border-color:color-mix(in srgb,var(--mantis-warn) 52%,var(--mantis-border)); } .claim-synthesis>div { display:flex; justify-content:space-between; gap:.6rem; } .claim-synthesis strong { color:var(--mantis-ok); font-size:.72rem; } .claim-synthesis.contrary strong { color:var(--mantis-warn); } .claim-synthesis span,.claim-synthesis small { color:var(--mantis-text-muted); font-size:.66rem; overflow-wrap:anywhere; } .claim-synthesis p { margin:.4rem 0; font-size:.76rem; line-height:1.45; }
  .claim-review-actions { display:flex; flex-wrap:wrap; gap:.35rem; margin-top:.65rem; } .claim-review-actions .wf-btn { margin:0; padding:.36rem .55rem; font-size:.7rem; }
  .review-workspace { margin:1rem 0; padding:1rem; border:1px solid var(--mantis-border); border-radius:10px; background:rgba(255,255,255,.018); } .evolution-grid { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:.55rem; margin-top:.8rem; } .evolution-grid>div { display:grid; gap:.22rem; padding:.65rem; border:1px solid color-mix(in srgb,var(--mantis-ok) 35%,var(--mantis-border)); border-radius:7px; background:rgba(0,0,0,.16); } .evolution-grid>div.unavailable { border-color:color-mix(in srgb,var(--mantis-warn) 50%,var(--mantis-border)); } .evolution-grid strong { color:var(--mantis-ok); font-size:.7rem; text-transform:uppercase; } .evolution-grid .unavailable strong { color:var(--mantis-warn); } .evolution-grid span { font-size:.76rem; overflow-wrap:anywhere; } .evolution-grid small { color:var(--mantis-text-muted); } .review-workspace details { margin-top:.8rem; } .review-workspace summary { cursor:pointer; color:var(--mantis-accent); font-size:.78rem; } .review-history { display:grid; gap:.4rem; margin-top:.6rem; } .review-history>div { padding:.55rem; border-left:2px solid var(--mantis-accent); background:rgba(0,0,0,.16); } .review-history span,.review-history strong { display:block; font-size:.68rem; } .review-history span,.review-history p { color:var(--mantis-text-muted); } .review-history p { margin:.2rem 0 0; font-size:.72rem; }
  .analysis-panel { order:1; display:grid; grid-template-columns:minmax(0,1fr) auto; gap:.75rem 1rem; margin:1rem 0; padding:1.05rem; border:1px solid rgba(151,137,255,.48); border-radius:16px; background:radial-gradient(circle at 92% 18%,rgba(151,137,255,.2),transparent 30%),linear-gradient(120deg,rgba(55,43,114,.38),rgba(23,53,63,.38)); box-shadow:inset 0 1px 0 rgba(255,255,255,.13),0 14px 34px rgba(14,11,35,.19); } .analysis-panel h3 { margin:.2rem 0; font-size:1.17rem; letter-spacing:-.025em; } .analysis-panel>.wf-btn { align-self:center; margin:0; } .analysis-panel .analysis-result,.analysis-panel .analysis-progress { grid-column:1/-1; } .analysis-result { margin-top:.15rem; padding:.8rem; border-left:3px solid var(--mantis-ok); border-radius:9px; background:rgba(255,255,255,.055); } .analysis-result.fallback { border-left-color:var(--mantis-warn); } .analysis-heading { display:flex; justify-content:space-between; gap:.7rem; } .analysis-heading span,.analysis-result small { display:block; color:var(--mantis-text-muted); font-size:.72rem; } .analysis-item { margin:.7rem 0; padding:.7rem; border:1px solid color-mix(in srgb,var(--mantis-accent) 35%,var(--mantis-border)); border-radius:7px; } .analysis-item div { display:flex; gap:.45rem; align-items:center; } .analysis-item span { color:var(--mantis-warn); font-weight:700; font-size:.75rem; } .analysis-item span.important { color:var(--mantis-danger); } .analysis-item em { color:var(--mantis-text-muted); font-size:.7rem; } .analysis-item p { margin:.35rem 0; } .analysis-item small { color:var(--mantis-text-muted); overflow-wrap:anywhere; }
  .analysis-progress { display:flex; gap:.75rem; align-items:center; margin-top:.8rem; padding:.75rem; border:1px solid var(--mantis-accent); border-radius:7px; } .analysis-progress p { margin:.2rem 0 0; color:var(--mantis-text-muted); font-size:.78rem; } .analysis-spinner { width:1rem; height:1rem; flex:0 0 auto; border:2px solid var(--mantis-border); border-top-color:var(--mantis-accent); border-radius:50%; animation:spin .8s linear infinite; } @keyframes spin { to { transform:rotate(360deg); } }
  .ddgs-leads { margin:1rem 0; border:1px solid color-mix(in srgb,var(--ui-link) 42%,var(--mantis-border)); border-radius:var(--radius-md); background:var(--ui-surface-1); } .ddgs-leads summary { display:flex; align-items:center; justify-content:space-between; gap:1rem; min-height:48px; padding:.65rem .8rem; cursor:pointer; list-style:none; } .ddgs-leads summary::-webkit-details-marker { display:none; } .ddgs-leads summary span { display:grid; gap:.12rem; } .ddgs-leads summary strong { font-size:.82rem; } .ddgs-leads summary small { color:var(--ui-link); font:.66rem/1.3 var(--font-meta); text-transform:uppercase; letter-spacing:.06em; } .ddgs-leads summary>b { color:var(--ui-text-tertiary); font-size:1rem; transition:transform var(--duration-fast); } .ddgs-leads[open] summary>b { transform:rotate(180deg); } .ddgs-leads>p { margin:0; padding:0 .8rem .7rem; color:var(--ui-text-secondary); font-size:.76rem; line-height:1.45; } .ddgs-leads>div { display:grid; gap:.35rem; padding:.5rem; border-top:1px solid var(--ui-border-subtle); } .ddgs-leads article { display:grid; gap:.2rem; padding:.55rem .65rem; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-sm); background:var(--ui-canvas-elevated); } .ddgs-leads article>span { color:var(--ui-link); font:.62rem/1.2 var(--font-meta); text-transform:uppercase; letter-spacing:.06em; } .ddgs-leads article>strong { font-size:.8rem; } .ddgs-leads article>a { color:var(--ui-text-secondary); font-size:.72rem; overflow-wrap:anywhere; }
  .scan-progress { margin-top:1rem; overflow:hidden; position:relative; border-color:color-mix(in srgb,var(--mantis-accent) 30%,var(--mantis-border)); }
  .scan-progress.ai-phase { border-color:color-mix(in srgb,#9b8cff 55%,var(--mantis-border)); background:radial-gradient(circle at 85% 15%,rgba(135,111,255,.16),transparent 42%),color-mix(in srgb,#302563 12%,var(--mantis-panel)); }
  .scan-progress-heading { display:flex; align-items:center; justify-content:space-between; gap:1rem; }
  .scan-progress-heading h2 { margin:.15rem 0 0; }
  .progress-pulse { width:.7rem; height:.7rem; border-radius:50%; background:var(--mantis-accent); box-shadow:0 0 0 .3rem color-mix(in srgb,var(--mantis-accent) 14%,transparent),0 0 1.2rem var(--mantis-accent); animation:progress-pulse 1.5s ease-in-out infinite; }
  .ai-phase .progress-pulse { background:#b7a8ff; box-shadow:0 0 0 .35rem rgba(155,140,255,.14),0 0 1.8rem #9b8cff; }
  .scan-stages { display:grid; grid-template-columns:1fr 1fr; gap:.75rem; margin:1rem 0 .8rem; }
  .scan-stage { display:flex; align-items:center; gap:.7rem; padding:.8rem; border:1px solid var(--mantis-border); border-radius:10px; background:rgba(255,255,255,.025); opacity:.55; transition:all .35s ease; }
  .scan-stage.active { opacity:1; border-color:var(--mantis-accent); box-shadow:0 0 1.2rem color-mix(in srgb,var(--mantis-accent) 14%,transparent); animation:stage-lift 2.8s ease-in-out infinite; }
  .scan-stage.complete { opacity:.9; border-color:color-mix(in srgb,var(--mantis-ok) 45%,var(--mantis-border)); }
  .ai-stage.active { border-color:#9b8cff; box-shadow:0 0 1.5rem rgba(155,140,255,.2); animation:stage-lift-ai 2.2s ease-in-out infinite; }
  .scan-stage strong,.scan-stage small { display:block; } .scan-stage small { margin-top:.2rem; color:var(--mantis-text-muted); font-size:.75rem; }
  .stage-visual { position:relative; display:block; flex:0 0 2.25rem; width:2.25rem; height:2.25rem; border-radius:50%; background:rgba(255,255,255,.04); overflow:visible; }
  .stage-visual::before,.stage-visual::after,.stage-visual i,.stage-visual b { content:''; position:absolute; display:block; border-radius:50%; }
  .tools-visual { background:conic-gradient(from 0deg,rgba(84,230,197,.04),rgba(84,230,197,.34),rgba(84,230,197,.03) 28%,rgba(84,230,197,.04) 100%); animation:tools-sweep 2.2s linear infinite; }
  .tools-visual::before { inset:.72rem; background:#54e6c5; box-shadow:0 0 .8rem #54e6c5; }
  .tools-visual::after { inset:.24rem; border:1px solid rgba(84,230,197,.55); animation:tools-radar 2.4s ease-out infinite; }
  .tools-visual i { width:.32rem; height:.32rem; background:#a9fff0; box-shadow:0 0 .45rem #54e6c5; }
  .tools-visual i:nth-child(1) { top:.1rem; left:1rem; animation:node-blink 1.7s ease-in-out infinite; }
  .tools-visual i:nth-child(2) { right:.08rem; bottom:.5rem; animation:node-blink 1.7s .35s ease-in-out infinite; }
  .tools-visual i:nth-child(3) { left:.15rem; bottom:.3rem; animation:node-blink 1.7s .7s ease-in-out infinite; }
  .tools-visual b { inset:.48rem; border:1px dashed rgba(123,245,222,.55); animation:tools-spin 5s linear infinite; }
  .ai-stage-visual { background:conic-gradient(from 0deg,rgba(155,140,255,.1),rgba(218,211,255,.55),rgba(104,80,214,.12),rgba(155,140,255,.1)),radial-gradient(circle,#efeaff 0 7%,#a89aff 12%,#6551cf 36%,rgba(101,81,207,.14) 65%,transparent 68%); box-shadow:0 0 1.1rem rgba(155,140,255,.75); animation:ai-stage-breathe 2.2s ease-in-out infinite,ai-stage-spin 5.5s linear infinite; }
  .ai-stage-visual::before { inset:-.18rem; border:1px solid rgba(203,194,255,.7); animation:ai-stage-orbit 3.2s linear infinite; }
  .ai-stage-visual::after { inset:-.45rem .1rem; border:1px solid rgba(138,224,218,.48); transform:rotate(55deg); animation:ai-stage-orbit 4.8s linear infinite reverse; }
  .ai-stage-visual i { inset:-.62rem .48rem; border:1px solid rgba(188,169,255,.35); animation:ai-stage-orbit 5.8s linear infinite; }
  .ai-stage-visual i:nth-child(2) { inset:.05rem -.48rem; animation-duration:4.1s; animation-direction:reverse; }
  .ai-stage-visual i:nth-child(3) { inset:-.4rem .05rem; animation-duration:3.7s; border-color:rgba(255,255,255,.35); }
  .ai-stage-visual b { inset:.82rem; background:#fff; box-shadow:0 0 .55rem #e3ddff; }
  .scan-stage.complete .stage-visual { filter:saturate(.72); opacity:.75; }
  @keyframes tools-radar { 0% { transform:scale(.35); opacity:.8; } 80%,100% { transform:scale(1.35); opacity:0; } }
  @keyframes tools-sweep { to { transform:rotate(360deg); } }
  @keyframes tools-spin { to { transform:rotate(360deg); } }
  @keyframes node-blink { 50% { transform:scale(1.55); opacity:.45; } }
  @keyframes ai-stage-breathe { 50% { transform:scale(1.1); filter:saturate(1.25); } }
  @keyframes ai-stage-spin { to { background-position:100% 0,0 0; } }
  @keyframes ai-stage-orbit { to { transform:rotate(360deg); } }
  @keyframes stage-lift { 50% { transform:translateY(-2px); } }
  @keyframes stage-lift-ai { 50% { transform:translateY(-3px) scale(1.01); } }
  .scan-progress-track { height:.42rem; overflow:hidden; border-radius:99px; background:rgba(255,255,255,.08); }
  .scan-progress-fill { height:100%; border-radius:inherit; background:linear-gradient(90deg,var(--mantis-accent),#71f5d4); transition:width .8s ease; position:relative; overflow:hidden; }
  .scan-progress-fill::after { content:''; position:absolute; inset:0; background:linear-gradient(90deg,transparent,rgba(255,255,255,.65),transparent); transform:translateX(-100%); animation:progress-sheen 1.8s ease-in-out infinite; }
  .scan-progress-fill.ai-fill { background:linear-gradient(90deg,#6f61d9,#c9c0ff,#8debdc); box-shadow:0 0 1rem rgba(155,140,255,.55); }
  .scan-progress-detail { margin:.6rem 0 0; color:var(--mantis-text-muted); font-size:.78rem; }
  .filtered-candidates { margin:1rem 0; padding:.85rem 1rem; border:1px dashed color-mix(in srgb,var(--mantis-warn) 45%,var(--mantis-border)); border-radius:8px; background:rgba(245,158,11,.05); } .filtered-candidates p { margin:.3rem 0 .65rem; color:var(--mantis-text-muted); font-size:.82rem; }
  @keyframes progress-pulse { 50% { transform:scale(1.18); opacity:.7; } }
  @keyframes progress-sheen { to { transform:translateX(100%); } }
  .advanced-toggle { margin-top:1.25rem; padding:.75rem 0; } .advanced-toggle label { font-weight:700; } .advanced { margin-top:.2rem; } .tool-list { display:flex; flex-direction:column; gap:.4rem; } .tool-list button { text-align:left; padding:.7rem; color:var(--mantis-text); background:transparent; border:1px solid var(--mantis-border); border-radius:6px; } .tool-list button.active { border-color:var(--mantis-accent); } .tool-list span { display:block; margin-top:.2rem; color:var(--mantis-text-muted); font-size:.75rem; } .log { border-top:1px solid var(--mantis-border); padding:.55rem 0; font: .73rem/1.4 monospace; overflow-wrap:anywhere; } .builtin-badge { align-self:center; border:1px solid var(--mantis-ok); border-radius:999px; padding:.32rem .55rem; color:var(--mantis-ok); font-size:.72rem; } .wf-btn.danger { border-color:color-mix(in srgb,var(--mantis-danger) 55%,var(--mantis-border)); color:var(--mantis-danger); } .catalog-maintenance { order:8; display:grid; grid-template-columns:1fr auto; gap:.8rem 1rem; align-items:start; margin-top:1rem; } .catalog-maintenance h2 { margin:.15rem 0 .35rem; font-size:1.08rem; } .maintenance-notice,.archive-grid { grid-column:1/-1; } .archive-grid { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:.6rem; } .archive-grid>div { padding:.7rem; border:1px solid var(--mantis-border); border-radius:7px; background:rgba(0,0,0,.18); } .archive-grid span,.archive-grid strong,.archive-grid small { display:block; } .archive-grid span { color:var(--mantis-warn); font-size:.65rem; font-weight:700; text-transform:uppercase; } .archive-grid strong { margin-top:.25rem; font-size:.8rem; } .archive-grid p,.archive-grid small { color:var(--mantis-text-muted); font-size:.68rem; line-height:1.45; }
  .candidate-audit { margin-top:1rem; } .candidate-audit>div { display:grid; grid-template-columns:minmax(0,1fr) minmax(0,2fr) auto; gap:.7rem; align-items:start; padding:.7rem 0; border-top:1px solid var(--mantis-border); } .candidate-audit span,.candidate-audit a { color:var(--mantis-text-muted); font-size:.76rem; overflow-wrap:anywhere; }
  .routine-card { margin-top:.8rem; padding-top:.8rem; border-top:1px solid var(--mantis-border); }
  .results { border-top-color:color-mix(in srgb,var(--mantis-accent) 52%,var(--mantis-border)); background:var(--ui-surface-1); }
  .result-summary div { border-color:var(--mantis-border); border-radius:10px; background:var(--ui-surface-2); box-shadow:none; }
  .result-summary div::after { display:none; }
  .signal-card { border-color:var(--mantis-border); border-radius:10px; background:var(--ui-surface-2); box-shadow:none; }
  .signal-card:hover { transform:none; border-color:var(--mantis-border-strong); box-shadow:none; }
  .advanced-toggle { display:flex; align-items:center; justify-content:space-between; gap:1rem; padding:.8rem .9rem; border:1px solid var(--mantis-border); border-radius:12px; background:var(--ui-surface-1); } .advanced-toggle strong { display:block; font-size:.82rem; } .advanced-toggle p { margin:.18rem 0 0; } .advanced-toggle label { display:flex; align-items:center; gap:.45rem; margin:0; color:var(--mantis-accent); }
  @media (max-width:900px) { .scan-page-header { align-items:stretch; flex-direction:column; } .scan-page-header .history-panel { align-self:flex-end; } .identity-command { grid-template-columns:1fr; align-items:stretch; } .identity-selector { grid-template-columns:1fr; } .scan-command { justify-content:space-between; } .claims-grid { grid-template-columns:1fr; } }
  @media (max-width:760px) { .main-grid,.advanced-grid,.routine-card,.catalog-maintenance,.analysis-panel { grid-template-columns:1fr; } .archive-grid { grid-template-columns:1fr; } .result-summary { grid-template-columns:repeat(2,minmax(0,1fr)); } .evolution-grid { grid-template-columns:1fr; } .scan-command { align-items:stretch; flex-direction:column; } .advanced-toggle,.ai-view-banner { align-items:stretch; flex-direction:column; } .scan-stages { grid-template-columns:1fr; } .analysis-panel>.wf-btn { width:100%; } }
  .ai-gate { display:flex; gap:.9rem; align-items:flex-start; margin:1rem 0; padding:1rem; border:1px solid color-mix(in srgb,var(--mantis-accent) 45%,var(--mantis-border)); border-radius:10px; background:color-mix(in srgb,var(--mantis-accent) 8%,transparent); }
  .ai-gate strong { display:block; }
  .ai-gate p { margin:.25rem 0; color:var(--mantis-text-muted); }
  .ai-gate small { color:var(--mantis-text-muted); }
  .premium-ai-gate { position:relative; overflow:hidden; min-height:5.1rem; border-color:rgba(168,151,255,.72); background:radial-gradient(circle at 2% 50%,rgba(137,113,255,.2),transparent 28%),linear-gradient(110deg,rgba(53,44,103,.26),rgba(21,45,55,.3)); box-shadow:inset 0 0 2.2rem rgba(155,140,255,.08),0 .65rem 2.5rem rgba(20,14,54,.2); }
  .premium-ai-gate::after { content:''; position:absolute; inset:0; pointer-events:none; background:linear-gradient(100deg,transparent 15%,rgba(216,210,255,.11) 48%,transparent 82%); transform:translateX(-110%); animation:ai-sheen 3.2s ease-in-out infinite; }
  .ai-orb { position:relative; flex:0 0 2.8rem; width:2.8rem; height:2.8rem; margin:.1rem .2rem 0 0; border-radius:50%; background:radial-gradient(circle at 42% 35%,#f3f0ff 0 5%,#b8aaff 16%,#7461df 42%,rgba(95,72,181,.25) 67%,transparent 70%); box-shadow:0 0 1.1rem #9d8aff,0 0 2.6rem rgba(130,104,255,.5); animation:ai-breathe 2.5s ease-in-out infinite; }
  .ai-orb::before,.ai-orb::after,.ai-orb i { content:''; position:absolute; inset:-.28rem; border:1px solid rgba(202,193,255,.65); border-radius:50%; transform:rotate(35deg); animation:ai-orbit 3.4s linear infinite; }
  .ai-orb::after { inset:-.58rem .12rem; transform:rotate(-48deg); animation-duration:4.8s; border-color:rgba(142,226,219,.5); }
  .ai-orb i { inset:-.05rem -.42rem; transform:rotate(82deg); animation-duration:2.8s; border-color:rgba(255,255,255,.38); }
  .ai-orb i:nth-child(2) { inset:-.42rem -.05rem; animation-duration:3.9s; animation-direction:reverse; }
  .ai-orb i:nth-child(3) { inset:-.72rem .4rem; animation-duration:5.6s; border-color:rgba(174,156,255,.34); }
  @keyframes ai-breathe { 50% { transform:scale(1.08); filter:saturate(1.2); } }
  @keyframes ai-orbit { to { transform:rotate(395deg); } }
  @keyframes ai-sheen { 55%,100% { transform:translateX(110%); } }
  .leak-annotation { display:grid; gap:.2rem; margin:.55rem 0; padding:.55rem .7rem; border-left:3px solid var(--mantis-danger); background:color-mix(in srgb,var(--mantis-danger) 8%,transparent); }
  .leak-annotation strong { color:var(--mantis-danger); font-size:.75rem; text-transform:uppercase; }
  .leak-annotation span { color:var(--mantis-text-muted); font-size:.78rem; }
  .post-scan-workspace{margin:1rem 0;padding:1rem;border:1px solid color-mix(in srgb,var(--mantis-danger) 42%,var(--mantis-border));border-radius:15px;background:radial-gradient(circle at 96% 4%,rgba(245,112,112,.13),transparent 30%),linear-gradient(125deg,rgba(119,32,40,.2),rgba(255,255,255,.02))}.post-scan-heading{display:flex;justify-content:space-between;gap:1rem;align-items:flex-start}.post-scan-heading h3{margin:.18rem 0;font-size:1.16rem;letter-spacing:-.025em}.post-scan-heading p:not(.eyebrow){max-width:650px;margin:.25rem 0 0;color:var(--mantis-text-muted);font-size:.76rem;line-height:1.45}.post-scan-count{display:grid;place-items:end;flex:0 0 auto}.post-scan-count strong{color:var(--mantis-danger);font:750 1.7rem/1 var(--font-meta)}.post-scan-count span{color:var(--mantis-text-muted);font-size:.64rem;white-space:nowrap}.post-scan-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(285px,1fr));gap:.65rem;margin-top:.9rem}.post-scan-item{display:flex;flex-direction:column;padding:.85rem;border:1px solid var(--mantis-border);border-left:3px solid var(--mantis-warn);border-radius:10px;background:rgba(7,9,11,.5)}.post-scan-item.severity-critique{border-left-color:var(--mantis-danger)}.post-scan-item.severity-elevee,.post-scan-item.severity-élevée{border-left-color:var(--mantis-warn)}.post-scan-item-head{display:flex;justify-content:space-between;gap:.6rem;align-items:center}.signal-kind{color:var(--mantis-text-muted);font:700 .62rem/1 var(--font-meta);letter-spacing:.07em;text-transform:uppercase}.severity-pill{padding:.22rem .45rem;border:1px solid currentColor;border-radius:999px;color:var(--mantis-warn);font-size:.64rem}.severity-critique .severity-pill{color:var(--mantis-danger)}.post-scan-item h4{margin:.55rem 0;font-size:.9rem;line-height:1.35}.post-scan-item dl{display:grid;gap:.4rem;margin:0}.post-scan-item dl>div{display:grid;gap:.1rem;padding:.42rem .5rem;border-radius:6px;background:rgba(255,255,255,.04)}.post-scan-item dt{color:var(--mantis-text-muted);font:700 .58rem/1 var(--font-meta);letter-spacing:.06em;text-transform:uppercase}.post-scan-item dd{margin:0;color:var(--mantis-text);font-size:.7rem;line-height:1.35;overflow-wrap:anywhere}.post-scan-evidence{margin:.55rem 0}.post-scan-evidence summary{cursor:pointer;color:var(--mantis-text-muted);font-size:.68rem;list-style:none}.post-scan-evidence summary::-webkit-details-marker{display:none}.post-scan-evidence summary::before{content:'+';display:inline-block;margin-right:.35rem;color:var(--mantis-accent);font-weight:800}.post-scan-evidence[open] summary::before{content:'−'}.post-scan-evidence p{max-height:12rem;overflow:auto;margin:.5rem 0 0;padding:.55rem;border-radius:7px;background:rgba(255,255,255,.035);color:var(--mantis-text-muted);font-size:.7rem;line-height:1.42;overflow-wrap:anywhere}.post-scan-actions{display:flex;flex-wrap:wrap;gap:.38rem;margin-top:auto}.post-scan-actions .wf-btn{margin:0;padding:.42rem .55rem;font-size:.68rem}.post-scan-done{display:flex;justify-content:space-between;align-items:center;gap:.6rem;margin-top:auto;padding:.6rem;border:1px solid color-mix(in srgb,var(--mantis-ok) 45%,var(--mantis-border));border-radius:7px;background:rgba(76,175,127,.07)}.post-scan-done span{color:var(--mantis-ok);font-size:.7rem;font-weight:700}.post-scan-done a{color:var(--mantis-ok);font-size:.7rem}.post-scan-disclaimer{margin:.8rem 0 0;color:var(--mantis-text-muted);font-size:.68rem;line-height:1.4}.breach-sites{display:grid;gap:.4rem;margin-top:.35rem}.breach-site{border:1px solid var(--mantis-border);border-radius:8px;background:rgba(255,255,255,.025)}.breach-site summary{display:flex;justify-content:space-between;gap:.8rem;align-items:center;padding:.58rem .65rem;cursor:pointer;list-style:none}.breach-site summary::-webkit-details-marker{display:none}.breach-site summary::before{content:'+';color:var(--mantis-warn);font-weight:800}.breach-site[open] summary::before{content:'−'}.breach-site summary strong{font-size:.74rem}.breach-site summary span{color:var(--mantis-text-muted);font-size:.64rem;white-space:nowrap}.breach-site-detail{padding:0 .7rem .7rem}.breach-site-detail p{margin:.35rem 0 .55rem;color:var(--mantis-text-muted);font-size:.7rem;line-height:1.4}.breach-site-detail .post-scan-actions{margin-top:.6rem}.post-scan-grouped-services{padding:.6rem}.post-scan-grouped-services h4{margin:.3rem 0;font-size:.82rem}.post-scan-grouped-services>p{margin:.35rem 0;font-size:.66rem}.post-scan-grouped-services .breach-sites{grid-template-columns:repeat(2,minmax(0,1fr));gap:.25rem}.post-scan-grouped-services .breach-site summary{padding:.4rem .45rem}.post-scan-grouped-services .breach-site summary strong{font-size:.68rem}.post-scan-grouped-services .breach-site summary span{font-size:.58rem}.post-scan-grouped-services .breach-site-detail{padding:0 .5rem .5rem}
  .post-scan-grid{grid-auto-rows:360px}.post-scan-item{height:360px;min-height:0;overflow-x:hidden;overflow-y:auto}.post-scan-grouped-breaches{padding:.65rem}.post-scan-grouped-breaches h4{margin:.3rem 0 .45rem;font-size:.84rem}@media(max-width:760px){.post-scan-grid{grid-auto-rows:340px}.post-scan-item{height:340px}}
  .prevention-path{margin:1rem 0;padding:1rem;border:1px solid color-mix(in srgb,var(--mantis-ok) 32%,var(--mantis-border));border-radius:12px;background:linear-gradient(120deg,rgba(62,170,127,.07),rgba(255,255,255,.012))}.prevention-heading{display:grid;grid-template-columns:1fr minmax(240px,420px);gap:1rem;align-items:end}.prevention-heading h3{margin:.2rem 0 0;font-size:1.05rem}.prevention-heading>p{margin:0;color:var(--mantis-text-muted);font-size:.74rem;line-height:1.45}.prevention-guides{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:.55rem;margin-top:.8rem}.prevention-guides a{display:grid;grid-template-columns:54px 1fr auto;gap:.65rem;align-items:center;padding:.55rem;border:1px solid var(--mantis-border);border-radius:9px;color:inherit;text-decoration:none;background:rgba(0,0,0,.16)}.prevention-guides a:hover{border-color:var(--prevent-accent)}.prevention-guides img{width:54px;height:54px;border-radius:7px;object-fit:cover;background:#080a0c}.prevention-guides span{display:grid;gap:.12rem;min-width:0}.prevention-guides small{color:var(--prevent-accent);font-size:.6rem}.prevention-guides strong{font-size:.76rem}.prevention-guides p{margin:0;color:var(--mantis-text-muted);font-size:.64rem;line-height:1.35}.prevention-guides b{color:var(--prevent-accent)}
  @media(max-width:900px){.prevention-guides{grid-template-columns:1fr}.prevention-heading{grid-template-columns:1fr}}

	/* Scanner stays evidence-first: active processing is visible, but its
	   decoration does not compete with source, proof or uncertainty. */
	.scan-progress,
	.scan-stage,
	.results,
	.signal-card,
	.advanced-toggle,
	.ai-view-banner { background:var(--ui-material-panel); }
	.scan-progress.ai-phase,
	.premium-ai-gate { background:var(--ui-material-panel); box-shadow:inset 0 1px 0 var(--ui-rim-light),var(--shadow-1); }
	.scan-stage.active,
	.ai-stage.active { box-shadow:inset 0 1px 0 var(--ui-rim-light); }
	.progress-pulse,
	.ai-phase .progress-pulse { box-shadow:0 0 0 .22rem color-mix(in srgb,var(--ui-accent) 10%,transparent); }
	.scan-progress-track { background:var(--ui-surface-3); }
	.scan-progress-fill,
	.scan-progress-fill.ai-fill { background:var(--ui-accent); }
	.post-scan-workspace { background:var(--ui-material-panel); }
	.post-scan-item,
	.breach-site,
	.post-scan-item dl>div,
	.breach-site-detail p { background:var(--ui-material-solid); }
	/* Decision cards stay equal-height and use a controlled red review material:
	   the red is reserved for items requiring a decision, while the text keeps
	   the high-contrast neutral treatment. */
	.post-scan-item {
		box-sizing:border-box;
		min-width:0;
		background:linear-gradient(145deg,color-mix(in srgb,var(--ui-danger) 15%,var(--ui-material-solid)),color-mix(in srgb,var(--ui-danger) 6%,var(--ui-material-solid)));
		border-color:color-mix(in srgb,var(--ui-danger) 58%,var(--ui-border-default));
		border-left-color:var(--ui-danger);
	}
	.post-scan-item.severity-critique,
	.post-scan-item.severity-elevee,
	.post-scan-item.severity-élevée { border-left-color:var(--ui-danger); }
	.post-scan-item h4,
	.post-scan-item p,
	.post-scan-item dd,
	.post-scan-item dt,
	.post-scan-item a,
	.post-scan-item summary,
	.post-scan-item strong,
	.post-scan-item span { min-width:0; max-width:100%; overflow-wrap:anywhere; word-break:break-word; }
	.post-scan-item h4 { color:var(--ui-text-primary); }
	.post-scan-item dl>div { background:color-mix(in srgb,var(--ui-danger) 8%,var(--ui-material-solid)); }
	.post-scan-item .breach-site { border-color:color-mix(in srgb,var(--ui-danger) 36%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-danger) 7%,var(--ui-material-solid)); }
	.post-scan-item .breach-site summary { align-items:flex-start; }
	.post-scan-item .breach-site summary strong { flex:1 1 auto; }
	.post-scan-item .breach-site summary span { flex:0 1 42%; white-space:normal; text-align:right; }
	.post-scan-item .post-scan-actions { min-width:0; }
	.results { background:radial-gradient(circle at 88% 9%,color-mix(in srgb,var(--ui-accent) 10%,transparent),transparent 27%),linear-gradient(145deg,rgba(255,255,255,.07),rgba(255,255,255,.018)); }
	.result-summary div::after,
	.claim-card::after { background:radial-gradient(circle,color-mix(in srgb,var(--ui-accent) 15%,transparent),transparent 65%); }
	.signal-card:hover { border-color:color-mix(in srgb,var(--ui-accent) 32%,var(--ui-border-default)); }
	.session-summary div { background:linear-gradient(145deg,color-mix(in srgb,var(--ui-accent) 8%,transparent),rgba(255,255,255,.025)); }
	.claims-panel { background:radial-gradient(circle at 94% 5%,color-mix(in srgb,var(--ui-accent) 11%,transparent),transparent 31%),linear-gradient(135deg,color-mix(in srgb,var(--ui-accent) 9%,transparent),rgba(255,255,255,.025)); }
</style>
