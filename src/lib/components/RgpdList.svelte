<script lang="ts">
  import { onMount } from 'svelte';
  import { listRgpdRequests, type RgpdRequest } from '$lib/api';

  let requests = $state<RgpdRequest[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      requests = await listRgpdRequests();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function getStatusColor(status_id: string): string {
    switch (status_id) {
      case 'status_002': return 'var(--mantis-warn)';
      case 'status_003': return 'var(--mantis-accent)';
      case 'status_004': return 'var(--mantis-ok)';
      default: return 'var(--mantis-text-muted)';
    }
  }
</script>

<div class="glass-card">
  <h2>Demandes RGPD</h2>
  
  {#if loading}
    <p class="muted">Chargement...</p>
  {:else if error}
    <p class="error">Erreur: {error}</p>
  {:else if requests.length === 0}
    <p class="muted">Aucune demande RGPD enregistrée.</p>
  {:else}
    <div class="list">
      {#each requests as req (req.id)}
        <a class="item" href={`/dpo?id=${req.id}`}>
          <div class="item-header">
            <div>
              <h3>{req.target}</h3>
              <p class="sub">Type: {req.type_id}</p>
            </div>
            <span class="badge" style={`color: ${getStatusColor(req.status_id)}; border-color: ${getStatusColor(req.status_id)};`}>
              {req.status_id}
            </span>
          </div>
          <div class="contact">
            <span class="label">Contact DPO:</span> {req.dpo_contact}
          </div>
          {#if req.data_summary}
            <p class="summary">"{req.data_summary}"</p>
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
    margin: 0 0 0.15rem;
    font-size: 1rem;
    font-weight: 600;
  }

  .sub {
    margin: 0;
    font-size: 0.75rem;
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

  .contact {
    font-size: 0.85rem;
    color: var(--mantis-text-muted);
    margin-bottom: 0.5rem;
  }

  .label {
    color: var(--mantis-text);
    font-weight: 500;
  }

  .summary {
    margin: 0;
    font-size: 0.85rem;
    color: var(--mantis-text-muted);
    font-style: italic;
  }
</style>
