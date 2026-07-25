<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listActions, updateActionStatus, type Action } from '$lib/api';

  let actions = $state<Action[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      actions = await listActions();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? actions[0]?.id ?? null);
  const selected = $derived(actions.find((a) => a.id === selectedId));

  function getPriorityLabel(id: string): string {
    switch (id) {
      case 'prio_001': return 'Basse';
      case 'prio_002': return 'Moyenne';
      case 'prio_003': return 'Haute';
      case 'prio_004': return 'Critique';
      default: return id;
    }
  }

  function getDifficultyLabel(id: string): string {
    switch (id) {
      case 'diff_001': return 'Facile';
      case 'diff_002': return 'Moyenne';
      case 'diff_003': return 'Difficile';
      default: return id;
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

  function getPriorityColor(id: string): string {
    switch (id) {
      case 'prio_004': return 'var(--mantis-danger)';
      case 'prio_003': return '#e67e22';
      case 'prio_002': return 'var(--mantis-warn)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'faite': return 'var(--mantis-ok)';
      case 'en_cours': return 'var(--mantis-accent)';
      default: return 'var(--mantis-text-muted)';
    }
  }

  function parseGuidance(guidance: string): string[] {
    try {
      return JSON.parse(guidance);
    } catch {
      return [];
    }
  }

  async function updateStatus(newStatus: string) {
    if (!selected) return;
    try {
      await updateActionStatus(selected.id, newStatus);
      // Update local state to reflect change immediately
      actions = actions.map(a => a.id === selected.id ? { ...a, status: newStatus } : a);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Actions"
    question="Que dois-je faire concrètement, et comment ?"
    intro="Chaque action est une checklist. Suivez les étapes (souvent hors MANTIS), puis marquez-la comme faite. Aucun mot de passe n’est demandé ici."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des actions...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if actions.length === 0}
    <div class="glass-card"><p class="muted">Aucune action enregistrée dans la base.</p></div>
  {:else}
    <div class="split-layout">
      <!-- Liste des actions -->
      <div class="glass-card list-panel">
        <h2>File d’actions</h2>
        <ul class="item-list">
          {#each actions as act (act.id)}
            <li>
              <a class="list-item" class:active={selectedId === act.id} href={`/actions?id=${act.id}`}>
                <div class="item-header">
                  <span class="item-title">{act.title}</span>
                  <span class="badge" style={`color: ${getPriorityColor(act.priority_id)}; border-color: ${getPriorityColor(act.priority_id)};`}>
                    {getPriorityLabel(act.priority_id)}
                  </span>
                </div>
                <p class="item-desc">
                  {getDifficultyLabel(act.difficulty_id)} · avant le {new Date(act.deadline).toLocaleDateString('fr-FR')}
                </p>
                <span class="status-pill" style={`color: ${getStatusColor(act.status)}; border-color: ${getStatusColor(act.status)};`}>
                  {getStatusLabel(act.status)}
                </span>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Détail de l'action -->
      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.title}</h3>
            <div class="badges">
              <span class="badge" style={`color: ${getPriorityColor(selected.priority_id)}; border-color: ${getPriorityColor(selected.priority_id)};`}>
                {getPriorityLabel(selected.priority_id)}
              </span>
              <span class="badge" style={`color: ${getStatusColor(selected.status)}; border-color: ${getStatusColor(selected.status)};`}>
                {getStatusLabel(selected.status)}
              </span>
            </div>
          </div>
          
          <p class="summary">
            Difficulté : {getDifficultyLabel(selected.difficulty_id)} · Échéance : {new Date(selected.deadline).toLocaleDateString('fr-FR')}
          </p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Comment faire (étapes)</dt>
              <dd>
                <ol class="steps-list">
                  {#each parseGuidance(selected.guidance) as step, i (i)}
                    <li>{step}</li>
                  {/each}
                </ol>
              </dd>
            </div>
            <div class="field">
              <dt>Preuve attendue (optionnelle)</dt>
              <dd>{selected.proof_expected}</dd>
            </div>
          </div>

          <div class="actions-section">
            <h4>Statut de l'action</h4>
            <div class="action-buttons">
              {#if selected.status !== 'en_cours' && selected.status !== 'faite'}
                <button class="wf-btn" onclick={() => updateStatus('en_cours')}>
                  Marquer en cours
                </button>
              {/if}
              {#if selected.status !== 'faite'}
                <button class="wf-btn primary" onclick={() => updateStatus('faite')}>
                  Marquer comme faite
                </button>
              {/if}
            </div>
          </div>

          {#if selected.incident_id}
            <div class="actions-section">
              <h4>Contexte</h4>
              <p class="muted">Cette action est liée à un incident spécifique.</p>
              <a href={`/incidents?id=${selected.incident_id}`} class="wf-btn" style="margin-top: 0.75rem;">
                Voir l'incident lié
              </a>
            </div>
          {/if}
        {:else}
          <p class="muted">Sélectionnez une action dans la liste.</p>
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
    position: relative;
  }

  .list-item:hover {
    border-color: var(--mantis-accent);
  }

  .list-item.active {
    border-color: var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 10%, transparent);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
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

  .status-pill {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    border: 1px solid;
    background: var(--mantis-bg-solid);
    display: none; /* Hidden to avoid clutter with badge, can be re-enabled */
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

  .badges {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
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
    margin-bottom: 0.3rem;
  }

  .field dd {
    margin: 0;
    font-size: 0.88rem;
    color: var(--mantis-text);
  }

  .steps-list {
    margin: 0;
    padding-left: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .actions-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--mantis-border);
  }

  .actions-section h4 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .action-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
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
</style>
