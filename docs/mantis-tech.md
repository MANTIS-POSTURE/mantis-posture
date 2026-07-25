# MANTIS POSTURE Tech specs
# MANTIS POSTURE – Spécifications techniques

MANTIS POSTURE est une application desktop locale, construite pour être légère, sécurisée et explicable.
Cette fiche décrit le socle technique, les contraintes de sécurité et les choix d’architecture.

## Stack technologique

- **Desktop / shell** :
  - Tauri 2 (framework Rust pour applications desktop multi-plateformes).
  - Cible initiale : Windows (plus tard Linux).

- **Frontend** :
  - SvelteKit (meta-framework Svelte, compilé en HTML/CSS/JS).
  - TypeScript (typage fort côté frontend).
  - Tailwind CSS (design systématique, thème sombre cockpit). [web:119][web:278][web:281]

- **Backend / logique métier** :
  - Rust (commandes Tauri pour la logique, l’accès aux données et certaines opérations OSINT locales).

- **Base de données** :
  - SQLite locale (fichier unique ou par profil).
  - Intégration via :
    - Plugin SQL Tauri, ou
    - Crate Rust type `rusqlite` / `sqlx`, avec migrations au démarrage. [web:7][web:278][web:281][web:276]

Le but est de produire un binaire desktop compact, rapide et sécurisé, avec une base de données locale lisible et portable.

## Architecture générale

- **Frontend SvelteKit** :
  - Affiche le cockpit (sidebar, Centre de posture, graphe, tables, guides).
  - Interagit avec le backend via des commandes Tauri (invocations Rust).
  - Ne gère pas de secrets ; les formulaires et composants sont orientés posture, pas gestion de mots de passe.

- **Backend Tauri + Rust** :
  - Gère la persistance en SQLite.
  - Implémente les opérations métier :
    - Création et gestion des dossiers, identités, expositions, incidents, actions, démarches RGPD, chronologie.
    - Calcul du score de posture.
    - Génération des rapports HTML/PDF.
    - Intégration progressive des modules OSINT et RGPD.

- **Base SQLite** :
  - Fichier local (par exemple dans un dossier de données de l’utilisateur).
  - Schéma structuré avec tables :
    - `folders`, `identities`, `exposures`, `incidents`, `actions`, `rgpd_requests`, `timeline_entries`.
  - Migrations exécutées au démarrage pour garantir la cohérence du schéma. [web:7][web:278][web:281]

## Contraintes de sécurité et de confidentialité

- **Local-first** :
  - L’application doit fonctionner entièrement en local.
  - Aucun cloud obligatoire, aucun backend distant imposé.
  - Les données restent sur la machine de l’utilisateur.

- **Pas de télémétrie ni d’analytics** :
  - Pas de collecte automatique de métriques d’usage.
  - Pas de tracking des actions de l’utilisateur vers un serveur externe.
  - Les éventuels journaux de diagnostic restent locaux et minimalistes. [web:277][web:284][web:286]

- **Pas de secrets stockés** :
  - MANTIS ne doit jamais stocker :
    - Mots de passe de comptes.
    - Seed phrases, clés privées, codes MFA.
    - Données bancaires ou documents d’identité.
    - Dumps de bases de données de fuites brutes.
  - Les domaines, e-mails, pseudos, numéros de téléphone, identités et résumés de fuites sont autorisés, mais pas les secrets eux-mêmes.

- **Données autorisées en base** :
  - Dossiers.
  - Identités.
  - Expositions.
  - Incidents.
  - Actions.
  - Démarches RGPD.
  - Chronologie des recherches, décisions, actions et revues.

Ces contraintes doivent être respectées dans toute évolution du code, des schémas et des modules OSINT.

## Intégration Tauri + SvelteKit

- **Mode développement** :
  - SvelteKit fournit un dev server (par défaut `http://localhost:5173`).
  - Tauri est configuré pour utiliser cette URL en `devPath` et la build SvelteKit en `distDir` lors du mode production. [web:119][web:281]
  - La commande principal de dev est `npm run tauri dev`.

- **Build production** :
  - SvelteKit est buildé en assets statiques.
  - Tauri embarque ces assets dans son binaire.
  - Le résultat est une application desktop autonome, sans besoin de serveur Node.

## SQLite et persistance

- **Choix de SQLite** :
  - SQLite est adapté aux applications desktop local-first, avec un fichier unique et un moteur intégré.
  - La base peut utiliser le mode WAL pour de meilleures performances et la robustesse (option). [web:281][web:7]

- **Accès aux données** :
  - Les commandes Rust/Tauri gèrent :
    - La création de la base si elle n’existe pas.
    - L’application des migrations.
    - Les opérations CRUD sur les tables (création, lecture, mise à jour, suppression).
  - Le frontend ne manipule pas directement SQLite ; il passe par des commandes ou des APIs Tauri dédiées.

- **Sauvegarde et restauration** :
  - MANTIS devra proposer (dans une phase ultérieure) :
    - Export de la base SQLite (ou d’un subset) pour sauvegarde.
    - Restauration à partir d’un fichier exporté.
  - Les sauvegardes doivent rester chiffrables par l’utilisateur, mais MANTIS ne gère pas de clés pour lui.

## Architecture OSINT (technique)

MANTIS intègre des éléments d’OSINT, mais de manière contrôlée :

- **Cadre OSINT** :
  - Utilisation d’outils et d’APIs d’OSINT dans le respect du cycle OSINT :
    - Planification et direction.
    - Collecte.
    - Traitement et validation.
    - Analyse et production.
    - Diffusion et action. [web:269][web:271][web:272][web:287][web:275]

- **Types de sources** :
  - Moteurs de recherche : requêtes sur noms, pseudos, e-mails, domaines.
  - Services de breach intelligence : vérification d’e-mails, domaines, pseudos dans des fuites. [web:250][web:261][web:272]
  - Vérification technique de domaines : DNS, certificats TLS, sous-domaines, bannières.

- **Architecture technique** :
  - Les appels vers des APIs externes sont encapsulés dans des modules Rust/Tauri.
  - Chaque module :
    - A des paramètres clairs (identifiants à vérifier, limites de requêtes).
    - Journalise localement les appels et les résultats.
  - Les résultats sont transformés en :
    - Résultats OSINT bruts.
    - Expositions (après filtrage).
    - Incidents (si le critère de gravité est atteint).

- **Gestion du bruit et des doublons** :
  - Déduplication des résultats.
  - Filtrage des expositions déjà traitées.
  - Seuils de confiance pour éviter les correspondances ambiguës.

## Automatisation RGPD (niveau technique)

L’automatisation RGPD repose sur :

- **Modèle de données** :
  - Table `rgpd_requests` pour stocker :
    - Type de demande (accès, rectification, effacement, opposition, déréférencement).
    - Cible (entreprise / service).
    - Identités et données concernées.
    - Dates (création, envoi, réponse).
    - Statut (en cours, terminé, refusé, etc.). [web:254][web:256][web:259][web:262]

- **Génération de documents** :
  - Création de modèles de courrier / texte de demande à partir des informations stockées.
  - Export en format texte ou HTML (par exemple pour coller dans un mail ou une lettre).
  - Génération de pièces jointes (extraits de rapport) en HTML/PDF via Tauri.

- **Aucun envoi automatique en production** :
  - MANTIS prépare les requêtes RGPD ; l’envoi (mail, formulaire, courrier) reste sous contrôle de l’utilisateur.
  - Le module ne doit pas envoyer des requêtes à des DPO ou services privacy sans action explicite de l’utilisateur.

## Résumé pour les agents et développeurs

- MANTIS est une application **local-first** : Tauri + SvelteKit + SQLite, sans cloud obligatoire.
- Elle est centrée sur la **posture numérique** et la réduction de l’exposition, pas sur la gestion de secrets.
- La stack et l’architecture doivent :
  - Respecter une séparation claire frontend/backend.
  - Utiliser SQLite comme source de vérité locale.
  - Intégrer OSINT et RGPD de manière progressive, contrôlée, traçable.

Toute nouvelle fonctionnalité doit être alignée avec ces principes techniques et ces contraintes de sécurité.