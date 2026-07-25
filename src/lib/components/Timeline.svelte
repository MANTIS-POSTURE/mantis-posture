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
        return 'bg-orange-500';
      case 'action':
        return 'bg-blue-500';
      case 'rgpd':
        return 'bg-purple-500';
      default:
        return 'bg-slate-500';
    }
  }
</script>

<div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
  <h2 class="text-xl font-bold mb-4 text-slate-100">Chronologie</h2>
  
  {#if loading}
    <p class="text-slate-400">Chargement...</p>
  {:else if error}
    <p class="text-red-400">Erreur: {error}</p>
  {:else if entries.length === 0}
    <p class="text-slate-400">Aucun événement enregistré.</p>
  {:else}
    <div class="relative border-l border-slate-700 pl-6 space-y-6">
      {#each entries as entry (entry.id)}
        <div class="relative">
          <div class="absolute -left-[31px] top-1 w-3 h-3 rounded-full {getEventColor(entry.event_type)} ring-2 ring-slate-800"></div>
          <div class="flex flex-col">
            <span class="text-xs text-slate-500">{formatDate(entry.created_at)}</span>
            <h3 class="text-sm font-semibold text-slate-200 mt-1">{entry.event_type}</h3>
            <p class="text-sm text-slate-400 mt-1">{entry.description}</p>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
