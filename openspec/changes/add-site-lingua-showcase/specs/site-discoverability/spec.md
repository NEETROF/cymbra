## ADDED Requirements

### Requirement: Every indexable page names its canonical address
Every page of the site that is not marked `noindex` SHALL carry a canonical link and an `og:url` naming the address the host serves for it: `https://cymbra.app` followed by its path with a trailing slash, the root being `https://cymbra.app/`. Every `hreflang` link SHALL name its translation in the same form. A page marked `noindex` SHALL carry no canonical link.

#### Scenario: The Lingua page
- **WHEN** `/lingua/` is built
- **THEN** its canonical link and `og:url` are `https://cymbra.app/lingua/`, and its `hreflang` links name `https://cymbra.app/lingua/`, `https://cymbra.app/en/lingua/` and `https://cymbra.app/es/lingua/`

#### Scenario: A legal page
- **WHEN** `/en/privacy/` is built
- **THEN** its canonical link is `https://cymbra.app/en/privacy/` and its French translation is named `https://cymbra.app/confidentialite/`

#### Scenario: A personal page
- **WHEN** `/account/` is built
- **THEN** it is marked `noindex` and carries no canonical link

### Requirement: Every page has a share preview
Every page of the site SHALL carry an `og:image` of 1200×630 pixels in JPEG or PNG, with its width, its height and an `og:image:alt` in the page's language, an `og:site_name`, and a `twitter:card` of type `summary_large_image`. The Lingua pages SHALL use a capture of the extension in their own language, the Music pages a capture of Cymbra Music in their own language, and every other page one image of the brand that holds no words but the name Cymbra. The page SHALL NOT need any script for its preview.

#### Scenario: Sharing the English Lingua page
- **WHEN** `https://cymbra.app/en/lingua/` is pasted in a messaging app that shows previews
- **THEN** the preview shows the English interface of the extension, the page's title and description

#### Scenario: Sharing the Spanish privacy policy
- **WHEN** `/es/privacidad/` is built
- **THEN** its `og:image` is the brand image and its `og:image:alt` is in Spanish

### Requirement: The site publishes a sitemap and a robots file
The site SHALL serve `/sitemap.xml`, listing every built page that is not marked `noindex` at its canonical address, each with one alternate per translation the page names, and SHALL serve `/robots.txt`, allowing every crawler and naming the sitemap. The sitemap SHALL be generated from the built pages, so that a page enters it by existing and leaves it by being marked `noindex`, and the build SHALL fail if a page that is not marked `noindex` has no canonical link. The robots file SHALL NOT disallow the pages marked `noindex`.

#### Scenario: A new page
- **WHEN** a page is added to the site without `noindex`
- **THEN** the next build lists it in `/sitemap.xml` with its translations, with no list to edit

#### Scenario: The personal pages
- **WHEN** the site is built
- **THEN** `/sitemap.xml` lists neither `/account/`, `/redeem/`, `/checkout/` nor any not-found page, and `/robots.txt` does not disallow them

#### Scenario: The robots file
- **WHEN** a crawler requests `/robots.txt`
- **THEN** it reads that every agent is allowed and that the sitemap is `https://cymbra.app/sitemap.xml`
