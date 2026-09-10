# Design — add-lingua-connected-clients

## Context

La pile locale a acté deux choses qui contraignent ce change : (1) chaque surface a un **état local versionné** dont les schémas partagent les types de `lingua-core` — « pour que la fusion soit mécanique » (décisions de `add-lingua-extension-review` et `add-lingua-apple`) : ce change est cette fusion ; (2) le transport extension avait déjà été tranché à l'exploration : **bearer gRPC-web** (Connect-ES, template back-office), jamais le cookie `/web/auth` (SameSite=Strict + CORS exact-origin = hostile aux extensions par design).

Le serveur est entièrement livré par `add-lingua-backend` : audience `lingua` en configuration, `CYMBRA_ALLOWED_WEB_ORIGINS`, les trois services `cymbra.lingua.v1`, le protocole (op-log, LWW, curseur, snapshot), la purge. Les décisions de protocole et de vie privée serveur (allow-list) y sont prises et ne sont pas rediscutées ici — ce design ne couvre que le **côté client** : comment on se connecte, où vivent les jetons, comment l'outbox et la fusion s'orchestrent, et l'écran de stats.

## Goals / Non-Goals

**Goals :**
- Un utilisateur connecté retrouve les mêmes statuts de mots, les mêmes cartes et des stats consolidées sur tous ses appareils (extension Chromium/Firefox, Safari + app Apple).
- Sans compte, rien ne change : le local-first de la pile reste le mode par défaut ; la sync est opt-in à la connexion.
- L'UI lit toujours le store local ; la sync est un échange d'arrière-plan invisible.

**Non-Goals :**
- Sync du plugin Claude Code (`~/.lingua/`) — les transcripts sont confidentiels ; change ultérieur avec son propre design (auth CLI loopback PKCE).
- Médias/images de cartes (v1 : le champ `media` du schéma ne se synchronise pas ; la sync chiffrée viendra avec la capture d'image).
- Tout ce qui est serveur (protocole, schéma, purge, CORS) — livré par `add-lingua-backend`.
- Multi-comptes par appareil, partage entre utilisateurs.

## Decisions

### D1 — Transport extension : gRPC-web bearer, tokens séparés par volatilité
Connect-ES + `createGrpcWebTransport`, clone du template back-office (`transport.ts` : intercepteur auth `Authorization: Bearer`, refresh **single-flight** avec retry unique, session-expiry en dernier recours ; `api.ts` : seam `setClientsForTest`). Rangement des jetons par volatilité : **access token en `chrome.storage.session`** (mémoire, purgé à la fermeture du navigateur, invisible du disque) ; **refresh token en `chrome.storage.local`** (la session survit au redémarrage — c'est lui qui est rotatif et révocable serveur). Alternative rejetée : les deux en `storage.local` — un access token persisté ne vaut que quelques minutes mais traîne sur disque ; les deux en `session` — re-login à chaque redémarrage, inacceptable pour une extension de lecture quotidienne.

### D2 — OIDC dans l'extension : `chrome.identity.launchWebAuthFlow`, client id du CSV
Google uniquement côté extension : `launchWebAuthFlow` ouvre le flow OAuth (redirect `https://<ext-id>.chromiumapp.org/`), l'extension récupère l'`id_token` et appelle `SignInOidc(provider=google, audience=lingua)`. Le client id (type Web, redirect chromiumapp.org) est déjà au **CSV `CYMBRA_GOOGLE_AUDIENCE`** (livré par `add-lingua-backend`, précédent exact du client desktop). Email/mot de passe = `SignInLocal` existant, mêmes écrans de reset que le site. Firefox : `browser.identity.launchWebAuthFlow` existe — même code. **Pas de Sign in with Apple dans l'extension** : la règle App Store ne s'applique qu'aux apps ; sur Safari, la connexion vit dans l'app conteneur (D3).

### D3 — App Apple : connexion native, Sign in with Apple obligatoire
L'app conteneur se connecte en **tonic natif** (`SignInOidc`/`SignInLocal` — pas de gRPC-web : c'est une app, pas un navigateur), jetons dans le Keychain, refresh partagé avec l'extension Safari via l'App Group (un seul compte par appareil, l'extension consomme la session de l'app par le handler natif — pas de flow OAuth dans Safari). Dès qu'un login tiers (Google) est proposé sur iOS, **Sign in with Apple doit l'être aussi** (guideline App Store) : le backend le supporte déjà (`CYMBRA_APPLE_AUDIENCE`), l'app utilise `ASAuthorizationController` natif. Ordre des boutons : Apple d'abord sur iOS, conformément aux attentes de review.

### D4 — Fusion du store pré-compte au premier sign-in : upload puis merge, le local reste maître d'affichage
À la première connexion d'un appareil : l'état local (statuts, cartes, stats de la pile locale) est **poussé intégralement** comme autant d'ops horodatées (les horodatages locaux d'origine sont conservés), puis le client tire le snapshot fusionné. Le merge serveur est le LWW ordinaire du protocole (`add-lingua-backend`) — le premier sign-in n'est pas un cas spécial, juste une grosse outbox. Ensuite le modèle reste **local-first** : l'UI lit toujours le store local ; la sync est un échange d'arrière-plan (au réveil du service worker, après un lot de mutations, à l'ouverture du side panel). Une déconnexion arrête la sync sans toucher l'état local.

### D5 — Le plugin Claude Code reste local-only — réaffirmé, pas oublié
Le store `~/.lingua/` ne gagne **aucun** chemin réseau dans ce change : les transcripts sont du code employeur, et l'invariant « aucune connexion réseau » de `add-lingua-agent` (spec `lingua-agent-capture`) est testé. Sa sync sera un change dédié (auth CLI loopback PKCE, opt-in explicite par machine). Conséquence assumée : les statuts extension et plugin continuent de diverger — c'était déjà l'état de la pile locale, et la fusion mécanique reste possible le jour venu (mêmes types `lingua-core`).

## Risks / Trade-offs

- [Refresh token en `storage.local` : lisible par un malware local] → même exposition que tout secret de profil navigateur ; mitigé par la rotation + détection de réutilisation (la famille est révoquée au premier replay) et `RevokeAllSessions` accessible.
- [Poussée initiale volumineuse au premier sign-in (des années de statuts)] → lots bornés + reprise par offset d'outbox (portés par le protocole de `add-lingua-backend`) ; l'ordre des ops préserve les horodatages, donc une interruption est reprise sans corruption.
- [Origine `chrome-extension://<id>` : l'id change entre dev (unpacked) et store] → l'id publié est stable (clé au manifest) ; en dev, l'origine locale s'ajoute à `CYMBRA_ALLOWED_WEB_ORIGINS` de l'environnement de dev seulement. Firefox (`moz-extension://<uuid>` aléatoire par install) : à vérifier à l'intégration, le repli documenté étant un id d'origine par `browser_specific_settings`.
- [Divergence extension ↔ plugin Claude Code maintenue] → assumée et réaffirmée (D5) ; documentée dans l'UI de stats (« hors sessions d'agents »).
- [Horloge client fausse : un appareil décalé « gagne » des conflits LWW] → tie-break et clamp côté serveur (`add-lingua-backend`) ; le pire cas reste corrigeable d'un clic (reposer le statut).

## Migration Plan

1. Prérequis : `add-lingua-backend` déployé et provisionné (audience, origines, `CYMBRA_LINGUA_DATABASE_URL` active).
2. Extension d'abord (UI compte + outbox + stats) — dogfooding Chrome macOS du fondateur.
3. App Apple ensuite (TestFlight interne avec le backend de dev, puis review App Store — Sign in with Apple présent, privacy labels à jour).
4. Rollback : la déconnexion (ou le retrait de `CYMBRA_LINGUA_DATABASE_URL` côté serveur) fait retomber les clients en local-only — c'est leur mode par défaut ; les schémas locaux ne migrent pas destructivement, donc un client « sync » continue de fonctionner seul.

## Open Questions

- Périodicité exacte de la sync d'arrière-plan côté extension (au réveil du SW + `chrome.alarms` ? seuil d'ops ?) — à mesurer au dogfooding, sans impact sur le protocole.
- Le partage de session app ↔ extension Safari via App Group : l'extension consomme-t-elle les jetons par le handler natif à chaque appel, ou une copie App Group avec invalidation ? À trancher à l'implémentation (le contrat — un seul compte par appareil Apple — ne bouge pas).
- Faut-il exposer un bouton « Synchroniser maintenant » ou rester silencieux (indicateur discret seulement) ? Penché : indicateur + action manuelle dans les réglages, jamais de friction dans la lecture.
