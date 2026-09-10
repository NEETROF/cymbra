# Tasks — add-lingua-connected-clients

## 1. Extension — compte et session

- [ ] 1.1 Transport gRPC-web Connect-ES dans `apps/lingua-extension` : clone adapté de `apps/back-office/src/lib/transport.ts` + `api.ts` (intercepteurs auth / refresh single-flight / session-expiry, seam de test), URL backend par variable de build
- [ ] 1.2 Stockage des jetons : access en `chrome.storage.session`, refresh en `chrome.storage.local` ; reprise de session au démarrage (refresh → nouveau `TokenPair`) ; tests vitest du cycle
- [ ] 1.3 UI compte (popup d'icône + page de réglages, tokens Cymbra) : « Continuer avec Google » (`chrome.identity.launchWebAuthFlow` → `SignInOidc`), email/mot de passe (`SignInLocal`), état connecté (email masquable), déconnexion (`Logout` + purge des jetons, état local intact)
- [ ] 1.4 Doc dev : origine `chrome-extension://` stable par clé de manifest ; ajout de l'origine dev à `CYMBRA_ALLOWED_WEB_ORIGINS` de l'environnement local uniquement

## 2. Extension — outbox et synchronisation

- [ ] 2.1 Outbox locale (op-log versionné dans `chrome.storage.local`) : chaque mutation de statut/carte enfile une op horodatée (device_id généré à l'install) ; vidage par lots avec reprise par offset
- [ ] 2.2 Pull par curseur + application au store local (LWW côté client symétrique au serveur) ; propagation cross-onglets existante (`storage.onChanged`) déclenchée par les changements tirés
- [ ] 2.3 Fusion du premier sign-in : poussée intégrale de l'état pré-compte (horodatages d'origine conservés), puis pull du snapshot fusionné ; test du scénario « deux appareils avec états locaux disjoints »
- [ ] 2.4 Orchestration d'arrière-plan : sync au réveil du service worker, après un lot de mutations et à l'ouverture du side panel ; indicateur discret d'état de sync + action manuelle dans les réglages ; aucune requête tant que non connecté
- [ ] 2.5 Stats : agrégation locale par (jour, langue) → `UpsertDailyStats` avec device_id ; tests vitest (idempotence du re-push)

## 3. Extension — écran de stats

- [ ] 3.1 Écran de stats (page d'extension, tokens Cymbra) : séries par jour × langue (mots appris, révisions, expositions), plage 7/30/90 jours ; connecté = `GetStats` consolidé (portée « tous les appareils »), sinon agrégats locaux (portée « cet appareil ») ; mention « hors sessions d'agents »
- [ ] 3.2 Lint des chaînes UI étendu aux nouveaux écrans (compte, sync, stats) : aucune occurrence de « lemme » (« forme du dictionnaire », « mots différents »)

## 4. App Apple — connexion native et sync

- [ ] 4.1 Écran de connexion natif (SwiftUI, charte Cymbra) : Sign in with Apple (`ASAuthorizationController` → `SignInOidc` Apple) + « Continuer avec Google » + email/mot de passe, via tonic natif d'audience `lingua` ; jetons en Keychain
- [ ] 4.2 Partage de session app ↔ extension Safari via App Group (un compte par appareil) : l'extension synchronise sous la session de l'app, aucun flow OAuth dans Safari ; trancher jetons-par-handler vs copie App Group (question ouverte du design) et documenter
- [ ] 4.3 Sync app : même outbox/pull/fusion que l'extension (types `lingua-core` partagés) pour statuts, cartes, stats ; premier sign-in fusionne l'état local de l'appareil
- [ ] 4.4 Écran de stats de l'app (mêmes règles que 3.1) ; vérification manuelle croisée : mot marqué sur Mac visible sur iPhone, carte iOS révisable sur desktop, stats consolidées justes
- [ ] 4.5 TestFlight interne avec le backend de dev ; parcours de review App Store re-déroulé (Sign in with Apple présent, privacy labels mis à jour : données de compte + contenu utilisateur synchronisé)

## 5. Gates et finitions

- [ ] 5.1 vitest vert sur `apps/lingua-extension` ; `python3 scripts/check_ci_units.py --list` confirme que toutes les unités touchées restent surveillées
- [ ] 5.2 `openspec validate add-lingua-connected-clients --strict` final + mise à jour des specs si l'implémentation a fait bouger un contrat
