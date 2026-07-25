<script lang="ts">
  import { onMount } from 'svelte';
  import { getPostureScore, type PostureScore } from '$lib/api';

  let scoreData: PostureScore | null = null;
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      scoreData = await getPostureScore();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function getScoreColor(score: number): string {
    if (score >= 80) return 'text-green-400';
    if (score >= 50) return 'text-yellow-400';
    return 'text-red-400';
  }
</script>

<div class="p-4 bg-slate-800 rounded-lg shadow-md text-slate-200">
  <h2 class="text-xl font-bold mb-4 text-slate-100">Score de posture</h2>
  
  {#if loading}
    <p class="text-slate-400">Calcul du score...</p>
  {:else if error}
    <p class="text-red-400">Erreur: {error}</p>
  {:else if scoreData}
    <div class="flex flex-col items-center">
      <div class={`text-5xl font-bold ${getScoreColor(scoreData.score)}`}>
        {scoreData.score}
      </div>
      <div class="mt-4 w-full space-y-2 text-sm">
        <div class="flex justify-between bg-slate-900/50 p-2 rounded">
          <span class="text-slate-400">Incidents ouverts</span>
          <span class="font-semibold text-red-300">{scoreData.open_incidents}</span>
        </div>
        <div class="flex justify-between bg-slate-900/50 p-2 rounded">
          <span class="text-slate-400">Expositions élevées</span>
          <span class="font-semibold text-orange-300">{scoreData.high_exposures}</span>
        </div>
        <div class="flex justify-between bg-slate-900/50 p-2 rounded">
          <span class="text-slate-400">Actions terminées</span>
          <span class="font-semibold text-green-300">{scoreData.completed_actions}</span>
        </div>
      </div>
    </div>
  {/if}
</div>
