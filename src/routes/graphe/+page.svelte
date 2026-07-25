<script lang="ts">
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import { graphEdges } from '$lib/mock/posture';
</script>

<section class="wf-view">
	<GuideHeader
		title="Graphe"
		question="Comment mes traces sont-elles reliées entre elles ?"
		intro="Carte simple des liens entre identités, expositions, incidents et actions. Cliquez une relation pour ouvrir la fiche — sans outil d’analyste complexe."
	/>

	<div class="wf-panel">
		<h2>Relations (aperçu)</h2>
		<p class="panel-help">
			Chaque ligne se lit : <em>élément A</em> — relation — <em>élément B</em>.
		</p>
		<ul class="wf-list">
			{#each graphEdges as edge (edge.id)}
				<li class="edge">
					<a class="node" href={edge.fromHref}>{edge.fromLabel}</a>
					<span class="rel">{edge.relation}</span>
					<a class="node" href={edge.toHref}>{edge.toLabel}</a>
				</li>
			{/each}
		</ul>
	</div>

	<NextStepBar
		hint="Pour agir, repartez du Centre de posture : alertes et priorités y sont ordonnées."
		primaryHref="/posture"
		primaryLabel="Retour au centre"
	>
		<a class="wf-btn" href="/expositions">Voir les expositions</a>
	</NextStepBar>
</section>

<style>
	.panel-help {
		margin: -0.35rem 0 0.85rem;
		font-size: 0.82rem;
		color: var(--mantis-text-muted);
	}

	.edge {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.55rem;
		padding: 0.75rem 0.9rem;
		border: 1px solid var(--mantis-border);
		border-radius: 8px;
		background: var(--mantis-bg);
	}

	.node {
		padding: 0.35rem 0.65rem;
		border-radius: 6px;
		border: 1px solid var(--mantis-border);
		background: var(--mantis-bg-raised);
		color: var(--mantis-text);
		text-decoration: none;
		font-size: 0.85rem;
		font-weight: 600;
	}

	.node:hover {
		border-color: var(--mantis-accent);
	}

	.rel {
		font-size: 0.75rem;
		color: var(--mantis-text-muted);
		font-style: italic;
	}
</style>
