<script lang="ts">
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import {
		postureScore,
		alerts,
		priorities,
		nextActions,
		alertLevelLabel
	} from '$lib/mock/posture';

	const scoreIntro =
		postureScore.value >= 80
			? 'Votre posture est globalement solide. Traitez les points restants pour la consolider.'
			: postureScore.value >= 60
				? 'Votre posture est correcte mais perfectible. Commencez par la priorité n°1 ci-dessous.'
				: 'Plusieurs points demandent votre attention. Suivez les priorités dans l’ordre.';
</script>

<section class="wf-view">
	<GuideHeader
		title="Centre de posture"
		question="Où en suis-je, et que dois-je faire en premier ?"
		intro="Ce hub résume votre exposition numérique et vous oriente vers la prochaine action utile — sans jargon technique."
	/>

	<div class="score-block wf-panel">
		<div class="score-main">
			<div class="score-value">{postureScore.value}</div>
			<div class="score-label">{postureScore.label}</div>
		</div>
		<div class="score-factors">
			<p class="score-plain">{scoreIntro}</p>
			<div>
				<p class="factor-label">Ce qui va bien</p>
				<ul>
					{#each postureScore.factorsPositive as f (f)}
						<li>{f}</li>
					{/each}
				</ul>
			</div>
			<div>
				<p class="factor-label">Ce qui tire le score vers le bas</p>
				<ul>
					{#each postureScore.factorsNegative as f (f)}
						<li>{f}</li>
					{/each}
				</ul>
			</div>
		</div>
	</div>

	<div class="wf-grid">
		<div class="wf-panel">
			<h2>Priorités — commencez ici</h2>
			<ul class="wf-list">
				{#each priorities as p (p.id)}
					<li>
						<a class="wf-row" href={`/actions?id=${p.actionId}`}>
							<span class="rank">{p.rank}</span>
							<span>
								<span class="wf-title">{p.title}</span>
								<p class="wf-desc">{p.reason}</p>
								<span class="wf-cta-hint">Faire cette action →</span>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel">
			<h2>Prochaines actions</h2>
			<ul class="wf-list">
				{#each nextActions as a (a.id)}
					<li>
						<a class="wf-row" href={`/actions?id=${a.actionId}`}>
							<span>
								<span class="wf-title">{a.label}</span>
								<p class="wf-desc">À faire avant le {a.deadline}</p>
								<span class="wf-cta-hint">Ouvrir le guide →</span>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>
	</div>

	<div class="wf-panel">
		<h2>Alertes</h2>
		<p class="panel-help">
			Chaque alerte actionnable ouvre l’incident correspondant pour comprendre le risque, puis
			choisir quoi faire.
		</p>
		<ul class="wf-list">
			{#each alerts as alert (alert.id)}
				<li>
					{#if alert.incidentId}
						<a class="wf-row" href={`/incidents?id=${alert.incidentId}`}>
							<span class="wf-dot" data-level={alert.level}></span>
							<span>
								<span class="wf-meta">{alertLevelLabel[alert.level]}</span>
								<span class="wf-title" style="display:block">{alert.title}</span>
								<p class="wf-desc">{alert.summary}</p>
								<span class="wf-cta-hint">Comprendre et traiter →</span>
							</span>
						</a>
					{:else}
						<div class="wf-row static">
							<span class="wf-dot" data-level={alert.level}></span>
							<span>
								<span class="wf-meta">{alertLevelLabel[alert.level]}</span>
								<span class="wf-title" style="display:block">{alert.title}</span>
								<p class="wf-desc">{alert.summary}</p>
								<p class="wf-desc">Aucune action requise pour le moment.</p>
							</span>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	</div>
</section>

<style>
	.score-block {
		display: grid;
		grid-template-columns: 180px 1fr;
		gap: 1.5rem;
		align-items: start;
	}

	@media (max-width: 700px) {
		.score-block {
			grid-template-columns: 1fr;
			text-align: center;
		}
	}

	.score-value {
		font-size: 3.5rem;
		font-weight: 800;
		color: var(--mantis-warn);
		line-height: 1;
	}

	.score-label {
		margin-top: 0.4rem;
		font-size: 0.75rem;
		color: var(--mantis-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.14em;
	}

	.score-plain {
		margin: 0 0 0.85rem;
		grid-column: 1 / -1;
		font-size: 0.9rem;
		line-height: 1.45;
	}

	.score-factors {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
	}

	@media (max-width: 700px) {
		.score-factors {
			grid-template-columns: 1fr;
			text-align: left;
		}
	}

	.factor-label {
		margin: 0 0 0.4rem;
		font-size: 0.72rem;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--mantis-text-muted);
	}

	.score-factors ul {
		margin: 0;
		padding-left: 1.1rem;
		font-size: 0.85rem;
		color: var(--mantis-text-muted);
	}

	.rank {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		margin-top: 0.1rem;
		border-radius: 4px;
		border: 1px solid var(--mantis-border);
		font-size: 0.75rem;
		font-weight: 700;
		color: var(--mantis-accent);
		flex-shrink: 0;
	}

	.panel-help {
		margin: -0.35rem 0 0.85rem;
		font-size: 0.82rem;
		color: var(--mantis-text-muted);
	}
</style>
