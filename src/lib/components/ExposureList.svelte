<script lang="ts">
  import { onMount } from 'svelte';
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
</script>

<div class="glass-card">
  <h2>Expositions</h2>
  
  {#if loading}
    <p class="muted">Chargement...</p>
  {:else if error}
    <p class="error">Erreur: {error}</p>
  {:else if exposures.length === 0}
    <p class="muted">Aucune exposition enregistrée.</p>
  {:else}
    <div class="list">
      {#each exposures as exposure (exposure.id)}
        <a class="item" href={`/expositions?id=${exposure.id}`}>
          <div class="item-header">
            <div>
              <h3>{exposure.title}</h3>
              <p class="date">Détecté le {formatDate(exposure.discovered_at)}</p>
            </div>
            <span class="badge" style={`color: ${getSeverityColor(exposure.severity)}; border-color: ${getSeverityColor(exposure.severity)};`}>
              {exposure.severity}
            </span>
          </div>
          <p class="item-desc">{exposure.what}</p>
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
    margin: 0 0 0.15rem;
    font-size: 1rem;
    font-weight: 600;
  }

  .date {
    margin: 0;
    font-size: 0.75rem;
    color: var(--mantis-text-muted);
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
</style>
