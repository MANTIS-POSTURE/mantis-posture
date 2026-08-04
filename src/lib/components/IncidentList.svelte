<script lang="ts">
  import { onMount } from 'svelte';
  import { listIncidents, listActions, type Incident, type Action } from '$lib/api';
	import StatePanel from './StatePanel.svelte';
  import { t } from '$lib/i18n';

  let incidents = $state<Incident[]>([]);
  let actions = $state<Action[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  const visibleIncidents = $derived(incidents.slice(0, 3));

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
      case 'élevée': return 'var(--ui-danger)';
      case 'modérée': return 'var(--mantis-warn)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  function getStatusLabel(status: string): string {
    switch (status) {
      case 'a_faire': return t('À faire');
      case 'en_cours': return t('En cours');
      case 'effectue_moi': return t('Effectuée par moi');
      case 'effectue_site': return t('Effectuée par le site');
      case 'en_attente': return t('En attente');
      case 'impossible': return t('Impossible');
      case 'ignore': return t('Ignorée');
      case 'faite': return t('Faite');
      default: return status;
    }
  }
</script>

<div class="glass-card">
  <div class="list-heading"><div><p class="eyebrow">{t('Décisions')}</p><h2>{t('Incidents et actions')}</h2></div>{#if incidents.length}<span>{incidents.length}</span>{/if}</div>
  
  {#if loading}
    <StatePanel compact tone="info" title={t('Chargement des incidents')} />
  {:else if error}
    <StatePanel compact tone="danger" title={t('Incidents indisponibles')} message={error} />
  {:else if incidents.length === 0}
    <StatePanel compact tone="success" title={t('Aucun incident ouvert')} message={t('Les signaux nécessitant un suivi apparaîtront ici.')} />
  {:else}
    <div class="list">
      {#each visibleIncidents as incident (incident.id)}
        <article class="item">
          <div class="item-header">
            <h3><a class="incident-link" href={`/incidents?id=${incident.id}`}>{incident.title}</a></h3>
            <span class="badge" style={`color: ${getSeverityColor(incident.severity)}; border-color: ${getSeverityColor(incident.severity)};`}>
              {incident.severity}
            </span>
          </div>
          <p class="item-desc">{incident.what}</p>
          
          {#if getActionsForIncident(incident.id).length > 0}
            <div class="actions-section">
              <h4>{t('Actions requises:')}</h4>
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
        </article>
      {/each}
    </div>
	<a class="list-footer" href="/incidents">{t('Voir tous les incidents →')}</a>
  {/if}
</div>

<style>
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
	.list-heading { display:flex; align-items:start; justify-content:space-between; gap:1rem; margin-bottom:.85rem; }.list-heading h2 { margin:.2rem 0 0; }.list-heading .eyebrow { margin:0; color:var(--ui-danger); text-transform:uppercase; }.list-heading>span { display:grid; place-items:center; min-width:28px; height:28px; border:1px solid var(--ui-border-default); border-radius:var(--radius-pill); color:var(--ui-text-secondary); font-family:var(--font-meta); font-size:.72rem; }.list-footer { display:inline-block; margin-top:.85rem; color:var(--ui-link); font-size:.78rem; font-weight:620; text-decoration:none; }

  .incident-link { color: inherit; text-decoration: none; }
  .incident-link:hover { color: var(--ui-accent); }

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
