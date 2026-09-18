## ADDED Requirements

### Requirement: Client-facing routes are permanent

The site SHALL keep serving, at their exact paths, the routes that shipped clients and store listings point at. A route in this set MAY gain or change content; it SHALL NOT be moved, renamed or deleted.

The obligation comes from the clients, not from the site: a build already installed on a device requests the path it was compiled with for as long as it exists, and a store listing field is read by reviewers and users without the site being consulted. Losing a path means a broken legal link, a broken subscription-management action or a failed checkout return, on a version nobody can update retroactively.

The failure is worse than a 404. The host currently answers **any** unmatched path with the French home page and status `200` — verified against production: `/route-qui-nexiste-pas/` returns `200` and the home page's title. A deleted `/en/privacy/` would therefore not fail; it would quietly serve French marketing copy to someone who asked for the privacy policy in English, with a status code that no uptime check would flag.

The set, with the consumer that pins each one:

| Route | Pinned by |
|---|---|
| `/cgu/`, `/confidentialite/` | Music, `services/legal_links.dart` — French locale |
| `/en/terms/`, `/en/privacy/` | Music, `services/legal_links.dart` — every other locale |
| `/account` | Music, `screens/plan_screen.dart` — managing a web subscription |
| `/checkout`, `/checkout/done` | Backend `CYMBRA_PADDLE_CHECKOUT_PAGE` and the Paddle return URL |
| `/redeem` | Access-code links handed out to users |
| `/support/`, `/en/support/` | App Store Connect support URL |
| `/en/delete-account/` | Play Console data-deletion URL and the Apple account-deletion requirement |
| `/`, `/en/` | The locale roots; entry points for every other surface |

#### Scenario: A route in the set is removed

- **WHEN** a change deletes or renames one of the listed routes without shipping a redirect for the old path
- **THEN** the change SHALL be rejected, because clients already installed cannot be updated to follow it

#### Scenario: A removed route is not detected by its status code

- **WHEN** a pinned route is deleted and the host answers the old path with the fallback page
- **THEN** the path SHALL be treated as broken even though it returns `200`, and detection SHALL compare the served page against the expected one rather than its status code

#### Scenario: Content is rewritten in place

- **WHEN** a change replaces what `/` serves, as the two-product hub did to the former Music landing page
- **THEN** the change is allowed, because the path still resolves and no client is asking for particular content at it

#### Scenario: A new page is added next to a pinned route

- **WHEN** a change adds `/music` and `/lingua` alongside the existing routes
- **THEN** no obligation is created for the new routes until a shipped client or a store listing field points at one

### Requirement: An unmatched path SHALL NOT serve the home page

The site SHALL answer a path it does not serve with a dedicated not-found page and a `404` status, in the locale of the requested path where one is discernible. It SHALL NOT answer with the home page, and SHALL NOT answer with `200`.

Today it does both. That turns every routing mistake into a silent one: the visitor sees plausible marketing content instead of an error, the operator sees a healthy status code, and a broken pinned route — the one failure this capability exists to prevent — becomes invisible to any check that looks at status codes.

#### Scenario: A path the site does not serve

- **WHEN** a request arrives for a path with no page behind it
- **THEN** the site SHALL return `404` with the not-found page

#### Scenario: An English path is missing

- **WHEN** the unmatched path begins with `/en/`
- **THEN** the not-found page SHALL be served in English

### Requirement: Moving a pinned route ships its redirect in the same change

A move of a pinned route SHALL ship a permanent redirect from the old path in the same change that performs the move, so that no revision of the site exists in which the old path fails. The redirect SHALL be served by the host, not by client-side script, so that it applies to a request made by a native app opening an external browser.

Cloudflare Pages reads redirects from `public/_redirects`. The site has no such file today; the first move that needs one creates it.

#### Scenario: A pinned route is relocated

- **WHEN** a change moves `/support/` to another path
- **THEN** the same change SHALL add a `301` from `/support/` to the new path in `public/_redirects`

#### Scenario: A redirect is proposed as a follow-up

- **WHEN** a change moves a pinned route and defers its redirect to a later change
- **THEN** the change SHALL be rejected, because the deployed gap would break installed clients for the whole interval

### Requirement: The route contract is recorded in the repository

The repository SHALL carry the list of client-facing routes and the consumer that pins each one, so that a person restructuring the site can see the obligation without reading the Flutter sources or opening a store console.

#### Scenario: Restructuring the site

- **WHEN** an author plans a change to the site's page structure
- **THEN** the repository SHALL let them enumerate the pinned routes and their consumers from checked-in documentation

#### Scenario: A consumer starts pinning a new route

- **WHEN** a change makes a shipped client or a store listing field point at a route not yet in the set
- **THEN** that change SHALL add the route and its consumer to the recorded list
