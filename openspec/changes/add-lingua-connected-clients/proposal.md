# add-lingua-connected-clients — Cymbra Lingua : sign-in, synchronisation et stats dans l'extension et l'app

## Why

`add-lingua-backend` a livré un serveur complet mais **inerte pour l'utilisateur** : audience `lingua`, trois services de sync, protocole LWW — et aucun client qui s'y connecte. Ce change ferme la boucle : la connexion Cymbra ID dans l'extension et l'app Apple, l'outbox cliente, la fusion du store pré-compte et l'écran de stats consolidé. Le cas réel du fondateur — Chrome sur macOS + Safari iOS — devient enfin cohérent : un mot appris sur le Mac est « connu » sur l'iPhone, une carte créée sur mobile est révisable dans le side panel desktop. Le local-first de la pile reste le mode par défaut — l'extension et l'app vivent entièrement sans compte, la synchronisation est **opt-in** à la connexion.

**Position dans la pile** (12 changes) : **12e et dernier**. **Prérequis explicites : `add-lingua-extension-review`** (side panel, cartes, statuts — les surfaces qui gagnent l'UI compte et l'outbox), **`add-lingua-apple`** (app conteneur et extension Safari) et **`add-lingua-backend`** (audience, services, protocole). `add-lingua-firefox` est couvert par ricochet (même code d'extension ; `browser.identity.launchWebAuthFlow` existe).

## What Changes

- **Connexion dans l'extension** : bearer `TokenPair` sur gRPC-web (Connect-ES — template `apps/back-office/src/lib/transport.ts` + `api.ts` : intercepteurs auth / refresh single-flight / session-expiry), refresh token rotatif (la détection de réutilisation avec révocation de famille existe déjà côté serveur), access token en `storage.session`, refresh en `storage.local`. Deux méthodes : **OIDC Google** via `chrome.identity.launchWebAuthFlow` (client id déjà au CSV `CYMBRA_GOOGLE_AUDIENCE` depuis `add-lingua-backend`) et **email/mot de passe** existants (`SignInLocal`).
- **Connexion dans l'app Apple** : native dans l'app conteneur (`SignInOidc` via tonic natif, jetons en Keychain). **Sign in with Apple est obligatoire dès qu'un login tiers existe sur iOS** (règle App Store) — le backend supporte déjà l'OIDC Apple (`CYMBRA_APPLE_AUDIENCE`). L'extension Safari consomme la session de l'app via l'App Group (un compte par appareil, pas de flow OAuth dans Safari).
- **Sync opt-in, local-first préservé** : sans compte, rien ne change ; une déconnexion arrête la sync sans toucher l'état local.
- **Outbox client + pull delta** : op-log local des mutations vidé par lots idempotents, pull par curseur, application LWW symétrique au serveur ; au premier sign-in, le store local pré-compte est **fusionné** (poussée intégrale aux horodatages d'origine, puis pull du snapshot fusionné).
- **Écran de stats consolidé** dans l'extension et l'app, alimenté par `StatsService` quand connecté, par l'état local sinon ; portée affichée (« tous les appareils » / « cet appareil ») ; mention « hors sessions d'agents ».
- **Le plugin Claude Code reste local-only** : le store `~/.lingua/` ne gagne aucun chemin réseau (sa sync = change ultérieur dédié) ; l'invariant « aucune connexion réseau » reste testé.
- Vocabulaire UI : jamais « lemme » dans les nouveaux écrans (compte, sync, stats) — « forme du dictionnaire », « mots différents ».

## Capabilities

### New Capabilities
- `lingua-sync` (moitié clients) : compte optionnel et local-first préservé, connexion extension (gRPC-web bearer, OIDC Google + email/mdp, jetons rangés par volatilité), connexion native app Apple (Sign in with Apple inclus, session partagée avec l'extension Safari), fusion du store pré-compte au premier sign-in, plugin Claude Code hors sync. _La moitié serveur (audience, module, protocole, vie privée serveur, purge) est le change `add-lingua-backend`._
- `lingua-stats` (moitié clients) : l'écran de stats dans l'extension et l'app — consolidé quand connecté, local sinon, portée affichée — et la règle de vocabulaire sans jargon. _Le stockage et la lecture consolidée serveur sont le change `add-lingua-backend`._

### Modified Capabilities
_Aucune. Les capabilities `lingua-*` de la pile locale ne sont pas modifiées : leur mode local reste le défaut, ce change ajoute le mode connecté par-dessus. Le socle (`backend-auth`, flags, analytics) est consommé tel quel._

## Impact

- **Produits** : Lingua (extension + app Apple : UI compte, outbox, écran stats) ; **Cymbra ID consommé** (OIDC Google/Apple, refresh rotatif) ; **backend Lingua consommé** (`add-lingua-backend`) ; Music / Live / back-office / site : **intacts**.
- **Arborescence** : `apps/lingua-extension` (transport Connect-ES, UI compte, outbox, écran stats), `apps/lingua-apple` (écran de connexion SwiftUI, App Group, sync, écran stats).
- **Env/deploy** : rien de nouveau côté serveur (tout est livré par `add-lingua-backend`) ; doc dev : ajout de l'origine d'extension de dev à `CYMBRA_ALLOWED_WEB_ORIGINS` de l'environnement local uniquement.
- **CI** : aucune unité nouvelle — `apps/lingua-extension` et `apps/lingua-apple` sont déjà surveillées par leurs lanes de la pile ; vitest étendu (session, outbox, stats) ; parcours TestFlight re-déroulé (Sign in with Apple, privacy labels : données de compte + contenu utilisateur synchronisé).
- **Hors périmètre (changes ultérieurs)** : sync du plugin Claude Code (transcripts confidentiels — local-only réaffirmé ici), sync des médias/images de cartes (chiffrée, pattern cache scores), console back-office Lingua et rôles/scope `lingua`, TextProfile/catalogue de livres, web de gestion de compte Lingua (le site existant couvre déjà la gestion Cymbra ID).
