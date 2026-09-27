# `#announcements` — introducing each app / présenter chaque app

Post these **once**, when the server opens, as **separate messages** in `#announcements`, in this
order: (1) Cymbra Music EN, (2) Cymbra Music FR, (3) Cymbra Lingua EN, (4) Cymbra Lingua FR. Then
press **Publish** on each so servers following the channel receive them.

**Before posting** — every block ends with "new versions are announced right here", which is true
only once:

1. the pull request adding the release announcements is merged (the announce steps of
   `music-release`, `lingua-extension-release` and `lingua-apple-release`, and the manual
   `release-announce` workflow);
2. the repository secret `DISCORD_WEBHOOK_ANNOUNCEMENTS` holds the `#announcements` webhook —
   without it every announcement is skipped with a warning, silently for readers;
3. a `release-announce` preview (`publish` unticked) of the latest Music tag renders correctly.

If any of these is not done, drop that last line from each block rather than promise it.

Each block stays under Discord's 2000-character cap. What the copy deliberately says, and why:

- **Drums** are behind the `drums.enabled` flag during their beta, so the electronic kit appears
  only in the Drums bullet, marked beta. The microphone input and the leaderboards are
  flag-gated too and are left out.
- **Windows and Linux** are "coming soon", as on the site (`apps/site/src/lib/stores.ts`). The
  GitHub releases carry preview builds, but the automatic announcements do not list them either
  until the site marks them live.
- **Cymbra Lingua's interface is in French**: it teaches English to French speakers, and its word
  card buttons read « Je connais », « + Deck », « Ignorer » in every language. The English block
  says so instead of inventing English labels.
- **Privacy**: the analysis runs in the browser and pages stay on the device, but a signed-in
  reader's deck cards carry the sentence they were captured from (privacy policy, Annex B). The
  copy lists what syncs rather than claim nothing ever leaves.
- **Stores, as checked on 2026-09-27**: Chrome Web Store serves **1.0.2**, Firefox Add-ons
  **1.2.1** (desktop only), the App Store listing of the Safari app is not public yet. **Check
  every link still opens before posting**, and replace the Safari line with the App Store link if
  it is live. Chrome 1.0.2 does not yet analyse content a page adds after it loads (fixed in
  1.2.0, #551) nor translate a selected phrase (#520, #522) — worth knowing in `#lingua-help`, not
  worth a disclaimer here; better, put 1.2.1 on the Chrome Web Store first.
- **The site's Lingua page** still says "coming soon / private beta" with its store buttons
  disabled. Post the Lingua blocks only once it agrees, or the two contradict each other.

---

## 1. Cymbra Music (EN)

```
# 🎹 Cymbra Music

Learn music connected to your instrument. Plug in a USB MIDI keyboard — or play on screen — and Cymbra follows the score in real time.

**What it does**
• **Two ways to read** — standard staff notation, or falling notes to learn a piece fast
• **Wait Mode** — the score waits for the right note; practise hands separately or together
• **Real-time scoring** — accuracy and timing while you play, a summary after every run
• **A growing library** — public-domain scores from beginner to advanced, high-quality piano sounds, and your own SoundFonts
• **Drums** — plug in an electronic drum kit and play real drum parts, on the staff or on an animated kit (beta — ask in #general)
• English, French, Italian and Spanish
• Data hosted in France (EU), never sold, no advertising trackers

**Get it** — free to start, Premium optional
• iPhone, iPad, Mac: <https://apps.apple.com/app/id6789557194>
• Android: <https://play.google.com/store/apps/details?id=com.cymbra.music>
• Windows and Linux: coming soon

**Here on Discord**
• **#music-help** — stuck? tell us your platform and MIDI device
• **#music-ideas** / **#music-bugs** — one post per idea or bug
• **#scores-and-soundfonts** — catalog requests and additions

New versions are announced right here.
```

---

## 2. Cymbra Music (FR)

```
# 🎹 Cymbra Music

Apprends la musique connecté à ton instrument. Branche un clavier MIDI USB — ou joue à l'écran — et Cymbra suit la partition en temps réel.

**Ce que ça fait**
• **Deux façons de lire** — la portée classique, ou les notes qui tombent pour apprendre un morceau vite
• **Mode Attente** — la partition attend la bonne note ; main gauche, main droite ou les deux
• **Score en temps réel** — justesse et rythme pendant que tu joues, un bilan après chaque passage
• **Une bibliothèque qui grandit** — des partitions du domaine public, du débutant au confirmé, des sons de piano de qualité, et tes propres SoundFonts
• **Batterie** — branche une batterie électronique et joue de vraies parties, sur la portée ou sur un kit animé (bêta — demande dans #general)
• Français, anglais, italien et espagnol
• Données hébergées en France (UE), aucune revente, aucun pisteur publicitaire

**L'installer** — gratuit pour commencer, Premium en option
• iPhone, iPad, Mac : <https://apps.apple.com/app/id6789557194>
• Android : <https://play.google.com/store/apps/details?id=com.cymbra.music>
• Windows et Linux : bientôt

**Ici, sur Discord**
• **#music-help** — bloqué ? précise ta plateforme et ton appareil MIDI
• **#music-ideas** / **#music-bugs** — un post par idée ou par bug
• **#scores-and-soundfonts** — demandes et ajouts au catalogue

Les nouvelles versions sont annoncées ici même.
```

---

## 3. Cymbra Lingua (EN)

```
# 📖 Cymbra Lingua — beta

Read the English web and grow your vocabulary as you go. Cymbra Lingua is a browser extension that highlights, right on the page, the English words you don't know yet. **It teaches English to French speakers: its interface and translations are in French.**

**What it does**
• **Highlighted in place** — without breaking the site you are reading
• **An honest percentage** — how much of this page you actually know
• **One click to understand** — dictionary form, French translation and how common the word is; then mark it known, add it to your deck, or ignore it
• **Deck and review** — captured words and phrases become cards, with the sentence you met them in
• **Your level on the CEFR scale**, from A1 to C2
• **Private by design** — the analysis runs in your browser, and the pages you read and their addresses stay on your device
• **No account needed** — sign in with your Cymbra account and only your word statuses, your cards and your daily counts sync across devices

**Get it**
• Chrome, Edge and other Chromium browsers: <https://chromewebstore.google.com/detail/cymbra-lingua/lodgdmkjlbpieomelpdkfaifdbipfncd>
• Firefox on desktop: <https://addons.mozilla.org/firefox/addon/cymbra-lingua/>
• Safari on iPhone, iPad and Mac: coming soon

**Here on Discord**
• **#lingua-help** — questions and bugs; say which browser you use

New versions are announced right here.
```

---

## 4. Cymbra Lingua (FR)

```
# 📖 Cymbra Lingua — bêta

Enrichis ton vocabulaire anglais en lisant le web. Cymbra Lingua est une extension de navigateur qui surligne, directement dans la page, les mots anglais que tu ne connais pas encore.

**Ce que ça fait**
• **Surligné sur place** — sans casser le site que tu lis
• **Un pourcentage honnête** — la part de cette page que tu connais vraiment
• **Un clic pour comprendre** — la forme du dictionnaire, la traduction et la fréquence du mot, puis « Je connais », « + Deck » ou « Ignorer »
• **Deck et révisions** — les mots et expressions capturés deviennent des cartes, avec la phrase où tu les as rencontrés
• **Ton niveau en CEFR**, de A1 à C2
• **Privé par construction** — l'analyse tourne dans ton navigateur ; les pages que tu lis et leurs adresses restent sur ton appareil
• **Aucun compte nécessaire** — connecte-toi avec ton compte Cymbra et seuls tes statuts de mots, tes cartes et tes compteurs du jour sont synchronisés entre tes appareils

**L'installer**
• Chrome, Edge et les autres navigateurs Chromium : <https://chromewebstore.google.com/detail/cymbra-lingua/lodgdmkjlbpieomelpdkfaifdbipfncd>
• Firefox sur ordinateur : <https://addons.mozilla.org/firefox/addon/cymbra-lingua/>
• Safari sur iPhone, iPad et Mac : bientôt

**Ici, sur Discord**
• **#lingua-help** — questions et bugs ; précise ton navigateur

Les nouvelles versions sont annoncées ici même.
```
