//! Discord announcements in the worker (change: add-discord-notifications) —
//! composition glue, excluded from the coverage gate: the decisions live in
//! `cymbra_discord::Publisher`, unit-tested there. This file only binds its ports
//! to the worker's pools and flag service.

use cymbra_discord::pg::{PgAnnouncementLedger, PgAnnouncementSource};
use cymbra_discord::webhook::WebhookDiscordSender;
use cymbra_discord::{AnnouncementEvent, FlagView, Locale, Outcome, Publisher, Retry, Routing};
use cymbra_feature_flags::{EvalContext, FlagService};
use sqlx::PgPool;

/// Everything `discord_notify` needs. Built only when at least one webhook is
/// configured; otherwise the job is a logged no-op.
pub struct Announcer {
    routing: Routing,
    locale: Locale,
    source: PgAnnouncementSource,
    ledger: PgAnnouncementLedger,
    sender: WebhookDiscordSender,
}

impl Announcer {
    /// `queue_pool` is `worker_svc` (owner of the `jobs.discord_announcements`
    /// ledger); `admin_pool` is `admin_svc`, the worker's cross-schema reader of
    /// the accepted catalog items.
    pub fn new(
        sender: WebhookDiscordSender,
        locale: Locale,
        queue_pool: PgPool,
        admin_pool: PgPool,
    ) -> Self {
        Self {
            routing: Routing::default(),
            locale,
            source: PgAnnouncementSource::new(admin_pool),
            ledger: PgAnnouncementLedger::new(queue_pool),
            sender,
        }
    }

    pub async fn publish(
        &self,
        flags: &FlagService,
        event: &AnnouncementEvent,
    ) -> Result<Outcome, Retry> {
        Publisher {
            routing: &self.routing,
            locale: self.locale,
            flags: &Flags(flags),
            source: &self.source,
            ledger: &self.ledger,
            sender: &self.sender,
        }
        .publish(event)
        .await
    }
}

/// The flag service seen through the publisher's port. Server-side keys are
/// declared under `APP_ALL`, evaluated without a user; unknown ⇒ off.
struct Flags<'a>(&'a FlagService);

impl FlagView for Flags<'_> {
    fn enabled(&self, key: &str) -> bool {
        self.0.bool(key, false, &EvalContext::anonymous(""))
    }
}

#[cfg(test)]
mod tests {
    use cymbra_discord::Category;
    use cymbra_discord::flags::{DISCORD_ENABLED, category_flag};
    use cymbra_feature_flags::{FlagValue, Registry};

    #[test]
    fn every_key_the_publisher_reads_is_registered_and_off() {
        let registry = Registry::default();
        let keys = std::iter::once(DISCORD_ENABLED.to_string())
            .chain(Category::ALL.into_iter().map(category_flag));
        for key in keys {
            let def = registry
                .get_by_key(&key)
                .unwrap_or_else(|| panic!("{key} is not registered"));
            assert_eq!(def.default, FlagValue::Bool(false), "{key}");
        }
    }
}
