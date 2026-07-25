# MANTIS POSTURE — Journal technique

## Phase 1 — Socle local-first et cockpit UI

### Étape 1 à 9 : Socle et Mocks
- Mise en place Tauri 2 + SvelteKit.
- Thème sombre "cockpit" (graphite/ardoise) avec variables CSS.
- Création de toutes les vues sidebar en mode mock (`src/lib/mock/posture.ts`).
- Composants guidés (`GuideHeader`, `NextStepBar`, `ReadOnlyField`).

## Phase 2 — Modèle de données SQLite

### Étape 1 : Schéma et Backend Rust
- Création de `src-tauri/migrations/0001_init.sql` (tables, relations, index).
- Implémentation de `init_database` dans `lib.rs` (migrations idempotentes, seed de données).
- Création des commandes Tauri : `list_folders`, `list_incidents`, `list_actions`, `list_identities`, `list_exposures`, `list_rgpd_requests`, `list_timeline_entries`, `get_posture_score`.
- Mise à jour de `src/lib/api.ts` avec les types TypeScript correspondants.

### Étape 2 : Migration du Cockpit (`/posture`)
- Remplacement des données mock par des appels à l'API Tauri.
- Refonte du design avec `glass-card` et grid layout.
- Composants : `PostureScore`, `AlertList`, `IncidentList`, `ExposureList`, `Timeline`.

### Étape 3 : Migration de la vue Incidents (`/incidents`)
- Branchement sur `listIncidents` et `listActions`.
- Design split-layout (liste + détail) cohérent avec le cockpit.

### Étape 4 : Migration de la vue Expositions (`/expositions`)
- Branchement sur `listExposures`.
- Application du design split-layout avec `glass-card`.
- Affichage des détails (Quoi, Pourquoi, Source, Sévérité, Statut).
- Lien vers les incidents associés.

### Étape 5 : Migration de la vue Actions (`/actions`)
- Branchement sur `listActions`.
- Application du design split-layout avec `glass-card`.
- Affichage des détails (Priorité, Difficulté, Échéance, Étapes, Preuve attendue).
- Lien vers l'incident associé.
- Parsing du JSON `guidance` pour afficher les étapes numérotées.

### Étape 6 : Migration des vues Dossiers et Identités
- Branchement de `/dossiers` sur `listFolders`, `listIdentities`, `listExposures`, `listIncidents`.
- Branchement de `/identites` sur `listIdentities`, `listFolders`, `listExposures`.
- Application du design split-layout avec `glass-card`.
- Filtrage des entités liées par `folder_id`.

### Prochaines étapes
- [ ] Migrer `/dpo` sur SQLite.
- [ ] Débloquer l'édition (CRUD) pour les identités et dossiers.
