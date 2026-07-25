<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import {
		actions as seedActions,
		actionStatusLabel,
		getIncident,
		toneClass,
		type ActionItem,
		type ActionStatus
	} from '$lib/mock/posture';

	let items = $state<ActionItem[]>(seedActions.map((a) => ({ ...a })));

	const selectedId = $derived($page.url.searchParams.get('id') ?? items[0]?.id ?? null);
	const selected = $derived(items.find((a) => a.id === selectedId));

	function setStatus(id: string, status: ActionStatus) {
		items = items.map((a) => (a.id === id ? { ...a, status } : a));
	}
</script>

<section class="wf-view">
	<GuideHeader
		title="Actions"
		question="Que dois-je faire concrètement, et comment ?"
		intro="Chaque action est une checklist. Suivez les étapes (souvent hors MANTIS), puis marquez-la comme faite. Aucun mot de passe n’est demandé ici."
	/>

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>File d’actions</h2>
			<ul class="wf-list">
				{#each items as act (act.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === act.id}
							href={`/actions?id=${act.id}`}
						>
							<span>
								<span class="badges">
									<span class="wf-badge {toneClass(act.priority)}">{act.priority}</span>
									<span class="wf-badge">{actionStatusLabel[act.status]}</span>
								</span>
								<span class="wf-title" style="display:block;margin-top:0.35rem"
									>{act.title}</span
								>
								<p class="wf-desc">
									{act.difficulty} · avant le {act.deadline}
								</p>
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
					Priorité {selected.priority} · {selected.difficulty} · échéance {selected.deadline}
				</p>
				<p class="wf-meta" style="margin-bottom:1rem">
					{selected.dossier} · {actionStatusLabel[selected.status]}
				</p>

				<div class="wf-field">
					<dt>Comment faire (étapes)</dt>
					<ol class="wf-steps">
						{#each selected.guidance as step, i (i)}
							<li>{step}</li>
						{/each}
					</ol>
				</div>
				<div class="wf-field">
					<dt>Preuve attendue (optionnelle)</dt>
					<dd>{selected.proofExpected}</dd>
				</div>

				{#if selected.status !== 'faite'}
					<NextStepBar
						hint="Suivez les étapes ci-dessus, puis confirmez ici quand c’est fait."
						primaryLabel="Marquer comme faite"
						onPrimary={() => setStatus(selected.id, 'faite')}
					>
						<button
							type="button"
							class="wf-btn"
							disabled={selected.status === 'en_cours'}
							onclick={() => setStatus(selected.id, 'en_cours')}
						>
							Marquer en cours
						</button>
						{#if selected.rgpdId}
							<a class="wf-btn" href={`/dpo?id=${selected.rgpdId}`}>Aller au DPO</a>
						{/if}
					</NextStepBar>
				{:else}
					<NextStepBar
						hint="Action terminée pour cette session. Vous pouvez la rouvrir si besoin."
						primaryLabel="Rouvrir l’action"
						onPrimary={() => setStatus(selected.id, 'a_faire')}
					/>
				{/if}

				<div class="wf-secondary-links">
					{#if selected.incidentId}
						{@const inc = getIncident(selected.incidentId)}
						{#if inc}
							<a class="wf-btn" href={`/incidents?id=${inc.id}`}>Voir l’incident</a>
						{/if}
					{/if}
					<a class="wf-btn" href="/posture">Centre de posture</a>
				</div>
				<p class="wf-note">
					État local à la session (mock). Aucun secret ni preuve sensible n’est stocké.
				</p>
			{:else}
				<p class="wf-empty">Sélectionnez une action.</p>
			{/if}
		</div>
	</div>
</section>

<style>
	.badges {
		display: flex;
		flex-wrap: wrap;
		gap: 0.35rem;
	}
</style>
