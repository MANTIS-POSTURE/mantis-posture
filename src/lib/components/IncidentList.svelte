<script lang="ts">
  import { onMount } from 'svelte';
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

  function getActionsForIncident(incidentId: string): Action[] {
    return actions.filter(a => a.incident_id === incidentId);
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

<div class="glass-card">
  <h2>Incidents & Actions</h2>
  
  {#if loading}
    <p class="muted">Chargement...</p>
  {:else if error}
    <p class="error">Erreur: {error}</p>
  {:else if incidents.length === 0}
    <p class="muted">Aucun incident enregistré.</p>
  {:else}
    <div class="list">
      {#each incidents as incident (incident.id)}
        <a class="item" href={`/incidents?id=${incident.id}`}>
          <div class="item-header">
            <h3>{incident.title}</h3>
            <span class="badge" style={`color: ${getSeverityColor(incident.severity)}; border-color: ${getSeverityColor(incident.severity)};`}>
              {incident.severity}
            </span>
          </div>
          <p class="item-desc">{incident.what}</p>
          
          {#if getActionsForIncident(incident.id).length > 0}
            <div class="actions-section">
              <h4>Actions requises:</h4>
              <ul>
                {#each getActionsForIncident(incident.id) as action (action.id)}
                  <li>
                    <a href={`/actions?id=${action.id}`} class="action-link" onclick={(e) => e.stopPropagation()}>
                      {action.title}
                    </a>
                    <span class="status-badge">{getStatusLabel(action.status)}</span>
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .item {
    display: block;
    padding: 1rem;
    border: 1px solid var(--mantis-border);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    text-decoration: none;
    color: inherit;
    transition: border-color 0.12s;
  }

  .item:hover {
    border-color: var(--mantis-accent);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 0.5rem;
  }

  .item-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .item-desc {
    margin: 0;
    font-size: 0.85rem;
    color: var(--mantis-text-muted);
  }

  .badge {
    padding: 0.15rem 0.45rem;
    border: 1px solid;
    border-radius: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .actions-section {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px dashed var(--mantis-border);
  }

  .actions-section h4 {
    margin: 0 0 0.5rem;
    font-size: 0.8rem;
    color: var(--mantis-text);
  }

  .actions-section ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .actions-section li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
  }

  .action-link {
    color: var(--mantis-text);
    text-decoration: none;
  }

  .action-link:hover {
    text-decoration: underline;
  }

  .status-badge {
    margin-left: auto;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-size: 0.7rem;
    background: var(--mantis-bg-solid);
    border: 1px solid var(--mantis-border);
    color: var(--mantis-text-muted);
  }
</style>
