<script lang="ts">
	import { page } from '$app/stores';
	import '$lib/workflow.css';
	import GuideHeader from '$lib/GuideHeader.svelte';
	import NextStepBar from '$lib/NextStepBar.svelte';
	import {
		folders,
		getFolder,
		identitiesForFolder,
		exposuresForFolder,
		incidentsForFolder
	} from '$lib/mock/posture';

	const selectedId = $derived($page.url.searchParams.get('id') ?? folders[0]?.id ?? null);
	const selected = $derived(selectedId ? getFolder(selectedId) : undefined);
	const folderIdentities = $derived(selected ? identitiesForFolder(selected.id) : []);
	const folderExposures = $derived(selected ? exposuresForFolder(selected.id) : []);
	const folderIncidents = $derived(selected ? incidentsForFolder(selected.id) : []);
</script>

<section class="wf-view">
	<GuideHeader
		title="Dossiers"
		question="Dans quel contexte je regarde ma posture ?"
		intro="Un dossier regroupe un périmètre (personnel, emploi…). Choisissez-en un, puis explorez ses identités ou ses expositions."
	/>

	<div class="wf-grid wf-split">
		<div class="wf-panel">
			<h2>Mes contextes</h2>
			<ul class="wf-list">
				{#each folders as folder (folder.id)}
					<li>
						<a
							class="wf-row"
							class:active={selectedId === folder.id}
							href={`/dossiers?id=${folder.id}`}
						>
							<span>
								<span class="wf-title">{folder.name}</span>
								<p class="wf-desc">{folder.context}</p>
								<p class="wf-desc">
									{folder.identityIds.length} identité(s) ·
									{exposuresForFolder(folder.id).length} exposition(s)
								</p>
							</span>
						</a>
					</li>
				{/each}
			</ul>
		</div>

		<div class="wf-panel wf-detail">
			{#if selected}
				<h3>{selected.name}</h3>
				<p class="wf-summary">{selected.context}</p>

				<div class="wf-field">
					<dt>Identités de ce dossier</dt>
					<dd>
						<ul class="wf-list">
							{#each folderIdentities as idn (idn.id)}
								<li>
									<a class="wf-row" href={`/identites?id=${idn.id}`}>
										<span>
											<span class="wf-title">{idn.label}</span>
											<p class="wf-desc">{idn.value}</p>
										</span>
									</a>
								</li>
							{/each}
						</ul>
					</dd>
				</div>

				<div class="wf-field">
					<dt>Expositions</dt>
					<dd>
						{#if folderExposures.length === 0}
							<p class="wf-empty">Aucune exposition dans ce dossier.</p>
						{:else}
							<ul class="wf-list">
								{#each folderExposures as exp (exp.id)}
									<li>
										<a class="wf-row" href={`/expositions?id=${exp.id}`}>
											<span>
												<span class="wf-title">{exp.title}</span>
												<p class="wf-desc">{exp.severity} · {exp.status}</p>
											</span>
										</a>
									</li>
								{/each}
							</ul>
						{/if}
					</dd>
				</div>

				{#if folderIncidents.length > 0}
					<div class="wf-field">
						<dt>Incidents en cours</dt>
						<dd>
							<ul class="wf-list">
								{#each folderIncidents as inc (inc.id)}
									<li>
										<a class="wf-row" href={`/incidents?id=${inc.id}`}>
											<span class="wf-title">{inc.title}</span>
										</a>
									</li>
								{/each}
							</ul>
						</dd>
					</div>
				{/if}

				<NextStepBar
					hint="Pour comprendre ce qui est exposé dans ce contexte, ouvrez les expositions."
					primaryHref={`/expositions?folder=${selected.id}`}
					primaryLabel="Voir les expositions"
				>
					<a class="wf-btn" href={`/identites?folder=${selected.id}`}>Voir les identités</a>
				</NextStepBar>
			{:else}
				<p class="wf-empty">Sélectionnez un dossier.</p>
			{/if}
		</div>
	</div>
</section>
