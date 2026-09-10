# admin-lingua-console — section Lingua du back-office (OPS)

## ADDED Requirements

### Requirement: Autorisation scopée des RPC admin Lingua
Chaque RPC d'administration Lingua (`LinguaAdminService`) SHALL exiger le rôle `admin` **dans le scope `lingua`** (`require_admin_in_scope`), le break-glass `global/admin` restant accepté ; un jeton sans ce rôle scopé — y compris un jeton console portant `admin` dans un autre scope — SHALL être refusé avec `PermissionDenied`. Le service SHALL être monté derrière l'intercepteur d'authentification strict (jeton valide obligatoire, audience `back-office` admise).

#### Scenario: Admin d'un autre produit refusé
- **WHEN** un porteur de `music/admin` (sans rôle dans le scope `lingua`) appelle `AdminGetLinguaUsage`
- **THEN** la réponse est `PermissionDenied`, bien que son set de rôles plat contienne `admin`

#### Scenario: Admin lingua et break-glass acceptés
- **WHEN** un porteur de `lingua/admin` ou de `global/admin` appelle un RPC de `LinguaAdminService`
- **THEN** l'appel est autorisé

### Requirement: Écran réservé aux admins du scope lingua
Le back-office SHALL n'afficher l'entrée de navigation « Lingua » et ne servir la route `/lingua` qu'aux administrateurs du scope `lingua` (`meta: { admin: true, adminScope: "lingua" }`) ; tout autre profil SHALL être redirigé sans erreur brute. Ce gate de route est une commodité UX : chaque RPC reste indépendamment gaté côté serveur.

#### Scenario: Modérateur sans le lien ni l'accès
- **WHEN** un modérateur non-admin est connecté à la console et navigue vers `/lingua`
- **THEN** le lien « Lingua » est absent de la navigation et la route le redirige hors de `/lingua`

#### Scenario: Admin music-only redirigé
- **WHEN** un administrateur ne détenant `admin` que dans le scope `music` navigue vers `/lingua`
- **THEN** la route le redirige (le scope `lingua` lui manque), comme le ferait le serveur s'il appelait les RPC directement

### Requirement: Agrégats d'usage Lingua
L'écran « Lingua » SHALL présenter, sur une fenêtre de dates filtrable (30 jours par défaut), des agrégats servis par les RPC admin du backend Lingua : tuiles (comptes actifs synchronisés, mots appris, révisions), séries temporelles par jour, et répartition par langue étudiée. Les agrégats sont calculés côté serveur sur les données de synchronisation existantes — aucune télémétrie nouvelle — et l'écran mentionne que seuls les comptes qui synchronisent sont comptés. Les libellés sont vulgarisés : aucune chaîne UI ne contient le mot « lemme » (dire « mots appris », « mots différents »).

#### Scenario: Consultation d'une fenêtre
- **WHEN** un admin lingua ouvre l'écran avec la fenêtre par défaut
- **THEN** les tuiles, les séries par jour et la répartition par langue étudiée de la fenêtre s'affichent, avec la mention « comptes synchronisés »

#### Scenario: Vocabulaire vulgarisé
- **WHEN** les chaînes UI de l'écran (en et fr) sont passées au lint du vocabulaire
- **THEN** aucune occurrence de « lemme » n'est trouvée

### Requirement: Vie privée — agrégats seulement, garantie par le schéma
La console Lingua SHALL ne présenter que des agrégats : aucun RPC de `LinguaAdminService` ne SHALL retourner de donnée attribuable à un compte individuel (mots rencontrés, decks, statistiques ou activité d'un compte), aucun message de réponse de `lingua_admin.proto` ne SHALL comporter de champ d'identifiant de compte, et l'écran ne SHALL offrir aucune recherche ni vue par compte. Cette limite est le périmètre du produit admin Lingua, pas une restriction provisoire.

#### Scenario: Le schéma des réponses ne peut pas fuiter un compte
- **WHEN** les messages de réponse de `lingua_admin.proto` sont inspectés
- **THEN** aucun ne contient de champ d'identifiant de compte ni de contenu par compte — une fuite exigerait un changement de `.proto`, visible en review et au gate proto

#### Scenario: Aucune vue par compte dans la console
- **WHEN** un admin lingua parcourt l'écran « Lingua »
- **THEN** aucune recherche par compte, aucun lien vers un compte et aucun détail individuel ne sont proposés

### Requirement: Registre des versions de packs de données
L'écran SHALL afficher un registre **en lecture seule** des packs de données publiés — `pack_version`, `analyzer_version`, paire L2→L1, date de build CI, taille, NOTICE — servi par `AdminListDataPacks` depuis le manifeste produit par le pipeline de packs et versionné avec le repo. La distribution des packs SHALL rester embarquée dans l'extension en v1 : le registre est informatif et prépare l'OTA futur, il ne pilote aucune distribution.

#### Scenario: Consultation du registre
- **WHEN** un admin lingua ouvre la section packs de l'écran
- **THEN** chaque pack publié apparaît avec sa version, son `analyzer_version`, sa paire L2→L1, sa date de build CI, sa taille et sa NOTICE consultable, sans aucune action d'écriture ni de publication

### Requirement: Flags Lingua via la console existante
Les feature flags Lingua SHALL être des clés déclarées dans le registry backend (`KeyDef`, app `lingua`) et administrées par la console `/flags` existante avec son gating actuel ; ce change ne SHALL introduire aucune interface de flags nouvelle.

#### Scenario: Une clé lingua apparaît dans /flags
- **WHEN** une clé de flag est déclarée dans le registry backend avec l'app `lingua`
- **THEN** elle est listée et administrable dans la console `/flags` existante, sans écran ni composant nouveau

### Requirement: États async localisés de l'écran
Chaque ressource asynchrone de l'écran (agrégats, séries, packs) SHALL être modélisée en union discriminée `Async<T>` matchée exhaustivement, chargée exclusivement via un store Pinia derrière le seam `api()` ; un échec RPC SHALL aboutir à un message d'erreur localisé dans l'union — jamais un code gRPC ou une exception brute à l'écran, jamais un appel API depuis un composant.

#### Scenario: Échec d'un RPC d'agrégats
- **WHEN** `AdminGetLinguaUsage` échoue pendant le chargement
- **THEN** l'écran rend l'état d'erreur avec un message localisé, la cause technique n'étant que journalisée
