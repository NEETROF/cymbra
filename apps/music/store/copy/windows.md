# Fiche Microsoft Store — Windows (fr · en · es · it)

What differs from the App Store / Play listings, field by field. Everything else is
reused as is from `fr.md`, `en.md`, `es.md`, `it.md` (change: add-desktop-auto-update,
design D4).

## Status (2026-10-26)

First submission made **without** the subscription paragraph: web billing is not live
in production (`billing.web.enabled` off, no `CYMBRA_PADDLE_*`), so the Windows app
offers no purchase and the certification notes say so. Add the paragraph below to the
four listings (and update the notes) in the same submission that turns web billing on.

## Description

The **"Description complète"** of each language file, with its last section
(ABONNEMENT / SUBSCRIPTION / SUSCRIPCIÓN / ABBONAMENTO — Apple billing, Apple EULA)
**replaced** by the block below. Store policy 10.8.4 requires the listing to state the
kind of in-app purchase and its price range; `<MONTHLY>` / `<YEARLY>` are the Paddle
prices.

### fr

ABONNEMENT
L'app est gratuite. Cymbra Premium, optionnel, est un abonnement mensuel (<MONTHLY>)
ou annuel (<YEARLY>), souscrit sur cymbra.app via notre prestataire de paiement
Paddle, après connexion à votre compte. Il se renouvelle automatiquement ; gérez-le ou
résiliez-le à tout moment depuis https://cymbra.app/account.
Politique de confidentialité : https://cymbra.app/confidentialite/

### en

SUBSCRIPTION
The app is free. Cymbra Premium, optional, is a monthly (<MONTHLY>) or yearly
(<YEARLY>) subscription, purchased on cymbra.app through our payment provider Paddle,
after signing in to your account. It renews automatically; manage or cancel it at any
time from https://cymbra.app/en/account.
Privacy policy: https://cymbra.app/en/privacy/

### es

SUSCRIPCIÓN
La app es gratuita. Cymbra Premium, opcional, es una suscripción mensual (<MONTHLY>)
o anual (<YEARLY>), contratada en cymbra.app a través de nuestro proveedor de pago
Paddle, tras iniciar sesión en tu cuenta. Se renueva automáticamente; gestiónala o
cancélala en cualquier momento desde https://cymbra.app/en/account.
Política de privacidad: https://cymbra.app/es/privacidad/

### it

ABBONAMENTO
L'app è gratuita. Cymbra Premium, facoltativo, è un abbonamento mensile (<MONTHLY>)
o annuale (<YEARLY>), sottoscritto su cymbra.app tramite il nostro fornitore di
pagamento Paddle, dopo l'accesso al tuo account. Si rinnova automaticamente; gestiscilo
o annullalo in qualsiasi momento da https://cymbra.app/en/account.
Privacy policy: https://cymbra.app/en/privacy/

## Other fields

| Partner Center field | Value |
|---|---|
| Product features (≤ 20 × 200 chars) | the bullet lines (•) of the description, one per entry |
| Search terms (≤ 7) | the first 7 entries of the language's *Mots-clés / Keywords* line |
| Short description | the language's *Description courte* (Play) |
| Copyright and trademark info | `© 2026 NEETROF` |
| Developed by | `NEETROF` |
| Screenshots (desktop, ≥ 1366×768) | `store/macos/desktop_1440x900/<lang>/01…08_*.png` — the Flutter UI, no window chrome |
| Website / support / privacy | see *Listing URLs* in `fr.md` — the support URL must stay the support page |
| Category | Music (or Education, as on the App Store) |
