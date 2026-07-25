<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listIncidents, listActions, type Incident, type Action } from '$lib/api';

  let incidents = $state<Incident[]>([]);
  let actions = $state<Action[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      incidents = await listIncidents();
      actions = await listActions();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? incidents[0]?.id ?? null);
  const selected = $derived(incidents.find((i) => i.id === selectedId));

  function getActionsForIncident(incidentId: string): Action[] {
    return actions.filter((a) => a.incident_id === incidentId);
  }

  function getSeverityColor(sev: string): string {
    switch (sev) {
      case 'critique': return 'var(--mantis-danger)';
      case 'élevée': return '#e67e22';
      case 'modérée': return 'var(--mantis-warn)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  function getStatusLabel(status: string): string {
    switch (status) {
      case 'a_faire': return 'À faire';
      case 'en_cours': return 'En cours';
      case 'faite': return 'Faite';
      default: return status;
    }
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Incidents"
    question="Qu’est-ce qui me demande un vrai suivi, et pourquoi ?"
    intro="Un incident est une exposition assez importante pour être traitée. Lisez le contexte, puis lancez l’action recommandée."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des incidents...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if incidents.length === 0}
    <div class="glass-card"><p class="muted">Aucun incident enregistré dans la base.</p></div>
  {:else}
    <div class="split-layout">
      <!-- Liste des incidents -->
      <div class="glass-card list-panel">
        <h2>À traiter</h2>
        <ul class="item-list">
          {#each incidents as inc (inc.id)}
            <li>
              <a class="list-item" class:active={selectedId === inc.id} href={`/incidents?id=${inc.id}`}>
                <span class="badge" style={`color: ${getSeverityColor(inc.severity)}; border-color: ${getSeverityColor(inc.severity)};`}>{inc.severity}</span>
                <span class="item-title">{inc.title}</span>
                <p class="item-desc">Détecté le {new Date(inc.discovered_at).toLocaleDateString('fr-FR')}</p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Détail de l'incident -->
      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.title}</h3>
            <span class="badge large" style={`color: ${getSeverityColor(selected.severity)}; border-color: ${getSeverityColor(selected.severity)};`}>{selected.severity}</span>
          </div>
          
          <p class="summary">{selected.next_step}</p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Quoi</dt>
              <dd>{selected.what}</dd>
            </div>
            <div class="field">
              <dt>Pourquoi c’est important</dt>
              <dd>{selected.why}</dd>
            </div>
            <div class="field">
              <dt>Impact possible</dt>
              <dd>{selected.impact}</dd>
            </div>
            <div class="field">
              <dt>Confiance</dt>
              <dd>{selected.confidence}</dd>
            </div>
          </div>

          {#if getActionsForIncident(selected.id).length > 0}
            <div class="actions-section">
              <h4>Actions requises</h4>
              <ul class="action-list">
                {#each getActionsForIncident(selected.id) as action (action.id)}
                  <li>
                    <a href={`/actions?id=${action.id}`} class="action-link">
                      {action.title}
                    </a>
                    <span class="status-pill">{getStatusLabel(action.status)}</span>
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        {:else}
          <p class="muted">Sélectionnez un incident dans la liste.</p>
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

  .action-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .action-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--mantis-border);
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.2);
  }

  .action-link {
    color: var(--mantis-text);
    text-decoration: none;
    font-size: 0.88rem;
    font-weight: 500;
  }

  .action-link:hover {
    text-decoration: underline;
    color: var(--mantis-accent);
  }

  .status-pill {
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    font-size: 0.7rem;
    background: var(--mantis-bg-solid);
    border: 1px solid var(--mantis-border);
    color: var(--mantis-text-muted);
  }

  .badge {
    display: inline-block;
    padding: 0.15rem 0.45rem;
    border: 1px solid;
    border-radius: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    align-self: flex-start;
  }

  .badge.large {
    font-size: 0.75rem;
    padding: 0.25rem 0.6rem;
  }
</style>
