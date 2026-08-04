<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { getAppLanguage, setAppLanguage, type AppLanguage, type Identity } from '$lib/api';
	import { t } from '$lib/i18n';
	import { onMount } from 'svelte';

	let language = $state<AppLanguage>('fr');
	onMount(() => { void getAppLanguage().then((value) => language = value).catch(() => {}); });
	async function changeLanguage(next: AppLanguage) {
		const previous = language;
		language = next;
		window.dispatchEvent(new CustomEvent('mantis-language-changed', { detail: next }));
		try { await setAppLanguage(next); } catch { language = previous; window.dispatchEvent(new CustomEvent('mantis-language-changed', { detail: previous })); }
	}

	let {
		identities = [],
		activeIdentityId = null,
		onIdentityChange,
		currentSection = 'MANTIS'
	}: {
		identities?: Identity[];
		activeIdentityId?: string | null;
		onIdentityChange?: (identityId: string | null) => void;
		currentSection?: string;
	} = $props();

	function minimize() {
		getCurrentWindow().minimize();
	}

	function toggleMaximize() {
		getCurrentWindow().toggleMaximize();
	}

	function closeWindow() {
		getCurrentWindow().close();
	}

	function changeIdentity(event: Event) {
		onIdentityChange?.((event.currentTarget as HTMLSelectElement).value || null);
	}
</script>

<header class="titlebar" data-tauri-drag-region>
	<div class="titlebar-brand">
		<img class="brand-mark" src="/mantis-logo.png" alt="" />
		<span class="brand-name">MANTIS</span>
		<span class="brand-separator" aria-hidden="true"></span>
		<span class="current-section">{currentSection}</span>
	</div>
	<div class="titlebar-identity" data-tauri-drag-region="false">
		<span class="identity-orbit" aria-hidden="true"></span>
		<select aria-label={t('Identité affichée dans MANTIS')} value={activeIdentityId ?? ''} onchange={changeIdentity} disabled={!identities.length}>
			{#if !identities.length}<option value="">{t('Aucune identité')}</option>{/if}
			{#each identities as identity (identity.id)}<option value={identity.id}>{identity.label}{identity.status === 'inactive' ? ' · inactive' : ''}</option>{/each}
		</select>
		<a href="/identites" aria-label={t('Gérer les identités')}>{t('Gérer')}</a>
	</div>
	<div class="titlebar-context"><span class="context-pulse"></span><span>{t('Local et privé')}</span></div>
	<div class="titlebar-language" role="group" aria-label={t('Langue / Language')}>
		<button class:active={language === 'fr'} type="button" onclick={() => changeLanguage('fr')} aria-pressed={language === 'fr'} title="Français"><span class="flag-icon flag-fr" aria-hidden="true"></span><b>FR</b></button>
		<button class:active={language === 'en'} type="button" onclick={() => changeLanguage('en')} aria-pressed={language === 'en'} title="English"><span class="flag-icon flag-en" aria-hidden="true"><svg viewBox="0 0 60 40" preserveAspectRatio="none"><rect width="60" height="40" fill="#012169"/><path d="M0 0h7l53 32v8h-7L0 8zM53 0h7v8L7 40H0v-8z" fill="#fff"/><path d="M0 0h3l57 35v5h-3L0 5zM57 0h3v5L3 40H0v-5z" fill="#c8102e"/><path d="M25 0h10v40H25zM0 15h60v10H0z" fill="#fff"/><path d="M27 0h6v40h-6zM0 17h60v6H0z" fill="#c8102e"/></svg></span><b>EN</b></button>
	</div>
	<div class="titlebar-controls">
		<button class="ctrl-btn minimize" onclick={minimize} aria-label={t('Réduire')} title={t('Réduire')}>
			<svg viewBox="0 0 12 12" width="12" height="12"><rect x="1" y="5.5" width="10" height="1" fill="currentColor"/></svg>
		</button>
		<button class="ctrl-btn maximize" onclick={toggleMaximize} aria-label={t('Agrandir')} title={t('Agrandir')}>
			<svg viewBox="0 0 12 12" width="12" height="12"><rect x="1.5" y="1.5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
		</button>
		<button class="ctrl-btn close" onclick={closeWindow} aria-label={t('Fermer')} title={t('Fermer')}>
			<svg viewBox="0 0 12 12" width="12" height="12"><path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.2"/></svg>
		</button>
	</div>
</header>

<style>
	.titlebar {
		display: grid;
		grid-template-columns: minmax(150px,1fr) auto minmax(110px,1fr) auto auto;
		align-items: center;
		height: var(--titlebar-height);
		min-height: var(--titlebar-height);
		padding: 0 .8rem 0 .95rem;
		background: var(--ui-material-shell);
		border-bottom: 1px solid var(--ui-border-subtle);
		-webkit-app-region: drag;
		user-select: none;
		box-shadow: inset 0 -1px 0 rgba(231,250,249,.035);
	}
	.titlebar-language { display:flex; align-items:center; gap:3px; margin:0 .45rem; padding:2px; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-sm); background:var(--ui-material-solid); -webkit-app-region:no-drag; }
	.titlebar-language button { display:flex; align-items:center; gap:4px; min-height:26px; padding:3px 5px; border:1px solid transparent; border-radius:4px; color:var(--ui-text-tertiary); background:transparent; cursor:pointer; transition:background var(--duration-fast),border-color var(--duration-fast),color var(--duration-fast); }
	.titlebar-language button:hover,.titlebar-language button.active { border-color:var(--ui-border-default); color:var(--ui-text-primary); background:var(--ui-surface-2); }
	.titlebar-language b { font:700 .58rem/1 var(--font-meta); letter-spacing:.04em; }
	.flag-icon { display:block; width:19px; height:13px; flex:0 0 auto; border-radius:2px; box-shadow:0 0 0 1px rgba(255,255,255,.2); }
	.flag-fr { background:linear-gradient(90deg,#1d3f8f 0 33.33%,#f5f5f2 33.33% 66.66%,#e2545b 66.66%); }
	.flag-en { overflow:hidden; background:#012169; }.flag-en svg { display:block; width:100%; height:100%; }

	.titlebar-brand {
		display: flex;
		align-items: center;
		min-width: 0;
		gap: 0.5rem;
	}

	.brand-mark {
		display: block;
		width: 44px;
		height: 44px;
		object-fit: contain;
	}

	.brand-name {
		font-size: 0.78rem;
		font-weight: 700;
		letter-spacing: 0.1em;
	}
	.brand-separator { width:1px; height:14px; background:var(--ui-border-strong); }
	.current-section { min-width:0; overflow:hidden; color:var(--ui-text-secondary); font-size:.76rem; font-weight:550; text-overflow:ellipsis; white-space:nowrap; }

	.titlebar-context {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		justify-self:end;
		font-size: 0.72rem;
		color: var(--ui-text-tertiary);
		letter-spacing: 0.04em;
		white-space: nowrap;
	}

	.titlebar-identity { display:flex; align-items:center; gap:.48rem; min-width:0; -webkit-app-region:no-drag; }.identity-orbit { width:8px; height:8px; flex:0 0 auto; border:2px solid var(--ui-accent); border-radius:50%; }.titlebar-identity select { width:clamp(175px,25vw,350px); min-width:0; min-height:30px; padding:.3rem 1.8rem .3rem .62rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-sm); color:var(--ui-text-primary); background:var(--ui-material-solid); font:inherit; font-size:.8rem; outline:none; transition:border-color var(--duration-fast),box-shadow var(--duration-fast),background var(--duration-fast); }.titlebar-identity select:hover { border-color:var(--ui-border-strong); background:var(--ui-surface-2); }.titlebar-identity select:focus { border-color:var(--ui-accent); box-shadow:0 0 0 3px var(--ui-focus-ring); }.titlebar-identity a { color:var(--ui-link); font-size:.74rem; font-weight:650; text-decoration:none; }.titlebar-identity a:hover { color:var(--ui-link-hover); }

	.context-pulse { width:6px; height:6px; border-radius:50%; background:var(--ui-success); box-shadow:0 0 6px color-mix(in srgb,var(--ui-success) 36%,transparent); }

	.titlebar-controls {
		display: flex;
		gap: 2px;
		margin-left:.5rem;
		-webkit-app-region: no-drag;
	}

	.ctrl-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 28px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--ui-text-tertiary);
		cursor: pointer;
		transition: background var(--duration-fast), color var(--duration-fast);
	}

	.ctrl-btn:hover {
		background: var(--ui-surface-2);
		color: var(--ui-text-primary);
	}

	.ctrl-btn.close:hover {
		background: color-mix(in srgb,var(--ui-danger) 82%,#180b0b);
		color: #fff;
	}
	@media (max-width:900px) { .titlebar { grid-template-columns:minmax(120px,1fr) auto auto auto; }.titlebar-context { display:none; }.titlebar-identity a { display:none; }.titlebar-identity select { width:clamp(130px,34vw,240px); } }
	@media (max-width:620px) { .brand-separator,.current-section,.identity-orbit { display:none; }.titlebar { grid-template-columns:1fr auto auto; }.titlebar-language { margin:0 .15rem; }.titlebar-language button { padding:3px; }.titlebar-language b { display:none; }.titlebar-identity select { width:130px; } }
</style>
