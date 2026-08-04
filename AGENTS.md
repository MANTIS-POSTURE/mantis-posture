# MANTIS POSTURE — contexte utile

MANTIS est une application desktop Windows locale, construite avec Tauri 2,
SvelteKit, TypeScript, Rust et SQLite. Elle aide une personne à comprendre les
traces publiques liées à ses propres identifiants, sans faire de pentest ni
d’attribution automatique d’identité.

## Principes produit

- Local-first : l’utilisateur ne doit installer ni Python, ni pip, ni Git,
  Docker, WSL ou navigateur automatisé.
- Une Identité contient uniquement les données saisies par l’utilisateur.
  Les résultats web restent des observations séparées.
- Aucun collecteur, modèle IA ou scan ne crée automatiquement une Exposition,
  un Incident, une Action ou une demande RGPD.
- Toute conclusion reste traçable : source, date, preuve et niveau
  d’incertitude. Un profil est toujours « potentiel » tant que l’utilisateur
  ne l’a pas revu.
- L’IA locale explique, trie et priorise ; elle ne crée ni fait, ni preuve, ni
  donnée métier et n’envoie rien.

## Travailler dans l’existant

Avant toute modification, inspecter uniquement la zone concernée : route,
composant, commande Tauri, migration et test associés. Réutiliser les modèles
existants `Dossier → Identité → Exposition → Incident → Action`, la navigation,
les composants d’état et les conventions SQLite/Tauri.

Ne pas créer de seconde base, navigation, thème, notion d’identité ou pipeline
OSINT. Ne pas remplacer des données réelles par des mocks. Toute migration doit
être additive, rejouable et préserver les relations existantes.

## Collecte et sécurité

- Les collecteurs production sont XposedOrNot, User Scanner, DDGS et Maigret.
- Les détails de collecte, journaux, noms d’outil et preuves brutes appartiennent
  au mode avancé ; le mode normal présente une explication claire.
- Chaque exécution doit rester bornée : timeout, taille de sortie, logs et
  échec isolé. Aucune collecte de mot de passe, token, cookie ou secret.
- Ne jamais consulter une adresse locale ou privée lors d’une vérification web.

## Documentation à charger au besoin

Les documents de référence sont `README.md`, `SECURITY.md`,
`THIRD-PARTY-NOTICES.md` et les notes de contribution. Les anciennes notes de
conception ne sont pas une source de vérité.

## Vérifier avant livraison

Exécuter les contrôles adaptés : `npm run check`, `npm run build`, `cargo check`
et les tests Rust lorsque le backend est touché. Ne pas committer si un contrôle
échoue. Préserver les modifications utilisateur déjà présentes dans le dépôt.
