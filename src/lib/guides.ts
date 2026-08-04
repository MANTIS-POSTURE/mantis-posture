export const PROTON_REFERRAL_URL = 'https://pr.tn/ref/CQQ64MYC';

export type GuideStep = { title: string; detail: string };
export type GuideResource = { label: string; url: string; note: string; featured?: boolean };

export type Guide = {
  id: string;
  title: string;
  shortTitle: string;
  category: 'Comptes' | 'Réseau' | 'Systèmes' | 'Communications' | 'Méthode' | 'Droits';
  summary: string;
  when: string;
  outcome: string;
  minutes: number;
  level: 'Essentiel' | 'Intermédiaire' | 'Avancé';
  image: string;
  imageAlt: string;
  gallery?: { src: string; alt: string }[];
  accent: string;
  tags: string[];
  steps: GuideStep[];
  checklist: string[];
  cautions: string[];
  resources: GuideResource[];
  relatedHref?: string;
};

const privacyGuides: GuideResource = {
  label: 'Privacy Guides',
  url: 'https://www.privacyguides.org/en/',
  note: 'Comparatifs indépendants, modèles de menace et recommandations maintenues.',
  featured: true
};

const exitChatControl: GuideResource = {
  label: 'Exit Chat Control',
  url: 'https://exitchatcontrol.org/',
  note: 'Manuel public de souveraineté numérique et de migration vers des outils respectueux.',
  featured: true
};

export const guides: Guide[] = [
  {
    id: 'guide-email-aliases', title: 'Séparer son adresse privée de ses inscriptions', shortTitle: 'Mail chiffré & alias', category: 'Comptes',
    summary: 'Utiliser une boîte chiffrée et un alias différent par service pour réduire le spam, la corrélation et l’impact d’une fuite.',
    when: 'Une adresse apparaît dans une fuite, sert partout, ou révèle directement votre identité.', outcome: 'Une adresse principale protégée et des alias révocables par usage.', minutes: 20, level: 'Essentiel',
    image: '/guides/proton-pass.avif', imageAlt: 'Interface Proton Pass sur ordinateur et téléphone', accent: '#8876ff', tags: ['email', 'alias', 'Proton Mail', 'fuite'], relatedHref: '/identites',
    steps: [
      { title: 'Garder une adresse racine privée', detail: 'Réservez-la à la récupération des comptes sensibles et ne la publiez pas.' },
      { title: 'Créer un alias par contexte', detail: 'Un alias pour les achats, un pour l’administratif et, idéalement, un par site important.' },
      { title: 'Migrer progressivement', detail: 'Commencez par banque, santé, opérateur, cloud et réseaux sociaux. Ne tentez pas tout en une soirée.' },
      { title: 'Révoquer ce qui fuit', detail: 'Si un alias reçoit du spam ou apparaît dans une fuite, remplacez-le sans changer votre boîte principale.' }
    ],
    checklist: ['Adresse racine absente des profils publics', 'Alias distinct pour les comptes critiques', 'Adresse de récupération vérifiée', 'MFA activée sur la boîte principale'],
    cautions: ['Le chiffrement ne protège pas un appareil déjà compromis.', 'Un alias réduit la corrélation mais ne rend pas anonyme si le même nom, téléphone ou moyen de paiement est réutilisé.'],
    resources: [{ label: 'Découvrir Proton Mail et ses alias', url: PROTON_REFERRAL_URL, note: 'Lien partenaire MANTIS vers l’écosystème Proton.' }, privacyGuides, exitChatControl]
  },
  {
    id: 'guide-proton-pass', title: 'Un mot de passe unique pour chaque compte', shortTitle: 'Proton Pass', category: 'Comptes',
    summary: 'Centraliser des mots de passe longs, uniques et des alias sans avoir à les mémoriser.',
    when: 'Vous réutilisez un mot de passe, stockez des codes dans des notes ou venez de découvrir une fuite.', outcome: 'Un coffre chiffré, une récupération testée et aucun mot de passe réutilisé.', minutes: 25, level: 'Essentiel',
    image: '/guides/proton-pass.avif', imageAlt: 'Coffre Proton Pass sur plusieurs appareils', accent: '#6d5dfc', tags: ['mot de passe', 'passkey', 'MFA', 'Proton Pass'], relatedHref: '/actions',
    steps: [
      { title: 'Créer une phrase maîtresse', detail: 'Longue, unique et jamais utilisée ailleurs. Conservez le moyen de récupération hors ligne.' },
      { title: 'Importer puis trier', detail: 'Commencez par les comptes sensibles et supprimez l’ancien export dès que l’import est contrôlé.' },
      { title: 'Remplacer les doublons', detail: 'Générez un secret unique, site par site, en priorité pour les comptes exposés.' },
      { title: 'Ajouter MFA ou passkey', detail: 'Préférez une passkey ou une clé physique quand le service le permet, sinon une application TOTP.' }
    ],
    checklist: ['Phrase maîtresse unique', 'Récupération testée', 'Doublons critiques remplacés', 'Export d’import supprimé', 'Verrouillage automatique activé'],
    cautions: ['Ne copiez jamais votre phrase maîtresse dans MANTIS.', 'Synchronisation chiffrée ne signifie pas protection contre une session déjà ouverte sur un appareil volé.'],
    resources: [{ label: 'Découvrir Proton Pass', url: PROTON_REFERRAL_URL, note: 'Lien partenaire MANTIS vers Proton Pass.' }, privacyGuides]
  },
  {
    id: 'guide-mfa', title: 'Renforcer ses comptes avec MFA et passkeys', shortTitle: 'MFA & passkeys', category: 'Comptes',
    summary: 'Ajouter une seconde preuve qui bloque la majorité des prises de compte après une fuite de mot de passe.',
    when: 'Un compte important ne dépend encore que d’un mot de passe.', outcome: 'Les comptes critiques protégés et des codes de secours conservés hors ligne.', minutes: 15, level: 'Essentiel',
    image: '/guides/proton-pass.avif', imageAlt: 'Gestionnaire de mots de passe et authentification', accent: '#a792ff', tags: ['MFA', '2FA', 'passkey', 'compte'], relatedHref: '/actions',
    steps: [
      { title: 'Prioriser la boîte mail', detail: 'Elle réinitialise souvent tous les autres comptes : sécurisez-la en premier.' },
      { title: 'Choisir le facteur', detail: 'Passkey ou clé physique si disponible, sinon application TOTP. Évitez le SMS quand une meilleure option existe.' },
      { title: 'Conserver les secours', detail: 'Imprimez ou notez les codes de récupération et gardez-les dans un lieu sûr et distinct.' },
      { title: 'Tester avant de fermer', detail: 'Ouvrez une session privée et vérifiez la connexion ainsi que la récupération.' }
    ],
    checklist: ['Mail principal protégé', 'Comptes financiers protégés', 'Codes de secours hors ligne', 'Anciennes sessions révoquées'],
    cautions: ['Le SMS est préférable à l’absence de MFA, mais reste vulnérable au détournement de numéro.', 'Ne perdez pas simultanément appareil, clé et codes de secours.'], resources: [privacyGuides]
  },
  {
    id: 'guide-vpn', title: 'Choisir un VPN selon son vrai besoin', shortTitle: 'Mullvad ou Proton VPN', category: 'Réseau',
    summary: 'Protéger le trafic sur un réseau non fiable et réduire la visibilité du fournisseur d’accès, sans promettre l’anonymat.',
    when: 'Vous utilisez un Wi-Fi public, voyagez ou ne souhaitez pas exposer votre IP domestique aux services consultés.', outcome: 'Un VPN configuré avec coupe-circuit et un usage cohérent avec votre modèle de menace.', minutes: 20, level: 'Intermédiaire',
    image: '/guides/proton-vpn.avif', imageAlt: 'Proton VPN sur ordinateur et téléphone', accent: '#52d6a7', tags: ['VPN', 'IP', 'Mullvad', 'Proton VPN'], relatedHref: '/veille',
    steps: [
      { title: 'Nommer le risque', detail: 'Wi-Fi hostile, IP domestique visible ou censure : le besoin détermine le choix.' },
      { title: 'Comparer sans chercher un gagnant absolu', detail: 'Mullvad minimise l’identification du compte ; Proton s’intègre à une suite plus large et propose une offre gratuite.' },
      { title: 'Activer le coupe-circuit', detail: 'Vérifiez que le trafic s’arrête si le tunnel tombe et activez le démarrage automatique sur les réseaux inconnus.' },
      { title: 'Tester les fuites', detail: 'Contrôlez IP et DNS avant/après, puis recommencez après une mise à jour importante.' }
    ],
    checklist: ['Coupe-circuit actif', 'Démarrage auto défini', 'IP et DNS testés', 'Compte choisi selon le besoin'],
    cautions: ['Un VPN déplace la confiance du fournisseur d’accès vers le fournisseur VPN.', 'Il ne supprime ni cookies, ni empreinte du navigateur, ni connexion à vos comptes habituels.'],
    resources: [{ label: 'Découvrir Proton VPN', url: PROTON_REFERRAL_URL, note: 'Lien partenaire MANTIS vers Proton VPN.' }, { label: 'Mullvad VPN', url: 'https://mullvad.net/', note: 'Compte numéroté et politique de collecte minimale.' }, privacyGuides]
  },
  {
    id: 'guide-linux', title: 'Passer à Linux sans perdre ses repères', shortTitle: 'Linux au quotidien', category: 'Systèmes',
    summary: 'Préparer une migration réaliste vers un système maintenu, chiffré et mis à jour.',
    when: 'Votre système n’est plus maintenu, collecte trop de télémétrie ou ne correspond plus à votre modèle de menace.', outcome: 'Une distribution adaptée, testée avant migration, avec sauvegarde et chiffrement disque.', minutes: 35, level: 'Intermédiaire',
    image: '/guides/linux-workstation.svg', imageAlt: 'Poste de travail Linux abstrait et sécurisé', accent: '#5aa9ff', tags: ['Linux', 'Fedora', 'chiffrement', 'système'], relatedHref: '/actions',
    steps: [
      { title: 'Lister les logiciels indispensables', detail: 'Vérifiez navigateur, bureautique, périphériques et éventuelles applications métier.' },
      { title: 'Tester sur une clé USB', detail: 'Essayez une distribution grand public maintenue, comme Fedora Workstation, sans modifier le disque.' },
      { title: 'Sauvegarder et chiffrer', detail: 'Faites une sauvegarde vérifiée puis activez le chiffrement complet pendant l’installation.' },
      { title: 'Installer peu, mettre à jour souvent', detail: 'Utilisez les dépôts de la distribution et évitez les scripts inconnus copiés depuis un forum.' }
    ],
    checklist: ['Compatibilité matérielle testée', 'Sauvegarde restaurable', 'Chiffrement disque actif', 'Mises à jour automatiques', 'Applications obtenues de sources fiables'],
    cautions: ['Linux n’est pas invulnérable et ne corrige pas de mauvaises habitudes de compte.', 'Qubes OS et Whonix répondent à des besoins spécifiques et demandent davantage d’apprentissage.'], resources: [privacyGuides, exitChatControl]
  },
  {
    id: 'guide-grapheneos', title: 'Durcir son smartphone avec GrapheneOS', shortTitle: 'GrapheneOS', category: 'Systèmes',
    summary: 'Installer un système mobile renforcé sur un Google Pixel compatible et isoler les applications sensibles.',
    when: 'Le smartphone concentre vos identités, messages, photos et facteurs de connexion.', outcome: 'Un Pixel compatible, installé par la méthode officielle et organisé en profils séparés.', minutes: 40, level: 'Avancé',
    image: '/guides/grapheneos-phone.svg', imageAlt: 'Téléphone sécurisé inspiré de GrapheneOS', accent: '#86dba8', tags: ['GrapheneOS', 'Pixel', 'mobile', 'profils'], relatedHref: '/actions',
    steps: [
      { title: 'Vérifier le modèle et le support', detail: 'Utilisez uniquement un Pixel encore pris en charge et consultez la liste officielle avant tout achat.' },
      { title: 'Sauvegarder les données', detail: 'L’installation efface le téléphone. Testez la restauration des éléments indispensables.' },
      { title: 'Utiliser l’installateur officiel', detail: 'Suivez le WebUSB officiel depuis un navigateur compatible et reverrouillez le chargeur de démarrage.' },
      { title: 'Créer des profils', detail: 'Séparez usages personnel, professionnel et applications à forte collecte ; accordez les permissions au minimum.' }
    ],
    checklist: ['Pixel officiellement supporté', 'Sauvegarde vérifiée', 'Bootloader reverrouillé', 'Profils séparés', 'Redémarrage automatique configuré'],
    cautions: ['GrapheneOS n’est officiellement destiné qu’aux appareils supportés.', 'Les services Google sandboxés peuvent améliorer la compatibilité mais restent un choix à évaluer application par application.'],
    resources: [{ label: 'Site officiel GrapheneOS', url: 'https://grapheneos.org/', note: 'Compatibilité, installateur et documentation de référence.' }, privacyGuides]
  },
  {
    id: 'guide-private-messaging', title: 'Choisir Signal, Element ou Session', shortTitle: 'Messageries privées', category: 'Communications',
    summary: 'Choisir une messagerie selon les personnes à joindre, les métadonnées à minimiser et le niveau de complexité acceptable.',
    when: 'Des échanges sensibles passent encore par SMS, messagerie sociale ou un canal non maîtrisé.', outcome: 'Un canal principal correctement vérifié et un canal de secours convenu.', minutes: 25, level: 'Essentiel',
    image: '/guides/signal-pixel.avif', imageAlt: 'Conversation chiffrée dans Signal sur un téléphone', accent: '#3a76f0', tags: ['Signal', 'Element', 'Session', 'messagerie'], relatedHref: '/actions',
    gallery: [
      { src: '/guides/signal-logo.avif', alt: 'Logo Signal' },
      { src: '/guides/signal-iphone.avif', alt: 'Appel de groupe Signal sur iPhone' },
      { src: '/guides/session-messenger.avif', alt: 'Messagerie Session sur trois téléphones' }
    ],
    steps: [
      { title: 'Choisir par contexte', detail: 'Signal convient à la plupart des proches ; Element aux communautés Matrix ; Session minimise l’identification à l’inscription.' },
      { title: 'Régler la confidentialité', detail: 'Contrôlez visibilité du numéro, notifications, aperçus, sauvegardes et verrouillage de l’application.' },
      { title: 'Vérifier les contacts sensibles', detail: 'Comparez le numéro de sécurité ou le QR code par un second canal fiable.' },
      { title: 'Prévoir le repli', detail: 'Convenez d’un canal alternatif avant une panne, un changement de téléphone ou une perte de compte.' }
    ],
    checklist: ['Application à jour', 'Verrouillage actif', 'Contacts sensibles vérifiés', 'Sauvegarde comprise', 'Canal de secours convenu'],
    cautions: ['Le chiffrement protège le transport, pas un écran déverrouillé ni le destinataire.', 'Signal requiert toujours un numéro pour l’inscription ; Session présente des compromis, notamment pour certains appels.'],
    resources: [{ label: 'Signal', url: 'https://signal.org/', note: 'Choix simple et mature pour les conversations courantes.' }, { label: 'Element / Matrix', url: 'https://element.io/', note: 'Messagerie décentralisée adaptée aux groupes et organisations.' }, { label: 'Session', url: 'https://getsession.org/', note: 'Inscription sans numéro ni e-mail, avec compromis spécifiques.' }, privacyGuides, exitChatControl]
  },
  {
    id: 'guide-compartmentalization', title: 'Empêcher ses identités de se recoller', shortTitle: 'Compartimentalisation', category: 'Méthode',
    summary: 'Séparer les contextes personnel, public et professionnel pour limiter la corrélation après une fuite.',
    when: 'Le même pseudo, mail, photo, navigateur ou numéro relie plusieurs facettes de votre vie.', outcome: 'Des compartiments nommés, cohérents et faciles à maintenir.', minutes: 30, level: 'Intermédiaire',
    image: '/guides/compartments.svg', imageAlt: 'Identités numériques séparées en compartiments', accent: '#f2b36d', tags: ['OPSEC', 'identité', 'alias', 'profils'], relatedHref: '/identites',
    steps: [
      { title: 'Dessiner trois compartiments', detail: 'Par exemple : privé, professionnel et public. Notez ce qui peut être relié entre eux.' },
      { title: 'Séparer les identifiants', detail: 'Utilisez adresses, alias et pseudos différents lorsque la continuité d’identité n’est pas utile.' },
      { title: 'Séparer les sessions', detail: 'Créez des profils de navigateur distincts, voire des profils système ou mobiles pour les usages sensibles.' },
      { title: 'Réduire les ponts invisibles', detail: 'Évitez de réutiliser photo, bio, numéro, domaine, horaires ou liens croisés.' }
    ],
    checklist: ['Compartiments nommés', 'Alias séparés', 'Profils navigateur distincts', 'Photos et bios non réutilisées', 'Règles simples documentées'],
    cautions: ['La séparation parfaite est difficile : visez une réduction mesurable de corrélation.', 'Une erreur ponctuelle peut relier deux compartiments ; ne surévaluez jamais la protection.'], resources: [privacyGuides, exitChatControl]
  },
  {
    id: 'guide-privacy-social', title: 'Réduire ce que racontent ses profils publics', shortTitle: 'Profils publics', category: 'Méthode',
    summary: 'Limiter les informations qui facilitent l’usurpation, le recoupement ou les questions de récupération.',
    when: 'Un scan retrouve des profils, anciennes biographies, photos ou informations géographiques.', outcome: 'Des profils nettoyés, des audiences revues et moins de signaux corrélables.', minutes: 20, level: 'Essentiel',
    image: '/guides/compartments.svg', imageAlt: 'Profils numériques compartimentés', accent: '#e89c64', tags: ['profil', 'réseaux sociaux', 'photo', 'OSINT'], relatedHref: '/expositions',
    steps: [
      { title: 'Voir comme un inconnu', detail: 'Ouvrez le profil hors connexion et notez ce qui révèle lieu, proches, employeur ou habitudes.' },
      { title: 'Retirer l’ancien', detail: 'Supprimez bios, publications et comptes qui n’ont plus d’utilité.' },
      { title: 'Réviser les audiences', detail: 'Limitez l’indexation et les anciens contenus publics, puis contrôlez les applications tierces.' },
      { title: 'Casser les recoupements', detail: 'Changez les pseudos ou photos réutilisés entre des contextes qui doivent rester distincts.' }
    ],
    checklist: ['Profil vu hors connexion', 'Anciennes publications revues', 'Audience et indexation contrôlées', 'Applications tierces révoquées'],
    cautions: ['Une suppression locale ne garantit pas la disparition des copies ou archives.', 'Ne signalez une usurpation qu’après avoir conservé les preuves nécessaires.'], resources: [privacyGuides]
  },
  {
    id: 'guide-rgpd', title: 'Exercer ses droits sans perdre la preuve', shortTitle: 'DPO & RGPD', category: 'Droits',
    summary: 'Identifier le bon organisme, formuler une demande proportionnée et conserver les éléments de suivi.',
    when: 'Une organisation publie des données inexactes, excessives ou que vous souhaitez faire supprimer.', outcome: 'Une demande relue, envoyée manuellement au bon contact et suivie avec ses preuves.', minutes: 25, level: 'Intermédiaire',
    image: '/guides/rgpd-rights.svg', imageAlt: 'Dossier de droits numériques et protection des données', accent: '#d6b36a', tags: ['RGPD', 'DPO', 'effacement', 'CNIL'], relatedHref: '/dpo',
    steps: [
      { title: 'Conserver la source', detail: 'Notez URL, date, capture et donnée concernée avant toute demande.' },
      { title: 'Contacter l’organisme', detail: 'Cherchez le contact vie privée ou DPO dans sa politique de confidentialité et vérifiez le destinataire.' },
      { title: 'Choisir le bon droit', detail: 'Accès, rectification, effacement, opposition ou déréférencement ne répondent pas au même objectif.' },
      { title: 'Suivre sans surpartager', detail: 'Conservez envoi et réponse. Ne fournissez une preuve d’identité que si elle est nécessaire et proportionnée.' }
    ],
    checklist: ['Source datée', 'Organisme et contact vérifiés', 'Droit adapté', 'Brouillon relu', 'Envoi et réponse conservés'],
    cautions: ['MANTIS prépare un brouillon informatif et n’envoie rien à votre place.', 'L’effacement n’est pas absolu et certaines obligations légales peuvent justifier une conservation.'],
    resources: [{ label: 'CNIL — exercer ses droits', url: 'https://www.cnil.fr/fr/comprendre-mes-droits', note: 'Référence officielle française sur les droits et les démarches.' }, { label: 'Trouver un DPO', url: 'https://www.cnil.fr/fr/designation-dpo', note: 'Informations officielles sur le rôle et la désignation du DPO.' }, privacyGuides]
  }
];

export function getGuide(id: string): Guide | undefined {
  return guides.find((guide) => guide.id === id);
}

function assertGuideLibrary(): void {
  const ids = new Set<string>();
  for (const guide of guides) {
    if (ids.has(guide.id)) throw new Error(`Identifiant de guide dupliqué : ${guide.id}`);
    ids.add(guide.id);
    for (const resource of guide.resources) {
      const concernsProton = /proton/i.test(`${resource.label} ${resource.note}`);
      if (concernsProton && resource.url !== PROTON_REFERRAL_URL) {
        throw new Error(`Le lien Proton du guide ${guide.id} ne passe pas par le parrainage configuré.`);
      }
    }
  }
}

assertGuideLibrary();

export function recommendGuideForContext(value: string): Guide {
  const text = value.toLocaleLowerCase('fr');
  const id = /rgpd|dpo|effacement|déréférencement|rectification/.test(text) ? 'guide-rgpd'
    : /vpn|adresse ip|réseau|wifi|wi-fi/.test(text) ? 'guide-vpn'
    : /signal|session|element|message|sms|conversation/.test(text) ? 'guide-private-messaging'
    : /téléphone|mobile|android|pixel/.test(text) ? 'guide-grapheneos'
    : /mail|email|courriel|adresse électronique/.test(text) ? 'guide-email-aliases'
    : /mot de passe|password|compte|authentification|mfa/.test(text) ? 'guide-proton-pass'
    : /profil|pseudo|photo|identité|corrél/.test(text) ? 'guide-compartmentalization'
    : 'guide-privacy-social';
  return getGuide(id) ?? guides[0];
}
