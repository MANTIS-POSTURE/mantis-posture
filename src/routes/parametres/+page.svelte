<script lang="ts">
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import ReadOnlyField from '$lib/ReadOnlyField.svelte';
	import { appPrinciples } from '$lib/mock/posture';
</script>

<section class="wf-view">
	<GuideHeader
		title="Paramètres"
		question="Comment MANTIS traite mes données, et que puis-je configurer ?"
		intro="Pour l’instant : transparence sur les principes. Les réglages avancés (export base, modules OSINT, thème) viendront avec la persistance locale."
	/>

	<div class="wf-panel">
		<h2>Principes de confiance</h2>
		<ul class="principles">
			{#each appPrinciples as p (p.title)}
				<li>
					<span class="wf-title">{p.title}</span>
					<p class="wf-desc">{p.text}</p>
				</li>
			{/each}
		</ul>
	</div>

	<div class="wf-panel">
		<h2>Application</h2>
		<ReadOnlyField label="Version" value="0.0.1 (Phase 1 — cockpit mock)" />
		<ReadOnlyField
			label="Stockage"
			value="Données fictives en mémoire (src/lib/mock). SQLite local prévu en Phase 2."
			hint="Aucun fichier de base utilisateur n’est encore créé."
		/>
		<ReadOnlyField
			label="Télémétrie"
			value="Désactivée — aucune collecte d’usage."
			hint="Ce réglage restera non négociable."
		/>
		<ReadOnlyField
			label="OSINT / réseau"
			value="Inactif en Phase 1. Les routines de Veille sont affichées sans exécution."
		/>
	</div>

	<NextStepBar
		hint="Rien à configurer pour commencer : suivez le Centre de posture."
		primaryHref="/posture"
		primaryLabel="Ouvrir le centre"
	>
		<a class="wf-btn" href="/guides">Lire un guide</a>
	</NextStepBar>
</section>

<style>
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
</style>
