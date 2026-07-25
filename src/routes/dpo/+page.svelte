<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import ReadOnlyField from '$lib/ReadOnlyField.svelte';
	import {
		rgpdRequests as seed,
		rgpdTypeLabel,
		getIncident,
		getAction,
		type RgpdRequest,
		type RgpdStatus
	} from '$lib/mock/posture';

	let requests = $state<RgpdRequest[]>(seed.map((r) => ({ ...r })));
	let copied = $state(false);

	const selectedId = $derived($page.url.searchParams.get('id') ?? requests[0]?.id ?? null);
	const selected = $derived(requests.find((r) => r.id === selectedId));

	function setStatus(id: string, status: RgpdStatus) {
		requests = requests.map((r) => (r.id === id ? { ...r, status } : r));
	}

	async function copyDraft(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			/* clipboard may be unavailable outside secure context */
		}
	}

	const nextHint = $derived(
		!selected
			? ''
			: selected.status === 'brouillon'
				? 'Vérifiez le brouillon, puis marquez la demande comme prête.'
				: selected.status === 'prete'
					? 'Copiez le brouillon, envoyez-le vous-même, puis indiquez que c’est envoyé.'
					: 'Demande suivie. Aucun envoi automatique depuis MANTIS.'
	);
</script>

<section class="wf-view">
	<GuideHeader
		title="DPO / RGPD"
		question="Comment préparer ma demande de droits, sans me tromper ?"
		intro="MANTIS prépare le texte et le suivi. Vous seul envoyez la demande (mail, formulaire ou courrier)."
	/>

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>Démarches</h2>
			<ul class="wf-list">
				{#each requests as req (req.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === req.id}
							href={`/dpo?id=${req.id}`}
						>
							<span>
								<span class="wf-badge">{rgpdTypeLabel[req.type]}</span>
								<span class="wf-title" style="display:block;margin-top:0.35rem"
									>{req.target}</span
								>
								<p class="wf-desc">{req.status} · {req.createdAt}</p>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel wf-detail">
			{#if selected}
				<h3>{rgpdTypeLabel[selected.type]} — {selected.target}</h3>
				<p class="wf-summary">
					Étapes : vérifier la cible → copier le brouillon → envoyer vous-même → noter
					l’envoi ici.
				</p>

				<ReadOnlyField label="Organisation cible" value={selected.target} />
				<ReadOnlyField
					label="Contact DPO / privacy"
					value={selected.dpoContact}
					hint="E-mail non éditable pour l’instant — saisie et validation prévues en Phase 2."
				/>
				<ReadOnlyField
					label="Données concernées"
					value={`${selected.dataSummary} (${selected.identities.join(', ')})`}
				/>

				<ol class="guide">
					<li class="guide-step done">
						<strong>1. Type</strong> — {rgpdTypeLabel[selected.type]}
					</li>
					<li class="guide-step done">
						<strong>2. Cible & contact</strong> — vérifiés (lecture seule)
					</li>
					<li class="guide-step" class:done={selected.status !== 'brouillon'}>
						<strong>3. Brouillon</strong> — vérifier puis copier
					</li>
					<li
						class="guide-step"
						class:done={selected.status === 'envoyee' || selected.status === 'repondue'}
					>
						<strong>4. Envoi</strong> — vous seul envoyez
					</li>
				</ol>

				<div class="wf-field" style="margin-top:1rem">
					<p class="wf-meta">Aperçu du brouillon</p>
					<pre class="wf-draft">{selected.draftPreview}</pre>
				</div>

				{#if selected.status === 'brouillon'}
					<NextStepBar
						hint={nextHint}
						primaryLabel="Marquer prête à envoyer"
						onPrimary={() => setStatus(selected.id, 'prete')}
					>
						<button
							type="button"
							class="wf-btn"
							onclick={() => copyDraft(selected.draftPreview)}
						>
							{copied ? 'Copié' : 'Copier le brouillon'}
						</button>
					</NextStepBar>
				{:else if selected.status === 'prete'}
					<NextStepBar
						hint={nextHint}
						primaryLabel={copied ? 'Brouillon copié' : 'Copier le brouillon'}
						onPrimary={() => copyDraft(selected.draftPreview)}
					>
						<button
							type="button"
							class="wf-btn"
							onclick={() => setStatus(selected.id, 'envoyee')}
						>
							J’ai envoyé
						</button>
					</NextStepBar>
				{:else}
					<NextStepBar hint={nextHint} primaryHref="/posture" primaryLabel="Retour au centre" />
				{/if}

				<div class="wf-secondary-links">
					{#if selected.incidentId}
						{@const inc = getIncident(selected.incidentId)}
						{#if inc}
							<a class="wf-btn" href={`/incidents?id=${inc.id}`}>Incident lié</a>
						{/if}
					{/if}
					{#if selected.actionId}
						{@const act = getAction(selected.actionId)}
						{#if act}
							<a class="wf-btn" href={`/actions?id=${act.id}`}>Action liée</a>
						{/if}
					{/if}
				</div>
				<p class="wf-note">Aucun envoi automatique. Aucun secret stocké.</p>
			{:else}
				<p class="wf-empty">Sélectionnez une démarche.</p>
			{/if}
		</div>
	</div>
</section>

<style>
	.guide {
		margin: 0;
		padding: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 0.55rem;
	}

	.guide-step {
		padding: 0.65rem 0.8rem;
		border-radius: 8px;
		border: 1px solid var(--mantis-border);
		font-size: 0.85rem;
		color: var(--mantis-text-muted);
	}

	.guide-step.done {
		border-color: color-mix(in srgb, var(--mantis-ok) 35%, var(--mantis-border));
		color: var(--mantis-text);
	}

	.guide-step strong {
		color: var(--mantis-text);
	}
</style>
