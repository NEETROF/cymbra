# add-admin-lingua-language-labels — the back office names the studied languages

## Why

The back office's Lingua screen breaks its usage report down by studied language, and filters its
series by one (`admin-lingua-console`). It shows each language as its code: « en » in the
breakdown's rows and in the filter's options. With Spanish arriving (`docs/lingua/spanish-programme.md`,
change 3), a lingua admin would read « en » and « es » beside counts, in a console whose every other
word is French or English.

This is change 3 of the programme, in R1. It changes nothing a reader of Lingua sees.

## What Changes

- **A name per studied language, in both of the console's languages**: `lingua.languages` in
  `fr.json` (« Anglais », « Espagnol ») and `en.json` (« English », « Spanish »).
- **`languageLabel`**, beside `appLabel`, names a code, and leaves a code the console has no name
  for as it is: a language that starts reporting before it is named still shows.
- **The Lingua screen** names the languages in its breakdown's rows and its filter's options. The
  filter's values stay the codes, so a selection and the request are unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `admin-lingua-console`: ADDED — *The studied languages are named in the console's language*.

## Impact

- **Products.** The back office (`apps/back-office`): `src/i18n/`, `src/views/LinguaView.vue`, and
  their unit and end-to-end tests. The server, the protos and the extension are not affected; the
  usage report still carries codes.
- **Release.** A back-office deployment, the owner's.
