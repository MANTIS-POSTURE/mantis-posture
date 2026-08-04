<script lang="ts">
	import { onMount } from 'svelte';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import PostureScore from '$lib/components/PostureScore.svelte';
	import AlertList from '$lib/components/AlertList.svelte';
	import IncidentList from '$lib/components/IncidentList.svelte';
	import ExposureList from '$lib/components/ExposureList.svelte';
	import Timeline from '$lib/components/Timeline.svelte';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import { getPublicIpContext, getVeilleRoutine, listIdentities, listIdentityScanSessions, updateVeilleRoutine, type PublicIpContext, type VeilleRoutine } from '$lib/api';
	import { activeIdentityId } from '$lib/active-identity';
	import { t } from '$lib/i18n';

	let identityLabel = $state('Aucune identité active');
	let lastScanAt = $state<string | null>(null);
	let scanStatus = $state<'none'|'ready'|'partial'>('none');
	let contextRequest = 0;
	let terminalOutput = $state('> initialisation liaison réseau…');
	let terminalStatus = $state('Analyse du contexte réseau en cours.');
	let networkLoading = $state(true);
	let typingTimer: ReturnType<typeof setTimeout> | null = null;
	let networkRefreshTimer: ReturnType<typeof setInterval> | null = null;
	let typingRun = 0;
	let networkRequest = 0;
	let routine = $state<VeilleRoutine | null>(null);
	let routineBusy = $state(false);
	let routineNotice = $state('');
	let routineError = $state('');
	const TERMINAL_TYPING_MS = 10_000;
	const TERMINAL_HOLD_MS = 7_000;
	const NETWORK_REFRESH_MS = 30_000;

	onMount(() => activeIdentityId.subscribe((identityId) => { void loadContext(identityId); }));
	onMount(() => {
		void loadNetworkContext();
		networkRefreshTimer = window.setInterval(() => { void loadNetworkContext(); }, NETWORK_REFRESH_MS);
		return () => {
			typingRun += 1;
			if (typingTimer) clearTimeout(typingTimer);
			if (networkRefreshTimer) clearInterval(networkRefreshTimer);
		};
	});
	onMount(() => { void loadRoutine(); });

	async function loadRoutine() {
		try { routine = await getVeilleRoutine(); }
		catch { routineError = t('Impossible de charger la planification des scans.'); }
	}

	async function saveRoutine(frequency: string, paused = routine?.paused ?? false) {
		routineBusy = true;
		routineNotice = '';
		routineError = '';
		try {
			routine = await updateVeilleRoutine(frequency, paused);
			routineNotice = t('Planification enregistrée.');
		} catch { routineError = t('Impossible d’enregistrer la planification.'); }
		finally { routineBusy = false; }
	}

	function animateTerminal(lines: string[]) {
		const text = lines.join('\n');
		const run = ++typingRun;
		if (typingTimer) clearTimeout(typingTimer);
		terminalOutput = '';
		let index = 0;
		const characterDelay = TERMINAL_TYPING_MS / Math.max(1, text.length);
		const typeNext = () => {
			if (run !== typingRun) return;
			index = Math.min(text.length, index + 1);
			terminalOutput = text.slice(0, index);
			if (index < text.length) typingTimer = setTimeout(typeNext, characterDelay);
			else typingTimer = setTimeout(() => { if (run === typingRun) animateTerminal(lines); }, TERMINAL_HOLD_MS);
		};
		typingTimer = setTimeout(typeNext, characterDelay);
	}

	function networkTerminalLines(context: PublicIpContext) {
		const location = [...new Set([context.city, context.region, context.country].filter((value): value is string => Boolean(value)))].join(' · ');
		const network = [context.isp ?? context.organization, context.asn ? `AS${context.asn}` : null].filter(Boolean).join(' · ');
		const routeSignals = [context.vpn ? 'VPN' : null, context.proxy ? 'PROXY' : null, context.tor ? 'TOR' : null, context.hosting ? t('HÉBERGEMENT') : null].filter(Boolean);
		const lines = [
			`> IP     ${context.ip}${context.ip_type ? ` · ${context.ip_type}` : ''}`,
		`GEO ≈    ${location || t('position non déterminée')}`,
		`NETWORK   ${network || t('opérateur non déterminé')}`,
		`SIGNAL   ${routeSignals.length ? `${routeSignals.join(' · ')} ${t('signalé')}` : t('aucun relais signalé')}`
		];
		return lines;
	}

	async function loadNetworkContext() {
		const request = ++networkRequest;
		networkLoading = true;
		terminalStatus = t('Analyse du contexte réseau en cours.');
		animateTerminal(['> uplink.inspect --public', t('… résolution du point de sortie')]);
		try {
			const context = await getPublicIpContext();
			if (request !== networkRequest) return;
			animateTerminal(networkTerminalLines(context));
		terminalStatus = t(`Contexte réseau chargé pour l’adresse IP publique ${context.ip}. La position affichée est une estimation.`);
		} catch {
			if (request !== networkRequest) return;
			animateTerminal(['> uplink.inspect --public', `[!] ${t('CONTEXTE RÉSEAU INDISPONIBLE')}`, `    ${t('aucune donnée conservée')}`]);
		terminalStatus = t('Le contexte réseau est indisponible. Aucune donnée n’a été conservée.');
		} finally { if (request === networkRequest) networkLoading = false; }
	}

	async function loadContext(identityId: string | null) {
		const request = ++contextRequest;
		if (!identityId) { identityLabel = t('Aucune identité active'); lastScanAt = null; scanStatus = 'none'; return; }
		try {
			const [identities, sessions] = await Promise.all([listIdentities(), listIdentityScanSessions(identityId, 1, 0)]);
			if (request !== contextRequest) return;
			identityLabel = identities.find((identity) => identity.id === identityId)?.label ?? t('Identité active');
			lastScanAt = sessions[0]?.completed_at ?? sessions[0]?.started_at ?? null;
			scanStatus = sessions[0] ? (sessions[0].status === 'termine' ? 'ready' : 'partial') : 'none';
		} catch { if (request === contextRequest) { identityLabel = t('Identité active'); lastScanAt = null; scanStatus = 'none'; } }
	}

	function formatDate(value: string | null) {
		if (!value) return t('Aucun scan enregistré');
		const parsed = new Date(value.replace(' ', 'T') + (value.includes('Z') ? '' : 'Z'));
		return Number.isNaN(parsed.getTime()) ? value : `${t('Dernier scan')} · ${parsed.toLocaleDateString(document.documentElement.lang === 'en' ? 'en-US' : 'fr-FR', { day:'2-digit', month:'short', year:'numeric' })}`;
	}
</script>

<svelte:head><title>Tableau de bord - MANTIS</title></svelte:head>

<section class="wf-view cockpit-view">
	<div class="dashboard-hero">
		<GuideHeader title="Tableau de bord" question="Que dois-je regarder et faire maintenant ?" intro="Votre situation, les priorités à examiner et l’activité récente — sans transformer une observation en conclusion." />
		<div class="network-terminal" aria-label={t('Contexte réseau public estimé')}><pre aria-hidden="true">{terminalOutput}<span class:busy={networkLoading} class="terminal-caret"></span></pre><p class="sr-only" aria-live="polite">{terminalStatus}</p></div>
	</div>

	<section class="command-center" aria-label={t('Situation de l’identité active')}>
		<div class="context-copy">
			<div class="context-heading">
				<p class="eyebrow">{t('Situation actuelle')}</p>
				<StatusBadge tone={scanStatus === 'ready' ? 'success' : scanStatus === 'partial' ? 'warning' : 'neutral'} dot label={scanStatus === 'ready' ? 'À jour' : scanStatus === 'partial' ? 'À vérifier' : 'À initialiser'} />
			</div>
			<h2>{identityLabel}</h2>
			<p>{formatDate(lastScanAt)} · {t('Données conservées sur cet appareil')}</p>
			<div class="context-links"><a href="/identites">{t('Changer d’identité')}</a><a href="/rapports">{t('Ouvrir le bilan')}</a></div>
		</div>
		<section class="scan-automation dashboard-automation" aria-labelledby="scan-automation-title">
			<div class="scan-automation-heading"><div><p class="eyebrow">{t('Automatisation des scans')}</p><h3 id="scan-automation-title">{t('Planifier des vérifications régulières')}</h3></div><span class="automation-status">{routine?.paused ? t('En pause') : routine?.frequency ?? t('Manuelle')}</span></div>
			<div class="scan-automation-controls">
				<label for="dashboard-frequency">{t('Fréquence')}</label>
				<select id="dashboard-frequency" value={routine?.frequency ?? 'Manuelle'} onchange={(event) => saveRoutine((event.currentTarget as HTMLSelectElement).value)} disabled={routineBusy}>
					<option value="Manuelle">{t('Manuelle')}</option><option value="Quotidienne">{t('Quotidienne')}</option><option value="Hebdomadaire">{t('Hebdomadaire')}</option><option value="Mensuelle">{t('Mensuelle')}</option>
				</select>
				<div class="automation-actions"><button class="wf-btn" type="button" onclick={() => saveRoutine(routine?.frequency ?? 'Manuelle', !(routine?.paused ?? false))} disabled={routineBusy}>{routine?.paused ? t('Reprendre') : t('Mettre en pause')}</button><button class="wf-btn primary" type="button" onclick={() => saveRoutine(routine?.frequency ?? 'Manuelle', false)} disabled={routineBusy}>{routineBusy ? t('Enregistrement…') : t('Programmer')}</button></div>
				{#if routineNotice}<p class="automation-feedback success" role="status" title={routineNotice}>{routineNotice}</p>{/if}
				{#if routineError}<p class="automation-feedback error" role="alert" title={routineError}>{routineError}</p>{/if}
			</div>
		</section>

		<a class="scan-launcher" href="/veille" aria-label={t('Ouvrir Scanner pour l’identité active')}>
			<span class="scan-radar" aria-hidden="true"><i></i><i></i><i></i></span>
			<span class="scan-copy"><small>{t('Nouvelle observation')}</small><strong>SCAN</strong></span>
			<svg class="scan-arrow" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14"/><path d="m13 6 6 6-6 6"/></svg>
		</a>
	</section>

	<section class="dashboard-columns" aria-label={t('Vue d’ensemble de la posture')}>
		<div class="dashboard-column primary-column">
			<div class="situation-slot"><PostureScore /></div>
			<div class="timeline-slot"><Timeline /></div>
		</div>
		<div class="dashboard-column secondary-column">
			<div class="priority-slot"><AlertList /></div>
			<section class="glass-card actions-card">
			<div><p class="eyebrow">{t('Passer à l’action')}</p><h2>{t('Raccourcis utiles')}</h2><p>{t('Créez uniquement les objets métier dont vous avez besoin. Aucun scan ne le fera automatiquement.')}</p></div>
			<nav aria-label={t('Actions rapides du tableau de bord')}>
				<a class="action-link" href="/actions"><span>01</span><div><strong>{t('Continuer mon plan d’action')}</strong><small>{t('Suivre les mesures déjà décidées')}</small></div><b>→</b></a>
				<a class="action-link" href="/incidents"><span>02</span><div><strong>{t('Déclarer un incident')}</strong><small>{t('Documenter un événement confirmé')}</small></div><b>→</b></a>
				<a class="action-link" href="/dpo"><span>03</span><div><strong>{t('Préparer une demande RGPD')}</strong><small>{t('Agir auprès d’un organisme identifié')}</small></div><b>→</b></a>
			</nav>
		</section>
		</div>
	</section>

	<section class="follow-up" aria-labelledby="follow-up-title">
		<div class="section-heading"><div><p class="eyebrow">{t('Suivi')}</p><h2 id="follow-up-title">{t('Éléments conservés dans MANTIS')}</h2></div><a href="/actions">{t('Voir toutes les actions →')}</a></div>
		<div class="follow-up-grid"><IncidentList /><ExposureList /></div>
	</section>
</section>

<style>
	.cockpit-view { max-width:1580px; width:100%; container-type:inline-size; }
	.dashboard-hero { display:grid; grid-template-columns:minmax(0,1fr) minmax(360px,440px); gap:1rem; align-items:center; }
	.network-terminal { min-width:0; min-height:62px; display:flex; align-items:center; padding:0; overflow:hidden; }
	.network-terminal pre { margin:0; color:color-mix(in srgb,var(--ui-accent) 84%,white); font-family:var(--font-meta); font-size:clamp(.78rem,.9vw,.9rem); font-weight:570; line-height:1.55; white-space:pre-wrap; overflow-wrap:anywhere; text-shadow:0 0 10px color-mix(in srgb,var(--ui-accent) 32%,transparent); }
	.terminal-caret { display:inline-block; width:.62em; height:1em; margin-left:3px; vertical-align:-.14em; background:var(--ui-accent); animation:terminal-blink .85s steps(1,end) infinite; box-shadow:0 0 8px var(--ui-accent); }.terminal-caret.busy { animation-duration:.42s; }
	.sr-only { position:absolute; width:1px; height:1px; padding:0; margin:-1px; overflow:hidden; clip:rect(0,0,0,0); white-space:nowrap; border:0; }
	@keyframes terminal-blink { 50% { opacity:0; } }
	.command-center { display:grid; grid-template-columns:minmax(250px,1fr) minmax(235px,.88fr) minmax(220px,.82fr); gap:.75rem; align-items:stretch; padding:.75rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-lg); background:linear-gradient(110deg,var(--ui-surface-1),color-mix(in srgb,var(--ui-accent) 4%,var(--ui-surface-1))); box-shadow:var(--shadow-1); }
	.context-copy { display:flex; flex-direction:column; justify-content:center; min-width:0; padding:.35rem .55rem; }.context-heading { display:flex; align-items:center; gap:.75rem; }.context-heading .eyebrow { margin:0; color:var(--ui-text-tertiary); text-transform:uppercase; }.context-copy h2 { margin:.35rem 0 .15rem; font-size:clamp(1.2rem,2vw,1.55rem); }.context-copy>p { margin:0; color:var(--ui-text-secondary); font-size:.8rem; }.context-links { display:flex; flex-wrap:wrap; gap:1rem; margin-top:.75rem; }.context-links a,.section-heading>a { color:var(--ui-link); font-size:.78rem; font-weight:620; text-decoration:none; }.context-links a:hover,.section-heading>a:hover { color:var(--ui-link-hover); }
	.scan-launcher { position:relative; isolation:isolate; display:grid; grid-template-columns:auto 1fr auto; align-items:center; gap:.75rem; min-height:88px; padding:.75rem 1rem; overflow:hidden; border:1px solid color-mix(in srgb,var(--ui-accent) 76%,var(--ui-border-default)); border-radius:var(--radius-md); color:var(--ui-text-primary); text-decoration:none; background:linear-gradient(112deg,color-mix(in srgb,var(--ui-accent) 8%,var(--ui-canvas)),var(--ui-canvas) 52%,color-mix(in srgb,var(--ui-accent) 5%,var(--ui-canvas))); box-shadow:inset 0 1px 0 color-mix(in srgb,var(--ui-accent) 30%,transparent),0 10px 22px rgba(0,0,0,.24); transition:transform var(--duration-normal),border-color var(--duration-normal),box-shadow var(--duration-normal); animation:scan-invitation 5s ease-in-out infinite; }
	.scan-launcher::before { content:''; position:absolute; z-index:0; inset:1px; pointer-events:none; background:linear-gradient(105deg,transparent 18%,color-mix(in srgb,var(--ui-accent) 30%,transparent) 43%,rgba(255,255,255,.16) 49%,transparent 61%); opacity:.12; transform:translateX(-110%); animation:scan-sheen 5s ease-in-out infinite; }.scan-launcher:hover { transform:translateY(-1px); border-color:var(--ui-accent); box-shadow:0 14px 32px rgba(0,0,0,.28),var(--shadow-glow-accent); }.scan-launcher:focus-visible { outline-color:var(--ui-link); }.scan-launcher>* { position:relative; z-index:1; }
	.scan-radar { position:relative; display:grid; place-items:center; width:50px; height:50px; overflow:hidden; border:1px solid color-mix(in srgb,var(--ui-accent) 78%,transparent); border-radius:50%; background:conic-gradient(from 30deg,transparent 0 70%,color-mix(in srgb,var(--ui-accent) 64%,transparent) 91%,transparent),radial-gradient(circle,color-mix(in srgb,var(--ui-accent) 18%,var(--ui-canvas)),var(--ui-canvas) 68%); box-shadow:inset 0 1px 0 color-mix(in srgb,var(--ui-accent) 42%,transparent),0 0 12px color-mix(in srgb,var(--ui-accent) 10%,transparent); animation:scan-radar-invitation 5s ease-in-out infinite; }.scan-radar::before,.scan-radar::after { content:''; position:absolute; border:1px solid color-mix(in srgb,var(--ui-accent) 48%,transparent); border-radius:50%; }.scan-radar::before { inset:10px; }.scan-radar::after { inset:20px; background:var(--ui-accent); }.scan-radar i { position:absolute; width:4px; height:4px; border-radius:50%; background:var(--ui-link-hover); }.scan-radar i:nth-child(1) { top:7px; left:24px; }.scan-radar i:nth-child(2) { right:7px; bottom:13px; }.scan-radar i:nth-child(3) { left:9px; bottom:11px; }
	.scan-copy { display:grid; }.scan-copy small { color:var(--ui-text-secondary); font-size:.72rem; }.scan-copy strong { color:var(--ui-accent-hover); font-size:1.45rem; font-weight:800; letter-spacing:.2em; text-shadow:0 0 12px color-mix(in srgb,var(--ui-accent) 24%,transparent); animation:scan-label-invitation 5s ease-in-out infinite; }.scan-arrow { color:var(--ui-accent-hover); transition:transform var(--duration-normal); }.scan-launcher:hover .scan-arrow { transform:translateX(4px); }
	@keyframes scan-invitation { 0%,68%,100% { border-color:color-mix(in srgb,var(--ui-accent) 70%,var(--ui-border-default)); box-shadow:inset 0 1px 0 color-mix(in srgb,var(--ui-accent) 28%,transparent),0 10px 22px rgba(0,0,0,.24); } 76% { border-color:var(--ui-accent-hover); box-shadow:0 0 0 4px color-mix(in srgb,var(--ui-accent) 18%,transparent),0 12px 28px color-mix(in srgb,var(--ui-accent) 18%,transparent),inset 0 1px 0 color-mix(in srgb,var(--ui-accent-hover) 55%,transparent); } 84% { border-color:color-mix(in srgb,var(--ui-accent) 82%,var(--ui-border-default)); box-shadow:0 0 0 2px color-mix(in srgb,var(--ui-accent) 10%,transparent),0 10px 24px color-mix(in srgb,var(--ui-accent) 10%,transparent),inset 0 1px 0 color-mix(in srgb,var(--ui-accent) 34%,transparent); } } @keyframes scan-sheen { 0% { opacity:.12; transform:translateX(-110%); } 30% { opacity:.2; transform:translateX(-35%); } 58% { opacity:.08; transform:translateX(20%); } 72%,100% { opacity:0; transform:translateX(115%); } } @keyframes scan-radar-invitation { 0%,68%,100% { transform:scale(1); box-shadow:inset 0 1px 0 color-mix(in srgb,var(--ui-accent) 42%,transparent),0 0 12px color-mix(in srgb,var(--ui-accent) 10%,transparent); } 76% { transform:scale(1.045); box-shadow:0 0 17px color-mix(in srgb,var(--ui-accent) 30%,transparent),inset 0 1px 0 color-mix(in srgb,var(--ui-accent-hover) 58%,transparent); } 84% { transform:scale(1.015); box-shadow:0 0 10px color-mix(in srgb,var(--ui-accent) 16%,transparent),inset 0 1px 0 color-mix(in srgb,var(--ui-accent) 44%,transparent); } } @keyframes scan-label-invitation { 0%,68%,100% { letter-spacing:.2em; filter:none; } 76% { letter-spacing:.22em; filter:drop-shadow(0 0 7px color-mix(in srgb,var(--ui-accent) 58%,transparent)); } 84% { letter-spacing:.205em; filter:none; } }
	/* Independent stacks prevent a short alert list from creating a tall, empty
	   card or a dead zone before the next row of useful work. */
	.dashboard-columns { display:grid; grid-template-columns:minmax(480px,1.28fr) minmax(340px,.92fr); gap:1rem; align-items:start; }.dashboard-column { display:grid; align-content:start; gap:1rem; min-width:0; }.timeline-slot { min-width:0; }
	.actions-card { display:grid; grid-template-columns:minmax(190px,.7fr) minmax(280px,1.3fr); gap:1rem; align-items:start; }.actions-card .eyebrow,.section-heading .eyebrow { margin:0; color:var(--ui-accent); text-transform:uppercase; }.actions-card h2,.section-heading h2 { margin:.25rem 0 .45rem; }.actions-card>div>p:last-child { color:var(--ui-text-secondary); font-size:.8rem; line-height:1.5; }.actions-card nav { display:grid; gap:.4rem; }.action-link { display:grid; grid-template-columns:auto 1fr auto; align-items:center; gap:.7rem; padding:.7rem; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-md); color:var(--ui-text-primary); text-decoration:none; background:var(--ui-canvas); }.action-link:hover { border-color:color-mix(in srgb,var(--ui-link) 35%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-link) 5%,var(--ui-canvas)); }.action-link>span { color:var(--ui-text-tertiary); font-family:var(--font-meta); font-size:.67rem; }.action-link strong,.action-link small { display:block; }.action-link strong { font-size:.82rem; }.action-link small { margin-top:.1rem; color:var(--ui-text-tertiary); font-size:.7rem; }.action-link b { color:var(--ui-link); font-weight:500; }
	.scan-automation { grid-column:1/-1; display:grid; gap:.7rem; margin-top:.2rem; padding:.85rem; border:1px solid color-mix(in srgb,var(--ui-accent) 28%,var(--ui-border-subtle)); border-radius:var(--radius-md); background:linear-gradient(135deg,color-mix(in srgb,var(--ui-accent) 7%,var(--ui-material-solid)),var(--ui-material-solid)); box-shadow:inset 0 1px 0 var(--ui-rim-light); }.scan-automation-heading { display:flex; align-items:flex-start; justify-content:space-between; gap:1rem; }.scan-automation-heading h3 { margin:.2rem 0 0; font-size:.95rem; }.automation-status { flex:0 0 auto; padding:.3rem .5rem; border:1px solid var(--ui-border-default); border-radius:999px; color:var(--ui-text-secondary); font:650 .64rem/1 var(--font-meta); }.scan-automation-controls { display:grid; grid-template-columns:auto minmax(150px,220px) 1fr auto minmax(140px,.8fr); align-items:center; gap:.6rem; }.scan-automation-controls>label:first-child { color:var(--ui-text-secondary); font-size:.74rem; }.scan-automation-controls select { min-width:0; padding:.48rem .55rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-sm); background:var(--ui-canvas); color:var(--ui-text-primary); font:inherit; }.automation-actions { display:flex; flex-wrap:wrap; gap:.4rem; }.automation-actions .wf-btn { margin:0; padding:.45rem .6rem; font-size:.68rem; }.automation-feedback { min-width:0; margin:0; overflow:hidden; font-size:.7rem; line-height:1.35; text-overflow:ellipsis; white-space:nowrap; }.automation-feedback.success { color:var(--ui-success); }.automation-feedback.error { color:var(--ui-danger); }
	.dashboard-automation { grid-column:auto; width:auto; max-width:none; box-sizing:border-box; margin:0; padding:.45rem .6rem; grid-template-columns:minmax(180px,.64fr) minmax(0,1.36fr); align-items:center; gap:.45rem .8rem; }.dashboard-automation .scan-automation-heading { align-items:center; min-width:0; }.dashboard-automation .scan-automation-heading h3 { font-size:.76rem; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }.dashboard-automation .scan-automation-controls { min-width:0; grid-template-columns:minmax(0,auto) minmax(84px,104px) minmax(0,1fr) auto minmax(120px,.8fr); gap:.35rem; align-items:center; }.dashboard-automation .scan-automation-controls>label:first-child { align-self:center; font-size:.64rem; }.dashboard-automation .scan-automation-controls select { width:100%; min-width:0; padding:.35rem .4rem; font-size:.68rem; }.dashboard-automation .automation-actions { min-width:0; grid-column:auto; justify-content:flex-end; }.dashboard-automation .automation-actions .wf-btn { max-width:100%; padding:.32rem .4rem; font-size:.58rem; white-space:nowrap; }.dashboard-automation .automation-feedback { min-width:0; grid-column:auto; font-size:.58rem; }
	.follow-up { display:grid; gap:.75rem; }.section-heading { display:flex; align-items:end; justify-content:space-between; gap:1rem; }.section-heading h2 { font-size:1.08rem; }.follow-up-grid { display:grid; grid-template-columns:1fr 1fr; gap:1rem; align-items:start; }.follow-up-grid> :global(*) { min-width:0; }
	@container (min-width:1181px) {
		.command-center { grid-template-columns:minmax(0,1fr) minmax(260px,.8fr); }
		.command-center>.context-copy { grid-column:1; grid-row:1; }
		.command-center>.dashboard-automation { grid-column:1/-1; grid-row:2; }
		.command-center>.scan-launcher { grid-column:2; grid-row:1; }
	}
	@container (max-width:1180px) {
		.command-center { grid-template-columns:minmax(0,1fr) minmax(220px,.8fr); }
		.command-center>.context-copy { grid-column:1; grid-row:1; }
		.command-center>.dashboard-automation { grid-column:1/-1; grid-row:2; }
		.command-center>.scan-launcher { grid-column:2; grid-row:1; }
	}
	@container (max-width:1050px) {
		.dashboard-hero,.dashboard-columns { grid-template-columns:1fr; }
		.actions-card { grid-template-columns:1fr 1.5fr; }
		.dashboard-automation .scan-automation-controls { grid-template-columns:minmax(0,auto) minmax(150px,220px) minmax(0,1fr) auto minmax(120px,.8fr); }
		.dashboard-automation .automation-actions { grid-column:auto; }
		.scan-automation-controls { grid-template-columns:auto minmax(150px,220px); }
		.scan-automation .automation-actions { grid-column:1/-1; }
	}
	@container (max-width:760px) {
		.command-center { grid-template-columns:1fr; }
		.command-center>.context-copy,.command-center>.dashboard-automation,.command-center>.scan-launcher { grid-column:auto; grid-row:auto; }
		.dashboard-automation { grid-template-columns:1fr; }
		.dashboard-automation .scan-automation-controls { grid-template-columns:minmax(0,1fr); }
		.dashboard-automation .automation-actions { grid-column:auto; justify-content:flex-start; }
		.dashboard-automation .scan-automation-controls { grid-template-columns:minmax(0,1fr); }
		.dashboard-automation .automation-feedback { grid-column:auto; }
		.actions-card,.follow-up-grid { grid-template-columns:1fr; }
		.section-heading { align-items:start; flex-direction:column; }
		.scan-launcher { min-height:92px; }
	}
	@media (prefers-reduced-motion:reduce) {
		.terminal-caret { animation:none; }
		.scan-arrow { transition:none; }
		/* Explicitly requested CTA motion remains available; unrelated decorative
		   motion is still reduced by the global accessibility limiter. */
		.scan-launcher { animation:scan-invitation 5s ease-in-out infinite !important; transition-duration:.35s !important; }
		.scan-launcher::before { animation:scan-sheen 5s ease-in-out infinite !important; }
		.scan-copy strong { animation:scan-label-invitation 5s ease-in-out infinite !important; }
		.scan-radar { animation:scan-radar-invitation 5s ease-in-out infinite !important; }
	}

	/* The live public-IP context is an intentional cockpit instrument. Keep it
	   visible: it is real, refreshed locally, and remains assistive-tech readable. */
	.cockpit-view .network-terminal { display:flex; }
	.cockpit-view .dashboard-hero { display:grid; grid-template-columns:minmax(0,1fr) minmax(360px,440px); }
	.cockpit-view .command-center { background:var(--ui-material-panel); border-color:var(--ui-border-subtle); box-shadow:inset 0 1px 0 var(--ui-rim-light),var(--shadow-1); }
	.cockpit-view .scan-launcher { background:var(--ui-material-solid); border-color:color-mix(in srgb,var(--ui-accent) 34%,var(--ui-border-default)); animation:scan-invitation 5s ease-in-out infinite; }
	.cockpit-view .scan-launcher::before { display:block; }
	.cockpit-view .scan-launcher:hover { transform:none; box-shadow:inset 0 1px 0 var(--ui-rim-light),var(--shadow-1); }
	.cockpit-view .scan-radar { background:radial-gradient(circle,var(--ui-accent-soft) 0 18%,transparent 20% 50%,color-mix(in srgb,var(--ui-accent) 30%,transparent) 51% 52%,transparent 53%); animation:scan-radar-invitation 5s ease-in-out infinite; box-shadow:inset 0 0 0 1px color-mix(in srgb,var(--ui-accent) 28%,transparent); }
	.cockpit-view .scan-copy strong { letter-spacing:.14em; animation:scan-label-invitation 5s ease-in-out infinite; }
	.cockpit-view .actions-card,
	.cockpit-view .follow-up-grid :global(.glass-card) { background:var(--ui-material-panel); }
	/* Dashboard material roles: the shell carries translucency, evidence stays deep,
	   and utility actions are deliberately flatter than the primary surfaces. */
	.cockpit-view .situation-slot :global(.glass-card) {
		background:linear-gradient(145deg,color-mix(in srgb,var(--ui-material-panel) 86%,var(--ui-accent) 14%),var(--ui-material-solid));
		border-color:color-mix(in srgb,var(--ui-accent) 24%,var(--ui-border-default));
		box-shadow:inset 0 1px 0 var(--ui-rim-light),inset 0 -18px 34px rgba(0,0,0,.2),var(--shadow-1);
	}
	.cockpit-view .priority-slot :global(.glass-card),
	.cockpit-view .follow-up-grid :global(.glass-card) {
		background:var(--ui-material-solid);
		border-color:var(--ui-border-subtle);
		box-shadow:inset 0 1px 0 var(--ui-rim-shadow),var(--shadow-1);
	}
	.cockpit-view .priority-slot :global(.glass-card)::before,
	.cockpit-view .priority-slot :global(.glass-card)::after,
	.cockpit-view .follow-up-grid :global(.glass-card)::before,
	.cockpit-view .follow-up-grid :global(.glass-card)::after {
		display:none;
	}
	.cockpit-view .timeline-slot :global(.glass-card) {
		background:linear-gradient(160deg,var(--ui-material-panel),color-mix(in srgb,var(--ui-material-solid) 86%,var(--ui-accent) 14%));
		border-color:color-mix(in srgb,var(--ui-link) 18%,var(--ui-border-default));
	}
	.cockpit-view .actions-card {
		align-self:start;
		height:auto;
		background:var(--ui-surface-1);
		border-color:var(--ui-border-subtle);
		box-shadow:none;
	}
	.cockpit-view .actions-card::before,
	.cockpit-view .actions-card::after { display:none; }
	@media (max-width:760px) { .cockpit-view .scan-launcher { min-height:88px; } }
</style>
