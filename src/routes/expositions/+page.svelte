<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import {
		exposures,
		getExposure,
		getFolder,
		getIdentity,
		getIncident,
		toneClass,
		exposureKindLabel,
		exposureStatusLabel
	} from '$lib/mock/posture';

	const folderFilter = $derived($page.url.searchParams.get('folder'));
	const list = $derived(
		folderFilter ? exposures.filter((e) => e.folderId === folderFilter) : exposures
	);

	const selectedId = $derived($page.url.searchParams.get('id') ?? list[0]?.id ?? null);
	const selected = $derived(selectedId ? getExposure(selectedId) : undefined);
	const folder = $derived(selected ? getFolder(selected.folderId) : undefined);
	const filterFolder = $derived(folderFilter ? getFolder(folderFilter) : undefined);
	const incident = $derived(
		selected?.incidentId ? getIncident(selected.incidentId) : undefined
	);

	function exposureHref(id: string): string {
		const base = `/expositions?id=${id}`;
		return folderFilter ? `${base}&folder=${folderFilter}` : base;
	}
</script>

<section class="wf-view">
	<GuideHeader
		title="Expositions"
		question="Qu’est-ce qui a été trouvé sur moi, et est-ce important ?"
		intro="Une exposition est une trace pertinente. Lisez le quoi et le pourquoi, puis ouvrez l’incident pour être guidé vers une action."
	/>

	{#if folderFilter && filterFolder}
		<p class="filter-hint">
			Filtre dossier : <strong>{filterFolder.name}</strong>
			<a href="/expositions">Tout afficher</a>
		</p>
	{/if}

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>Traces relevées</h2>
			<ul class="wf-list">
				{#each list as exp (exp.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === exp.id}
							href={exposureHref(exp.id)}
						>
							<span>
								<span class="badges">
									<span class="wf-badge {toneClass(exp.severity)}">{exp.severity}</span>
									<span class="wf-badge">{exposureStatusLabel[exp.status]}</span>
								</span>
								<span class="wf-title" style="display:block;margin-top:0.35rem"
									>{exp.title}</span
								>
								<p class="wf-desc">
									{exposureKindLabel[exp.kind]} · {exp.discoveredAt}
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
				<p class="wf-summary">{selected.why}</p>
				<p class="wf-meta" style="margin-bottom:1rem">
					{exposureKindLabel[selected.kind]} · {exposureStatusLabel[selected.status]}
					{#if folder}
						· {folder.name}
					{/if}
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
						<dt>Source</dt>
						<dd>{selected.source}</dd>
					</div>
					<div class="wf-field">
						<dt>Gravité</dt>
						<dd>
							<span class="wf-badge {toneClass(selected.severity)}"
								>{selected.severity}</span
							>
						</dd>
					</div>
					<div class="wf-field">
						<dt>Identités concernées</dt>
						<dd>
							<ul class="wf-list">
								{#each selected.identityIds as iid (iid)}
									{@const idn = getIdentity(iid)}
									{#if idn}
										<li>
											<a class="wf-row" href={`/identites?id=${idn.id}`}>
												<span>
													<span class="wf-title">{idn.label}</span>
													<p class="wf-desc">{idn.value}</p>
												</span>
											</a>
										</li>
									{/if}
								{/each}
							</ul>
						</dd>
					</div>
				</dl>

				{#if incident}
					<NextStepBar
						hint="Cette exposition est suivie comme un incident. Ouvrez-le pour la prochaine étape."
						primaryHref={`/incidents?id=${incident.id}`}
						primaryLabel="Ouvrir l’incident"
					>
						{#if folder}
							<a class="wf-btn" href={`/dossiers?id=${folder.id}`}>Dossier</a>
						{/if}
					</NextStepBar>
				{:else}
					<div class="wf-secondary-links">
						{#if folder}
							<a class="wf-btn" href={`/dossiers?id=${folder.id}`}>Dossier</a>
						{/if}
						<a class="wf-btn" href="/posture">Centre de posture</a>
					</div>
				{/if}
				<p class="wf-note">Résumés uniquement — pas de dumps de fuites ni de secrets.</p>
			{:else}
				<p class="wf-empty">Sélectionnez une exposition.</p>
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
