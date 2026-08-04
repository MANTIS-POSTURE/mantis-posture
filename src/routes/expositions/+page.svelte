<script lang="ts">
	import { onMount } from 'svelte';
	import { listExposures, type Exposure } from '$lib/api';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import StatePanel from '$lib/components/StatePanel.svelte';
	import RemediationJourney from '$lib/components/RemediationJourney.svelte';
	import { activeIdentityId } from '$lib/active-identity';
	import { t } from '$lib/i18n';

	let exposures = $state<Exposure[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function refresh() {
		loading = true;
		try { exposures = await listExposures($activeIdentityId); }
		catch (e) { error = String(e); }
		finally { loading = false; }
	}
	onMount(() => activeIdentityId.subscribe(() => { void refresh(); }));

	const statusLabels: Record<string,string> = { nouvelle:'Nouvelle', en_suivi:'En suivi', acceptee:'Acceptée', reduite:'Réduite' };
	function statusLabel(value:string) { return statusLabels[value] ?? value; }
	function severityTone(value:string) { return value === 'critique' ? 'danger' : value === 'élevée' || value === 'modérée' ? 'warning' : 'success'; }
	function formatDate(value:string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString('fr-FR',{day:'2-digit',month:'short',year:'numeric'}); }
</script>

<section class="wf-view exposures-view">
	<GuideHeader title="Expositions" question="Quelles informations demandent mon attention ?" intro="Chaque élément retenu indique son origine, son niveau de priorité et la prochaine étape possible." />
	<RemediationJourney current="exposition" />

	<div class="exposure-summary">
		<div><span class="summary-value">{exposures.length}</span><span class="summary-label">exposition{exposures.length > 1 ? 's' : ''}</span></div>
		<div><span class="summary-value attention">{exposures.filter((item) => item.severity === 'critique' || item.severity === 'élevée').length}</span><span class="summary-label">prioritaire{exposures.filter((item) => item.severity === 'critique' || item.severity === 'élevée').length > 1 ? 's' : ''}</span></div>
		<a class="wf-btn" href="/veille">Lancer une vérification</a>
	</div>

	<div class="glass-card exposure-panel">
		<div class="panel-heading"><div><h2>Constats retenus</h2><p>Vérifiez la source et le niveau de risque avant de décider d’un suivi.</p></div><span class="count-badge">{exposures.length} résultat{exposures.length > 1 ? 's' : ''}</span></div>
		{#if loading}
			<StatePanel tone="info" title="Chargement des expositions" message="Lecture de la base locale…" />
		{:else if error}
			<StatePanel tone="danger" title="Expositions indisponibles" message={error} />
		{:else if exposures.length === 0}
			<StatePanel tone="success" title="Rien de préoccupant à suivre" message="Lancez une vérification depuis Veille pour rechercher de nouvelles informations publiques." />
		{:else}
			<div class="table-wrap">
				<table>
					<thead><tr><th>Exposition</th><th>Nature</th><th>Risque</th><th>Suivi</th><th>Détectée</th><th>Source</th><th><span class="sr-only">Ouvrir</span></th></tr></thead>
					<tbody>{#each exposures as exp (exp.id)}
						<tr>
							<td data-label="Exposition"><a class="exposure-title" href={`/expositions/${exp.id}`}>{t(exp.title)}</a></td>
							<td data-label="Nature"><span class="badge kind">{exp.kind}</span></td>
							<td data-label="Risque"><span class="badge tone-{severityTone(exp.severity)}"><span class="badge-dot"></span>{exp.severity}</span></td>
							<td data-label="Suivi"><span class="badge status">{statusLabel(exp.status)}</span></td>
							<td data-label="Détectée"><time>{formatDate(exp.discovered_at)}</time></td>
							<td data-label="Source"><span class="source">{exp.source}</span></td>
							<td class="action-cell"><a href={`/expositions/${exp.id}`} class="open-link" aria-label={`Ouvrir ${exp.title}`}>Ouvrir <span aria-hidden="true">→</span></a></td>
						</tr>
					{/each}</tbody>
				</table>
			</div>
		{/if}
	</div>
</section>

<style>
	.exposures-view { max-width:1440px; }
	.exposure-summary { display:flex; align-items:center; gap:.75rem; padding:.7rem; border:1px solid var(--ui-border-subtle); border-radius:var(--radius-md); background:rgba(255,255,255,.022); }
	.exposure-summary>div { display:flex; align-items:baseline; gap:.4rem; min-width:130px; padding:.35rem .65rem; border-right:1px solid var(--ui-border-subtle); }.summary-value { font-size:1.35rem; font-weight:750; color:var(--ui-text-primary); font-variant-numeric:tabular-nums; }.summary-value.attention { color:var(--ui-warning); }.summary-label { color:var(--ui-text-secondary); font-size:.75rem; }.exposure-summary .wf-btn { margin-left:auto; }
	.exposure-panel { padding:0; }.panel-heading { display:flex; align-items:center; justify-content:space-between; gap:1rem; padding:1rem 1.1rem; border-bottom:1px solid var(--ui-border-subtle); }.panel-heading h2 { margin:0 0 .2rem; font-size:1rem; }.panel-heading p { margin:0; color:var(--ui-text-secondary); font-size:.78rem; }.count-badge { padding:.25rem .55rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-pill); color:var(--ui-text-secondary); background:rgba(255,255,255,.035); font-size:.7rem; }
	.table-wrap { overflow-x:auto; } table { width:100%; border-collapse:separate; border-spacing:0; } th { padding:.65rem 1rem; text-align:left; color:var(--ui-text-tertiary); background:rgba(0,0,0,.18); border-bottom:1px solid var(--ui-border-default); font-size:.67rem; font-weight:700; letter-spacing:.1em; text-transform:uppercase; } td { padding:.85rem 1rem; border-bottom:1px solid var(--ui-border-subtle); color:var(--ui-text-secondary); font-size:.8rem; vertical-align:middle; } tbody tr { transition:background var(--duration-fast),box-shadow var(--duration-fast); } tbody tr:hover { background:rgba(255,255,255,.035); box-shadow:inset 2px 0 0 var(--ui-accent); } tbody tr:last-child td { border-bottom:0; }
  .exposure-title { color:var(--ui-text-primary); font-weight:650; text-decoration:none; line-height:1.35; }.exposure-title:hover { color:var(--ui-accent); }.badge { display:inline-flex; align-items:center; gap:.35rem; padding:.25rem .5rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-pill); color:var(--ui-text-secondary); background:rgba(255,255,255,.035); font-size:.69rem; font-weight:650; text-transform:capitalize; white-space:nowrap; }.badge.kind { color:var(--ui-info); border-color:color-mix(in srgb,var(--ui-info) 28%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-info) 8%,transparent); }.badge.status { color:var(--ui-warning); border-color:color-mix(in srgb,var(--ui-warning) 30%,var(--ui-border-default)); background:color-mix(in srgb,var(--ui-warning) 8%,transparent); }.badge-dot { width:5px; height:5px; border-radius:50%; background:currentColor; box-shadow:0 0 7px currentColor; }.tone-danger { color:var(--ui-danger); }.tone-warning { color:var(--ui-warning); }.tone-success { color:var(--ui-success); }.source,time { white-space:nowrap; }.source { color:var(--ui-text-secondary); }.action-cell { width:1%; }.open-link { display:inline-flex; align-items:center; gap:.35rem; padding:.4rem .6rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-sm); color:var(--ui-text-primary); background:rgba(0,0,0,.2); text-decoration:none; font-size:.74rem; font-weight:650; }.open-link:hover { border-color:color-mix(in srgb,var(--ui-accent) 45%,var(--ui-border-default)); color:var(--ui-accent); background:var(--ui-accent-soft); }.sr-only { position:absolute; width:1px; height:1px; padding:0; margin:-1px; overflow:hidden; clip:rect(0,0,0,0); white-space:nowrap; border:0; }
	@media(max-width:1000px) { .exposure-summary { flex-wrap:wrap; }.exposure-summary .wf-btn { width:100%; margin-left:0; } thead { display:none; } table,tbody,tr,td { display:block; } tbody { display:grid; gap:.7rem; padding:.7rem; } tr { padding:.8rem; border:1px solid var(--ui-border-default); border-radius:var(--radius-md); background:rgba(0,0,0,.16); } td { display:flex; justify-content:space-between; gap:1rem; padding:.42rem 0; border:0; } td::before { content:attr(data-label); color:var(--ui-text-tertiary); font-size:.65rem; font-weight:700; letter-spacing:.08em; text-transform:uppercase; }.action-cell { width:auto; margin-top:.4rem; }.action-cell::before { display:none; }.open-link { width:100%; justify-content:center; } }

	.exposures-view .exposure-summary { background:var(--ui-material-solid); border-color:var(--ui-border-subtle); box-shadow:inset 0 1px 0 var(--ui-rim-light); }
	.exposures-view .exposure-panel { background:var(--ui-material-panel); }
	.exposures-view .count-badge,
	.exposures-view .badge,
	.exposures-view .open-link { background:var(--ui-material-solid); }
	.exposures-view tbody tr:hover { background:color-mix(in srgb,var(--ui-accent) 4%,var(--ui-material-solid)); box-shadow:inset 2px 0 0 var(--ui-accent); }
	@media(max-width:1000px) { .exposures-view tr { background:var(--ui-material-solid); } }
</style>
