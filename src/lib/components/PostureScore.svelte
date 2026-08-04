<script lang="ts">
  import { onMount } from 'svelte';
  import { getPostureScore, type PostureScore } from '$lib/api';
  import StatePanel from './StatePanel.svelte';
  import { t } from '$lib/i18n';
  import { activeIdentityId } from '$lib/active-identity';

  let scoreData = $state<PostureScore | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let requestId = 0;

  async function loadScore(identityId: string | null) {
    const request = ++requestId;
    loading = true;
    error = null;
    scoreData = null;
    try {
      scoreData = await getPostureScore(identityId);
    } catch (e) {
      if (request === requestId) error = String(e);
    } finally {
      if (request === requestId) loading = false;
    }
  }

  onMount(() => activeIdentityId.subscribe((identityId) => { void loadScore(identityId); }));

  function scoreTone(score: number) {
    if (score >= 90) return { color: 'var(--mantis-ok)', label: t('Solide') };
    if (score >= 70) return { color: 'var(--ui-warning)', label: t('À consolider') };
    if (score >= 40) return { color: 'var(--mantis-danger)', label: t('Fragile') };
    return { color: 'var(--mantis-danger)', label: t('Critique') };
  }

  function getScoreInsight(score: number): string {
    if (score < 40) return t('Des actions urgentes sont nécessaires.');
    if (score < 70) return t('Plusieurs risques méritent un examen prioritaire.');
    if (score < 90) return t('La situation est maîtrisée, avec quelques actions à finaliser.');
    return t('La situation est solide. Conservez ce rythme de vérification.');
  }

  function scoreProgress(score: number) { return Math.max(0, Math.min(100, score)); }
</script>

<section class="glass-card posture-instrument" aria-labelledby="posture-score-title">
  <header class="instrument-header">
    <div>
      <p class="eyebrow">{t('Posture')}</p>
      <h2 id="posture-score-title">{t('Indice de situation')}</h2>
    </div>
    {#if scoreData?.score !== null && scoreData}
      {@const tone = scoreTone(scoreData.score)}
      <span class="score-status" style={`--score-color:${tone.color}`}><i></i>{tone.label}</span>
    {/if}
  </header>

  {#if loading}
    <StatePanel compact tone="info" title={t('Calcul de la posture')} message={t('Agrégation locale des incidents, expositions et actions…')} />
  {:else if error}
    <StatePanel compact tone="danger" title={t('Score indisponible')} message={error} />
  {:else if scoreData && scoreData.score === null}
    <StatePanel compact tone="info" title={t('Score indisponible pour le moment')} message={t('Lancez un premier scan pour calculer la posture à partir de données observées.')} />
  {:else if scoreData && scoreData.score !== null}
    {@const tone = scoreTone(scoreData.score)}
    <div class="instrument-body">
      <div class="score-dial" style={`--score-color:${tone.color};--score-progress:${scoreProgress(scoreData.score)}%`} role="img" aria-label={`${t('Indice de posture')} : ${scoreData.score} ${t('sur 100')}, ${tone.label}`}>
        <div class="dial-ticks" aria-hidden="true"></div>
        <div class="dial-face">
          <span class="score-number">{scoreData.score}</span>
          <span class="score-unit">{t('sur 100')}</span>
        </div>
      </div>

      <div class="instrument-reading">
        <p class="reading-label">{t('Lecture actuelle')}</p>
        <h3 style={`color:${tone.color}`}>{tone.label}</h3>
        <p class="score-insight">{getScoreInsight(scoreData.score)}</p>
        <p class="score-method">{t('Indice local pondéré : chaque gravité compte progressivement ; les actions terminées apportent une mitigation plafonnée.')}</p>
      </div>
    </div>

    <dl class="score-factors" aria-label={t('Facteurs qui composent la posture')}>
      <div class="factor factor-danger"><dt>{t('Incidents ouverts')}</dt><dd>{scoreData.open_incidents}</dd></div>
      <div class="factor factor-warning"><dt>{t('Expositions élevées')}</dt><dd>{scoreData.high_exposures}</dd></div>
      <div class="factor factor-success"><dt>{t('Actions terminées')}</dt><dd>{scoreData.completed_actions}</dd></div>
    </dl>
  {/if}
</section>

<style>
  .posture-instrument { position:relative; isolation:isolate; overflow:hidden; min-height:100%; padding:1.1rem; container-type:inline-size; }
  .posture-instrument::before { content:''; position:absolute; z-index:-1; inset:0; pointer-events:none; background:radial-gradient(circle at 22% 48%,color-mix(in srgb,var(--ui-accent) 13%,transparent),transparent 34%),linear-gradient(135deg,color-mix(in srgb,var(--ui-accent) 5%,transparent),transparent 56%); }
  .instrument-header { display:flex; align-items:flex-start; justify-content:space-between; gap:1rem; }
  .eyebrow { margin:0 0 .18rem; color:var(--ui-accent); font:650 .63rem/1 var(--font-meta); letter-spacing:.14em; text-transform:uppercase; }
  h2 { margin:0; font-size:1.05rem; letter-spacing:-.02em; }
  .score-status { display:inline-flex; align-items:center; gap:.4rem; flex:0 0 auto; margin-top:.05rem; padding:.32rem .5rem; border:1px solid color-mix(in srgb,var(--score-color) 40%,var(--ui-border-default)); border-radius:var(--radius-pill); color:var(--score-color); background:color-mix(in srgb,var(--score-color) 9%,transparent); font:650 .61rem/1 var(--font-meta); letter-spacing:.06em; text-transform:uppercase; }
  .score-status i { width:6px; height:6px; border-radius:50%; background:currentColor; box-shadow:0 0 9px currentColor; }
  .instrument-body { display:grid; grid-template-columns:minmax(160px,.83fr) minmax(0,1.17fr); align-items:center; gap:clamp(.8rem,2vw,1.7rem); margin:1rem 0; }
  .score-dial { position:relative; display:grid; place-items:center; width:min(100%,218px); aspect-ratio:1; margin:auto; border-radius:50%; background:conic-gradient(from 218deg,var(--score-color) 0 var(--score-progress),rgba(255,255,255,.09) var(--score-progress) 78%,transparent 78% 100%); filter:drop-shadow(0 0 18px color-mix(in srgb,var(--score-color) 16%,transparent)); }
  .score-dial::before { content:''; position:absolute; inset:8px; border-radius:50%; background:radial-gradient(circle at 37% 25%,rgba(255,255,255,.12),transparent 25%),radial-gradient(circle,var(--ui-material-panel),var(--ui-material-solid) 70%); box-shadow:inset 0 1px 0 var(--ui-rim-light),inset 0 -18px 25px rgba(0,0,0,.55),0 8px 22px rgba(0,0,0,.38); }
  .score-dial::after { content:''; position:absolute; inset:20%; border:1px solid color-mix(in srgb,var(--score-color) 28%,transparent); border-radius:50%; box-shadow:inset 0 0 20px color-mix(in srgb,var(--score-color) 8%,transparent); }
  .dial-ticks { position:absolute; z-index:1; inset:4px; border-radius:50%; background:repeating-conic-gradient(from 218deg,rgba(255,255,255,.54) 0deg 1deg,transparent 1deg 9deg,transparent 9deg 10deg); -webkit-mask:radial-gradient(transparent 0 79%,#000 80% 83%,transparent 84%); mask:radial-gradient(transparent 0 79%,#000 80% 83%,transparent 84%); opacity:.55; }
  .dial-face { position:relative; z-index:2; display:grid; justify-items:center; gap:.35rem; }
  .score-number { color:var(--ui-text-primary); font:500 clamp(3.6rem,6vw,5.1rem)/.8 var(--font-display); letter-spacing:-.09em; text-shadow:0 0 22px color-mix(in srgb,var(--score-color) 20%,transparent); }
  .score-unit,.reading-label { color:var(--ui-text-tertiary); font:600 .58rem/1 var(--font-meta); letter-spacing:.14em; text-transform:uppercase; }
  .instrument-reading { min-width:0; }.reading-label { margin:0 0 .35rem; }.instrument-reading h3 { margin:0; font-size:clamp(1.25rem,2vw,1.65rem); letter-spacing:-.04em; }.score-insight { max-width:34ch; margin:.55rem 0 0; color:var(--ui-text-secondary); font-size:.82rem; line-height:1.55; }.score-method { margin:.85rem 0 0; color:var(--ui-text-tertiary); font-size:.68rem; line-height:1.45; }
  .score-factors { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:.5rem; margin:0; padding-top:.85rem; border-top:1px solid var(--ui-border-subtle); }.factor { position:relative; min-width:0; padding:.15rem .55rem .15rem .65rem; border-left:2px solid var(--ui-border-strong); }.factor-danger { border-color:var(--mantis-danger); }.factor-warning { border-color:var(--ui-warning); }.factor-success { border-color:var(--mantis-ok); }.factor dt { overflow:hidden; color:var(--ui-text-tertiary); font-size:.62rem; line-height:1.25; text-overflow:ellipsis; white-space:nowrap; }.factor dd { margin:.28rem 0 0; color:var(--ui-text-primary); font:650 1.35rem/1 var(--font-display); }.factor-danger dd { color:var(--mantis-danger); }.factor-warning dd { color:var(--ui-warning); }.factor-success dd { color:var(--mantis-ok); }
  @container (max-width:460px) { .instrument-body { grid-template-columns:1fr; }.score-dial { width:190px; }.instrument-reading { text-align:center; }.score-insight { margin-inline:auto; }.score-factors { gap:.25rem; }.factor { padding-inline:.4rem; }.factor dt { font-size:.55rem; }.factor dd { font-size:1.15rem; } }
  @media (prefers-reduced-motion:reduce) { .score-dial { filter:none; } }
</style>
