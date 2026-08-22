//! Channel configuration from the environment and the built provider clients —
//! shared by the server (routes + gRPC) and the worker (reconciliation, erasure).
//! Each block is `None` when its variables are absent: the channel is then
//! simply not wired.

use crate::billing::rc_client::RcClient;
use crate::billing::revenuecat::RcConfig;
use crate::billing::web::PaddleProvider;
use crate::ports::{StoreCustomerEraser, StoreCustomerSource};
use std::sync::Arc;

/// Everything the channels need, read from the environment. Each block is
/// `None` when its variables are absent — the channel is then not mounted.
#[derive(Clone, Default)]
pub struct BillingEnv {
    /// The store aggregator (App Store, Play, later Paddle).
    pub revenuecat: Option<RevenueCatEnv>,
    /// The web merchant-of-record, direct.
    pub web: Option<WebEnv>,
}

/// `CYMBRA_REVENUECAT_*` (change: swap-store-billing-to-revenuecat).
#[derive(Clone)]
pub struct RevenueCatEnv {
    /// Secret (v1) API key — customer reads + erasure.
    pub api_key: String,
    /// The value RevenueCat sends in the webhook `Authorization` header.
    pub webhook_secret: String,
    /// Project id (dashboard deep links only).
    pub project_id: Option<String>,
    /// Apply `SANDBOX` events/subscriptions — staging only, never production.
    pub allow_sandbox: bool,
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

fn flag(name: &str) -> bool {
    env(name).is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes"))
}

impl BillingEnv {
    /// Read the channel configuration from the process environment.
    pub fn from_env() -> Self {
        let revenuecat = match (
            env("CYMBRA_REVENUECAT_API_KEY"),
            env("CYMBRA_REVENUECAT_WEBHOOK_SECRET"),
        ) {
            (Some(api_key), Some(webhook_secret)) => Some(RevenueCatEnv {
                api_key,
                webhook_secret,
                project_id: env("CYMBRA_REVENUECAT_PROJECT_ID"),
                allow_sandbox: flag("CYMBRA_REVENUECAT_ALLOW_SANDBOX"),
            }),
            _ => None,
        };
        let web = match (
            env("CYMBRA_PADDLE_API_KEY"),
            env("CYMBRA_PADDLE_WEBHOOK_SECRET"),
            env("CYMBRA_PADDLE_CHECKOUT_PAGE"),
        ) {
            (Some(api_key), Some(webhook_secret), Some(checkout_page)) => Some(WebEnv {
                api_key,
                sandbox: flag("CYMBRA_PADDLE_SANDBOX"),
                webhook_secret,
                checkout_page,
            }),
            _ => None,
        };
        Self { revenuecat, web }
    }
}

/// The wired channels: what the gRPC adapter, the routes and the worker use.
#[derive(Clone, Default)]
pub struct BillingChannels {
    /// Aggregator config (webhook auth, sandbox rule) — `Some` mounts the route.
    pub revenuecat: Option<Arc<RcConfig>>,
    /// Aggregator customer reads (`SyncStorePlan`, reconciliation, TRANSFER).
    pub rc_customers: Option<Arc<dyn StoreCustomerSource>>,
    /// Aggregator customer deletion (account erasure).
    pub rc_eraser: Option<Arc<dyn StoreCustomerEraser>>,
    pub web: Option<Arc<PaddleProvider>>,
    pub web_secret: Option<String>,
}

impl BillingChannels {
    pub fn build(env: &BillingEnv) -> Self {
        let mut c = Self::default();
        if let Some(rc) = &env.revenuecat {
            c.revenuecat = Some(Arc::new(RcConfig {
                webhook_secret: rc.webhook_secret.clone(),
                api_key: rc.api_key.clone(),
                project_id: rc.project_id.clone(),
                allow_sandbox: rc.allow_sandbox,
            }));
            let client = Arc::new(RcClient::new(rc.api_key.clone()));
            c.rc_customers = Some(client.clone());
            c.rc_eraser = Some(client);
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
