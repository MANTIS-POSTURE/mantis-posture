<script lang="ts">
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import { reportSnapshot } from '$lib/mock/posture';
</script>

<section class="wf-view">
	<GuideHeader
		title="Rapports"
		question="Puis-je résumer ma posture clairement ?"
		intro="Aperçu d’une synthèse locale. L’export HTML/PDF arrivera plus tard ; pour l’instant, lisez et repartez agir."
	/>

	<div class="wf-panel">
		<h2>{reportSnapshot.title}</h2>
		<p class="wf-meta">Généré le {reportSnapshot.generatedAt}</p>

		<div class="stats">
			<div class="stat">
				<span class="stat-value">{reportSnapshot.score}</span>
				<span class="stat-label">Score</span>
			</div>
			<div class="stat">
				<span class="stat-value">{reportSnapshot.openIncidents}</span>
				<span class="stat-label">Incidents</span>
			</div>
			<div class="stat">
				<span class="stat-value">{reportSnapshot.openActions}</span>
				<span class="stat-label">Actions ouvertes</span>
			</div>
			<div class="stat">
				<span class="stat-value">{reportSnapshot.rgpdInProgress}</span>
				<span class="stat-label">Démarches RGPD</span>
			</div>
		</div>

		<div class="wf-field" style="margin-top:1.25rem">
			<dt>Points clés</dt>
			<ul class="highlights">
				{#each reportSnapshot.highlights as h (h)}
					<li>{h}</li>
				{/each}
			</ul>
		</div>
		<p class="wf-note">{reportSnapshot.note}</p>
	</div>

	<NextStepBar
		hint="Un rapport sert à comprendre ; le Centre sert à décider. Continuez par la priorité n°1."
		primaryHref="/posture"
		primaryLabel="Traiter les priorités"
	>
		<a class="wf-btn" href="/incidents">Voir les incidents</a>
		<a class="wf-btn" href="/dpo">Voir le DPO</a>
	</NextStepBar>
</section>

<style>
	.stats {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 0.75rem;
		margin-top: 1.25rem;
	}

	@media (max-width: 700px) {
		.stats {
			grid-template-columns: 1fr 1fr;
		}
	}

	.stat {
		padding: 0.9rem;
		border-radius: 8px;
		border: 1px solid var(--mantis-border);
		background: var(--mantis-bg);
		text-align: center;
	}

	.stat-value {
		display: block;
		font-size: 1.6rem;
		font-weight: 750;
		line-height: 1.1;
	}

	.stat-label {
		display: block;
		margin-top: 0.35rem;
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--mantis-text-muted);
	}

	.highlights {
		margin: 0.35rem 0 0;
		padding-left: 1.1rem;
		font-size: 0.88rem;
		color: var(--mantis-text-muted);
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}
</style>
