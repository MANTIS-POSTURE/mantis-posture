<script lang="ts">
  // Les alertes sont actuellement dérivées manuellement pour le cockpit.
  // Plus tard, elles pourront être calculées côté Rust ou SQL.
  const alerts = [
    {
      level: 'critical',
      title: 'Incident ouvert',
      message: 'Fuite d\'email détectée avec une sévérité élevée.',
      link: '/incidents?id=inc-leaks',
      linkLabel: 'Voir l\'incident'
    },
    {
      level: 'high',
      title: 'Exposition élevée',
      message: 'Email et mot de passe haché exposés dans Collection #1.',
      link: '/expositions?id=exp-leak-2019',
      linkLabel: 'Voir l\'exposition'
    },
    {
      level: 'moderate',
      title: 'Demande RGPD en cours',
      message: 'Demande d\'effacement envoyée à Réseau Social X.',
      link: '/dpo?id=rgpd-1',
      linkLabel: 'Ouvrir la demande'
    }
  ];

  function getColors(level: string) {
    switch (level) {
      case 'critical':
        return { border: 'var(--mantis-danger)', bg: 'color-mix(in srgb, var(--mantis-danger) 15%, transparent)', text: 'var(--mantis-danger)' };
      case 'high':
        return { border: '#e67e22', bg: 'color-mix(in srgb, #e67e22 15%, transparent)', text: '#e67e22' };
      case 'moderate':
        return { border: 'var(--mantis-warn)', bg: 'color-mix(in srgb, var(--mantis-warn) 15%, transparent)', text: 'var(--mantis-warn)' };
      default:
        return { border: 'var(--mantis-accent)', bg: 'color-mix(in srgb, var(--mantis-accent) 15%, transparent)', text: 'var(--mantis-accent)' };
    }
  }
</script>

<div class="card">
  <h2>Alertes</h2>
  <div class="alert-list">
    {#each alerts as alert}
      {@const c = getColors(alert.level)}
      <div class="alert-item" style={`border-left: 4px solid ${c.border}; background: ${c.bg};`}>
        <div class="alert-content">
          <p class="alert-title" style={`color: ${c.text}`}>{alert.title}</p>
          <p class="alert-msg">{alert.message}</p>
        </div>
        <a href={alert.link} class="alert-btn" style={`color: ${c.text}; border-color: ${c.border};`}>
          {alert.linkLabel}
        </a>
      </div>
    {/each}
  </div>
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

  .alert-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .alert-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
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
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    text-decoration: none;
    transition: opacity 0.12s;
  }

  .alert-btn:hover {
    opacity: 0.8;
  }
</style>
