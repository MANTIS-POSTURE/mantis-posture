<script lang="ts">
  import { onMount } from 'svelte';
  import { getPostureScore, type PostureScore } from '$lib/api';

  let scoreData = $state<PostureScore | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

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
    if (score >= 80) return 'var(--mantis-ok)';
    if (score >= 50) return 'var(--mantis-warn)';
    return 'var(--mantis-danger)';
  }
</script>

<div class="card">
  <h2>Score de posture</h2>
  
  {#if loading}
    <p class="muted">Calcul du score...</p>
  {:else if error}
    <p class="error">Erreur: {error}</p>
  {:else if scoreData}
    <div class="score-container">
      <div class="score-main" style={`color: ${getScoreColor(scoreData.score)}`}>
        {scoreData.score}
      </div>
      <div class="metrics-grid">
        <div class="metric">
          <span class="metric-value" style="color: var(--mantis-danger);">{scoreData.open_incidents}</span>
          <span class="metric-label">Incidents ouverts</span>
        </div>
        <div class="metric">
          <span class="metric-value" style="color: #e67e22;">{scoreData.high_exposures}</span>
          <span class="metric-label">Expositions élevées</span>
        </div>
        <div class="metric">
          <span class="metric-value" style="color: var(--mantis-ok);">{scoreData.completed_actions}</span>
          <span class="metric-label">Actions terminées</span>
        </div>
      </div>
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

  .score-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
  }

  .score-main {
    font-size: 3.5rem;
    font-weight: 700;
    line-height: 1;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
    width: 100%;
  }

  .metric {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.75rem;
    background: var(--mantis-bg);
    border-radius: 6px;
    border: 1px solid var(--mantis-border);
  }

  .metric-value {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .metric-label {
    margin-top: 0.25rem;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--mantis-text-muted);
    text-align: center;
  }
</style>
