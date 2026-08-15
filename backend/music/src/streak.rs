// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Practice-streak data port (change: add-practice-streak).
//!
//! [`StreakRepo`] is the storage seam [`crate::StreakModule`] composes the pure
//! rules of [`crate::streak_core`] over: read/write a user's streak row, read
//! their spendable points, and — as ONE atomic operation — charge a freeze and
//! restore the streak.
//!
//! The freeze deliberately lives *in the port* rather than as "debit, then
//! write": the debit (an append to the shared `music.curation_points` ledger) and
//! the streak restore must either both happen or neither, or a crash between
//! them charges a user for a streak they did not get back. Both tables are in the
//! `music` schema, so the Postgres adapter can do it in one transaction; the
//! in-memory fake mirrors the same all-or-nothing semantics (including the
//! balance re-check) so the module's tests prove the rule without a database.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::streak_core::{ReminderCandidate, StreakState};

/// The ledger `reward_key` a streak freeze is recorded under, so a spend is
/// identifiable in the activity feed (and never confused with a shop redemption).
pub const FREEZE_REWARD_KEY: &str = "streak_freeze";

/// The ledger **idempotency key** for a freeze on `day` — the same
/// `<kind>:<discriminator>` shape the play awards use (change: add-play-rewards).
///
/// A user can only ever pay for one freeze per local day: a granted recovery
/// stamps `last_played_date = today`, after which the streak reads as intact and
/// the module refuses a second one. Encoding that invariant in the key hands it to
/// the unique index `curation_points_award_key_once_idx`, which is the only guard
/// that survives two confirmations racing each other — the balance re-check under
/// the row lock stops an overdraft, not a double charge.
pub fn freeze_award_key(day: chrono::NaiveDate) -> String {
    format!("{FREEZE_REWARD_KEY}:{day}")
}

/// Storage surface for the practice streak. Keyed by the plain `user_id` string
/// (no cross-schema FK), like every other music-module port.
#[async_trait]
pub trait StreakRepo: Send + Sync {
    /// A user's streak state. A user who has never played has no row yet — that
    /// is [`StreakState::default`], not an error.
    async fn get(&self, user_id: &str) -> Result<StreakState>;

    /// Write a user's streak state (upsert).
    async fn put(&self, user_id: &str, state: &StreakState) -> Result<()>;

    /// The user's spendable points balance (the curation-rewards ledger sum) —
    /// what the freeze is paid from.
    async fn spendable_points(&self, user_id: &str) -> Result<i64>;

    /// Charge `cost` points under `award_key` and write `state`, atomically.
    ///
    /// Returns `false` (and writes NOTHING) when the balance no longer covers the
    /// cost. Charging is **idempotent on `award_key`**: a second call for the same
    /// key restores the streak but does not debit again, so two confirmations
    /// racing each other cost the user one freeze, not two.
    async fn spend_and_restore(
        &self,
        user_id: &str,
        state: &StreakState,
        cost: i64,
        award_key: &str,
    ) -> Result<bool>;

    /// Every user with a live streak (`current_streak > 0`), with the account's
    /// stored timezone — the raw input to the reminder sweep's at-risk filter
    /// ([`crate::streak_core::at_risk_user_ids`]).
    async fn live_streaks(&self) -> Result<Vec<ReminderCandidate>>;
}

// --- In-memory fake (tests) -------------------------------------------------

#[derive(Default)]
struct FakeState {
    streaks: HashMap<String, StreakState>,
    zones: HashMap<String, String>,
    locales: HashMap<String, String>,
    balances: HashMap<String, i64>,
    /// Freeze debits appended, newest last: `(user_id, cost)`.
    debits: Vec<(String, i64)>,
    /// Award keys already charged — the fake's stand-in for the ledger's partial
    /// unique index on `(user_id, award_key)`.
    charged: HashSet<(String, String)>,
}

/// In-memory [`StreakRepo`] for module tests (no Postgres). Mirrors the adapter's
/// atomic spend semantics: an unaffordable freeze writes neither the debit nor
/// the streak, and a repeated award key restores without charging twice.
#[derive(Default)]
pub struct FakeStreakRepo {
    state: Mutex<FakeState>,
}

impl FakeStreakRepo {
    /// Seed a user's streak row (as if they had been playing).
    pub fn seed(&self, user_id: &str, state: StreakState) {
        self.state
            .lock()
            .expect("streak fake lock")
            .streaks
            .insert(user_id.to_string(), state);
    }

    /// Seed a user's spendable points balance.
    pub fn seed_balance(&self, user_id: &str, balance: i64) {
        self.state
            .lock()
            .expect("streak fake lock")
            .balances
            .insert(user_id.to_string(), balance);
    }

    /// Seed a user's stored IANA timezone (the reminder sweep's day boundary).
    pub fn seed_timezone(&self, user_id: &str, timezone: &str) {
        self.state
            .lock()
            .expect("streak fake lock")
            .zones
            .insert(user_id.to_string(), timezone.to_string());
    }

    /// Seed a user's stored locale tag (which reminder batch they land in).
    pub fn seed_locale(&self, user_id: &str, locale: &str) {
        self.state
            .lock()
            .expect("streak fake lock")
            .locales
            .insert(user_id.to_string(), locale.to_string());
    }

    /// Test helper: the freeze debits recorded so far, as `(user_id, cost)`.
    /// Empty proves the "no silent debit" rule held.
    pub fn debits(&self) -> Vec<(String, i64)> {
        self.state.lock().expect("streak fake lock").debits.clone()
    }
}

#[async_trait]
impl StreakRepo for FakeStreakRepo {
    async fn get(&self, user_id: &str) -> Result<StreakState> {
        Ok(self
            .state
            .lock()
            .expect("streak fake lock")
            .streaks
            .get(user_id)
            .copied()
            .unwrap_or_default())
    }

    async fn put(&self, user_id: &str, state: &StreakState) -> Result<()> {
        self.state
            .lock()
            .expect("streak fake lock")
            .streaks
            .insert(user_id.to_string(), *state);
        Ok(())
    }

    async fn spendable_points(&self, user_id: &str) -> Result<i64> {
        Ok(self
            .state
            .lock()
            .expect("streak fake lock")
            .balances
            .get(user_id)
            .copied()
            .unwrap_or(0))
    }

    async fn spend_and_restore(
        &self,
        user_id: &str,
        state: &StreakState,
        cost: i64,
        award_key: &str,
    ) -> Result<bool> {
        let mut st = self.state.lock().expect("streak fake lock");
        let key = (user_id.to_string(), award_key.to_string());
        // Already charged under this key (the unique index's job): restore, but
        // never debit a second time.
        if st.charged.contains(&key) {
            st.streaks.insert(user_id.to_string(), *state);
            return Ok(true);
        }
        let balance = st.balances.get(user_id).copied().unwrap_or(0);
        if balance < cost {
            return Ok(false); // all-or-nothing: nothing written
        }
        st.charged.insert(key);
        st.balances.insert(user_id.to_string(), balance - cost);
        st.debits.push((user_id.to_string(), cost));
        st.streaks.insert(user_id.to_string(), *state);
        Ok(true)
    }

    async fn live_streaks(&self) -> Result<Vec<ReminderCandidate>> {
        let st = self.state.lock().expect("streak fake lock");
        let mut rows: Vec<ReminderCandidate> = st
            .streaks
            .iter()
            .filter(|(_, s)| s.current > 0)
            .map(|(user_id, s)| ReminderCandidate {
                user_id: user_id.clone(),
                state: *s,
                timezone: st.zones.get(user_id).cloned().unwrap_or_default(),
                locale: st.locales.get(user_id).cloned().unwrap_or_default(),
            })
            .collect();
        // Deterministic order for tests (the adapter's ORDER BY user_id).
        rows.sort_by(|a, b| a.user_id.cmp(&b.user_id));
        Ok(rows)
    }
}
