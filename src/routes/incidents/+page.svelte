<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import {
		incidents,
		getIncident,
		getAction,
		getExposure,
		toneClass,
		type Incident
	} from '$lib/mock/posture';

	const selectedId = $derived($page.url.searchParams.get('id') ?? incidents[0]?.id ?? null);
	const selected: Incident | undefined = $derived(
		selectedId ? getIncident(selectedId) : undefined
	);

	const primaryAction = $derived(
		selected?.actionIds[0] ? getAction(selected.actionIds[0]) : undefined
	);
</script>

<section class="wf-view">
	<GuideHeader
		title="Incidents"
		question="Qu’est-ce qui me demande un vrai suivi, et pourquoi ?"
		intro="Un incident, c’est une exposition assez importante pour être traitée. Lisez le contexte, puis lancez l’action recommandée."
	/>

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>À traiter</h2>
			<ul class="wf-list">
				{#each incidents as inc (inc.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === inc.id}
							href={`/incidents?id=${inc.id}`}
						>
							<span>
								<span class="wf-badge {toneClass(inc.severity)}">{inc.severity}</span>
								<span class="wf-title" style="display:block;margin-top:0.35rem"
									>{inc.title}</span
								>
								<p class="wf-desc">{inc.dossier} · détecté {inc.discoveredAt}</p>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel wf-detail">
			{#if selected}
				<h3>{selected.title}</h3>
				<p class="wf-summary">
					En bref : {selected.nextStep}
				</p>
				<p class="wf-meta" style="margin-bottom:1rem">
					{selected.dossier} · gravité {selected.severity}
				</p>

				<dl>
					<div class="wf-field">
						<dt>Quoi</dt>
						<dd>{selected.what}</dd>
					</div>
					<div class="wf-field">
						<dt>Pourquoi c’est important</dt>
						<dd>{selected.why}</dd>
					</div>
					<div class="wf-field">
						<dt>Impact possible</dt>
						<dd>{selected.impact}</dd>
					</div>
					<div class="wf-field">
						<dt>Niveau de confiance</dt>
						<dd>{selected.confidence}</dd>
					</div>
					{#if selected.exposureIds.length > 0}
						<div class="wf-field">
							<dt>Traces à l’origine</dt>
							<dd>
								<ul class="wf-list">
									{#each selected.exposureIds as eid (eid)}
										{@const exp = getExposure(eid)}
										{#if exp}
											<li>
												<a class="wf-row" href={`/expositions?id=${exp.id}`}>
													<span class="wf-title">{exp.title}</span>
												</a>
											</li>
										{/if}
									{/each}
								</ul>
							</dd>
						</div>
					{/if}
				</dl>

				{#if primaryAction}
					<NextStepBar
						hint={selected.nextStep}
						primaryHref={`/actions?id=${primaryAction.id}`}
						primaryLabel="Faire l’action recommandée"
					>
						{#if selected.rgpdId}
							<a class="wf-btn" href={`/dpo?id=${selected.rgpdId}`}>Préparer le RGPD</a>
						{/if}
					</NextStepBar>
				{/if}

				<div class="wf-secondary-links">
					<a class="wf-btn" href="/posture">Retour au centre</a>
				</div>
			{:else}
				<p class="wf-empty">Sélectionnez un incident dans la liste.</p>
			{/if}
		</div>
	</div>
</section>
