<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listFolders, listIdentities, listExposures, listIncidents, type Folder, type Identity, type Exposure, type Incident } from '$lib/api';

  let folders = $state<Folder[]>([]);
  let identities = $state<Identity[]>([]);
  let exposures = $state<Exposure[]>([]);
  let incidents = $state<Incident[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [folders, identities, exposures, incidents] = await Promise.all([
        listFolders(),
        listIdentities(),
        listExposures(),
        listIncidents()
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? folders[0]?.id ?? null);
  const selected = $derived(folders.find((f) => f.id === selectedId));

  function identitiesForFolder(folderId: string): Identity[] {
    return identities.filter((i) => i.folder_id === folderId);
  }
  function exposuresForFolder(folderId: string): Exposure[] {
    return exposures.filter((e) => e.folder_id === folderId);
  }
  function incidentsForFolder(folderId: string): Incident[] {
    return incidents.filter((i) => i.folder_id === folderId);
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Dossiers"
    question="Dans quel contexte je regarde ma posture ?"
    intro="Un dossier regroupe un périmètre (personnel, emploi…). Choisissez-en un, puis explorez ses identités ou ses expositions."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des dossiers...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if folders.length === 0}
    <div class="glass-card"><p class="muted">Aucun dossier enregistré.</p></div>
  {:else}
    <div class="split-layout">
      <div class="glass-card list-panel">
        <h2>Mes contextes</h2>
        <ul class="item-list">
          {#each folders as folder (folder.id)}
            <li>
              <a class="list-item" class:active={selectedId === folder.id} href={`/dossiers?id=${folder.id}`}>
                <span class="item-title">{folder.name}</span>
                <p class="item-desc">{folder.context}</p>
                <p class="item-desc">
                  {identitiesForFolder(folder.id).length} identité(s) ·
                  {exposuresForFolder(folder.id).length} exposition(s)
                </p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.name}</h3>
          </div>
          <p class="summary">{selected.context}</p>

          <div class="detail-grid">
            <div class="field">
              <dt>Identités de ce dossier</dt>
              <dd>
                <ul class="sub-list">
                  {#each identitiesForFolder(selected.id) as idn (idn.id)}
                    <li>
                      <a href={`/identites?id=${idn.id}`} class="sub-link">
                        {idn.label} <span class="muted">({idn.kind})</span>
                      </a>
                    </li>
                  {:else}
                    <li class="muted">Aucune identité.</li>
                  {/each}
                </ul>
              </dd>
            </div>

            <div class="field">
              <dt>Expositions</dt>
              <dd>
                <ul class="sub-list">
                  {#each exposuresForFolder(selected.id) as exp (exp.id)}
                    <li>
                      <a href={`/expositions?id=${exp.id}`} class="sub-link">
                        {exp.title} <span class="muted">({exp.severity})</span>
                      </a>
                    </li>
                  {:else}
                    <li class="muted">Aucune exposition.</li>
                  {/each}
                </ul>
              </dd>
            </div>

            <div class="field">
              <dt>Incidents en cours</dt>
              <dd>
                <ul class="sub-list">
                  {#each incidentsForFolder(selected.id) as inc (inc.id)}
                    <li>
                      <a href={`/incidents?id=${inc.id}`} class="sub-link">
                        {inc.title} <span class="muted">({inc.severity})</span>
                      </a>
                    </li>
                  {:else}
                    <li class="muted">Aucun incident.</li>
                  {/each}
                </ul>
              </dd>
            </div>
          </div>
        {:else}
          <p class="muted">Sélectionnez un dossier.</p>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .split-layout {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 1.25rem;
  }

  @media (max-width: 900px) {
    .split-layout {
      grid-template-columns: 1fr;
    }
  }

  .list-panel h2 {
    margin: 0 0 1rem;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .item-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list-item {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.75rem;
    border: 1px solid var(--mantis-border);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    text-decoration: none;
    color: inherit;
    transition: border-color 0.12s;
  }

  .list-item:hover {
    border-color: var(--mantis-accent);
  }

  .list-item.active {
    border-color: var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 10%, transparent);
  }

  .item-title {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .item-desc {
    margin: 0;
    font-size: 0.75rem;
    color: var(--mantis-text-muted);
  }

  .detail-header {
    margin-bottom: 1rem;
  }

  .detail-header h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .summary {
    margin: 0 0 1.25rem;
    padding: 0.75rem 1rem;
    border-left: 3px solid var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 5%, transparent);
    border-radius: 0 6px 6px 0;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.25rem;
  }

  .field {
    margin: 0;
  }

  .field dt {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--mantis-text-muted);
    margin-bottom: 0.5rem;
  }

  .sub-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .sub-link {
    display: block;
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--mantis-border);
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.2);
    color: var(--mantis-text);
    text-decoration: none;
    font-size: 0.88rem;
    transition: border-color 0.12s;
  }

  .sub-link:hover {
    border-color: var(--mantis-accent);
  }
</style>
