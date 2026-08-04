<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import StatePanel from '$lib/components/StatePanel.svelte';
  import RemediationJourney from '$lib/components/RemediationJourney.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { listIncidents, listActions, type Incident, type Action } from '$lib/api';
  import { activeIdentityId } from '$lib/active-identity';

  let incidents = $state<Incident[]>([]);
  let actions = $state<Action[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function refresh() {
    loading = true;
    try {
      [incidents, actions] = await Promise.all([listIncidents($activeIdentityId), listActions($activeIdentityId)]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
  onMount(() => activeIdentityId.subscribe(() => { void refresh(); }));

  const selectedId = $derived($page.url.searchParams.get('id') ?? incidents[0]?.id ?? null);
  const selected = $derived(incidents.find((i) => i.id === selectedId));

  function getActionsForIncident(incidentId: string): Action[] {
    return actions.filter((a) => a.incident_id === incidentId);
  }

  function severityTone(sev: string): 'danger' | 'warning' | 'neutral' {
    return sev === 'critique' ? 'danger' : sev === 'élevée' || sev === 'modérée' ? 'warning' : 'neutral';
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
    title="Points à traiter"
    question="Qu’est-ce qui mérite une attention particulière, et pourquoi ?"
    intro="Un point à traiter regroupe une situation suffisamment importante pour être suivie. Lisez le contexte, puis choisissez l’action utile."
  />
  <RemediationJourney current="incident" />

  {#if loading}
    <StatePanel tone="info" title="Chargement des points à traiter" message="Lecture du suivi local…" />
  {:else if error}
    <StatePanel tone="danger" title="Points à traiter indisponibles" message={error} />
  {:else if incidents.length === 0}
    <StatePanel tone="success" title="Aucune décision en attente" message="Une exposition vérifiée pourra être qualifiée ici si elle mérite un suivi." />
  {:else}
    <div class="split-layout">
      <!-- Liste des incidents -->
      <div class="glass-card list-panel">
        <div class="list-heading"><div><p class="eyebrow">Décider</p><h2>À traiter</h2></div><span>{incidents.length}</span></div>
        <ul class="item-list">
          {#each incidents as inc (inc.id)}
            <li>
              <a class="list-item" class:active={selectedId === inc.id} href={`/incidents?id=${inc.id}`}>
                <StatusBadge label={inc.severity} tone={severityTone(inc.severity)} dot />
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
            <StatusBadge label={selected.severity} tone={severityTone(selected.severity)} dot />
          </div>
          
          <div class="decision-callout"><span>Prochaine décision</span><strong>{selected.next_step}</strong></div>
          
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
              <h4>Actions liées</h4>
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
          <StatePanel compact title="Sélectionnez un point à traiter" message="Son contexte, son impact et les actions liées apparaîtront ici." />
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>

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

</style>
