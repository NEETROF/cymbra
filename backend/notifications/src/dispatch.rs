//! Dispatch — selection → send → prune (change: add-push-notifications, tasks
//! 2.4/3.2, design D5/D6).
//!
//! The one place a category send is executed. Both of its dependencies are traits
//! ([`PushRegistry`], [`PushSender`]), so the whole loop — including the
//! invalid-token pruning that protects the registry — is unit-tested against
//! mockall doubles with no DB and no FCM.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use cymbra_feature_flags::{EvalContext, FlagService, registry as flag_keys};

use crate::repo::{Audience, PushRegistry};
use crate::select_core::{DEFAULT_TIMEZONE, SelectionFlags, select_recipients};
use crate::sender::{PushMessage, PushSender, SendOutcome};

/// What one dispatch did — the numbers the job logs and the tests assert on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DispatchReport {
    /// Tokens that passed every gate.
    pub selected: usize,
    /// Sends the provider accepted.
    pub delivered: usize,
    /// Sends that failed transiently (the job's retry covers them).
    pub retryable: usize,
    /// Tokens removed because the provider reported them dead.
    pub pruned: usize,
}

/// Resolve a category's [`SelectionFlags`] from the hot-reloadable flag store
/// (design D4).
///
/// Every gate fails **safe**: an unset kill-switch, an undeclared category, or a
/// flag store that cannot be read all resolve to "disabled", so nothing is sent.
/// `default_pref` is the product default the owning feature declares for users
/// who never expressed a choice.
pub fn resolve_flags(flags: &FlagService, category: &str, default_pref: bool) -> SelectionFlags {
    let ctx = EvalContext::anonymous("");
    // -1 = "no schedule hour configured" → an event-triggered send with no hour
    // gate. A configured value outside 0..=23 is treated the same way rather than
    // silently firing at a wrong hour.
    let hour = flags.int(&flag_keys::category_hour_key(category), -1, &ctx);
    SelectionFlags {
        category: category.to_string(),
        kill_switch_on: flags.bool(flag_keys::NOTIFICATIONS_ENABLED, false, &ctx),
        category_enabled: flags.bool(&flag_keys::category_enabled_key(category), false, &ctx),
        schedule_hour: u32::try_from(hour).ok().filter(|h| *h <= 23),
        default_pref,
        default_timezone: DEFAULT_TIMEZONE.to_string(),
        foreground: flags.bool(&flag_keys::category_foreground_key(category), false, &ctx),
    }
}

/// The `data` key the dispatcher stamps with the category's resolved foreground
/// flag (change: add-foreground-notifications, design D4): `"true"` = the client
/// surfaces the message in-app when it arrives with the app open, anything else
/// (including absence, e.g. a hand-crafted payload) = silence. The Dart client
/// reads the same key — keep the two in sync.
pub const FOREGROUND_DATA_KEY: &str = "foreground";

/// [`PushMessage`] with the resolved foreground decision stamped on its `data`,
/// replacing any caller-supplied value: the flag store is the single source of
/// that decision, a feature cannot opt itself in from a job payload.
fn stamp_foreground(msg: &PushMessage, foreground: bool) -> PushMessage {
    let mut stamped = msg.clone();
    stamped.data.retain(|(k, _)| k != FOREGROUND_DATA_KEY);
    stamped
        .data
        .push((FOREGROUND_DATA_KEY.to_string(), foreground.to_string()));
    stamped
}

/// Runs one category send end to end.
pub struct Dispatcher {
    registry: Arc<dyn PushRegistry>,
    sender: Arc<dyn PushSender>,
}

impl Dispatcher {
    pub fn new(registry: Arc<dyn PushRegistry>, sender: Arc<dyn PushSender>) -> Self {
        Self { registry, sender }
    }

    /// Load the audience's candidates, select the eligible tokens, send, and
    /// prune whatever the provider reports as dead.
    ///
    /// Returns `Err` only when the registry read or the sender itself fails —
    /// a per-token failure is a [`SendOutcome`], not an error, so one bad device
    /// never aborts the batch. Re-running a dispatch is safe (at-least-once
    /// worker delivery): the worst case is a duplicate notification, and pruning
    /// is idempotent.
    pub async fn dispatch(
        &self,
        flags: &SelectionFlags,
        audience: &Audience,
        msg: &PushMessage,
        now: DateTime<Utc>,
    ) -> anyhow::Result<DispatchReport> {
        // Skip the candidate load entirely when a global gate is already closed —
        // a disabled category must not even read the registry.
        if !flags.kill_switch_on || !flags.category_enabled {
            tracing::debug!(
                category = %flags.category,
                kill_switch_on = flags.kill_switch_on,
                category_enabled = flags.category_enabled,
                "push dispatch suppressed by flags"
            );
            return Ok(DispatchReport::default());
        }

        let candidates = self.registry.candidates(&flags.category, audience).await?;
        let tokens = select_recipients(&candidates, flags, now);

        // Every message carries the category's foreground decision — stamped
        // here, centrally, so a feature cannot forget and scheduled and
        // event-triggered sends behave alike (design D4).
        let msg = stamp_foreground(msg, flags.foreground);

        let mut report = DispatchReport {
            selected: tokens.len(),
            ..DispatchReport::default()
        };
        for token in &tokens {
            match self.sender.send(token, &msg).await? {
                SendOutcome::Delivered => report.delivered += 1,
                SendOutcome::Retryable(reason) => {
                    report.retryable += 1;
                    tracing::warn!(category = %flags.category, reason, "push send failed (retryable)");
                }
                SendOutcome::Invalid(reason) => {
                    // The device is gone for good: drop the token so it is never
                    // selected again (design D2).
                    self.registry.prune_token(token).await?;
                    report.pruned += 1;
                    tracing::info!(category = %flags.category, reason, "pruned invalid push token");
                }
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::{Candidate, MockPushRegistry};
    use crate::sender::MockPushSender;

    fn now() -> DateTime<Utc> {
        // 20:00 in Paris (CEST).
        DateTime::parse_from_rfc3339("2026-08-06T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn live_flags() -> SelectionFlags {
        SelectionFlags {
            kill_switch_on: true,
            category_enabled: true,
            schedule_hour: Some(20),
            ..SelectionFlags::new("practice_streak")
        }
    }

    fn candidate(user: &str, token: &str) -> Candidate {
        Candidate {
            user_id: user.into(),
            token: token.into(),
            platform: "ios".into(),
            timezone: Some("Europe/Paris".into()),
            pref: None,
        }
    }

    #[tokio::test]
    async fn sends_to_every_selected_token() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_candidates()
            .returning(|_, _| Ok(vec![candidate("u1", "t1"), candidate("u2", "t2")]));
        let mut sender = MockPushSender::new();
        sender
            .expect_send()
            .times(2)
            .returning(|_, _| Ok(SendOutcome::Delivered));

        let d = Dispatcher::new(Arc::new(registry), Arc::new(sender));
        let report = d
            .dispatch(
                &live_flags(),
                &Audience::All,
                &PushMessage::new("t", "b"),
                now(),
            )
            .await
            .unwrap();
        assert_eq!(report.selected, 2);
        assert_eq!(report.delivered, 2);
        assert_eq!(report.pruned, 0);
    }

    /// Task 2.3 (add-foreground-notifications): every sent message carries the
    /// resolved foreground decision, correct for on and off, and the flag never
    /// changes who is selected.
    #[tokio::test]
    async fn the_message_carries_the_foreground_decision_without_affecting_selection() {
        for foreground in [true, false] {
            let mut registry = MockPushRegistry::new();
            registry
                .expect_candidates()
                .returning(|_, _| Ok(vec![candidate("u1", "t1"), candidate("u2", "t2")]));
            let mut sender = MockPushSender::new();
            let expected = foreground.to_string();
            sender
                .expect_send()
                .withf(move |_, msg| {
                    msg.data
                        == vec![
                            ("route".to_string(), "/practice".to_string()),
                            (FOREGROUND_DATA_KEY.to_string(), expected.clone()),
                        ]
                })
                .times(2)
                .returning(|_, _| Ok(SendOutcome::Delivered));

            let d = Dispatcher::new(Arc::new(registry), Arc::new(sender));
            let flags = SelectionFlags {
                foreground,
                ..live_flags()
            };
            let msg = PushMessage::new("t", "b").with_data("route", "/practice");
            let report = d
                .dispatch(&flags, &Audience::All, &msg, now())
                .await
                .unwrap();
            assert_eq!(report.selected, 2, "selection must ignore foreground");
            assert_eq!(report.delivered, 2);
        }
    }

    /// The flag store is the single source of the foreground decision: a value
    /// smuggled in through the job payload's `data` is replaced, not kept.
    #[tokio::test]
    async fn a_caller_supplied_foreground_entry_is_overridden() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_candidates()
            .returning(|_, _| Ok(vec![candidate("u1", "t1")]));
        let mut sender = MockPushSender::new();
        sender
            .expect_send()
            .withf(|_, msg| {
                msg.data == vec![(FOREGROUND_DATA_KEY.to_string(), "false".to_string())]
            })
            .times(1)
            .returning(|_, _| Ok(SendOutcome::Delivered));

        let d = Dispatcher::new(Arc::new(registry), Arc::new(sender));
        let flags = SelectionFlags {
            foreground: false,
            ..live_flags()
        };
        let msg = PushMessage::new("t", "b").with_data(FOREGROUND_DATA_KEY, "true");
        d.dispatch(&flags, &Audience::All, &msg, now())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn an_invalid_outcome_prunes_exactly_that_token() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_candidates()
            .returning(|_, _| Ok(vec![candidate("u1", "dead"), candidate("u2", "live")]));
        registry
            .expect_prune_token()
            .withf(|t| t == "dead")
            .times(1)
            .returning(|_| Ok(()));
        let mut sender = MockPushSender::new();
        sender.expect_send().returning(|token, _| {
            Ok(match token {
                "dead" => SendOutcome::Invalid("UNREGISTERED".into()),
                _ => SendOutcome::Delivered,
            })
        });

        let d = Dispatcher::new(Arc::new(registry), Arc::new(sender));
        let report = d
            .dispatch(
                &live_flags(),
                &Audience::All,
                &PushMessage::new("t", "b"),
                now(),
            )
            .await
            .unwrap();
        assert_eq!(report.delivered, 1);
        assert_eq!(report.pruned, 1);
        assert_eq!(report.retryable, 0);
    }

    #[tokio::test]
    async fn a_retryable_failure_keeps_the_token() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_candidates()
            .returning(|_, _| Ok(vec![candidate("u1", "t1")]));
        // No `expect_prune_token`: a call would fail the test.
        let mut sender = MockPushSender::new();
        sender
            .expect_send()
            .returning(|_, _| Ok(SendOutcome::Retryable("503".into())));

        let d = Dispatcher::new(Arc::new(registry), Arc::new(sender));
        let report = d
            .dispatch(
                &live_flags(),
                &Audience::All,
                &PushMessage::new("t", "b"),
                now(),
            )
            .await
            .unwrap();
        assert_eq!(report.retryable, 1);
        assert_eq!(report.pruned, 0);
    }

    #[tokio::test]
    async fn a_closed_global_gate_never_touches_the_registry_or_the_sender() {
        for flags in [
            SelectionFlags {
                kill_switch_on: false,
                ..live_flags()
            },
            SelectionFlags {
                category_enabled: false,
                ..live_flags()
            },
        ] {
            // Both mocks are strict: any call at all fails the test.
            let d = Dispatcher::new(
                Arc::new(MockPushRegistry::new()),
                Arc::new(MockPushSender::new()),
            );
            let report = d
                .dispatch(&flags, &Audience::All, &PushMessage::new("t", "b"), now())
                .await
                .unwrap();
            assert_eq!(report, DispatchReport::default());
        }
    }

    #[tokio::test]
    async fn out_of_hours_selects_nobody_but_still_reads_candidates() {
        let mut registry = MockPushRegistry::new();
        registry
            .expect_candidates()
            .times(1)
            .returning(|_, _| Ok(vec![candidate("u1", "t1")]));
        let d = Dispatcher::new(Arc::new(registry), Arc::new(MockPushSender::new()));
        let report = d
            .dispatch(
                &live_flags(),
                &Audience::All,
                &PushMessage::new("t", "b"),
                DateTime::parse_from_rfc3339("2026-08-06T09:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .await
            .unwrap();
        assert_eq!(report.selected, 0);
    }

    #[tokio::test]
    async fn a_registry_failure_aborts_the_dispatch() {
        let mut registry = MockPushRegistry::new();
        registry.expect_candidates().returning(|_, _| {
            Err(cymbra_platform::AppError::Internal(anyhow::anyhow!(
                "db down"
            )))
        });
        let d = Dispatcher::new(Arc::new(registry), Arc::new(MockPushSender::new()));
        assert!(
            d.dispatch(
                &live_flags(),
                &Audience::All,
                &PushMessage::new("t", "b"),
                now()
            )
            .await
            .is_err()
        );
    }

    /// The dispatch path never evaluates admin-scoped rollout.
    struct NoAdmins;

    #[async_trait::async_trait]
    impl cymbra_feature_flags::AdminScopeResolver for NoAdmins {
        async fn is_platform_admin(&self, _user_id: &str) -> cymbra_platform::Result<bool> {
            Ok(false)
        }
    }

    #[test]
    fn flags_resolve_safe_when_nothing_is_declared_or_stored() {
        // No override store at all: every key falls back to its code default.
        let service = FlagService::new(
            cymbra_feature_flags::Registry::default(),
            None,
            Arc::new(cymbra_feature_flags::NoopBus),
            Arc::new(NoAdmins),
        );
        let f = resolve_flags(&service, "practice_streak", true);
        assert_eq!(f.category, "practice_streak");
        // Kill-switch defaults off, and the category's keys are not declared by
        // the platform → disabled, no hour gate, silent in the foreground.
        assert!(!f.kill_switch_on);
        assert!(!f.category_enabled);
        assert_eq!(f.schedule_hour, None);
        assert!(f.default_pref);
        assert_eq!(f.default_timezone, DEFAULT_TIMEZONE);
        assert!(!f.foreground);
    }
}
