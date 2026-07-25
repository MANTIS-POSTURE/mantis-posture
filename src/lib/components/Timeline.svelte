<script lang="ts">
  import { onMount } from 'svelte';
  import { listTimelineEntries, type TimelineEntry } from '$lib/api';

  let entries = $state<TimelineEntry[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      entries = await listTimelineEntries();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleString('fr-FR', { 
      year: 'numeric', 
      month: 'short', 
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function getEventColor(eventType: string): string {
    switch (eventType.toLowerCase()) {
      case 'détection':
        return '#e67e22';
      case 'action':
        return 'var(--mantis-accent)';
      case 'rgpd':
        return 'var(--mantis-warn)';
      default:
        return 'var(--mantis-text-muted)';
    }
  }
</script>

<div class="card">
  <h2>Chronologie</h2>
  
  {#if loading}
    <p class="muted">Chargement...</p>
  {:else if error}
    <p class="error">Erreur: {error}</p>
  {:else if entries.length === 0}
    <p class="muted">Aucun événement enregistré.</p>
  {:else}
    <div class="timeline">
      {#each entries as entry (entry.id)}
        <div class="entry">
          <div class="dot" style={`background: ${getEventColor(entry.event_type)};`}></div>
          <div class="content">
            <span class="date">{formatDate(entry.created_at)}</span>
            <h3>{entry.event_type}</h3>
            <p>{entry.description}</p>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .card {
    background: var(--mantis-bg-raised);
    border: 1px solid var(--mantis-border);
    border-radius: 10px;
    padding: 1.25rem;
  }

  h2 {
    margin: 0 0 1rem;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .timeline {
    position: relative;
    border-left: 1px solid var(--mantis-border);
    padding-left: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .entry {
    position: relative;
  }

  .dot {
    position: absolute;
    left: -1.55rem;
    top: 0.25rem;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 2px solid var(--mantis-bg-raised);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .date {
    font-size: 0.75rem;
    color: var(--mantis-text-muted);
  }

  h3 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
  }

  p {
    margin: 0;
    font-size: 0.85rem;
    color: var(--mantis-text-muted);
  }
</style>
