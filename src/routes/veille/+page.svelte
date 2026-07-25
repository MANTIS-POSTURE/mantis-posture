<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import { watchRoutines, watchStatusLabel } from '$lib/mock/posture';

	const selectedId = $derived(
		$page.url.searchParams.get('id') ?? watchRoutines[0]?.id ?? null
	);
	const selected = $derived(watchRoutines.find((r) => r.id === selectedId));
</script>

<section class="wf-view">
	<GuideHeader
		title="Veille"
		question="Qu’est-ce qui tourne en arrière-plan pour me surveiller ?"
		intro="Ici vous verrez les routines de contrôle (fuites, mentions, pages). En Phase 1 tout est planifié en mock : aucune requête réseau n’est lancée."
	/>

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>Routines</h2>
			<ul class="wf-list">
				{#each watchRoutines as routine (routine.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === routine.id}
							href={`/veille?id=${routine.id}`}
						>
							<span>
								<span class="wf-badge">{watchStatusLabel[routine.status]}</span>
								<span class="wf-title" style="display:block;margin-top:0.35rem"
									>{routine.title}</span
								>
								<p class="wf-desc">{routine.frequency}</p>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel wf-detail">
			{#if selected}
				<h3>{selected.title}</h3>
				<p class="wf-summary">{selected.note}</p>

				<dl>
					<div class="wf-field">
						<dt>Fréquence</dt>
						<dd>{selected.frequency}</dd>
					</div>
					<div class="wf-field">
						<dt>Sources</dt>
						<dd>{selected.sources}</dd>
					</div>
					<div class="wf-field">
						<dt>Données transmises (principe)</dt>
						<dd>{selected.transmitted}</dd>
					</div>
					<div class="wf-field">
						<dt>Dernière exécution</dt>
						<dd>{selected.lastRun}</dd>
					</div>
					<div class="wf-field">
						<dt>Prochaine</dt>
						<dd>{selected.nextRun}</dd>
					</div>
				</dl>

				<NextStepBar
					hint="La veille réelle arrivera avec l’OSINT contrôlé. En attendant, traitez les priorités du Centre."
					primaryHref="/posture"
					primaryLabel="Voir le centre de posture"
				>
					<a class="wf-btn" href="/identites">Vérifier mes identités</a>
				</NextStepBar>
			{:else}
				<p class="wf-empty">Sélectionnez une routine.</p>
			{/if}
		</div>
	</div>
</section>
