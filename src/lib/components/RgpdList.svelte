<script lang="ts">
  import { onMount } from 'svelte';
  import { listRgpdRequests, type RgpdRequest } from '$lib/api';

  let requests: RgpdRequest[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      requests = await listRgpdRequests();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-FR', { year: 'numeric', month: 'long', day: 'numeric' });
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case 'en cours':
        return 'bg-yellow-900/50 text-yellow-300 border-yellow-700';
      case 'terminé':
        return 'bg-green-900/50 text-green-300 border-green-700';
      case 'refusé':
        return 'bg-red-900/50 text-red-300 border-red-700';
      default:
        return 'bg-slate-700 text-slate-300 border-slate-600';
    }
  }
</script>

<div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
  <h2 class="text-xl font-bold mb-4 text-slate-100">Demandes RGPD</h2>
  
  {#if loading}
    <p class="text-slate-400">Chargement...</p>
  {:else if error}
    <p class="text-red-400">Erreur: {error}</p>
  {:else if requests.length === 0}
    <p class="text-slate-400">Aucune demande RGPD enregistrée.</p>
  {:else}
    <div class="space-y-3">
      {#each requests as req (req.id)}
        <div class="border border-slate-700 rounded-md p-3 bg-slate-900/50">
          <div class="flex justify-between items-start mb-2">
            <div>
              <h3 class="text-md font-semibold text-slate-100">{req.target_entity}</h3>
              <p class="text-xs text-slate-500">Créée le {formatDate(req.created_at)}</p>
            </div>
            <span class="px-2 py-1 text-xs rounded-full border {getStatusColor(req.status)}">
              {req.status}
            </span>
          </div>
          <div class="text-sm text-slate-400 mb-2">
            <span class="font-medium text-slate-300">Type:</span> {req.request_type}
          </div>
          {#if req.notes}
            <p class="text-sm text-slate-400 italic">"{req.notes}"</p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
