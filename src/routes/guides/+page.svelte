<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import { guides, getGuide } from '$lib/mock/posture';

	const selectedId = $derived($page.url.searchParams.get('id') ?? guides[0]?.id ?? null);
	const selected = $derived(selectedId ? getGuide(selectedId) : undefined);

	const primaryHref = $derived(
		selected?.relatedActionId
			? `/actions?id=${selected.relatedActionId}`
			: (selected?.relatedHref ?? '/posture')
	);
	const primaryLabel = $derived(
		selected?.relatedActionId
			? 'Ouvrir l’action liée'
			: selected?.relatedHref
				? 'Ouvrir le module lié'
				: 'Retour au centre'
	);
</script>

<section class="wf-view">
	<GuideHeader
		title="Guides"
		question="Comment m’y prendre si je ne suis pas expert ?"
		intro="Mini-tutoriels pour les gestes fréquents. Ils complètent les Actions : ici on explique, là-bas on suit une checklist."
	/>

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>Catalogue</h2>
			<ul class="wf-list">
				{#each guides as guide (guide.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === guide.id}
							href={`/guides?id=${guide.id}`}
						>
							<span>
								<span class="wf-title">{guide.title}</span>
								<p class="wf-desc">{guide.summary}</p>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel wf-detail">
			{#if selected}
				<h3>{selected.title}</h3>
				<p class="wf-summary">{selected.summary}</p>

				<div class="wf-field">
					<dt>Quand l’utiliser</dt>
					<dd>{selected.when}</dd>
				</div>
				<div class="wf-field">
					<dt>Étapes</dt>
					<ol class="wf-steps">
						{#each selected.steps as step, i (i)}
							<li>{step}</li>
						{/each}
					</ol>
				</div>

				<NextStepBar hint="Passez à la checklist concrète quand vous êtes prêt." {primaryHref} {primaryLabel}>
					<a class="wf-btn" href="/actions">Toutes les actions</a>
				</NextStepBar>
			{:else}
				<p class="wf-empty">Sélectionnez un guide.</p>
			{/if}
		</div>
	</div>
</section>
