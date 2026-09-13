## 1. Sign-in surface

- [x] 1.1 Extract `_LegalConsent` from `entry_screen.dart` into a shared widget and show it on both `EntryScreen` and `SignInInvitationScreen`
- [x] 1.2 Add `SignInBenefit.subscribe` and its invitation copy to `app_en.arb`, `app_fr.arb`, `app_it.arb` and `app_es.arb` with the same key and placeholders; run the l10n drift check
- [x] 1.3 Verify that authenticating from a guest session clears the persisted guest choice, fix it in the session notifier if it does not, and cover it with a test
- [x] 1.4 Widget test: the consent notice is shown on `SignInInvitationScreen` and its references open the resolved URLs through the injected launcher

## 2. First run without the wall

- [x] 2.1 Welcome "Skip" and "Continue without an account": call `continueAsGuest()` before `completeWelcome()`, so the gate hands over with a guest session and no entry-screen frame
- [x] 2.2 Welcome "Sign in": open the contextual sign-in surface over the welcome, and complete the welcome only when authentication succeeds
- [x] 2.3 Update `onboarding_flow_test.dart`: skip and continue land on the library in guest mode (replacing "skipping lands on the entry screen"); signing in from the welcome ends it; leaving the sign-in surface returns to the welcome

## 3. Guest upgrade

- [x] 3.1 Guest account button (`account-signin`): open the contextual sign-in surface through `inviteSignIn` instead of calling `leaveGuest()`
- [x] 3.2 Widget tests: a guest who signs in from the library stays on the library, signed in; a guest who backs out stays a guest on the same screen
- [x] 3.3 Guest account control: keep the direct sign-in button and add an overflow menu beside it (`account-guest-menu`) reaching the paywall, language and legal pages — the paywall's guest state was otherwise unreachable (found on the 1.33.0 build); widget tests for the entries shown and hidden, the paywall route and a legal link

## 4. Paywall guest state

- [x] 4.1 `plan_screen.dart`: with no account session, render what premium includes, the account-bound explanation and a sign-in action using `SignInBenefit.subscribe`, and build neither the purchase card nor the restore action
- [x] 4.2 Confirm that returning to the paywall after sign-in re-reads the plan and shows the purchase card, with no extra invalidation (the plan provider is identity-scoped)
- [x] 4.3 Widget tests: guest paywall content; no plan-service or store call for a guest; sign-in returns to the paywall with purchase actions; declining keeps the guest state

## 5. Verification

- [x] 5.1 `auth_entry_test.dart`: the entry screen is still shown after sign-out and after account deletion when no guest choice is stored
- [ ] 5.2 Integration `app_test.dart`: a first run walks language → welcome → library as a guest
- [x] 5.3 `dart format` from the repo root, `melos run analyze`, `dart run custom_lint`, and `flutter test --coverage` at or above 80 %
- [ ] 5.4 On device, on the iPhone and Mac store builds: fresh install → language → welcome → "Continue without an account" → library; open Subscription as a guest → sign in → purchase buttons with prices
- [x] 5.5 `openspec validate open-app-without-sign-in-wall --strict`
