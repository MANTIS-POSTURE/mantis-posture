-- Phase 8: lifecycle metadata only. Historical scans and signals are never deleted.

INSERT OR IGNORE INTO osint_modules(id,name,description,target_kind,frequency,status,last_run,next_run,script_path,script_args,catalog_status,replacement_id,archived_reason) VALUES
('osint-github-profile','Profil GitHub public','Vérifie un profil GitHub public pour un pseudo déclaré. Les informations publiques restent une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL,'active',NULL,NULL),
('osint-gitlab-profile','Profil GitLab public','Vérifie un profil GitLab public pour un pseudo ou e-mail public déclaré. Les informations publiques restent une correspondance possible, jamais une identité confirmée.','pseudo ou e-mail','Manuel','actif',NULL,NULL,NULL,NULL,'active',NULL,NULL),
('osint-gravatar-profile','Profil public lié à l’e-mail','Recherche un profil Gravatar public à partir du condensat SHA-256 d’un e-mail autorisé. Les informations restent une correspondance possible, jamais une identité confirmée.','e-mail','Manuel','actif',NULL,NULL,NULL,NULL,'active',NULL,NULL),
('osint-keybase-profile','Preuves publiques liées au pseudo','Recherche un profil Keybase exact et ses preuves publiques vers d’autres services. Un réseau de comptes reste une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL,'active',NULL,NULL),
('osint-bluesky-profile','Profil Bluesky public','Vérifie un handle Bluesky public exact ou la variante explicite pseudo.bsky.social. Le profil reste une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL,'active',NULL,NULL),
('osint-hackernews-profile','Profil Hacker News public','Vérifie un compte Hacker News public correspondant exactement au pseudo déclaré et résume une activité publique limitée. Le profil reste une correspondance possible, jamais une identité confirmée.','pseudo','Manuel','actif',NULL,NULL,NULL,NULL,'active',NULL,NULL),
('osint-email-breaches-local','h8mail (historique)','Ancien collecteur local non distribué.','email','Manuel','desactive',NULL,NULL,NULL,NULL,'archived','osint-email-intel','Collecteur local non configuré et non distribué ; XposedOrNot couvre le besoin actif sans dépendance utilisateur.'),
('osint-gmail-profile','GHunt (historique)','Ancien collecteur non packagé.','email','Manuel','desactive',NULL,NULL,NULL,NULL,'archived','osint-web-footprint','Collecteur historique non packagé ; les recherches publiques passent par DDGS.'),
('osint-entity-corroboration','SpiderFoot (historique)','Entrée proposée mais jamais implémentée.','identité','Manuel','desactive',NULL,NULL,NULL,NULL,'archived',NULL,'Entrée proposée mais jamais implémentée ; la corrélation déterministe MANTIS remplit ce rôle.');

UPDATE osint_modules SET catalog_status='active',replacement_id=NULL,archived_reason=NULL
WHERE id IN ('osint-email-intel','osint-email-platforms','osint-web-footprint','osint-username-profiles','osint-github-profile','osint-gitlab-profile','osint-gravatar-profile','osint-keybase-profile','osint-bluesky-profile','osint-hackernews-profile');

UPDATE osint_modules SET catalog_status='test_only',status='desactive',archived_reason='Fixture synthétique conservée uniquement pour les tests.'
WHERE id='mock-osint';

UPDATE osint_modules SET catalog_status='archived',status='desactive',replacement_id='osint-email-intel',archived_reason='Collecteur local non configuré et non distribué ; XposedOrNot couvre le besoin actif sans dépendance utilisateur.'
WHERE id='osint-email-breaches-local';

UPDATE osint_modules SET catalog_status='archived',status='desactive',replacement_id='osint-web-footprint',archived_reason='Collecteur historique non packagé ; les recherches publiques passent par DDGS.'
WHERE id='osint-gmail-profile';

UPDATE osint_modules SET catalog_status='archived',status='desactive',replacement_id=NULL,archived_reason='Entrée proposée mais jamais implémentée ; la corrélation déterministe MANTIS remplit ce rôle.'
WHERE id='osint-entity-corroboration';
