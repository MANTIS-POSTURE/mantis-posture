<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listIdentities, listFolders, listExposures, type Identity, type Folder, type Exposure } from '$lib/api';

  let identities = $state<Identity[]>([]);
  let folders = $state<Folder[]>([]);
  let exposures = $state<Exposure[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [identities, folders, exposures] = await Promise.all([
        listIdentities(),
        listFolders(),
        listExposures()
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? identities[0]?.id ?? null);
  const selected = $derived(identities.find((i) => i.id === selectedId));

  function getFolderName(folderId: string | null): string {
    if (!folderId) return 'Aucun dossier';
    return folders.find(f => f.id === folderId)?.name ?? 'Inconnu';
  }

  function exposuresForIdentity(identity: Identity | undefined): Exposure[] {
    if (!identity || !identity.folder_id) return [];
    // Note: Since exposures don't link directly to identity_id in the current schema,
    // we show exposures in the same folder as the identity.
    return exposures.filter(e => e.folder_id === identity.folder_id);
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Identités"
    question="Quelles traces me concernent (noms, e-mails, profils) ?"
    intro="Inventaire de vos identifiants connus. Aucun mot de passe ni secret n’est stocké."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des identités...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if identities.length === 0}
    <div class="glass-card"><p class="muted">Aucune identité enregistrée.</p></div>
  {:else}
    <div class="split-layout">
      <div class="glass-card list-panel">
        <h2>Liste</h2>
        <ul class="item-list">
          {#each identities as idn (idn.id)}
            <li>
              <a class="list-item" class:active={selectedId === idn.id} href={`/identites?id=${idn.id}`}>
                <span class="item-title">{idn.label}</span>
                <p class="item-desc">{idn.kind} · {idn.value}</p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.label}</h3>
            <span class="badge">{selected.kind}</span>
          </div>
          
          <p class="summary">{selected.value}</p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Dossier</dt>
              <dd>{getFolderName(selected.folder_id)}</dd>
            </div>
            {#if selected.notes}
              <div class="field">
                <dt>Notes</dt>
                <dd>{selected.notes}</dd>
              </div>
            {/if}
          </div>

          <div class="actions-section">
            <h4>Expositions liées (même dossier)</h4>
            <ul class="sub-list">
              {#each exposuresForIdentity(selected) as exp (exp.id)}
                <li>
                  <a href={`/expositions?id=${exp.id}`} class="sub-link">
                    {exp.title} <span class="muted">({exp.severity})</span>
                  </a>
                </li>
              {:else}
                <li class="muted">Aucune exposition dans ce dossier.</li>
              {/each}
            </ul>
          </div>
        {:else}
          <p class="muted">Sélectionnez une identité.</p>
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
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
    gap: 1rem;
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
    font-family: 'JetBrains Mono', monospace;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 700px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }
  }

  .field {
    margin: 0;
  }

  .field dt {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--mantis-text-muted);
    margin-bottom: 0.3rem;
  }

  .field dd {
    margin: 0;
    font-size: 0.88rem;
    color: var(--mantis-text);
  }

  .actions-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--mantis-border);
  }

  .actions-section h4 {
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    font-weight: 600;
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

  .badge {
    display: inline-block;
    padding: 0.15rem 0.45rem;
    border: 1px solid var(--mantis-border);
    border-radius: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    align-self: flex-start;
  }
</style>
