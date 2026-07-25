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
</script>

<div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
  <h2 class="text-xl font-bold mb-4 text-slate-100">Expositions</h2>
  
  {#if loading}
    <p class="text-slate-400">Chargement...</p>
  {:else if error}
    <p class="text-red-400">Erreur: {error}</p>
  {:else if exposures.length === 0}
    <p class="text-slate-400">Aucune exposition enregistrée.</p>
  {:else}
    <div class="space-y-3">
      {#each exposures as exposure (exposure.id)}
        <div class="border border-slate-700 rounded-md p-3 bg-slate-900/50">
          <div class="flex justify-between items-start mb-2">
            <div>
              <h3 class="text-md font-semibold text-slate-100">{exposure.title}</h3>
              <p class="text-xs text-slate-500">Détecté le {formatDate(exposure.discovered_at)}</p>
            </div>
            <span class="px-2 py-1 text-xs rounded-full bg-orange-900/50 text-orange-300 border border-orange-700">
              {exposure.severity}
            </span>
          </div>
          <p class="text-sm text-slate-400">{exposure.what}</p>
        </div>
      {/each}
    </div>
  {/if}
</div>
