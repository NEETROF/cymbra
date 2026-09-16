## Why

Le back-office traite un compte à deux endroits qui s'ignorent : `/roles` liste tous
les comptes (rôles, plan, bêtas) mais ne sait rien montrer en détail, et `/plans`
sait tout d'**un** compte à la fois — à condition de retaper son handle dans un champ
de recherche. Un admin qui repère un compte dans la liste doit recopier son handle
dans l'autre écran pour voir son abonnement ; à l'inverse la console `/plans` ne
permet aucun parcours (pas de liste, pas de filtre, un compte à la fois).

Les deux moitiés parlent déjà du même objet — la colonne `PLAN` de `/roles` affiche
exactement l'abonnement effectif que `/plans` détaille. Il manque le lien entre elles.

## What Changes

- **BREAKING (URL back-office)** `/roles` devient `/users`, intitulé « Utilisateurs » :
  le même annuaire paginé (handle, nom, rôles, plan, bêtas), mais dont chaque ligne est
  **cliquable**. `/roles` redirige vers `/users` (favoris, liens existants).
- Nouvelle page de détail **`/users/{user_id}`** qui regroupe tout ce qui concerne un
  compte, aujourd'hui éclaté entre des boutons de ligne et l'autre écran :
  - **abonnement** — plan effectif, lignes de droit, adhésions bêta, attribution de
    premium, inscription à une campagne, révocations, lien RevenueCat (le bloc
    « Recherche de compte » de `/plans`, à l'identique) ;
  - **rôles** par scope autorisé, avec attribution/retrait ;
  - **historique d'audit** des rôles, **fiabilité curateur**, **révocation des sessions**.
- La liste `/users` n'expose plus que la lecture et le filtrage (recherche, scope, plan,
  bêta, pagination) : les cinq boutons d'action et les deux panneaux dépliants par ligne
  disparaissent au profit du clic vers le détail.
- **BREAKING (URL back-office)** `/plans` devient `/campaigns`, intitulé « Campagnes »,
  et ne garde que la gestion des campagnes : cycle de vie, frappe et révocation de codes,
  liste et export des membres. Sa recherche de compte est **supprimée** (elle vit dans
  `/users`). Les membres d'une campagne deviennent des liens vers `/users/{user_id}`.
- La page de détail reste **gardée par scope** : un admin non-`music` y voit rôles,
  historique, fiabilité et sessions, mais aucun bloc abonnement — comme aujourd'hui sur
  la liste.

Aucun changement de contrat serveur : `ListAccounts` (avec son filtre `ids`),
`LookupAccountPlan` (qui accepte déjà un `user_id`), `GetPlansForAccounts`,
`ListRoleGrants`, `GetCuratorReliability` et les mutations plan/rôle existent tels quels.

## Capabilities

### New Capabilities
<!-- Aucune : la restructuration porte sur des capabilities existantes. -->

### Modified Capabilities
- `admin-account-directory`: la page « Roles » devient la page « Utilisateurs » (`/users`)
  et ses actions par compte migrent vers une page de détail adressable
  `/users/{user_id}` ; l'annuaire devient une surface de lecture/filtrage/navigation.
- `admin-plan-console`: la consultation et les mutations d'abonnement d'un compte
  quittent la console `/plans` pour la page de détail utilisateur ; la console résiduelle
  est renommée « Campagnes » (`/campaigns`) et se limite aux campagnes, codes et membres.

## Impact

**Produit impacté : back-office (`apps/back-office`) uniquement.** Cymbra ID, Music, Live
et le site ne sont pas touchés — le back-office **consomme** les RPC existants d'identité
et de plans, aucun n'est modifié ni ajouté.

- Vues : `RolesView.vue` → `UsersView.vue` (liste), nouvelle `UserDetailView.vue`,
  `PlansView.vue` → `CampaignsView.vue` (amputée de son bloc de recherche).
- Router + nav : routes `/users`, `/users/:userId`, `/campaigns` ; redirections depuis
  `/roles` et `/plans` ; entrées de nav `nav.users` / `nav.campaigns`.
- Stores : `roles` gagne le chargement d'**un** compte par `ids:[userId]` (deep-link,
  F5) ; `plans` garde sa recherche mais adressée par `user_id`. Pas de nouvelle
  dépendance.
- i18n : espaces de clés `roles.*` → `users.*`, `plans.*` scindé entre le détail et les
  campagnes ; `en` et `fr` restent alignés.
- Tests : `test/roles.spec.ts`, `test/plans.spec.ts`, `e2e/roles.spec.ts`,
  `e2e/plans.spec.ts` réécrits sur les nouveaux écrans ; nouveaux tests de la page détail
  (dont l'accès direct par URL et le refus du bloc abonnement hors scope `music`).
