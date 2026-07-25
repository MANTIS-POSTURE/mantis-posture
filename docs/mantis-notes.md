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

### Étape 7 : Migration de la vue DPO (`/dpo`)
- Branchement sur `listRgpdRequests`.
- Application du design split-layout avec `glass-card`.
- Affichage des détails (Cible, Contact DPO, Type, Statut, Résumé, Brouillon).
- Bouton de copie du brouillon.
- Mise à jour locale du statut (non persisté en base pour l'instant).

### Étape 8 : Persistance des statuts (Actions & DPO)
- Ajout des commandes Rust `update_action_status` et `update_rgpd_request_status`.
- Ajout des fonctions API frontend correspondantes.
- Mise à jour de la vue `/actions` avec des boutons pour changer le statut (En cours, Faite) et appel à l'API.
- Mise à jour de la vue `/dpo` pour que les boutons de changement de statut appellent l'API et persistent en base.

### Étape 9 : CRUD Identités
- Ajout de la dépendance `uuid` dans `Cargo.toml`.
- Implémentation des commandes Rust `create_identity`, `update_identity`, `delete_identity`.
- Ajout des fonctions API frontend correspondantes.
- Refonte de l'UI `/identites` pour inclure un formulaire de création/édition dans le panneau de détail.
- Gestion des modes "view" et "edit" dans la page Svelte.

### Étape 10 : Adresses structurées & Modules OSINT
- **Schéma SQL** : Ajout des champs `address_line1`, `address_line2`, `city`, `postal_code`, `country` à la table `identities`. Ajout du type 'adresse' au CHECK constraint.
- **Schéma SQL** : Création de la table `osint_modules` pour définir les workflows de veille (fuites, mentions, etc.).
- **Backend Rust** : Mise à jour des structs et commandes pour gérer les nouveaux champs d'adresse. Ajout de `list_osint_modules`. Bump du schema version à 2.
- **Frontend API** : Mise à jour des types `Identity` et ajout de `OsintModule`.
- **UI Identités** : Ajout des champs d'adresse dans le formulaire et l'affichage.
- **UI Veille** : Migration sur SQLite via `listOsintModules`.

### Étape 11 : Exécution des Modules OSINT (Simulation)
- **Backend Rust** : Ajout de la commande `run_osint_module`. Elle met à jour `last_run`, et pour le module `osint-breaches`, elle simule la détection d'une fuite pour l'email `alex.martin.perso@example.com` en créant une entrée dans `exposures` et `timeline_entries`.
- **Frontend API** : Ajout de `runOsintModule`.
- **UI Veille** : Ajout d'un bouton "Lancer la routine" dans le panneau de détail. Affichage du résultat de l'exécution.

### Étape 12 : Intégration de scripts OSINT locaux
- **Schéma SQL** : Ajout des colonnes `script_path` et `script_args` à la table `osint_modules`. Bump du schema version à 3.
- **Backend Rust** : La commande `run_osint_module` vérifie si `script_path` est défini. Si oui, elle exécute le script via `std::process::Command`, capture stdout et le retourne. Si le script échoue, elle capture stderr et met le statut du module à 'erreur'. Si aucun script n'est défini, elle conserve la simulation.
- **Seed Data** : Le module `osint-breaches` est configuré avec `script_path='python'` et `script_args='scripts/check_breaches.py'` (simulation d'appel à un outil CLI).
- **Frontend API** : Mise à jour de l'interface `OsintModule` avec `script_path` et `script_args`.
- **UI Veille** : Affichage du script configuré dans le panneau de détail.

### Réflexion OSINT & Privacy (pour la suite)
- **Adresses** : Permettra de croiser avec des annuaires publics, des fuites de bases immobilières, ou des registres de sociétés.
- **Pseudos passés** : Important pour tracker l'évolution d'identité (ajouter un champ "ancien pseudo" ou gérer via les notes/relations).
- **Domaines** : Surveillance DNS, certificats TLS, sous-domaines exposés.
- **Workflow fuite** : Hasher l'email localement (Rust) avant de requêter une API de breach intelligence (ex: HaveIBeenPwned) pour ne jamais envoyer l'email en clair.

### Prochaines étapes
- [ ] Débloquer l'édition (CRUD) pour les dossiers.
- [ ] Ajouter la création d'incidents/expositions depuis l'UI.
- [ ] Intégrer une vraie API OSINT (ex: HIBP) en demandant la clé API à l'utilisateur.
