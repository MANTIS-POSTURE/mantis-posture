<script lang="ts">
	import '../app.css';
	import Titlebar from '$lib/Titlebar.svelte';
	import NavIcon from '$lib/NavIcon.svelte';
	import { page } from '$app/stores';
	import { onDestroy, onMount } from 'svelte';
	import { getAppLanguage, getAppTheme, getLocalAiStatus, isLocalAiEnabled, listIdentities, setLocalAiPreference, type AppLanguage, type Identity, type LocalAiStatus } from '$lib/api';
	import { activeIdentityId, setActiveIdentityId } from '$lib/active-identity';
	import { applyUiLanguage } from '$lib/i18n';

	let { children } = $props();
	const navGroups = [
		{ label: 'Accueil', items: [
			{ label: 'Tableau de bord', path: '/posture', icon: 'posture' }
		] },
		{ label: 'Observer', items: [
			{ label: 'Identités', path: '/identites', icon: 'identities' },
			{ label: 'Scanner', path: '/veille', icon: 'watch' },
			{ label: 'Connexions', path: '/graphe', icon: 'graph' }
		] },
		{ label: 'Agir', items: [
			{ label: 'Expositions', path: '/expositions', icon: 'exposures' },
			{ label: 'Incidents', path: '/incidents', icon: 'incidents' },
			{ label: 'Actions', path: '/actions', icon: 'actions' },
			{ label: 'Demandes RGPD', path: '/dpo', icon: 'dpo' },
		] },
		{ label: 'Organiser', items: [
			{ label: 'Rapports', path: '/rapports', icon: 'reports' }
		] },
		{ label: 'Système', items: [
			{ label: 'Guides', path: '/guides', icon: 'guides' },
			{ label: 'Paramètres', path: '/parametres', icon: 'settings' }
		] }
	];
	const navGroupsEn = [
		{ label: 'Home', items: [{ label: 'Dashboard', path: '/posture', icon: 'posture' }] },
		{ label: 'Observe', items: [{ label: 'Identities', path: '/identites', icon: 'identities' }, { label: 'Scanner', path: '/veille', icon: 'watch' }, { label: 'Connections', path: '/graphe', icon: 'graph' }] },
		{ label: 'Act', items: [{ label: 'Exposures', path: '/expositions', icon: 'exposures' }, { label: 'Incidents', path: '/incidents', icon: 'incidents' }, { label: 'Actions', path: '/actions', icon: 'actions' }, { label: 'GDPR requests', path: '/dpo', icon: 'dpo' }] },
		{ label: 'Organize', items: [{ label: 'Reports', path: '/rapports', icon: 'reports' }] },
		{ label: 'System', items: [{ label: 'Guides', path: '/guides', icon: 'guides' }, { label: 'Settings', path: '/parametres', icon: 'settings' }] }
	];

	const currentPath = $derived($page.url.pathname);
	function isActive(path: string) { return currentPath === path || currentPath.startsWith(`${path}/`); }
	let language = $state<AppLanguage>('fr');
	const activeNavGroups = $derived(language === 'en' ? navGroupsEn : navGroups);
	const currentLabel = $derived(activeNavGroups.flatMap((group) => group.items).find((item) => isActive(item.path))?.label ?? 'MANTIS');
	const shellCopy = $derived(language === 'en' ? {
		navigation: 'Main navigation', collapse: 'Collapse navigation', expand: 'Expand navigation', local: 'Your data stays here', localDetail: 'Stored on this device',
		guided: 'A guided reading, on your device', guidedText: 'Add local AI to sort and explain your results. You can also continue without it: no data is sent online.', discover: 'Discover local AI', continue: 'Continue for now'
	} : {
		navigation: 'Navigation principale', collapse: 'Réduire la navigation', expand: 'Déployer la navigation', local: 'Vos données restent ici', localDetail: 'Conservées sur cet appareil',
		guided: 'Une lecture guidée, sur votre appareil', guidedText: 'Ajoutez l’IA locale pour mieux trier et expliquer vos résultats. Vous pouvez aussi continuer sans elle : aucune donnée n’est envoyée en ligne.', discover: 'Découvrir l’IA locale', continue: 'Continuer pour l’instant'
	});
	let aiStatus = $state<LocalAiStatus | null>(null);
	let identities = $state<Identity[]>([]);
	let sidebarCollapsed = $state(false);
	let aiRefreshTimer: ReturnType<typeof setInterval> | null = null;
	function stopGlobalAiRefresh() {
		if (aiRefreshTimer) clearInterval(aiRefreshTimer);
		aiRefreshTimer = null;
	}
	async function refreshGlobalAiStatus() {
		try {
			if (!await isLocalAiEnabled()) {
				aiStatus = null;
				stopGlobalAiRefresh();
				return;
			}
			aiStatus = await getLocalAiStatus();
			// La barre globale n'a besoin d'être actualisée en continu que pendant un téléchargement.
			if (aiStatus.active_download_id && !aiRefreshTimer) {
				aiRefreshTimer = setInterval(() => { void refreshGlobalAiStatus(); }, 1500);
			} else if (!aiStatus.active_download_id) {
				stopGlobalAiRefresh();
			}
		} catch { /* L’application reste utilisable sans IA. */ }
	}
	async function loadIdentitySelector() {
		try {
			identities = await listIdentities();
			if (!$activeIdentityId || !identities.some((identity) => identity.id === $activeIdentityId)) {
				setActiveIdentityId(identities.find((identity) => identity.status === 'active')?.id ?? identities[0]?.id ?? null);
			}
		} catch { /* Les écrans restent disponibles si la base est momentanément occupée. */ }
	}
	function selectActiveIdentity(identityId: string | null) {
		setActiveIdentityId(identityId);
	}
	onMount(() => {
		document.documentElement.dataset.theme = 'obsidian';
		sidebarCollapsed = localStorage.getItem('mantis.sidebar.collapsed') === 'true';
		const onLanguageChanged = (event: Event) => { language = (event as CustomEvent<AppLanguage>).detail; applyUiLanguage(language); };
		const onIdentitiesChanged = () => { void loadIdentitySelector(); };
		window.addEventListener('mantis-language-changed', onLanguageChanged);
		window.addEventListener('mantis-identities-changed', onIdentitiesChanged);
		void getAppLanguage().then((value) => { language = value; applyUiLanguage(value); }).catch(() => {});
		void getAppTheme().then((theme) => document.documentElement.dataset.theme = theme).catch(() => {});
		void refreshGlobalAiStatus(); void loadIdentitySelector();
		return () => { window.removeEventListener('mantis-language-changed', onLanguageChanged); window.removeEventListener('mantis-identities-changed', onIdentitiesChanged); };
	});
	onDestroy(stopGlobalAiRefresh);
	async function continueWithoutAi() {
		try { await setLocalAiPreference(false, null, 'sans_ia'); aiStatus = null; } catch { aiStatus = null; }
	}
	function toggleSidebar() {
		sidebarCollapsed = !sidebarCollapsed;
		localStorage.setItem('mantis.sidebar.collapsed', String(sidebarCollapsed));
	}
</script>

<svelte:head><link rel="icon" type="image/png" href="/mantis-logo.png" /><title>MANTIS Posture</title></svelte:head>

{#key language}
<div class="app">
	<Titlebar {identities} activeIdentityId={$activeIdentityId} onIdentityChange={selectActiveIdentity} currentSection={currentLabel} />
	<div class="shell">
		<aside class="sidebar" class:collapsed={sidebarCollapsed}>
			<div class="sidebar-brand"><img class="brand-mark" src="/mantis-logo.png" alt="" /><div class="brand-text"><span class="brand-name">MANTIS</span><span class="brand-sub">POSTURE</span></div><button class="collapse-toggle" type="button" aria-label={sidebarCollapsed ? shellCopy.expand : shellCopy.collapse} aria-expanded={!sidebarCollapsed} onclick={toggleSidebar}><svg viewBox="0 0 20 20" aria-hidden="true"><path d="m12.5 5-5 5 5 5"/></svg></button></div>
			<nav class="sidebar-nav" aria-label={shellCopy.navigation}>
				{#each activeNavGroups as group (group.label)}
					<section class="nav-group">
						<h2>{group.label}</h2>
						<ul>{#each group.items as item (item.path)}<li><a href={item.path} class="nav-item" class:active={isActive(item.path)} aria-current={isActive(item.path) ? 'page' : undefined} title={item.label}><span class="nav-icon"><NavIcon name={item.icon} /></span><span class="nav-label">{item.label}</span></a></li>{/each}</ul>
					</section>
				{/each}
			</nav>
			<div class="sidebar-status"><span class="status-dot"></span><div><strong>{shellCopy.local}</strong><small>{shellCopy.localDetail}</small></div></div>
		</aside>
		<main class="content">
			{#if aiStatus?.onboarding_status === 'a_proposer' && currentPath !== '/parametres'}
				<section class="ai-onboarding" aria-labelledby="ai-onboarding-title"><div><strong id="ai-onboarding-title">{shellCopy.guided}</strong><p>{shellCopy.guidedText}</p></div><div class="ai-onboarding-actions"><a href="/parametres#ia-locale">{shellCopy.discover}</a><button onclick={continueWithoutAi}>{shellCopy.continue}</button></div></section>
			{/if}
			{@render children()}
		</main>
	</div>
</div>
{/key}

<style>
	.app { display:flex; flex-direction:column; height:100vh; overflow:hidden; background:var(--ui-canvas); }
	.shell { display:flex; flex:1; min-height:0; position:relative; }
	.sidebar { position:relative; z-index:2; width:var(--sidebar-width); flex-shrink:0; display:flex; flex-direction:column; gap:1rem; padding:1rem .7rem .75rem; background:var(--ui-material-shell); border-right:1px solid var(--ui-border-subtle); box-shadow:inset -1px 0 0 rgba(231,250,249,.035), 14px 0 38px rgba(0,0,0,.22); overflow-x:hidden; overflow-y:auto; contain:layout paint style; transition:width var(--duration-normal) var(--ease-emphasized); }
	.sidebar.collapsed { width:72px; }
	.sidebar-brand { display:flex; align-items:center; gap:.7rem; min-width:0; padding:0 .45rem; margin-bottom:.1rem; }
	.brand-mark { width:64px; height:64px; flex:0 0 64px; object-fit:contain; }
	.brand-text { display:flex; flex-direction:column; min-width:0; line-height:1.1; }
	.brand-name { font-weight:700; letter-spacing:.08em; font-size:.98rem; }
	.brand-sub { font-size:.67rem; letter-spacing:.28em; color:var(--ui-text-tertiary); }
	.collapse-toggle { display:grid; place-items:center; width:30px; height:30px; margin-left:auto; flex:0 0 auto; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-sm); color:var(--ui-text-tertiary); background:transparent; cursor:pointer; transition:background var(--duration-fast),border-color var(--duration-fast),color var(--duration-fast); }.collapse-toggle:hover { color:var(--ui-accent); border-color:var(--ui-border-default); background:var(--ui-accent-soft); }.collapse-toggle svg { width:15px; height:15px; fill:none; stroke:currentColor; stroke-width:1.7; transition:transform var(--duration-normal) var(--ease-standard); }.collapsed .collapse-toggle svg { transform:rotate(180deg); }
	.sidebar-nav { display:flex; flex-direction:column; gap:1rem; }
	.nav-group h2 { margin:0 0 .45rem .65rem; font-size:.66rem; font-weight:650; letter-spacing:.14em; text-transform:uppercase; color:var(--ui-text-tertiary); }
	.sidebar-nav ul { list-style:none; margin:0; padding:0; display:flex; flex-direction:column; gap:2px; }
	.nav-item { display:flex; align-items:center; gap:.72rem; min-height:40px; padding:.48rem .65rem; border:1px solid transparent; border-radius:var(--radius-sm); font-size:1rem; font-weight:540; color:var(--ui-text-secondary); text-decoration:none; transition:background var(--duration-fast),border-color var(--duration-fast),color var(--duration-fast); }
	.nav-item:hover { background:color-mix(in srgb,var(--ui-surface-2) 78%,transparent); border-color:var(--ui-border-subtle); color:var(--ui-text-primary); }
	.nav-item.active { background:color-mix(in srgb,var(--ui-accent) 9%,var(--ui-surface-2)); border-color:color-mix(in srgb,var(--ui-accent) 24%,var(--ui-border-subtle)); color:var(--ui-text-primary); font-weight:620; box-shadow:inset 2px 0 0 var(--ui-accent); }
	.nav-icon { display:grid; place-items:center; color:var(--ui-text-tertiary); transition:color var(--duration-fast) var(--ease-standard); }.nav-item:hover .nav-icon { color:var(--ui-text-secondary); }.nav-item.active .nav-icon { color:var(--ui-accent); }
	.sidebar-status { margin-top:auto; display:flex; align-items:flex-start; gap:.6rem; padding:.7rem; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-sm); background:color-mix(in srgb,var(--ui-surface-2) 72%,transparent); box-shadow:inset 0 1px 0 var(--ui-rim-light); }
	.status-dot { width:7px; height:7px; margin-top:.3rem; flex:0 0 auto; border-radius:50%; background:var(--ui-success); box-shadow:0 0 6px color-mix(in srgb,var(--ui-success) 36%,transparent); }
	.sidebar-status div { display:grid; gap:.1rem; min-width:0; }.sidebar-status strong { font-size:.75rem; font-weight:650; color:var(--ui-text-secondary); }.sidebar-status small { font-size:.67rem; line-height:1.35; color:var(--ui-text-tertiary); }
	.collapsed .brand-text,.collapsed .nav-label,.collapsed .nav-group h2,.collapsed .sidebar-status div { display:none; }.collapsed .sidebar-brand { padding:0; justify-content:center; }.collapsed .collapse-toggle { position:absolute; top:64px; right:5px; width:21px; height:21px; border-radius:50%; background:var(--ui-surface-2); }.collapsed .nav-group { padding-top:.55rem; border-top:1px solid var(--ui-border-subtle); }.collapsed .nav-item { justify-content:center; padding:.5rem; }.collapsed .nav-icon :global(svg) { width:20px; height:20px; }.collapsed .sidebar-status { justify-content:center; padding:.7rem .4rem; }
	.content { position:relative; z-index:1; flex:1; min-width:0; overflow-y:auto; background:transparent; scrollbar-gutter:stable; }
	.ai-onboarding { margin:1rem var(--content-gutter) 0; padding:.9rem 1rem; display:flex; justify-content:space-between; align-items:center; gap:1rem; border:1px solid color-mix(in srgb,var(--ui-ai) 30%,var(--ui-border-default)); border-radius:var(--radius-md); background:var(--ui-material-overlay); box-shadow:inset 0 1px 0 var(--ui-rim-light),var(--shadow-1); backdrop-filter:blur(var(--blur-overlay)) saturate(118%); -webkit-backdrop-filter:blur(var(--blur-overlay)) saturate(118%); }
	.ai-onboarding p { margin:.25rem 0 0; color:var(--ui-text-secondary); font-size:.82rem; }.ai-onboarding-actions { display:flex; gap:.5rem; flex-shrink:0; }
	.ai-onboarding a,.ai-onboarding button { min-height:36px; padding:.5rem .75rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-sm); background:transparent; color:var(--ui-text-primary); text-decoration:none; font:inherit; cursor:pointer; }.ai-onboarding a { background:var(--ui-ai); color:#090b1c; border-color:var(--ui-ai); font-weight:650; }
	@media (max-width:1100px) { .sidebar { width:72px; }.sidebar .brand-text,.sidebar .nav-label,.sidebar .nav-group h2,.sidebar .sidebar-status div { display:none; }.sidebar .brand-mark { width:52px; height:52px; flex-basis:52px; }.sidebar .sidebar-brand { padding:0; justify-content:center; }.sidebar .collapse-toggle { display:none; }.sidebar .nav-group { padding-top:.55rem; border-top:1px solid var(--ui-border-subtle); }.sidebar .nav-item { justify-content:center; padding:.5rem; }.sidebar .sidebar-status { justify-content:center; padding:.7rem .4rem; } }
	@media (max-width:800px) { .ai-onboarding { align-items:flex-start; flex-direction:column; } }
</style>
