<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listExposures, type Exposure } from '$lib/api';

  let exposures = $state<Exposure[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      exposures = await listExposures();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? exposures[0]?.id ?? null);
  const selected = $derived(exposures.find((e) => e.id === selectedId));

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-FR', { year: 'numeric', month: 'long', day: 'numeric' });
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
      case 'nouvelle': return 'Nouvelle';
      case 'en_suivi': return 'En suivi';
      case 'acceptee': return 'Acceptée';
      case 'reduite': return 'Réduite';
      default: return status;
    }
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Expositions"
    question="Qu’est-ce qui a été trouvé sur moi, et est-ce important ?"
    intro="Une exposition est une trace pertinente. Lisez le quoi et le pourquoi, puis ouvrez l’incident pour être guidé vers une action."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des expositions...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else if exposures.length === 0}
    <div class="glass-card"><p class="muted">Aucune exposition enregistrée dans la base.</p></div>
  {:else}
    <div class="split-layout">
      <!-- Liste des expositions -->
      <div class="glass-card list-panel">
        <h2>Traces relevées</h2>
        <ul class="item-list">
          {#each exposures as exp (exp.id)}
            <li>
              <a class="list-item" class:active={selectedId === exp.id} href={`/expositions?id=${exp.id}`}>
                <div class="item-header">
                  <span class="item-title">{exp.title}</span>
                  <span class="badge" style={`color: ${getSeverityColor(exp.severity)}; border-color: ${getSeverityColor(exp.severity)};`}>{exp.severity}</span>
                </div>
                <p class="item-desc">{exp.kind} · Détecté le {formatDate(exp.discovered_at)}</p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <!-- Détail de l'exposition -->
      <div class="glass-card detail-panel">
        {#if selected}
          <div class="detail-header">
            <h3>{selected.title}</h3>
            <div class="badges">
              <span class="badge" style={`color: ${getSeverityColor(selected.severity)}; border-color: ${getSeverityColor(selected.severity)};`}>{selected.severity}</span>
              <span class="badge">{getStatusLabel(selected.status)}</span>
            </div>
          </div>
          
          <p class="summary">{selected.why}</p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Quoi</dt>
              <dd>{selected.what}</dd>
            </div>
            <div class="field">
              <dt>Source</dt>
              <dd>{selected.source}</dd>
            </div>
            <div class="field">
              <dt>Détecté le</dt>
              <dd>{formatDate(selected.discovered_at)}</dd>
            </div>
            <div class="field">
              <dt>Type</dt>
              <dd>{selected.kind}</dd>
            </div>
          </div>

          <div class="actions-section">
            <h4>Prochaine étape</h4>
            <p class="muted">Si cette exposition nécessite un suivi, ouvrez ou créez un incident associé pour lancer une action de remédiation.</p>
            <a href="/incidents" class="wf-btn primary" style="margin-top: 0.75rem;">
              Voir les incidents
            </a>
          </div>
        {:else}
          <p class="muted">Sélectionnez une exposition dans la liste.</p>
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
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
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
