<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import ReadOnlyField from '$lib/ReadOnlyField.svelte';
	import {
		identities,
		getIdentity,
		getFolder,
		identityKindLabel,
		exposuresForIdentity
	} from '$lib/mock/posture';

	const folderFilter = $derived($page.url.searchParams.get('folder'));
	const list = $derived(
		folderFilter ? identities.filter((i) => i.folderId === folderFilter) : identities
	);

	const selectedId = $derived($page.url.searchParams.get('id') ?? list[0]?.id ?? null);
	const selected = $derived(selectedId ? getIdentity(selectedId) : undefined);
	const folder = $derived(selected ? getFolder(selected.folderId) : undefined);
	const filterFolder = $derived(folderFilter ? getFolder(folderFilter) : undefined);
	const linkedExposures = $derived(selected ? exposuresForIdentity(selected.id) : []);

	function identityHref(id: string): string {
		const base = `/identites?id=${id}`;
		return folderFilter ? `${base}&folder=${folderFilter}` : base;
	}
</script>

<section class="wf-view">
	<GuideHeader
		title="Identités"
		question="Quelles traces me concernent (noms, e-mails, profils) ?"
		intro="Inventaire de vos identifiants connus. Aucun mot de passe ni secret n’est stocké. Les valeurs ne sont pas encore modifiables."
	/>

	{#if folderFilter && filterFolder}
		<p class="filter-hint">
			Filtre dossier : <strong>{filterFolder.name}</strong>
			<a href="/identites">Tout afficher</a>
		</p>
	{/if}

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>Liste</h2>
			<ul class="wf-list">
				{#each list as idn (idn.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === idn.id}
							href={identityHref(idn.id)}
						>
							<span>
								<span class="wf-badge">{identityKindLabel[idn.kind]}</span>
								<span class="wf-title" style="display:block;margin-top:0.35rem"
									>{idn.label}</span
								>
								<p class="wf-desc">{idn.value}</p>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel wf-detail">
			{#if selected}
				<h3>{selected.label}</h3>
				<p class="wf-summary">
					{#if linkedExposures.length > 0}
						Cette identité apparaît dans {linkedExposures.length} exposition(s). Ouvrez-les
						pour comprendre le risque.
					{:else}
						Aucune exposition mock liée pour le moment.
					{/if}
				</p>

				<ReadOnlyField
					label="Type"
					value={identityKindLabel[selected.kind]}
					hint="Type figé en mock — édition après Phase 2."
				/>
				<ReadOnlyField label="Valeur" value={selected.value} />
				{#if selected.notes}
					<ReadOnlyField label="Notes" value={selected.notes} />
				{/if}
				{#if folder}
					<ReadOnlyField label="Dossier" value={folder.name} />
				{/if}

				{#if linkedExposures.length > 0}
					<div class="wf-field">
						<dt>Expositions liées</dt>
						<dd>
							<ul class="wf-list">
								{#each linkedExposures as exp (exp.id)}
									<li>
										<a class="wf-row" href={`/expositions?id=${exp.id}`}>
											<span>
												<span class="wf-title">{exp.title}</span>
												<p class="wf-desc">{exp.severity} · {exp.source}</p>
											</span>
										</a>
									</li>
								{/each}
							</ul>
						</dd>
					</div>
					<NextStepBar
						hint="Passez à l’exposition pour voir pourquoi c’est important et quoi faire."
						primaryHref={`/expositions?id=${linkedExposures[0].id}`}
						primaryLabel="Ouvrir la première exposition"
					>
						{#if folder}
							<a class="wf-btn" href={`/dossiers?id=${folder.id}`}>Dossier</a>
						{/if}
					</NextStepBar>
				{:else}
					<div class="wf-secondary-links">
						{#if folder}
							<a class="wf-btn" href={`/dossiers?id=${folder.id}`}>Ouvrir le dossier</a>
						{/if}
						<a class="wf-btn" href="/expositions">Toutes les expositions</a>
					</div>
				{/if}
			{:else}
				<p class="wf-empty">Sélectionnez une identité.</p>
			{/if}
		</div>
	</div>
</section>

<style>
	.filter-hint {
		margin: -0.5rem 0 0;
		font-size: 0.85rem;
		color: var(--mantis-text-muted);
	}

	.filter-hint a {
		margin-left: 0.75rem;
		color: var(--mantis-accent);
	}
</style>
