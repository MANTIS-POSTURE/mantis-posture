<script lang="ts">
  import { onMount } from 'svelte';
  import { listIdentities, type Identity } from '$lib/api';

  let identities = $state<Identity[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      identities = await listIdentities();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="glass-card">
  <h2>Identités</h2>
  
  {#if loading}
    <p class="muted">Chargement...</p>
  {:else if error}
    <p class="error">Erreur: {error}</p>
  {:else if identities.length === 0}
    <p class="muted">Aucune identité enregistrée.</p>
  {:else}
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>Type</th>
            <th>Valeur</th>
            <th>Label</th>
          </tr>
        </thead>
        <tbody>
          {#each identities as identity (identity.id)}
            <tr>
              <td class="capitalize">{identity.kind}</td>
              <td class="mono">{identity.value}</td>
              <td class="muted-text">{identity.label || '-'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .table-container {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
    text-align: left;
  }

  th {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--mantis-text-muted);
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--mantis-border);
  }

  td {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--mantis-border);
    color: var(--mantis-text);
  }

  tr:last-child td {
    border-bottom: none;
  }

  .capitalize { text-transform: capitalize; }
  .mono { font-family: 'JetBrains Mono', monospace; color: var(--mantis-text); }
  .muted-text { color: var(--mantis-text-muted); }
</style>
