<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import '$lib/workflow.css';
  import GuideHeader from '$lib/GuideHeader.svelte';
  import { listIdentities, listFolders, listExposures, createIdentity, updateIdentity, deleteIdentity, type Identity, type Folder, type Exposure } from '$lib/api';

  let identities = $state<Identity[]>([]);
  let folders = $state<Folder[]>([]);
  let exposures = $state<Exposure[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Form state
  let mode = $state<'view' | 'edit' | 'create'>('view');
  let formData = $state({
    id: '',
    label: '',
    kind: 'email',
    value: '',
    folder_id: '' as string | null,
    notes: '',
    address_line1: '',
    address_line2: '',
    city: '',
    postal_code: '',
    country: ''
  });

  onMount(async () => {
    try {
      [identities, folders, exposures] = await Promise.all([
        listIdentities(),
        listFolders(),
        listExposures()
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const selectedId = $derived($page.url.searchParams.get('id') ?? identities[0]?.id ?? null);
  const selected = $derived(identities.find((i) => i.id === selectedId));

  $effect(() => {
    if (selected && mode !== 'edit' && mode !== 'create') {
      formData = {
        id: selected.id,
        label: selected.label,
        kind: selected.kind,
        value: selected.value,
        folder_id: selected.folder_id,
        notes: selected.notes ?? '',
        address_line1: selected.address_line1 ?? '',
        address_line2: selected.address_line2 ?? '',
        city: selected.city ?? '',
        postal_code: selected.postal_code ?? '',
        country: selected.country ?? ''
      };
    }
  });

  function getFolderName(folderId: string | null): string {
    if (!folderId) return 'Aucun dossier';
    return folders.find(f => f.id === folderId)?.name ?? 'Inconnu';
  }

  function exposuresForIdentity(identity: Identity | undefined): Exposure[] {
    if (!identity || !identity.folder_id) return [];
    return exposures.filter(e => e.folder_id === identity.folder_id);
  }

  function startCreate() {
    mode = 'create';
    formData = {
      id: '',
      label: '',
      kind: 'email',
      value: '',
      folder_id: folders[0]?.id ?? null,
      notes: '',
      address_line1: '',
      address_line2: '',
      city: '',
      postal_code: '',
      country: ''
    };
  }

  function startEdit() {
    if (!selected) return;
    mode = 'edit';
    formData = {
      id: selected.id,
      label: selected.label,
      kind: selected.kind,
      value: selected.value,
      folder_id: selected.folder_id,
      notes: selected.notes ?? '',
      address_line1: selected.address_line1 ?? '',
      address_line2: selected.address_line2 ?? '',
      city: selected.city ?? '',
      postal_code: selected.postal_code ?? '',
      country: selected.country ?? ''
    };
  }

  function cancelEdit() {
    mode = 'view';
    if (selected) {
      formData = {
        id: selected.id,
        label: selected.label,
        kind: selected.kind,
        value: selected.value,
        folder_id: selected.folder_id,
        notes: selected.notes ?? '',
        address_line1: selected.address_line1 ?? '',
        address_line2: selected.address_line2 ?? '',
        city: selected.city ?? '',
        postal_code: selected.postal_code ?? '',
        country: selected.country ?? ''
      };
    }
  }

  async function saveIdentity() {
    try {
      const addr1 = formData.address_line1 || null;
      const addr2 = formData.address_line2 || null;
      const city = formData.city || null;
      const postal = formData.postal_code || null;
      const country = formData.country || null;
      const notes = formData.notes || null;

      if (mode === 'create') {
        const newIdentity = await createIdentity(
          formData.label,
          formData.kind,
          formData.value,
          formData.folder_id || null,
          notes,
          addr1,
          addr2,
          city,
          postal,
          country
        );
        identities = [...identities, newIdentity];
        // Select the new identity
        const newUrl = new URL(window.location.href);
        newUrl.searchParams.set('id', newIdentity.id);
        window.history.pushState({}, '', newUrl);
        mode = 'view';
      } else if (mode === 'edit') {
        await updateIdentity(
          formData.id,
          formData.label,
          formData.kind,
          formData.value,
          formData.folder_id || null,
          notes,
          addr1,
          addr2,
          city,
          postal,
          country
        );
        identities = identities.map(i => 
          i.id === formData.id 
            ? { 
                ...i, 
                ...formData, 
                folder_id: formData.folder_id || null, 
                notes: notes,
                address_line1: addr1,
                address_line2: addr2,
                city: city,
                postal_code: postal,
                country: country
              }
            : i
        );
        mode = 'view';
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function removeIdentity(id: string) {
    if (!confirm('Supprimer cette identité ?')) return;
    try {
      await deleteIdentity(id);
      identities = identities.filter(i => i.id !== id);
      // Clear selection if deleted
      const newUrl = new URL(window.location.href);
      newUrl.searchParams.delete('id');
      window.history.pushState({}, '', newUrl);
    } catch (e) {
      error = String(e);
    }
  }

  function formatAddress(identity: Identity): string {
    const parts = [
      identity.address_line1,
      identity.address_line2,
      identity.postal_code ? `${identity.postal_code} ${identity.city}` : identity.city,
      identity.country
    ].filter(p => p && p.trim() !== '');
    return parts.join(', ');
  }
</script>

<section class="wf-view">
  <GuideHeader
    title="Identités"
    question="Quelles traces me concernent (noms, e-mails, profils, adresses) ?"
    intro="Inventaire de vos identifiants connus. Aucun mot de passe ni secret n’est stocké."
  />

  {#if loading}
    <div class="glass-card"><p class="muted">Chargement des identités...</p></div>
  {:else if error}
    <div class="glass-card"><p class="error">Erreur: {error}</p></div>
  {:else}
    <div class="split-layout">
      <div class="glass-card list-panel">
        <div class="list-header">
          <h2>Liste</h2>
          <button class="wf-btn primary small" onclick={startCreate}>+ Nouvelle</button>
        </div>
        <ul class="item-list">
          {#each identities as idn (idn.id)}
            <li>
              <a class="list-item" class:active={selectedId === idn.id} href={`/identites?id=${idn.id}`}>
                <span class="item-title">{idn.label}</span>
                <p class="item-desc">{idn.kind} · {idn.value}</p>
              </a>
            </li>
          {/each}
        </ul>
      </div>

      <div class="glass-card detail-panel">
        {#if mode === 'view' && selected}
          <div class="detail-header">
            <h3>{selected.label}</h3>
            <span class="badge">{selected.kind}</span>
          </div>
          
          <p class="summary">{selected.value}</p>
          
          <div class="detail-grid">
            <div class="field">
              <dt>Dossier</dt>
              <dd>{getFolderName(selected.folder_id)}</dd>
            </div>
            {#if selected.kind === 'adresse' && formatAddress(selected)}
              <div class="field full-width">
                <dt>Adresse complète</dt>
                <dd>{formatAddress(selected)}</dd>
              </div>
            {/if}
            {#if selected.notes}
              <div class="field full-width">
                <dt>Notes</dt>
                <dd>{selected.notes}</dd>
              </div>
            {/if}
          </div>

          <div class="actions-section">
            <h4>Expositions liées (même dossier)</h4>
            <ul class="sub-list">
              {#each exposuresForIdentity(selected) as exp (exp.id)}
                <li>
                  <a href={`/expositions?id=${exp.id}`} class="sub-link">
                    {exp.title} <span class="muted">({exp.severity})</span>
                  </a>
                </li>
              {:else}
                <li class="muted">Aucune exposition dans ce dossier.</li>
              {/each}
            </ul>
          </div>

          <div class="actions-section">
            <div class="action-buttons">
              <button class="wf-btn" onclick={startEdit}>Modifier</button>
              <button class="wf-btn danger" onclick={() => removeIdentity(selected.id)}>Supprimer</button>
            </div>
          </div>

        {:else if mode === 'edit' || mode === 'create'}
          <div class="detail-header">
            <h3>{mode === 'create' ? 'Nouvelle identité' : 'Modifier l\'identité'}</h3>
          </div>

          <form onsubmit={(e) => { e.preventDefault(); saveIdentity(); }} class="identity-form">
            <div class="form-field">
              <label for="label">Label</label>
              <input id="label" type="text" bind:value={formData.label} required />
            </div>

            <div class="form-field">
              <label for="kind">Type</label>
              <select id="kind" bind:value={formData.kind}>
                <option value="nom">Nom</option>
                <option value="email">E-mail</option>
                <option value="telephone">Téléphone</option>
                <option value="pseudo">Pseudo</option>
                <option value="domaine">Domaine</option>
                <option value="url">URL / Profil</option>
                <option value="adresse">Adresse postale</option>
              </select>
            </div>

            <div class="form-field">
              <label for="value">Valeur (résumé)</label>
              <input id="value" type="text" bind:value={formData.value} required />
              <p class="muted" style="font-size: 0.75rem; margin-top: 0.25rem;">Pour une adresse, indiquez par exemple "12 rue de Paris, 75001".</p>
            </div>

            {#if formData.kind === 'adresse'}
              <div class="form-grid">
                <div class="form-field full-width">
                  <label for="address_line1">Adresse ligne 1</label>
                  <input id="address_line1" type="text" bind:value={formData.address_line1} />
                </div>
                <div class="form-field full-width">
                  <label for="address_line2">Adresse ligne 2 (optionnel)</label>
                  <input id="address_line2" type="text" bind:value={formData.address_line2} />
                </div>
                <div class="form-field">
                  <label for="city">Ville</label>
                  <input id="city" type="text" bind:value={formData.city} />
                </div>
                <div class="form-field">
                  <label for="postal_code">Code postal</label>
                  <input id="postal_code" type="text" bind:value={formData.postal_code} />
                </div>
                <div class="form-field full-width">
                  <label for="country">Pays</label>
                  <input id="country" type="text" bind:value={formData.country} />
                </div>
              </div>
            {/if}

            <div class="form-field">
              <label for="folder_id">Dossier</label>
              <select id="folder_id" bind:value={formData.folder_id}>
                <option value="">Aucun dossier</option>
                {#each folders as folder (folder.id)}
                  <option value={folder.id}>{folder.name}</option>
                {/each}
              </select>
            </div>

            <div class="form-field">
              <label for="notes">Notes</label>
              <textarea id="notes" bind:value={formData.notes}></textarea>
            </div>

            <div class="action-buttons">
              <button type="submit" class="wf-btn primary">Enregistrer</button>
              <button type="button" class="wf-btn" onclick={cancelEdit}>Annuler</button>
            </div>
          </form>
        {:else}
          <p class="muted">Sélectionnez une identité ou créez-en une nouvelle.</p>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .muted { color: var(--mantis-text-muted); font-size: 0.85rem; }
  .error { color: var(--mantis-danger); font-size: 0.85rem; }

  .split-layout {
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 1.25rem;
  }

  @media (max-width: 900px) {
    .split-layout {
      grid-template-columns: 1fr;
    }
  }

  .list-panel h2 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .item-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .list-item {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.75rem;
    border: 1px solid var(--mantis-border);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
    text-decoration: none;
    color: inherit;
    transition: border-color 0.12s;
  }

  .list-item:hover {
    border-color: var(--mantis-accent);
  }

  .list-item.active {
    border-color: var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 10%, transparent);
  }

  .item-title {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .item-desc {
    margin: 0;
    font-size: 0.75rem;
    color: var(--mantis-text-muted);
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
    gap: 1rem;
  }

  .detail-header h3 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  .summary {
    margin: 0 0 1.25rem;
    padding: 0.75rem 1rem;
    border-left: 3px solid var(--mantis-accent);
    background: color-mix(in srgb, var(--mantis-accent) 5%, transparent);
    border-radius: 0 6px 6px 0;
    font-size: 0.9rem;
    line-height: 1.5;
    font-family: 'JetBrains Mono', monospace;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  .field.full-width {
    grid-column: 1 / -1;
  }

  @media (max-width: 700px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }
  }

  .field {
    margin: 0;
  }

  .field dt {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--mantis-text-muted);
    margin-bottom: 0.3rem;
  }

  .field dd {
    margin: 0;
    font-size: 0.88rem;
    color: var(--mantis-text);
  }

  .actions-section {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--mantis-border);
  }

  .actions-section h4 {
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .sub-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .sub-link {
    display: block;
    padding: 0.6rem 0.8rem;
    border: 1px solid var(--mantis-border);
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.2);
    color: var(--mantis-text);
    text-decoration: none;
    font-size: 0.88rem;
    transition: border-color 0.12s;
  }

  .sub-link:hover {
    border-color: var(--mantis-accent);
  }

  .badge {
    display: inline-block;
    padding: 0.15rem 0.45rem;
    border: 1px solid var(--mantis-border);
    border-radius: 4px;
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    align-self: flex-start;
  }

  .action-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .wf-btn.danger {
    background: var(--mantis-danger);
    border-color: var(--mantis-danger);
    color: #fff;
  }

  .wf-btn.small {
    padding: 0.35rem 0.75rem;
    font-size: 0.8rem;
  }

  /* Form styles */
  .identity-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .form-field.full-width {
    grid-column: 1 / -1;
  }

  .form-field label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--mantis-text-muted);
  }

  .form-field input,
  .form-field select,
  .form-field textarea {
    width: 100%;
    padding: 0.6rem 0.8rem;
    background: var(--mantis-bg);
    border: 1px solid var(--mantis-border-strong);
    border-radius: 6px;
    color: var(--mantis-text);
    font-family: inherit;
    font-size: 0.9rem;
  }

  .form-field input:focus,
  .form-field select:focus,
  .form-field textarea:focus {
    outline: none;
    border-color: var(--mantis-accent);
  }

  .form-field textarea {
    min-height: 80px;
    resize: vertical;
  }
</style>
