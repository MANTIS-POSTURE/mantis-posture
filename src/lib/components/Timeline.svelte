<script lang="ts">
  import { onMount } from 'svelte';
  import { listTimelineEntries, type TimelineEntry } from '$lib/api';
	import StatePanel from './StatePanel.svelte';
  import { t } from '$lib/i18n';

  let entries = $state<TimelineEntry[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  const recentEntries = $derived(entries.slice(0, 5));

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
        return 'var(--ui-warning)';
      case 'action':
        return 'var(--mantis-accent)';
      case 'rgpd':
        return 'var(--mantis-warn)';
      default:
        return 'var(--mantis-text-muted)';
    }
  }
</script>

<div class="glass-card">
  <p class="eyebrow">{t('Activité')}</p><h2>{t('Événements récents')}</h2>
  
  {#if loading}
    <StatePanel compact tone="info" title="Chargement de la chronologie" />
  {:else if error}
    <StatePanel compact tone="danger" title="Chronologie indisponible" message={error} />
  {:else if entries.length === 0}
    <StatePanel compact title={t('Aucun événement enregistré')} message={t('Les scans et décisions alimenteront cette chronologie.')} />
  {:else}
    <div class="timeline">
      {#each recentEntries as entry (entry.id)}
        <div class="entry">
          <div class="dot" style={`background: ${getEventColor(entry.event_type)};`}></div>
          <div class="content">
            <span class="date">{formatDate(entry.created_at)}</span>
            <h3>{entry.event_type}</h3>
            <p>{t(entry.description)}</p>
          </div>
        </div>
      {/each}
    </div>
	{#if entries.length > recentEntries.length}<a class="timeline-link" href="/graphe">{t('Voir l’historique complet →')}</a>{/if}
  {/if}
</div>

<style>
  .timeline {
    position: relative;
    border-left: 1px solid var(--mantis-border);
    padding-left: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .entry {
    position: relative;
  }

  .dot {
    position: absolute;
    left: -1.6rem;
    top: 0.25rem;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 2px solid var(--mantis-bg-solid);
    box-shadow: 0 0 8px currentColor;
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
	.eyebrow { margin:0 0 .2rem; color:var(--ui-link); text-transform:uppercase; }.glass-card>h2 { margin-bottom:1rem; }.timeline-link { display:inline-block; margin-top:1rem; color:var(--ui-link); font-size:.78rem; font-weight:620; text-decoration:none; }
</style>
