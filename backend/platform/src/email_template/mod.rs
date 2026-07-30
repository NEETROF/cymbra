//! Branded, localized transactional-email rendering (change: template-backend-
//! emails). One shared layer both the inline auth senders and the
//! `verification_email` job use, so a given email is byte-identical regardless of
//! producer (design D1/D2). Emails carry the shared **"Cymbra ID"** identity
//! brand — never a single product's — and ship an HTML part plus a plain-text
//! alternative (design D5/D6).
//!
//! Rendering is pure and host-testable (no I/O); the SMTP multipart glue lives in
//! [`crate::email`].

use askama::Template;

/// Umbrella identity brand shown in the header wordmark + `From` display name.
pub const BRAND: &str = "Cymbra ID";

/// A fully-rendered transactional email: the localized subject, the branded HTML
/// body, and a plain-text alternative carrying the same code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedEmail {
    pub subject: String,
    pub html: String,
    pub text: String,
}

/// Locales the transactional emails are translated into (design D7). Anything
/// unknown or absent falls back to [`SupportedLocale::En`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLocale {
    En,
    Es,
    Fr,
    It,
}

impl SupportedLocale {
    /// Map an optional/arbitrary locale tag (e.g. `"fr"`, `"fr-FR"`, `"FR_ca"`) to a
    /// supported locale, defaulting to English when absent or unrecognized.
    pub fn parse(tag: Option<&str>) -> Self {
        match tag {
            Some(t) => {
                let primary = t
                    .trim()
                    .to_ascii_lowercase()
                    .split(['-', '_'])
                    .next()
                    .unwrap_or("")
                    .to_string();
                match primary.as_str() {
                    "fr" => Self::Fr,
                    "es" => Self::Es,
                    "it" => Self::It,
                    _ => Self::En,
                }
            }
            None => Self::En,
        }
    }
}

/// Locale-aware Terms + Privacy URLs on the `cymbra.app` legal site (mirrors the
/// app-side `legal-links` spec): French uses the French pages, every other locale
/// falls back to the English pages.
fn legal_links(locale: SupportedLocale) -> (&'static str, &'static str) {
    match locale {
        SupportedLocale::Fr => (
            "https://cymbra.app/cgu/",
            "https://cymbra.app/confidentialite/",
        ),
        _ => (
            "https://cymbra.app/en/terms/",
            "https://cymbra.app/en/privacy/",
        ),
    }
}

/// The localized, per-email copy the layout renders around the code.
struct Copy {
    subject: &'static str,
    heading: &'static str,
    intro: &'static str,
    code_note: &'static str,
    outro: &'static str,
    footer_note: &'static str,
    terms_label: &'static str,
    privacy_label: &'static str,
}

/// Which transactional email to render.
enum Kind {
    Verification,
    PasswordReset,
}

fn copy_for(kind: &Kind, locale: SupportedLocale) -> Copy {
    let (terms_label, privacy_label, footer_note) = shared_copy(locale);
    match (kind, locale) {
        // --- Verification ---
        (Kind::Verification, SupportedLocale::En) => Copy {
            subject: "Verify your Cymbra account",
            heading: "Confirm your email",
            intro: "Use the code below to verify your email address and activate your Cymbra account.",
            code_note: "Your verification code",
            outro: "This code expires soon. If you didn't create a Cymbra account, you can safely ignore this email.",
            footer_note,
            terms_label,
            privacy_label,
        },
        (Kind::Verification, SupportedLocale::Fr) => Copy {
            subject: "Vérifiez votre compte Cymbra",
            heading: "Confirmez votre adresse e-mail",
            intro: "Utilisez le code ci-dessous pour vérifier votre adresse e-mail et activer votre compte Cymbra.",
            code_note: "Votre code de vérification",
            outro: "Ce code expire bientôt. Si vous n'avez pas créé de compte Cymbra, ignorez cet e-mail.",
            footer_note,
            terms_label,
            privacy_label,
        },
        (Kind::Verification, SupportedLocale::Es) => Copy {
            subject: "Verifica tu cuenta de Cymbra",
            heading: "Confirma tu correo",
            intro: "Usa el código de abajo para verificar tu dirección de correo y activar tu cuenta de Cymbra.",
            code_note: "Tu código de verificación",
            outro: "Este código caduca pronto. Si no creaste una cuenta de Cymbra, ignora este correo.",
            footer_note,
            terms_label,
            privacy_label,
        },
        (Kind::Verification, SupportedLocale::It) => Copy {
            subject: "Verifica il tuo account Cymbra",
            heading: "Conferma la tua email",
            intro: "Usa il codice qui sotto per verificare il tuo indirizzo email e attivare il tuo account Cymbra.",
            code_note: "Il tuo codice di verifica",
            outro: "Questo codice scade a breve. Se non hai creato un account Cymbra, ignora questa email.",
            footer_note,
            terms_label,
            privacy_label,
        },
        // --- Password reset ---
        (Kind::PasswordReset, SupportedLocale::En) => Copy {
            subject: "Reset your Cymbra password",
            heading: "Reset your password",
            intro: "Use the code below to reset the password for your Cymbra account.",
            code_note: "Your password reset code",
            outro: "This code expires soon. If you didn't request a password reset, you can safely ignore this email.",
            footer_note,
            terms_label,
            privacy_label,
        },
        (Kind::PasswordReset, SupportedLocale::Fr) => Copy {
            subject: "Réinitialisez votre mot de passe Cymbra",
            heading: "Réinitialisez votre mot de passe",
            intro: "Utilisez le code ci-dessous pour réinitialiser le mot de passe de votre compte Cymbra.",
            code_note: "Votre code de réinitialisation",
            outro: "Ce code expire bientôt. Si vous n'avez pas demandé de réinitialisation, ignorez cet e-mail.",
            footer_note,
            terms_label,
            privacy_label,
        },
        (Kind::PasswordReset, SupportedLocale::Es) => Copy {
            subject: "Restablece tu contraseña de Cymbra",
            heading: "Restablece tu contraseña",
            intro: "Usa el código de abajo para restablecer la contraseña de tu cuenta de Cymbra.",
            code_note: "Tu código de restablecimiento",
            outro: "Este código caduca pronto. Si no solicitaste restablecer la contraseña, ignora este correo.",
            footer_note,
            terms_label,
            privacy_label,
        },
        (Kind::PasswordReset, SupportedLocale::It) => Copy {
            subject: "Reimposta la password di Cymbra",
            heading: "Reimposta la password",
            intro: "Usa il codice qui sotto per reimpostare la password del tuo account Cymbra.",
            code_note: "Il tuo codice di reimpostazione",
            outro: "Questo codice scade a breve. Se non hai richiesto la reimpostazione, ignora questa email.",
            footer_note,
            terms_label,
            privacy_label,
        },
    }
}

/// Locale-shared footer bits: legal-link labels + the "why am I getting this" note.
fn shared_copy(locale: SupportedLocale) -> (&'static str, &'static str, &'static str) {
    match locale {
        SupportedLocale::En => (
            "Terms",
            "Privacy",
            "You received this email because this address was used for a Cymbra account. Cymbra ID · © NEETROF.",
        ),
        SupportedLocale::Fr => (
            "CGU",
            "Confidentialité",
            "Vous recevez cet e-mail car cette adresse a été utilisée pour un compte Cymbra. Cymbra ID · © NEETROF.",
        ),
        SupportedLocale::Es => (
            "Términos",
            "Privacidad",
            "Recibes este correo porque esta dirección se usó para una cuenta de Cymbra. Cymbra ID · © NEETROF.",
        ),
        SupportedLocale::It => (
            "Termini",
            "Privacy",
            "Ricevi questa email perché questo indirizzo è stato usato per un account Cymbra. Cymbra ID · © NEETROF.",
        ),
    }
}

/// Askama context for the shared email layout. Owned `String`s keep call sites
/// simple; empty `logo_url`/`cta_url` mean "omit that element".
#[derive(Template)]
#[template(path = "email/layout.html")]
struct EmailHtml {
    brand: String,
    logo_url: String,
    preheader: String,
    heading: String,
    intro: String,
    code: String,
    code_note: String,
    outro: String,
    cta_url: String,
    cta_label: String,
    terms_url: String,
    terms_label: String,
    privacy_url: String,
    privacy_label: String,
    footer_note: String,
}

/// Plain-text alternative carrying the same code + instructions as the HTML.
fn render_text(c: &Copy, code: &str, terms_url: &str, privacy_url: &str) -> String {
    format!(
        "{heading}\n\n{intro}\n\n{code_note}: {code}\n\n{outro}\n\n—\n{brand}\n{terms_label}: {terms_url}\n{privacy_label}: {privacy_url}\n{footer_note}\n",
        heading = c.heading,
        intro = c.intro,
        code_note = c.code_note,
        code = code,
        outro = c.outro,
        brand = BRAND,
        terms_label = c.terms_label,
        terms_url = terms_url,
        privacy_label = c.privacy_label,
        privacy_url = privacy_url,
        footer_note = c.footer_note,
    )
}

fn render(
    kind: Kind,
    code: &str,
    locale: SupportedLocale,
    logo_url: Option<&str>,
) -> RenderedEmail {
    let c = copy_for(&kind, locale);
    let (terms_url, privacy_url) = legal_links(locale);
    let html = EmailHtml {
        brand: BRAND.to_string(),
        logo_url: logo_url.unwrap_or("").to_string(),
        preheader: c.intro.to_string(),
        heading: c.heading.to_string(),
        intro: c.intro.to_string(),
        code: code.to_string(),
        code_note: c.code_note.to_string(),
        outro: c.outro.to_string(),
        cta_url: String::new(),
        cta_label: String::new(),
        terms_url: terms_url.to_string(),
        terms_label: c.terms_label.to_string(),
        privacy_url: privacy_url.to_string(),
        privacy_label: c.privacy_label.to_string(),
        footer_note: c.footer_note.to_string(),
    }
    // Rendering a compile-time-checked template over owned strings is infallible in
    // practice; surface any formatter error as an empty body rather than panicking
    // in the mail path.
    .render()
    .unwrap_or_default();
    let text = render_text(&c, code, terms_url, privacy_url);
    RenderedEmail {
        subject: c.subject.to_string(),
        html,
        text,
    }
}

/// Render the account-verification email for `code` in `locale`. `logo_url` is the
/// hosted neutral "Cymbra ID" logo (config-driven); when `None` the header shows
/// the text wordmark alone.
pub fn verification_email(
    code: &str,
    locale: SupportedLocale,
    logo_url: Option<&str>,
) -> RenderedEmail {
    render(Kind::Verification, code, locale, logo_url)
}

/// Render the password-reset email for `code` in `locale`. See
/// [`verification_email`] for `logo_url`.
pub fn password_reset_email(
    code: &str,
    locale: SupportedLocale,
    logo_url: Option<&str>,
) -> RenderedEmail {
    render(Kind::PasswordReset, code, locale, logo_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_parse_maps_and_falls_back_to_english() {
        assert_eq!(SupportedLocale::parse(Some("fr")), SupportedLocale::Fr);
        assert_eq!(SupportedLocale::parse(Some("fr-FR")), SupportedLocale::Fr);
        assert_eq!(SupportedLocale::parse(Some("ES_es")), SupportedLocale::Es);
        assert_eq!(SupportedLocale::parse(Some("it")), SupportedLocale::It);
        assert_eq!(SupportedLocale::parse(Some("en-US")), SupportedLocale::En);
        assert_eq!(SupportedLocale::parse(Some("de")), SupportedLocale::En);
        assert_eq!(SupportedLocale::parse(Some("")), SupportedLocale::En);
        assert_eq!(SupportedLocale::parse(None), SupportedLocale::En);
    }

    #[test]
    fn verification_carries_code_and_brand_in_both_parts() {
        let e = verification_email("ABC-123", SupportedLocale::En, None);
        assert_eq!(e.subject, "Verify your Cymbra account");
        // Code present in HTML and text.
        assert!(e.html.contains("ABC-123"), "code missing from html");
        assert!(e.text.contains("ABC-123"), "code missing from text");
        // Brand signature: "Cymbra ID" wordmark + accent color + NEETROF footer.
        assert!(e.html.contains("Cymbra ID"));
        assert!(e.html.contains("#7C3AED"));
        assert!(e.html.to_lowercase().contains("neetrof"));
        assert!(e.text.contains("Cymbra ID"));
    }

    #[test]
    fn password_reset_uses_same_layout_signature() {
        let e = password_reset_email("XYZ-999", SupportedLocale::En, None);
        assert_eq!(e.subject, "Reset your Cymbra password");
        assert!(e.html.contains("XYZ-999"));
        assert!(e.text.contains("XYZ-999"));
        assert!(e.html.contains("Cymbra ID"));
        assert!(e.html.contains("#7C3AED"));
    }

    #[test]
    fn does_not_reference_any_product_asset() {
        let e = verification_email("CODE", SupportedLocale::En, None);
        let all = format!("{}{}", e.html, e.text).to_lowercase();
        assert!(!all.contains("music"), "must not mention a product brand");
        assert!(!all.contains("splash_logo"));
    }

    #[test]
    fn french_localizes_subject_and_legal_links() {
        let e = verification_email("code", SupportedLocale::Fr, None);
        assert_eq!(e.subject, "Vérifiez votre compte Cymbra");
        assert!(e.html.contains("https://cymbra.app/cgu/"));
        assert!(e.html.contains("https://cymbra.app/confidentialite/"));
        assert!(e.text.contains("https://cymbra.app/cgu/"));
    }

    #[test]
    fn non_french_uses_english_legal_links() {
        for loc in [
            SupportedLocale::En,
            SupportedLocale::Es,
            SupportedLocale::It,
        ] {
            let e = password_reset_email("code", loc, None);
            assert!(e.html.contains("https://cymbra.app/en/terms/"));
            assert!(e.html.contains("https://cymbra.app/en/privacy/"));
        }
    }

    #[test]
    fn each_locale_renders_its_language() {
        assert_eq!(
            verification_email("c", SupportedLocale::Es, None).subject,
            "Verifica tu cuenta de Cymbra"
        );
        assert_eq!(
            verification_email("c", SupportedLocale::It, None).subject,
            "Verifica il tuo account Cymbra"
        );
        assert_eq!(
            password_reset_email("c", SupportedLocale::Fr, None).subject,
            "Réinitialisez votre mot de passe Cymbra"
        );
    }

    /// Dev aid: `cargo test -p cymbra-platform emit_samples -- --ignored` writes the
    /// rendered HTML for both emails × all locales to `$EMAIL_SAMPLE_DIR` for a
    /// visual review (design D4/D5). Ignored by default.
    #[test]
    #[ignore]
    fn emit_samples() {
        let dir = std::env::var("EMAIL_SAMPLE_DIR").expect("set EMAIL_SAMPLE_DIR");
        let logo = Some("https://cymbra.app/brand/cymbra-id.png");
        for loc in [
            SupportedLocale::En,
            SupportedLocale::Fr,
            SupportedLocale::Es,
            SupportedLocale::It,
        ] {
            let v = verification_email("CYMBRA-4821", loc, logo);
            let r = password_reset_email("CYMBRA-7702", loc, None);
            let tag = format!("{loc:?}").to_lowercase();
            std::fs::write(format!("{dir}/verification_{tag}.html"), v.html).unwrap();
            std::fs::write(format!("{dir}/reset_{tag}.html"), r.html).unwrap();
        }
    }

    #[test]
    fn render_is_deterministic_for_same_code_and_locale() {
        // The job producer and the inline sender both call this pure function, so
        // identical (code, locale) inputs yield byte-identical emails (design D1/D2).
        let a = verification_email("SAME-CODE", SupportedLocale::Fr, None);
        let b = verification_email("SAME-CODE", SupportedLocale::Fr, None);
        assert_eq!(a, b);
    }

    #[test]
    fn logo_url_renders_img_only_when_present() {
        let without = verification_email("c", SupportedLocale::En, None);
        assert!(!without.html.contains("<img"));
        let with = verification_email(
            "c",
            SupportedLocale::En,
            Some("https://cymbra.app/id-logo.png"),
        );
        assert!(with.html.contains("<img"));
        assert!(with.html.contains("https://cymbra.app/id-logo.png"));
        assert!(with.html.contains("alt=\"Cymbra ID\""));
    }
}
