<script lang="ts">
  import { onMount } from 'svelte';
  import { listIncidents, listExposures, listRgpdRequests, type Incident, type Exposure, type RgpdRequest } from '$lib/api';
	import StatePanel from './StatePanel.svelte';
  import { t } from '$lib/i18n';

  let incidents = $state<Incident[]>([]);
  let exposures = $state<Exposure[]>([]);
  let rgpdRequests = $state<RgpdRequest[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  const MAX_DISPLAYED_ALERTS = 4;

  type Alert = {
    level: 'critical' | 'high' | 'moderate' | 'info';
    title: string;
    message: string;
    link: string;
    linkLabel: string;
    date: string;
  };

  function getAlerts(): Alert[] {
    const result: Alert[] = [];

    incidents.forEach(inc => {
      let level: Alert['level'] = 'info';
      if (inc.severity === 'critique') level = 'critical';
      else if (inc.severity === 'élevée') level = 'high';
      else if (inc.severity === 'modérée') level = 'moderate';

      result.push({
        level,
        title: `Incident : ${inc.title}`,
        message: inc.what,
        link: `/incidents?id=${inc.id}`,
        linkLabel: t('Voir l’incident'),
        date: inc.discovered_at
      });
    });

    exposures.forEach(exp => {
      let level: Alert['level'] = 'info';
      if (exp.severity === 'critique') level = 'critical';
      else if (exp.severity === 'élevée') level = 'high';
      else if (exp.severity === 'modérée') level = 'moderate';

      result.push({
        level,
        title: `Exposition : ${exp.title}`,
        message: exp.what,
        link: `/expositions?id=${exp.id}`,
        linkLabel: t('Voir l’exposition'),
        date: exp.discovered_at
      });
    });

    rgpdRequests.forEach(req => {
      if (req.status_id !== 'status_004') {
        result.push({
          level: 'moderate',
          title: `Demande RGPD : ${req.target}`,
          message: `${req.type_id} — ${req.target}`,
          link: `/dpo?id=${req.id}`,
          linkLabel: t('Ouvrir la demande'),
          date: 'Demande RGPD'
        });
      }
    });

    // Tri : sévérité d'abord (critical > high > moderate > info), puis date la plus récente en premier
    const severityWeight: Record<Alert['level'], number> = {
      critical: 0,
      high: 1,
      moderate: 2,
      info: 3
    };

    result.sort((a, b) => {
      const swDiff = severityWeight[a.level] - severityWeight[b.level];
      if (swDiff !== 0) return swDiff;
      // Dates les plus récentes en premier (tri décroissant)
      return b.date.localeCompare(a.date);
    });

    return result;
  }

  const allAlerts = $derived(getAlerts());
  const displayedAlerts = $derived(allAlerts.slice(0, MAX_DISPLAYED_ALERTS));
  const remainingCount = $derived(allAlerts.length - MAX_DISPLAYED_ALERTS);

  function getColors(level: string) {
    switch (level) {
      case 'critical':
        return { border: 'var(--mantis-danger)', bg: 'color-mix(in srgb, var(--mantis-danger) 15%, transparent)', text: 'var(--mantis-danger)' };
      case 'high':
        return { border: 'var(--ui-warning)', bg: 'color-mix(in srgb, var(--ui-warning) 9%, transparent)', text: 'var(--ui-warning)' };
      case 'moderate':
        return { border: 'var(--mantis-warn)', bg: 'color-mix(in srgb, var(--mantis-warn) 15%, transparent)', text: 'var(--mantis-warn)' };
      default:
        return { border: 'var(--mantis-accent)', bg: 'color-mix(in srgb, var(--mantis-accent) 15%, transparent)', text: 'var(--mantis-accent)' };
    }
  }

  onMount(async () => {
    try {
      [incidents, exposures, rgpdRequests] = await Promise.all([
        listIncidents(),
        listExposures(),
        listRgpdRequests()
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="glass-card">
  <div class="alert-header">
    <div><p class="eyebrow">{t('Priorité')}</p><h2>{t('À examiner maintenant')}</h2></div>
    {#if !loading && allAlerts.length > 0}
      <span class="alert-count">{allAlerts.length} alerte{allAlerts.length > 1 ? 's' : ''}</span>
    {/if}
  </div>

  {#if loading}
    <StatePanel compact tone="info" title={t('Chargement des alertes')} message={t('Priorisation locale des signaux en cours…')} />
  {:else if error}
    <StatePanel compact tone="danger" title={t('Alertes indisponibles')} message={error} />
  {:else if allAlerts.length === 0}
    <StatePanel compact tone="success" title={t('Aucune alerte active')} message={t('Les nouvelles détections importantes apparaîtront ici.')} />
  {:else}
    <div class="alert-list">
      {#each displayedAlerts as alert}
        {@const c = getColors(alert.level)}
        <div class="alert-item" style={`border-left: 4px solid ${c.border}; background: ${c.bg};`}>
          <div class="alert-content">
            <p class="alert-title" style={`color: ${c.text}`}>{t(alert.title)}</p>
            <p class="alert-msg">{alert.message}</p>
          </div>
          <a href={alert.link} class="alert-btn" style={`color: ${c.text}; border-color: ${c.border};`}>
            {alert.linkLabel}
          </a>
        </div>
      {/each}
    </div>

    {#if remainingCount > 0}
      <div class="alert-footer">
        <a href="/incidents" class="more-alerts-link">
          → Voir les {remainingCount} autre{remainingCount > 1 ? 's' : ''} alerte{remainingCount > 1 ? 's' : ''}
        </a>
      </div>
    {/if}
  {/if}
</div>

<style>
  .alert-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .alert-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
	.alert-header .eyebrow { margin:0 0 .2rem; color:var(--ui-warning); text-transform:uppercase; }

  .alert-count {
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    border-radius: 10px;
    background: var(--mantis-bg-raised);
    border: 1px solid var(--mantis-border);
    color: var(--mantis-text-muted);
    font-weight: 600;
  }

  .alert-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .alert-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.68rem 0.75rem;
    border-radius: 0 6px 6px 0;
  }

  .alert-content {
    flex: 1;
  }

  .alert-title {
    margin: 0 0 0.25rem;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .alert-msg {
    margin: 0;
    font-size: 0.82rem;
    color: var(--mantis-text-muted);
  }

  .alert-btn {
    flex-shrink: 0;
    padding: 0.35rem 0.65rem;
    border: 1px solid;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
    text-decoration: none;
    transition: opacity 0.12s;
    background: var(--ui-canvas);
  }

  .alert-btn:hover {
    opacity: 0.8;
  }

  .alert-footer {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--mantis-border);
    text-align: center;
  }

  .more-alerts-link {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--ui-link);
    text-decoration: none;
    transition: opacity 0.12s;
  }

  .more-alerts-link:hover {
    opacity: 0.8;
    text-decoration: underline;
  }

</style>
