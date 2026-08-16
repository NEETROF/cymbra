//! Channel configuration from the environment and the built provider clients —
//! shared by the server (routes + gRPC) and the worker (reconciliation, erasure
//! cancellation). Each block is `None` when its variables are absent: the channel
//! is then simply not wired.

use crate::billing::apple::{AppStoreServerApi, AppleConfig, AppleVerifier};
use crate::billing::google::{
    GoogleConfig, GooglePushAuth, GoogleVerifier, PlayApi, PlayDeveloperApi, PushAuth,
};
use crate::billing::web::PaddleProvider;
use std::sync::Arc;

/// Everything the three channels need, read from the environment. Each block is
/// `None` when its variables are absent — the channel is then not mounted.
#[derive(Clone, Default)]
pub struct BillingEnv {
    pub apple: Option<AppleEnv>,
    pub google: Option<GoogleEnv>,
    pub web: Option<WebEnv>,
}

#[derive(Clone)]
pub struct AppleEnv {
    pub bundle_id: String,
    pub allow_sandbox: bool,
    /// App Store Connect API key (PEM) + ids — for the Server API (reconciliation).
    pub asc_key_pem: Option<String>,
    pub asc_key_id: Option<String>,
    pub asc_issuer_id: Option<String>,
}

#[derive(Clone)]
pub struct GoogleEnv {
    pub package_name: String,
    pub sa_email: String,
    pub sa_key_pem: String,
    /// The RTDN push endpoint URL (OIDC `aud`) and the push service account.
    pub rtdn_audience: String,
    pub rtdn_sa_email: String,
}

#[derive(Clone)]
pub struct WebEnv {
    pub api_key: String,
    pub sandbox: bool,
    pub webhook_secret: String,
    pub checkout_page: String,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

impl BillingEnv {
    /// Read the channel configuration from the process environment.
    pub fn from_env() -> Self {
        let apple = env("CYMBRA_APPLE_BUNDLE_ID").map(|bundle_id| AppleEnv {
            bundle_id,
            allow_sandbox: env("CYMBRA_APPLE_ALLOW_SANDBOX")
                .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes")),
            asc_key_pem: env("CYMBRA_APPLE_ASC_KEY_PEM").map(|p| p.replace("\\n", "\n")),
            asc_key_id: env("CYMBRA_APPLE_ASC_KEY_ID"),
            asc_issuer_id: env("CYMBRA_APPLE_ASC_ISSUER_ID"),
        });
        let google = match (
            env("CYMBRA_GOOGLE_PACKAGE_NAME"),
            env("CYMBRA_GOOGLE_SA_EMAIL"),
            env("CYMBRA_GOOGLE_SA_KEY_PEM"),
            env("CYMBRA_GOOGLE_RTDN_AUDIENCE"),
            env("CYMBRA_GOOGLE_RTDN_SA_EMAIL"),
        ) {
            (Some(package_name), Some(sa_email), Some(key), Some(rtdn_audience), Some(rtdn_sa)) => {
                Some(GoogleEnv {
                    package_name,
                    sa_email,
                    sa_key_pem: key.replace("\\n", "\n"),
                    rtdn_audience,
                    rtdn_sa_email: rtdn_sa,
                })
            }
            _ => None,
        };
        let web = match (
            env("CYMBRA_PADDLE_API_KEY"),
            env("CYMBRA_PADDLE_WEBHOOK_SECRET"),
            env("CYMBRA_PADDLE_CHECKOUT_PAGE"),
        ) {
            (Some(api_key), Some(webhook_secret), Some(checkout_page)) => Some(WebEnv {
                api_key,
                sandbox: env("CYMBRA_PADDLE_SANDBOX")
                    .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes")),
                webhook_secret,
                checkout_page,
            }),
            _ => None,
        };
        Self { apple, google, web }
    }
}

/// The wired channels: what the gRPC adapter, the routes and the worker use.
#[derive(Clone, Default)]
pub struct BillingChannels {
    pub apple: Option<Arc<AppleConfig>>,
    pub apple_verifier: Option<Arc<AppleVerifier>>,
    pub apple_api: Option<Arc<AppStoreServerApi>>,
    pub google_api: Option<Arc<dyn PlayApi>>,
    pub google_verifier: Option<Arc<GoogleVerifier>>,
    pub google_push_auth: Option<Arc<dyn PushAuth>>,
    pub google_package: Option<String>,
    pub web: Option<Arc<PaddleProvider>>,
    pub web_secret: Option<String>,
}

impl BillingChannels {
    pub fn build(env: &BillingEnv) -> Self {
        let mut c = Self::default();
        if let Some(a) = &env.apple {
            let cfg = Arc::new(AppleConfig::new(a.bundle_id.clone(), a.allow_sandbox));
            c.apple_verifier = Some(Arc::new(AppleVerifier::new((*cfg).clone())));
            if let (Some(pem), Some(kid), Some(iss)) =
                (&a.asc_key_pem, &a.asc_key_id, &a.asc_issuer_id)
            {
                c.apple_api = Some(Arc::new(AppStoreServerApi::new(
                    cfg.clone(),
                    pem.clone(),
                    kid.clone(),
                    iss.clone(),
                    a.allow_sandbox,
                )));
            }
            c.apple = Some(cfg);
        }
        if let Some(g) = &env.google {
            let api: Arc<dyn PlayApi> = Arc::new(PlayDeveloperApi::new(GoogleConfig {
                package_name: g.package_name.clone(),
                client_email: g.sa_email.clone(),
                private_key_pem: g.sa_key_pem.clone(),
            }));
            c.google_verifier = Some(Arc::new(GoogleVerifier::new(api.clone())));
            c.google_api = Some(api);
            c.google_push_auth = Some(Arc::new(GooglePushAuth::new(
                g.rtdn_audience.clone(),
                g.rtdn_sa_email.clone(),
            )));
            c.google_package = Some(g.package_name.clone());
        }
        if let Some(w) = &env.web {
            c.web = Some(Arc::new(PaddleProvider::new(
                w.api_key.clone(),
                w.sandbox,
                w.checkout_page.clone(),
            )));
            c.web_secret = Some(w.webhook_secret.clone());
        }
        c
    }
}
