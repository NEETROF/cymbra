---
layout: ../layouts/Legal.astro
title: Politique de confidentialité
lang: fr
alternates:
  fr: /confidentialite
  en: /en/privacy
  es: /es/privacidad
updated: 14/10/2026
---

La présente politique explique quelles données personnelles les **services Cymbra**
(édités par **NEETROF**) traitent, pourquoi, sur quelle base légale, avec qui elles sont
partagées, combien de temps elles sont conservées, et quels sont vos droits. Elle couvre
le **compte Cymbra**, partagé entre les services Cymbra ; les traitements propres à un
produit figurent en **annexe** (voir *Annexe A — Cymbra Music* et *Annexe B — Cymbra Lingua*).

## 1. Responsable du traitement

**NEETROF — SASU, 948723887**, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCE.
Contact : **gfortin@neetrof.fr**.

## 2. Données que nous traitons

Nous appliquons la **minimisation** : nous ne collectons que ce qui est nécessaire au
fonctionnement du compte et de l'application.

| Donnée | Origine | Finalité |
|---|---|---|
| Adresse email | vous (inscription email) ou votre fournisseur (Google/Apple) | identifiant de compte, vérification, réinitialisation de mot de passe |
| Mot de passe (empreinte **argon2**, jamais en clair) | vous (inscription email) | authentification |
| Identifiant de connexion externe (identifiant Google/Apple « sub ») | Google / Apple | connexion via « Se connecter avec Google/Apple » |
| Pseudo (« handle ») et nom affiché | vous | identification dans l'application |
| Préférences applicatives | vous | mémoriser vos réglages |
| Jetons de session (refresh tokens) | généré au login | maintenir votre session connectée |
| Journaux techniques (adresse IP, horodatages, erreurs) | serveur | sécurité, prévention des abus, bon fonctionnement |
| Statut d'abonnement (offre, source, dates de début/fin), **identifiants opaques** de l'abonnement chez le canal d'achat (Apple, Google, Paddle — via RevenueCat pour l'App Store et Google Play), campagnes bêta rejointes, codes d'accès utilisés | canal d'achat / vous | activer l'offre Premium sur vos appareils, gérer essais et bêtas |

Nous **ne** collectons **pas** de données de localisation précise, ne vendons aucune
donnée, et n'utilisons pas de publicité tierce ni de traceurs publicitaires. Nous ne
recevons et ne stockons **jamais** vos numéros de carte, adresses de facturation ni
factures : ils sont traités exclusivement par le canal d'achat (Apple, Google, Paddle).

**Chiffres publiés sur la communauté Cymbra.** Nous publions sur les salons publics de la
communauté Cymbra (serveur Discord) des chiffres d'activité **agrégés**, calculés à partir
de l'usage des services : par exemple le nombre de parties jouées, la précision moyenne,
les partitions notées et les pièces les plus jouées de Cymbra Music, les mots lus, appris
et révisés dans Cymbra Lingua, ou la part de chaque méthode de connexion parmi les nouveaux
comptes. Ces chiffres ne comportent **ni nom, ni pseudo, ni identifiant, ni nombre de
personnes**. Tant que la communauté est petite, un chiffre peut toutefois ne refléter
l'activité que de très peu de personnes.

## 3. Bases légales (RGPD art. 6)

- **Exécution du contrat** : création et gestion de votre compte, fourniture de
  l'application.
- **Intérêt légitime** : sécurité, prévention de la fraude/des abus, limitation de
  débit, bon fonctionnement, publication de chiffres d'activité agrégés sur les canaux
  de la communauté Cymbra (voir §2).
- **Consentement** : connexion via Google/Apple (vous choisissez ce mode).

## 4. Sous-traitants et tiers

Nous partageons le strict nécessaire avec des prestataires agissant pour notre compte :

- **OVHcloud** (France, UE) — hébergement du serveur et des sauvegardes.
- **Brevo** (UE) — envoi des emails transactionnels (vérification, réinitialisation).
- **Google** / **Apple** — uniquement si vous utilisez leur connexion (vérification de
  votre identité via leur jeton), ou si vous vous abonnez via l'App Store / Google Play
  (ils sont les vendeurs officiels : ils encaissent et facturent ; nous ne voyons jamais
  vos données de paiement).
- **RevenueCat, Inc.** (États-Unis) — uniquement si vous vous abonnez via l'App Store ou
  Google Play : il vérifie l'achat auprès de la boutique et suit le cycle de vie de
  l'abonnement (renouvellement, période de grâce, résiliation, remboursement) pour notre
  compte, et nous fournit des statistiques agrégées d'abonnements et de revenus. Il
  reçoit un **identifiant de compte opaque**, des informations techniques sur
  l'application et l'appareil, et les faits de transaction de la boutique (produit,
  prix, devise, pays, dates) — jamais votre nom, email, pseudonyme ni vos données de
  paiement. Ce transfert hors UE est encadré par les clauses contractuelles types de la
  Commission européenne (accord de traitement des données signé avec RevenueCat). Sa
  fiche est supprimée à la suppression de votre compte.
- **Paddle** (vendeur officiel, Royaume-Uni/UE) — uniquement si vous vous abonnez depuis
  Linux, Windows ou le site web : il encaisse, facture et nous notifie l'état de
  l'abonnement associé à un identifiant opaque.
- **Google** (API Gemini) et **Anthropic, PBC** (API Claude, États-Unis) — uniquement si
  vous utilisez la **Traduction en ligne** de Cymbra Lingua (*Annexe B*) : ils reçoivent la
  phrase à traduire, sans votre compte ni votre email, la traduisent et ne la conservent pas.
  Ce transfert hors UE est encadré par les clauses contractuelles types de la Commission
  européenne prévues par leurs conditions de traitement des données.

Vos données sont **hébergées dans l'Union européenne** (France). En dehors de la
vérification des abonnements et de la Traduction en ligne décrites ci-dessus (RevenueCat,
Google et Anthropic, sous clauses
contractuelles types), nous ne procédons à aucun transfert hors UE.

## 5. Durée de conservation

- Données de compte : conservées **tant que votre compte existe**.
- **Suppression du compte** : lorsque vous supprimez votre compte (voir §7), vos
  données personnelles (email, empreinte de mot de passe, identité externe, pseudo,
  nom, sessions) sont **effacées**.
- Sauvegardes : les sauvegardes chiffrées tournent sur une fenêtre glissante
  (14 jours) puis sont écrasées ; une donnée supprimée disparaît donc au plus tard à
  l'expiration de cette fenêtre.
- Journaux techniques : 7 jours.

## 6. Sécurité

Chiffrement en transit (**TLS**), mots de passe stockés sous forme d'empreinte
**argon2** (jamais en clair), sauvegardes **chiffrées** et stockées hors du serveur,
accès serveur restreint. Aucune mesure n'étant infaillible, nous ne pouvons garantir
une sécurité absolue.

## 7. Vos droits (RGPD)

Vous disposez des droits d'**accès**, de **rectification**, d'**effacement**, de
**limitation**, d'**opposition** et de **portabilité**.

- **Effacement (droit à l'oubli)** : vous pouvez **supprimer votre compte directement
  dans l'application** (Réglages → Supprimer mon compte). La suppression est
  irréversible et efface vos données personnelles. Votre compte étant commun à toutes les
  apps Cymbra, sa suppression vaut pour chacune d'elles (Cymbra Music, Cymbra Lingua).
- **Cymbra Lingua** : vous pouvez aussi effacer **uniquement** vos données Lingua, sans
  supprimer votre compte (voir *Annexe B*).
- Pour toute autre demande, écrivez à **privacy@cymbra.app**. Vous pouvez aussi
  introduire une réclamation auprès de la **CNIL** (www.cnil.fr).

## 8. Mineurs

Cymbra n'est pas destinée aux enfants de moins de 12 ans ; nous ne collectons pas
sciemment leurs données.

## 9. Modifications

Nous pouvons mettre à jour cette politique ; la date « Dernière mise à jour » ci-dessus
sera modifiée en conséquence. En cas de changement important, nous vous en informerons.

## 10. Contact

**gfortin@neetrof.fr** — NEETROF, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCE.

---

## Annexe A — Cymbra Music

Le service **Cymbra Music** vous permet de téléverser vos propres contenus. À ce titre,
en complément du §2, nous traitons :

| Donnée | Origine | Finalité |
|---|---|---|
| Fichiers téléversés (partitions, sons de piano / *soundfonts*) | vous | fournir les fonctions de lecture et de pratique |
| Métadonnées associées (nom du fichier, attestation d'origine, horodatage) | vous | gestion et traçabilité de vos contenus |

- **Base légale** : exécution du contrat (fourniture de la fonctionnalité).
- **Conservation** : tant que vous conservez le contenu ; la **suppression** retire le
  fichier et son enregistrement (voir CGU, *Annexe A — Cymbra Music*).
- Ces fichiers sont **hébergés dans l'Union européenne** (France), comme le reste de vos
  données.

---

## Annexe B — Cymbra Lingua

**Cymbra Lingua** (extension de navigateur et app Safari) surligne, dans les pages que vous
lisez, les mots que vous ne connaissez pas encore dans les langues que vous étudiez. En
complément du §2 :

**Ce qui reste sur votre appareil.** Le texte des pages est analysé **sur votre appareil** et
n'est jamais envoyé, sauf la phrase que vous envoyez vous-même avec la **Traduction en ligne**
et la traduction d'un mot que vous choisissez de garder sur sa carte (ci-dessous). Restent aussi sur l'appareil : le détail de vos lectures (quels mots
vous avez rencontrés, sur quelles pages), l'**adresse de la page** où vous ajoutez un mot à
votre deck et, si vous n'êtes pas connecté, l'ensemble de vos données Lingua.

**Vos langues.** Vos réglages de langues restent sur l'appareil : les langues que vous
étudiez et votre langue maternelle ne sont pas enregistrées en tant que telles. Chaque statut,
niveau, carte et jour de statistiques synchronisé est rangé sous la langue étudiée à laquelle
il appartient, et votre langue maternelle parvient à Cymbra comme langue de la traduction de
chaque carte et avec vos statistiques de chaque jour ; la langue de l'interface de Lingua, ou
celle de votre navigateur, est transmise à votre compte Cymbra comme langue du compte (voir
ci-dessous).

**Traduction étendue.** Ce réglage, désactivé par défaut et propre à chaque appareil (il
n'est pas synchronisé), traduit sur votre appareil la phrase où se trouve votre sélection.
Quand vous l'activez, l'extension télécharge **une fois** le ou les modèles de vos paires de
langues — un modèle (environ 26 Mo) quand la traduction est directe, deux (environ 52 Mo)
quand elle passe par l'anglais —, les modèles de Firefox Translations publiés par Mozilla
sous licence MPL 2.0, depuis `models.cymbra.app` ; les modèles d'une paire ajoutée ensuite ne
sont téléchargés que lorsque vous le demandez. La traduction se fait ensuite **sur votre
appareil**, et le texte des pages n'est toujours jamais envoyé. Ces téléchargements ne
contiennent ni texte des pages, ni données Lingua, ni jeton de compte, ni identifiant
d'installation, et Cymbra ne les associe à aucun compte ni à aucune installation. Comme pour
toute page web, l'hébergeur de ces fichiers (Cloudflare) voit l'adresse IP de l'appareil qui
les demande. Désactiver le réglage supprime les modèles de l'appareil. L'app Cymbra Lingua
garde sa propre copie des modèles : elle ne la télécharge, depuis la même adresse et dans les
mêmes conditions, qu'après que vous l'avez acceptée dans l'app, quand le réglage est activé dans
l'extension Safari du même appareil, et la supprime quand il est désactivé.

**Traduction en ligne.** Ce réglage, désactivé par défaut et propre à chaque appareil (il
n'est pas synchronisé), n'est proposé qu'aux comptes qui y ont accès. Quand il est activé, un
bouton « Traduction en ligne » apparaît sur la traduction d'une sélection de plusieurs mots, et
sur une carte en révision. **Rien n'est envoyé tant que vous n'appuyez pas sur ce bouton.** Quand
vous appuyez, la phrase qui contient votre sélection — ou la phrase de la carte révisée, le mot
repéré par sa position —, la position de la sélection et la paire de langues sont envoyées
à Cymbra, avec votre jeton de compte, puis par Cymbra à **Google** (API Gemini) pour être
traduites, ou à **Anthropic** (API Claude) quand Google ne répond pas. Ni Cymbra ni ces
fournisseurs ne conservent la phrase ni sa traduction, et ces fournisseurs ne s'en servent pas
pour entraîner leurs modèles. Pour limiter l'usage et en connaître le coût, Cymbra compte, pour
votre compte, le nombre de traductions en ligne du jour (compteur effacé au bout d'un jour) et
tient des totaux quotidiens par fournisseur et par paire de langues, sans compte ni texte. Dans
l'app Cymbra Lingua, la Traduction en ligne demande de vous connecter **dans l'app** (avec Apple,
Google ou votre email et votre mot de passe) : l'app ouvre alors sa propre session de votre compte, distincte de celle de
l'extension, que vous pouvez fermer depuis l'app.

**Traduction gardée.** Une traduction automatique (sur l'appareil ou en ligne) n'est conservée
que si, en révision, vous choisissez de garder la traduction du mot comme traduction de sa
carte : seul ce mot traduit est gardé, sur la carte, et synchronisé avec elle ; la phrase
traduite ne l'est jamais. Une traduction que vous ne gardez pas n'est conservée nulle part.

**Lecture à voix haute.** La lecture à voix haute utilise une voix de la langue lue,
installée sur votre appareil : le texte lu ne le quitte pas. Si aucune voix de cette langue
n'y est installée, vous pouvez activer en secours les voix en ligne de votre navigateur
(réglage désactivé par défaut, propre à chaque appareil) : le texte lu est alors envoyé par
**votre navigateur** au fournisseur de ces voix (Google, pour Chrome), selon les conditions
de ce fournisseur. Cymbra ne reçoit pas ce texte. Dès qu'une voix de cette langue est
installée sur l'appareil, elle est utilisée à la place.

**Ce qui est synchronisé si vous êtes connecté** à votre compte Cymbra :

| Donnée | Origine | Finalité |
|---|---|---|
| Statut de chaque mot (connu, en apprentissage, ignoré) et niveau déclaré, pour chaque langue étudiée | vous | retrouver votre progression sur vos appareils |
| Deck de révision, pour chaque langue étudiée : le mot, la phrase où vous l'avez trouvé, sa traduction — celle du dictionnaire, ou la traduction automatique du mot que vous avez choisi de garder, jamais une phrase traduite —, la langue de cette traduction et son état de révision — **sans** l'adresse de la page | vous | réviser sur tous vos appareils |
| Statistiques par jour et par langue étudiée (mots appris, révisions, nombre de mots lus et, parmi eux, de mots nouveaux), avec votre langue maternelle ce jour-là (celle des traductions) | calculées sur l'appareil | afficher vos statistiques ; chiffres d'usage **agrégés** pour faire fonctionner le service et, sans nom ni nombre de personnes, publiés sur les canaux de la communauté Cymbra (voir §2) |
| Identifiant d'installation aléatoire | généré par l'extension, et par l'app Cymbra Lingua pour elle-même | départager deux appareils lors de la synchronisation |

C'est l'extension qui synchronise ces données. Si vous vous connectez **aussi dans l'app Cymbra
Lingua**, avec le compte qu'utilise l'extension de Safari, l'app synchronise elle-même les statuts
et les cartes des révisions faites dans l'app, sous sa propre session et son propre identifiant
d'installation, avec le même contenu (sans adresse de page) ; les statistiques du jour passent
toujours par l'extension. Rien de plus n'est collecté.

- **Base légale** : exécution du contrat (fourniture de la synchronisation) ; intérêt
  légitime pour la publication de chiffres agrégés (voir §2).
- **Conservation** : tant que votre compte existe, ou jusqu'à ce que vous effaciez vos
  données Lingua.
- **Effacer vos données Lingua sans supprimer votre compte** : dans l'extension,
  *Compte → Tes données → Effacer mes données Lingua*. L'effacement porte sur le serveur
  et sur tous vos appareils (chacun s'efface à sa prochaine synchronisation) ; votre
  compte Cymbra et vos autres apps Cymbra ne sont pas touchés.
- La **suppression de votre compte** (§7) efface aussi toutes vos données Lingua.
- Ces données sont **hébergées dans l'Union européenne** (France), comme le reste de vos
  données.

**Votre compte Cymbra.** Quand vous créez votre compte, demandez un code ou un nouveau mot de
passe, ou ajoutez un mot de passe depuis Lingua, l'extension indique à Cymbra une langue —
celle de l'interface de Lingua ou celle de votre navigateur — que votre compte garde comme sa
langue : Cymbra vous écrit dans cette langue (vérification, réinitialisation de mot de passe)
et vos autres apps Cymbra peuvent l'adopter.

**Règles du Chrome Web Store sur les données utilisateur.** « The use of information received
from Google APIs will adhere to the Chrome Web Store User Data Policy, including the Limited Use
requirements. » Autrement dit : l'utilisation des informations reçues des API Google respectera
les Règles du Chrome Web Store sur les données utilisateur (*Chrome Web Store User Data
Policy*), y compris les exigences d'utilisation limitée (*Limited Use*). Pour Cymbra Lingua, ces
informations sont : le texte des pages que vous lisez, analysé sur votre appareil ; votre compte
Cymbra (adresse email, identifiant de connexion, sessions, langue du compte) ; ce qui est
synchronisé si vous êtes connecté (statuts et niveaux des mots, deck de révision, statistiques
quotidiennes, identifiant d'installation) ; et la phrase envoyée par la **Traduction en
ligne**, seulement quand vous appuyez sur son bouton. Cymbra n'utilise ces données qu'aux fins
décrites dans la présente politique, ne les partage qu'avec les sous-traitants cités au §4 et
pour ces fins, ne les vend pas et ne s'en sert pas pour de la publicité. L'équipe de Cymbra ne
voit vos données Lingua que sous forme de totaux, sans compte, sauf pour vous aider à votre
demande, pour la sécurité ou pour respecter la loi.
