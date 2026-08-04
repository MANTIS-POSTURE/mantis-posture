<script lang="ts">
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import ReadOnlyField from '$lib/ReadOnlyField.svelte';
	import StatePanel from '$lib/components/StatePanel.svelte';
	import ProgressStatus from '$lib/components/ProgressStatus.svelte';
	import PageToolbar from '$lib/components/PageToolbar.svelte';
	import { appPrinciples } from '$lib/mock/posture';
	import { t } from '$lib/i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { onDestroy, onMount } from 'svelte';
	import { diagnoseLocalAi, getAppLanguage, getAppTheme, getLocalAiAnalysisMode, getLocalAiStatus, pauseLocalAiDownload, removeLocalAiComponents, setAppLanguage, setAppTheme, setLocalAiAnalysisMode, setLocalAiPreference, startLocalAiSetup, type AppLanguage, type AppTheme, type LocalAiStatus } from '$lib/api';

	let aiStatus = $state<LocalAiStatus | null>(null);
	let aiBusy = $state(false);
	let aiNotice = $state<string | null>(null);
	let aiError = $state<string | null>(null);
	let selectedModelId = $state('');
	let pollTimer: ReturnType<typeof setInterval> | null = null;
	let downloadWasActive = false;
	let analysisMode = $state<'automatic' | 'manual'>('automatic');
	let clearingData = $state(false);
	let clearingScanResults = $state(false);
	let dataNotice = $state<string | null>(null);
	let dataError = $state<string | null>(null);
	let appTheme = $state<AppTheme>('obsidian');
	let appLanguage = $state<AppLanguage>('fr');
	const themes: { id: AppTheme; name: string; description: string; swatches: string[] }[] = [
		{ id: 'obsidian', name: 'AMOLED Black', description: 'Noir profond, gris argent, vert positif et rouge alerte.', swatches: ['#000000', '#c6ccd4', '#63e39a', '#ff6f73'] },
		{ id: 'arctic', name: 'Arctique', description: 'Obsidienne froide, argent et contraste net.', swatches: ['#000000', '#c7ced1', '#f3f6f7'] },
		{ id: 'ember', name: 'Embraise', description: 'Obsidienne chaude, argent et alertes rouges.', swatches: ['#000000', '#aeb6ba', '#df7373'] },
		{ id: 'verdant', name: 'Verdoyant', description: 'Obsidienne profonde, argent et états verts.', swatches: ['#000000', '#c7ced1', '#78d9a6'] },
		{ id: 'cyber', name: 'Cybersécurité', description: 'AMOLED, argent de contrôle et gris de signal.', swatches: ['#000000', '#f3f6f7', '#899895'] },
		{ id: 'cyberpunk', name: 'Cyberpunk', description: 'AMOLED, argent sculpté et alertes rouges.', swatches: ['#000000', '#c7ced1', '#df7373'] },
		{ id: 'osint', name: 'OSINT', description: 'AMOLED, argent de lecture et états verts.', swatches: ['#000000', '#c7ced1', '#78d9a6'] }
	];

	function stopAiPolling() {
		if (pollTimer) clearInterval(pollTimer);
		pollTimer = null;
	}
	function updateAiPolling(downloadActive: boolean) {
		if (downloadActive && !pollTimer) {
			pollTimer = setInterval(() => { void loadAiStatus(); }, 1500);
		} else if (!downloadActive) {
			stopAiPolling();
		}
	}
	onMount(() => {
		void loadAiStatus();
		getLocalAiAnalysisMode().then((mode) => analysisMode = mode).catch(() => {});
		getAppTheme().then((theme) => { appTheme = theme; document.documentElement.dataset.theme = theme; }).catch(() => {});
		getAppLanguage().then((language) => { appLanguage = language; document.documentElement.lang = language; }).catch(() => {});
	});
	onDestroy(stopAiPolling);

	async function loadAiStatus() {
		try {
			const nextStatus = await getLocalAiStatus();
			if (downloadWasActive && !nextStatus.active_download_id && nextStatus.enabled && nextStatus.models.some((model) => model.component_id === nextStatus.selected_model_id && model.installed)) {
				aiNotice = 'Installation terminée : l’IA locale est opérationnelle.';
			}
			downloadWasActive = Boolean(nextStatus.active_download_id);
			aiStatus = nextStatus;
			updateAiPolling(downloadWasActive);
			if (!selectedModelId) selectedModelId = aiStatus.selected_model_id ?? aiStatus.recommended_model_id;
		}
		catch (error) { aiError = String(error); }
	}

	async function installAiFeatures() {
		aiBusy = true; aiError = null; aiNotice = null;
		try { await startLocalAiSetup(selectedModelId); aiNotice = 'Téléchargement démarré. Vous pouvez continuer à utiliser MANTIS.'; await loadAiStatus(); }
		catch (error) { aiError = String(error); await loadAiStatus(); }
		finally { aiBusy = false; }
	}

	async function pauseAi() {
		if (!aiStatus?.active_download_id) return;
		try { aiNotice = await pauseLocalAiDownload(aiStatus.active_download_id); await loadAiStatus(); } catch (error) { aiError = String(error); }
	}

	async function toggleAi(enabled: boolean) {
		try { aiNotice = await setLocalAiPreference(enabled, aiStatus?.selected_model_id ?? null, enabled ? 'configure' : 'sans_ia'); await loadAiStatus(); }
		catch (error) { aiError = String(error); }
	}
	async function changeAnalysisMode(mode: 'automatic' | 'manual') {
		try { aiNotice = await setLocalAiAnalysisMode(mode); analysisMode = mode; }
		catch (error) { aiError = String(error); }
	}
	async function changeTheme(theme: AppTheme) {
		const previous = appTheme;
		appTheme = theme;
		document.documentElement.dataset.theme = theme;
		try { await setAppTheme(theme); }
		catch (error) { appTheme = previous; document.documentElement.dataset.theme = previous; dataError = `Le thème n’a pas pu être enregistré : ${String(error)}`; }
	}
	async function changeLanguage(language: AppLanguage) {
		const previous = appLanguage;
		appLanguage = language;
		document.documentElement.lang = language;
		window.dispatchEvent(new CustomEvent('mantis-language-changed', { detail: language }));
		try { await setAppLanguage(language); }
		catch (error) { appLanguage = previous; document.documentElement.lang = previous; window.dispatchEvent(new CustomEvent('mantis-language-changed', { detail: previous })); dataError = `Language preference could not be saved: ${String(error)}`; }
	}

	async function runAiDiagnostic() {
		aiBusy = true; aiError = null;
		try { aiNotice = await diagnoseLocalAi(); await loadAiStatus(); }
		catch (error) { aiError = String(error); }
		finally { aiBusy = false; }
	}

	async function removeAiRuntime() {
		if (!confirm('Supprimer le runtime, les modèles et les téléchargements IA locaux ? Les fonctions normales de MANTIS resteront disponibles.')) return;
		aiBusy = true; aiError = null;
		try { aiNotice = await removeLocalAiComponents(); await loadAiStatus(); }
		catch (error) { aiError = String(error); }
		finally { aiBusy = false; }
	}

	function formatBytes(value: number) { return value >= 1_073_741_824 ? `${(value / 1_073_741_824).toFixed(2)} Go` : value > 0 ? `${(value / 1_048_576).toFixed(0)} Mo` : '0 Mo'; }
	function recommendedLabel(status: LocalAiStatus) { return status.models.find((model) => model.component_id === status.recommended_model_id)?.label ?? 'profil léger'; }
	function aiIsReady(status: LocalAiStatus) { return status.enabled && status.installed && status.models.some((model) => model.component_id === status.selected_model_id && model.installed); }
	function selectedModelLabel(status: LocalAiStatus) { return status.models.find((model) => model.component_id === status.selected_model_id)?.label ?? 'le modèle sélectionné'; }

	async function clearUserScanData() {
		if (!confirm('Remettre MANTIS dans l’état d’un nouvel utilisateur ?\n\nCette action efface définitivement les dossiers, identités, résultats et preuves de scans, analyses associées, expositions, incidents, actions, demandes RGPD et chronologie.\n\nLes modules OSINT, leurs installations, les réglages et l’IA locale seront conservés.')) return;
		clearingData = true; dataNotice = null; dataError = null;
		try {
			dataNotice = await invoke<string>('clear_user_scan_data');
		} catch (err) {
			dataError = 'La remise à zéro a échoué : ' + String(err);
		} finally { clearingData = false; }
	}

	async function clearScanResults() {
		if (!confirm('Effacer uniquement les résultats des scans Veille ?\n\nLes fuites, expositions, incidents, actions et preuves issus des scans seront supprimés. Vos dossiers, identités et valeurs saisies seront conservés.')) return;
		clearingScanResults = true; dataNotice = null; dataError = null;
		try {
			dataNotice = await invoke<string>('clear_scan_results');
		} catch (err) {
			dataError = 'Le nettoyage des résultats a échoué : ' + String(err);
		} finally { clearingScanResults = false; }
	}
</script>

<section class="wf-view">
	<GuideHeader
		title="Paramètres"
		question="Comment MANTIS protège mes données, et que puis-je choisir ?"
		intro="Retrouvez ici les choix qui concernent votre confidentialité, l’IA locale, la veille et vos données enregistrées."
	/>
	<PageToolbar label="Sections des paramètres"><nav class="settings-nav" aria-label="Accès rapide"><a href="#garanties">Garanties</a><a href="#application">Application</a><a href="#ia-locale">IA locale</a><a href="#donnees">Données locales</a></nav><span class="local-badge">Local et privé</span></PageToolbar>

	<div class="wf-panel settings-panel" id="garanties">
		<p class="eyebrow">Principes</p>
		<h2>Vos garanties</h2>
		<ul class="principles">
			{#each appPrinciples as p (p.title)}
				<li>
					<span class="wf-title">{t(p.title)}</span>
					<p class="wf-desc">{t(p.text)}</p>
				</li>
			{/each}
		</ul>
	</div>

	<div class="wf-panel settings-panel" id="application">
		<p class="eyebrow">Système</p>
		<h2>Application</h2>
		<section class="theme-picker" aria-labelledby="theme-title"><div><strong id="theme-title">Palette de l’application</strong><p>Choisissez une ambiance. Le choix est enregistré uniquement sur cet appareil.</p></div><div class="theme-options">{#each themes as theme (theme.id)}<button class:active={appTheme === theme.id} class={`theme-option theme-${theme.id}`} onclick={() => changeTheme(theme.id)} aria-pressed={appTheme === theme.id}><span class="theme-swatches">{#each theme.swatches as swatch}<i style={`--swatch:${swatch}`}></i>{/each}</span><span><strong>{theme.name}</strong><small>{theme.description}</small></span>{#if appTheme === theme.id}<b>✓</b>{/if}</button>{/each}</div></section>
		<section class="language-picker" aria-labelledby="language-title"><div><strong id="language-title">Langue / Language</strong><p>Change la langue de l’interface. Le choix est enregistré localement.</p></div><div class="language-options" role="group" aria-label="Application language"><button class:active={appLanguage === 'fr'} onclick={() => changeLanguage('fr')} aria-pressed={appLanguage === 'fr'} title="Français"><span class="flag-icon flag-fr" aria-hidden="true"></span><strong>FR</strong></button><button class:active={appLanguage === 'en'} onclick={() => changeLanguage('en')} aria-pressed={appLanguage === 'en'} title="English"><span class="flag-icon flag-en" aria-hidden="true"><svg viewBox="0 0 60 40" preserveAspectRatio="none"><rect width="60" height="40" fill="#012169"/><path d="M0 0h7l53 32v8h-7L0 8zM53 0h7v8L7 40H0v-8z" fill="#fff"/><path d="M0 0h3l57 35v5h-3L0 5zM57 0h3v5L3 40H0v-5z" fill="#c8102e"/><path d="M25 0h10v40H25zM0 15h60v10H0z" fill="#fff"/><path d="M27 0h6v40h-6zM0 17h60v6H0z" fill="#c8102e"/></svg></span><strong>EN</strong></button></div></section>
		<ReadOnlyField label="Version" value="0.1.0 — fondations IA locale" />
		<ReadOnlyField
			label="Où sont conservées vos données"
			value="Vos données et preuves restent dans l’espace privé de MANTIS sur cet appareil."
			hint="Les migrations sont additives et une sauvegarde est créée avant une montée de version."
		/>
		<ReadOnlyField
			label="Télémétrie"
			value="Désactivée — aucune collecte d'usage."
			hint="Ce réglage restera non négociable."
		/>
		<ReadOnlyField
			label="Recherche publique et réseau"
			value="Les vérifications autorisées peuvent consulter des sources publiques. L’IA locale ne navigue jamais seule sur Internet."
		/>
	</div>

	<div class="wf-panel settings-panel ai-panel" id="ia-locale">
		<p class="eyebrow ai">Intelligence locale</p>
		<h2>IA locale</h2>
		<p class="wf-desc">Facultative, sans compte ni clé. Elle reste sur cet appareil pour trier, expliquer et mettre vos résultats en contexte.</p>
		<StatePanel compact tone="warning" title="Fonction expérimentale — non fiable aujourd’hui" message="L’IA locale peut être lente ou échouer. Les scans et le classement déterministe de MANTIS restent la référence ; n’activez cette option que pour tester une synthèse locale." live="polite" />
		{#if aiStatus}
			{#if aiIsReady(aiStatus)}
				<StatePanel tone="success" title="IA locale prête" message={`${selectedModelLabel(aiStatus)} est installé et vérifié sur cet appareil.`} live="polite" />
			{/if}
			<div class="ai-status-grid">
				<ReadOnlyField label="État" value={aiStatus.enabled ? 'Activées localement' : 'Désactivées — MANTIS fonctionne normalement'} />
				<ReadOnlyField label="Moteur IA" value={`Version ${aiStatus.version} · ${aiStatus.platform} ${aiStatus.architecture}`} />
				<ReadOnlyField label="Mémoire détectée" value={formatBytes(aiStatus.total_memory_bytes)} hint={`Choix automatique : ${recommendedLabel(aiStatus)}.`} />
				<ReadOnlyField label="Espace disponible" value={formatBytes(aiStatus.available_disk_bytes)} hint="MANTIS vérifie l’espace avant le téléchargement." />
			</div>
			<details class="technical-details"><summary>Diagnostic technique</summary><p class:ai-ok={aiStatus.integrity_ok} class="ai-diagnostic">{aiStatus.diagnostic}</p></details>
			<fieldset class="model-picker" disabled={Boolean(aiStatus.active_download_id)}>
				<legend>Choisir un modèle prévalidé</legend>
				{#each aiStatus.models as model (model.component_id)}
					<label class:selected={selectedModelId === model.component_id}>
						<input type="radio" bind:group={selectedModelId} value={model.component_id} />
						<span><strong>{t(model.label)}</strong> {model.component_id === aiStatus.recommended_model_id ? `· ${t('choix automatique')}` : ''}<small>{t(`${formatBytes(model.expected_bytes)} · ${model.license} · ${model.min_ram_gb} Go RAM minimum · ${model.installed ? 'installé' : model.compatible ? 'compatible' : 'mémoire limitée'}`)}</small></span>
					</label>
				{/each}
			</fieldset>
			{@const selectedModel = aiStatus.models.find((model) => model.component_id === selectedModelId)}
			{#if selectedModel}
				<ProgressStatus tone="ai" value={selectedModel.downloaded_bytes} max={selectedModel.expected_bytes} label={selectedModel.status === 'telechargement' ? 'Téléchargement du modèle' : selectedModel.installed ? 'Modèle installé' : 'Préparation du modèle'} detail={`${formatBytes(selectedModel.downloaded_bytes)} / ${formatBytes(selectedModel.expected_bytes)} · ${selectedModel.status === 'telechargement' ? 'téléchargement en cours' : 'la reprise conserve le fichier partiel'}`} />
				{#if selectedModel.status === 'erreur'}<StatePanel compact tone="danger" title="Téléchargement interrompu" message={selectedModel.diagnostic} live="assertive" />{/if}
			{/if}
		{/if}
		{#if aiNotice}<StatePanel compact tone="ai" title="Fonctions IA" message={aiNotice} live="polite" />{/if}
		{#if aiError}<StatePanel compact tone="danger" title="Fonctions IA indisponibles" message={aiError} live="assertive" />{/if}
		<div class="ai-actions">
			<button class="wf-btn primary" onclick={installAiFeatures} disabled={aiBusy || Boolean(aiStatus?.active_download_id) || Boolean(aiStatus?.models.find((model) => model.component_id === selectedModelId)?.installed)}>{aiStatus?.models.find((model) => model.component_id === selectedModelId)?.installed ? 'Modèle à jour' : 'Télécharger les fonctions IA'}</button>
			{#if aiStatus?.active_download_id}<button class="wf-btn" onclick={pauseAi}>Mettre en pause</button>{/if}
			{#if aiStatus?.enabled}<button class="wf-btn" onclick={() => toggleAi(false)}>Désactiver</button>{:else if aiStatus?.models.some((model) => model.installed)}<button class="wf-btn" onclick={() => toggleAi(true)}>Réactiver</button>{/if}
			<button class="wf-btn" onclick={runAiDiagnostic} disabled={aiBusy}>Vérifier l’installation</button>
			{#if aiStatus?.installed || aiStatus?.models.some((model) => model.installed || model.downloaded_bytes > 0)}<button class="wf-btn wf-btn--danger" onclick={removeAiRuntime} disabled={aiBusy}>Supprimer les fonctions IA</button>{/if}
		</div>
		{#if aiStatus?.models.some((model) => model.installed)}
			<div class="analysis-mode"><div><strong>Après un scan manuel</strong><p>Les routines lancent toujours l’analyse locale automatiquement. Pour un scan lancé par vous, choisissez si elle démarre immédiatement ou attend votre clic.</p></div><select aria-label="Déclenchement de l’analyse après un scan manuel" value={analysisMode} onchange={(event) => changeAnalysisMode((event.currentTarget as HTMLSelectElement).value as 'automatic' | 'manual')}><option value="automatic">Automatique — recommandé</option><option value="manual">Manuel — afficher le bouton</option></select></div>
		{/if}
		<p class="wf-hint">MANTIS utilise uniquement des composants IA vérifiés. Vous n’avez aucun fichier ni réglage technique à gérer.</p>
	</div>

	<div class="wf-panel settings-panel data-panel" id="donnees">
		<p class="eyebrow danger">Données locales</p>
		<h2>Mes données locales</h2>
		<ReadOnlyField
			label="Emplacement"
			value="%APPDATA%/mantis/mantis.db (Windows) / ~/.local/share/mantis/mantis.db (Linux)"
			hint="Fichier unique, portable, chiffrable par l'utilisateur."
		/>
		<ReadOnlyField
			label="Mise à jour des données"
			value="Les améliorations sont appliquées automatiquement au démarrage, sans supprimer vos données."
		/>
		<div class="wf-actions">
			<button class="wf-btn wf-btn--warning" onclick={clearScanResults} disabled={clearingScanResults || clearingData}>
				{clearingScanResults ? 'Nettoyage des résultats…' : 'Effacer les résultats des scans (garder mes identités)'}
			</button>
			<span class="wf-hint">Supprime les fuites, expositions, incidents, actions, preuves et rapports issus de Veille. Les dossiers et identités restent en place.</span>
			<button class="wf-btn wf-btn--danger" onclick={clearUserScanData} disabled={clearingData}>
				{clearingData ? 'Remise à zéro…' : 'Effacer les données et repartir à zéro'}
			</button>
			<span class="wf-hint">Supprime dossiers, scans, preuves, analyses, brouillons et fichiers exportés. Les modules OSINT, leurs installations, les réglages et l’IA locale sont conservés.</span>
			{#if dataNotice}<StatePanel compact tone="success" title="Remise à zéro terminée" message={dataNotice} live="polite" />{/if}
			{#if dataError}<StatePanel compact tone="danger" title="Remise à zéro impossible" message={dataError} live="assertive" />{/if}
		</div>
	</div>

	<NextStepBar
		hint="Rien à configurer pour commencer : suivez le Centre de posture."
		primaryHref="/posture"
		primaryLabel="Ouvrir mon tableau de bord"
	>
		<a class="wf-btn" href="/guides">Lire un guide</a>
	</NextStepBar>
</section>

<style>
	.settings-nav{display:flex;flex-wrap:wrap;gap:.3rem}.settings-nav a{padding:.42rem .62rem;border-radius:var(--radius-sm);color:var(--ui-text-secondary);font-size:.76rem;text-decoration:none}.settings-nav a:hover{color:var(--ui-text-primary);background:var(--ui-surface-2)}.local-badge{color:var(--ui-success);font:700 .68rem/1 var(--font-meta);text-transform:uppercase;letter-spacing:.07em}.settings-panel{scroll-margin-top:5rem}.settings-panel>.eyebrow{margin:0 0 .3rem;color:var(--ui-link);font:700 .65rem/1.2 var(--font-meta);letter-spacing:.1em;text-transform:uppercase}.settings-panel>.eyebrow.ai{color:var(--ui-ai)}.settings-panel>.eyebrow.danger{color:var(--ui-danger)}.ai-panel{border-top-color:color-mix(in srgb,var(--ui-ai) 52%,var(--ui-border-default))}.data-panel{border-top-color:color-mix(in srgb,var(--ui-danger) 38%,var(--ui-border-default))}.technical-details{margin-top:.8rem;padding:.7rem .8rem;border:1px solid var(--ui-border-default);border-radius:var(--radius-sm);background:var(--ui-surface-2)}.technical-details summary{cursor:pointer;color:var(--ui-text-secondary);font-size:.76rem;font-weight:650}.technical-details .ai-diagnostic{margin:.7rem 0 0}.theme-picker{margin:1rem 0;padding:.85rem;border:1px solid var(--ui-border-default);border-radius:var(--radius-md);background:rgba(255,255,255,.025)}.theme-picker>div:first-child{display:flex;justify-content:space-between;gap:1rem;align-items:baseline}.theme-picker p{margin:.2rem 0 .75rem;color:var(--ui-text-secondary);font-size:.76rem}.theme-options{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:.5rem}.theme-option{display:grid;grid-template-columns:auto 1fr auto;gap:.55rem;align-items:center;min-width:0;padding:.62rem;border:1px solid var(--ui-border-default);border-radius:9px;color:var(--ui-text);background:rgba(0,0,0,.15);text-align:left;cursor:pointer;transition:border-color var(--duration-fast),background var(--duration-fast),transform var(--duration-fast)}.theme-option:hover{transform:translateY(-1px);border-color:var(--ui-border-strong);background:rgba(255,255,255,.05)}.theme-option.active{border-color:var(--theme-accent,var(--ui-accent));box-shadow:inset 0 0 0 1px color-mix(in srgb,var(--theme-accent,var(--ui-accent)) 28%,transparent)}.theme-option>span:last-of-type{display:grid;gap:.08rem;min-width:0}.theme-option small{overflow:hidden;color:var(--ui-text-tertiary);font-size:.65rem;line-height:1.35;text-overflow:ellipsis;white-space:nowrap}.theme-option b{color:var(--theme-accent,var(--ui-accent));font-size:.74rem}.theme-swatches{display:flex;gap:3px}.theme-swatches i{width:10px;height:26px;border-radius:999px;background:var(--swatch);box-shadow:inset 0 0 0 1px rgba(255,255,255,.14)}.theme-obsidian{--theme-accent:#d8bc82}.theme-arctic{--theme-accent:#69c9e8}.theme-ember{--theme-accent:#e29a62}.theme-verdant{--theme-accent:#75d6ad}.theme-cyber{--theme-accent:#31d7f4}.theme-cyberpunk{--theme-accent:#f05acb}.theme-osint{--theme-accent:#65df92}@media(max-width:620px){.theme-options{grid-template-columns:1fr}.theme-picker>div:first-child{display:block}}
	.language-picker{display:grid;grid-template-columns:1fr auto;gap:1rem;align-items:center;margin:1rem 0;padding:.85rem;border:1px solid var(--ui-border-default);border-radius:var(--radius-md);background:rgba(255,255,255,.025)}.language-picker p{margin:.2rem 0 0;color:var(--ui-text-secondary);font-size:.76rem}.language-options{display:flex;gap:.5rem}.language-options button{display:flex;align-items:center;gap:.55rem;min-width:82px;min-height:64px;padding:.58rem .72rem;border:1px solid var(--ui-border-default);border-radius:var(--radius-sm);color:var(--ui-text-secondary);background:rgba(0,0,0,.24);cursor:pointer;transition:border-color var(--duration-fast),background var(--duration-fast),transform var(--duration-fast)}.language-options button:hover,.language-options button.active{border-color:var(--ui-accent);color:var(--ui-text-primary);background:var(--ui-accent-soft);transform:translateY(-1px)}.language-options strong{font:700 .72rem/1 var(--font-meta)}.flag-icon{display:block;width:36px;height:24px;flex:0 0 auto;border-radius:4px;box-shadow:0 0 0 1px rgba(255,255,255,.22),0 2px 8px rgba(0,0,0,.35)}.flag-fr{background:linear-gradient(90deg,#1d3f8f 0 33.33%,#f5f5f2 33.33% 66.66%,#e2545b 66.66%)}.flag-en{overflow:hidden;background:#012169}.flag-en svg{display:block;width:100%;height:100%}
	@media(max-width:620px){.language-picker{grid-template-columns:1fr}.language-options{justify-content:flex-start}}
	.principles {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.principles li {
		padding: 0.75rem 0.9rem;
		border-radius: 8px;
		border: 1px solid var(--mantis-border);
		background: var(--mantis-bg);
	}

	.wf-actions {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.wf-btn--danger {
		background: var(--mantis-danger-bg, #3a1a1a);
		border: 1px solid var(--mantis-danger-border, #8b2a2a);
		color: var(--mantis-danger-text, #ff6b6b);
	}

	.wf-btn--danger:hover {
		background: var(--mantis-danger-hover, #4a1f1f);
	}

	.wf-btn--warning {
		background: color-mix(in srgb, var(--mantis-warn, #f59e0b) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--mantis-warn, #f59e0b) 45%, var(--mantis-border));
		color: var(--mantis-warn, #fbbf24);
	}

	.wf-hint {
		font-size: 0.8rem;
		color: var(--mantis-text-muted);
		line-height: 1.4;
	}

	.ai-status-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:.75rem; margin-top:1rem; }
	.ai-actions { display:flex; flex-wrap:wrap; gap:.5rem; margin-top:.75rem; }
	.ai-diagnostic { padding:.75rem; border:1px solid var(--mantis-border); border-radius:7px; color:var(--mantis-text-muted); }
	.ai-diagnostic.ai-ok { border-color:color-mix(in srgb,var(--mantis-ok) 45%,var(--mantis-border)); color:var(--mantis-ok); }
	.model-picker { border:0; padding:0; margin:1rem 0; display:grid; gap:.55rem; }
	.model-picker legend { font-weight:650; margin-bottom:.5rem; }
	.model-picker label { display:flex; gap:.7rem; align-items:flex-start; padding:.8rem; border:1px solid var(--mantis-border); border-radius:8px; cursor:pointer; }
	.model-picker label.selected { border-color:var(--mantis-accent); background:color-mix(in srgb,var(--mantis-accent) 8%,transparent); }
	.model-picker small { display:block; margin-top:.25rem; color:var(--mantis-text-muted); }
	.analysis-mode { display:grid; grid-template-columns:1fr minmax(220px,auto); gap:1rem; align-items:center; margin-top:1rem; padding: .85rem; border:1px solid var(--mantis-border); border-radius:8px; } .analysis-mode p { margin:.25rem 0 0; color:var(--mantis-text-muted); font-size:.8rem; } .analysis-mode select { padding:.5rem; border:1px solid var(--mantis-border); border-radius:6px; background:var(--mantis-bg); color:var(--mantis-text); }
	@media (max-width:700px) { .ai-status-grid { grid-template-columns:1fr; } }

	/* Keep static settings readable without making each option a tall card. */
	.settings-panel h2 { margin-bottom: .7rem; }
	.principles { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:.45rem; }
	.principles li { padding:.55rem .7rem; }
	.principles .wf-title { font-size:.78rem; }
	.principles .wf-desc { margin:.2rem 0 0; font-size:.72rem; line-height:1.35; }
	.theme-picker, .language-picker { margin:.7rem 0; padding:.65rem; }
	.theme-picker p, .language-picker p { margin:.15rem 0 .5rem; font-size:.7rem; }
	.theme-obsidian{--theme-accent:#c7ced1}.theme-arctic{--theme-accent:#c7ced1}.theme-ember{--theme-accent:#df7373}.theme-verdant{--theme-accent:#78d9a6}.theme-cyber{--theme-accent:#f3f6f7}.theme-cyberpunk{--theme-accent:#df7373}.theme-osint{--theme-accent:#78d9a6}.theme-options { grid-template-columns:repeat(3,minmax(0,1fr)); gap:.35rem; }
	.theme-option { gap:.4rem; padding:.45rem; }
	.theme-option small { display:none; }
	.theme-swatches i { width:8px; height:20px; }
	.language-picker { gap:.7rem; }
	.language-options { gap:.35rem; }
	.language-options button { min-width:64px; min-height:42px; padding:.35rem .5rem; gap:.4rem; }
	.flag-icon { width:27px; height:18px; }
	.settings-panel > :global(.ro-field) { margin:0 0 .45rem; padding:.48rem .65rem; }
	.settings-panel > :global(.ro-field) :global(.ro-value) { font-size:.8rem; }
	.settings-panel > :global(.ro-field) :global(.ro-hint) { margin-top:.18rem; font-size:.66rem; line-height:1.25; }
	#application > :global(.ro-field), #donnees > :global(.ro-field) { display:grid; grid-template-columns:minmax(8rem,.45fr) minmax(0,1fr); column-gap:.7rem; align-items:center; }
	#application > :global(.ro-field) :global(.ro-top), #donnees > :global(.ro-field) :global(.ro-top) { grid-row:1 / span 2; display:block; margin:0; }
	#application > :global(.ro-field) :global(.ro-badge), #donnees > :global(.ro-field) :global(.ro-badge) { display:inline-block; margin-top:.25rem; }
	#application > :global(.ro-field) :global(.ro-hint), #donnees > :global(.ro-field) :global(.ro-hint) { grid-column:2; }
	@media (max-width:760px) {
		.principles, .theme-options { grid-template-columns:1fr 1fr; }
	}
	@media (max-width:520px) {
		.principles, .theme-options { grid-template-columns:1fr; }
		#application > :global(.ro-field), #donnees > :global(.ro-field) { display:block; }
		#application > :global(.ro-field) :global(.ro-top), #donnees > :global(.ro-field) :global(.ro-top) { margin-bottom:.25rem; }
	}

	.settings-panel { background:var(--ui-material-panel); }
	.settings-nav a { border:1px solid transparent; }
	.settings-nav a:hover { border-color:var(--ui-border-subtle); }
	.theme-picker,
	.language-picker { background:var(--ui-material-solid); border-color:var(--ui-border-subtle); }
	.theme-option,
	.language-options button { background:var(--ui-canvas); border-color:var(--ui-border-subtle); transform:none; }
	.theme-option:hover,
	.language-options button:hover { background:var(--ui-surface-2); transform:none; }
	.theme-option.active,
	.language-options button.active { background:color-mix(in srgb,var(--ui-accent) 7%,var(--ui-material-solid)); border-color:color-mix(in srgb,var(--ui-accent) 40%,var(--ui-border-default)); box-shadow:inset 2px 0 0 var(--ui-accent); }
	.technical-details,
	.model-picker,
	.analysis-mode { background:var(--ui-material-solid); border-color:var(--ui-border-subtle); }
	.ai-panel { border-top-color:color-mix(in srgb,var(--ui-ai) 38%,var(--ui-border-default)); }
	.data-panel { border-top-color:color-mix(in srgb,var(--ui-danger) 38%,var(--ui-border-default)); }
</style>
