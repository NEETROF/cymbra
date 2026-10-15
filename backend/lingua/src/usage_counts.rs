// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Anonymous usage counts (change: add-lingua-usage-counts, spec `lingua-usage-counts`):
//! an installation's steps and the leaving page's answers added to week counters, and
//! nothing else kept.
//!
//! Every boundary is a port declared here (the consumer declares it): the counters
//! ([`UsageCountsRepo`] — no row per report, no address, no header), the runtime knobs
//! ([`UsageCountsPolicy`] — the kill switch, the ceiling and the latest version, from the
//! flag registry) and the time ([`Clock`]). Floods are refused by an in-memory
//! [`FloodGuard`] that holds only a keyed hash of each network, never the address, and
//! forgets it when the clock hour ends. Old counters are deleted by [`prune_expired`], run
//! by the worker once a day.
//!
//! Keeping nothing: the module never logs a report, an answer or an address. The address
//! reaches the guard alone and is dropped when the call returns.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::usage_counts_core::{self as core, Invalid};

#[cfg(any(test, feature = "mock"))]
use mockall::automock;

/// The week counters (a thin Postgres adapter). Values reach it already bounded.
#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait]
pub trait UsageCountsRepo: Send + Sync {
    /// Add one to the counter of each step for `(week, browser, version)`, unless the
    /// week's counter for that step and browser, every version summed, has reached
    /// `ceiling` — so a week marked capped in the console really is (design D10).
    async fn count_steps(
        &self,
        week: i32,
        browser: &str,
        version: &str,
        steps: &[&'static str],
        ceiling: i64,
    ) -> Result<()>;
    /// Add one to a leaving reason's counter for `(week, reason, browser, version)`, unless
    /// the week's counter for that reason and browser, versions summed, reached `ceiling`.
    async fn count_leave(
        &self,
        week: i32,
        reason: &str,
        browser: &str,
        version: &str,
        ceiling: i64,
    ) -> Result<()>;
    /// Delete every counter whose week starts before `week`.
    async fn delete_before(&self, week: i32) -> Result<()>;
}

/// The runtime knobs, read on every request.
#[cfg_attr(any(test, feature = "mock"), automock)]
pub trait UsageCountsPolicy: Send + Sync {
    /// `lingua.usage_counts.enabled`: off, requests are acknowledged and nothing is counted.
    fn enabled(&self) -> bool;
    /// `lingua.usage_counts.ceiling`: the value a week's counter for one step (or reason)
    /// and one browser family stops growing at, versions summed.
    fn ceiling(&self) -> i64;
    /// `lingua.usage_counts.latest_version`: the latest released `major.minor`, which
    /// bounds the versions the counters name (design D10).
    fn latest_version(&self) -> String;
}

/// The time, injected so the hour of the guard can be tested.
#[cfg_attr(any(test, feature = "mock"), automock)]
pub trait Clock: Send + Sync {
    /// Seconds since the Unix epoch.
    fn now_secs(&self) -> u64;
}

/// The system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// Requests one network may make within a clock hour (design D10).
pub const FLOOD_LIMIT: u32 = 60;
/// Networks the guard holds at most; a new one is refused while it is full.
pub const FLOOD_CAPACITY: usize = 50_000;
/// The default ceiling of a counter.
pub const DEFAULT_CEILING: i64 = 5_000;

const HOUR_SECS: u64 = 3_600;

type Key = [u8; 32];

/// The networks seen in one clock hour and how many requests each made. When the hour
/// ends the whole table is replaced at once — no entry is ever visited to expire it, so
/// no request scans the table, full or not (the review's DoS: a full table used to be
/// swept on every new network, under the lock).
struct Table {
    /// The clock hour (`now / 3600`) the entries belong to.
    hour: u64,
    entries: HashMap<Key, u32>,
}

/// Refuses more than [`FLOOD_LIMIT`] requests a clock hour from one network — an IPv4
/// address, or an IPv6 address's /64 prefix — holding only an HMAC of it under a key drawn
/// at start-up and never written, and at most [`FLOOD_CAPACITY`] entries. The address
/// itself is never stored, so it cannot leak into a log, a panic message or the disk.
///
/// Every request costs one HMAC (outside the lock) and one hash-map lookup (inside it):
/// constant time, whether the table is empty or full. A full table refuses a new network
/// until the hour ends; the previous hour's table is dropped after the lock is released.
pub struct FloodGuard {
    key: Key,
    limit: u32,
    capacity: usize,
    table: Mutex<Table>,
}

impl FloodGuard {
    /// A guard under a fresh random key.
    pub fn new() -> Result<Self> {
        let mut key = [0u8; 32];
        getrandom::getrandom(&mut key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("flood guard key: {e}")))?;
        Ok(Self::with_key(key, FLOOD_LIMIT, FLOOD_CAPACITY))
    }

    /// A guard under a given key and bounds (tests).
    pub fn with_key(key: Key, limit: u32, capacity: usize) -> Self {
        Self {
            key,
            limit,
            capacity,
            table: Mutex::new(Table {
                hour: 0,
                entries: HashMap::new(),
            }),
        }
    }

    /// The network an address belongs to: the IPv4 address itself, or an IPv6 address's
    /// /64 prefix (one home or phone holds a whole /64). An IPv4-mapped IPv6 address is its
    /// IPv4 address. What cannot be parsed is keyed as given (all such requests share one
    /// budget).
    fn network(addr: &str) -> Vec<u8> {
        match addr.trim().parse::<IpAddr>() {
            Ok(IpAddr::V4(v4)) => v4.octets().to_vec(),
            Ok(IpAddr::V6(v6)) => match v6.to_ipv4_mapped() {
                Some(v4) => v4.octets().to_vec(),
                None => {
                    let mut prefix = vec![6u8];
                    prefix.extend_from_slice(&v6.octets()[..8]);
                    prefix
                }
            },
            Err(_) => {
                let mut raw = vec![0u8];
                raw.extend_from_slice(addr.as_bytes());
                raw
            }
        }
    }

    fn hash(&self, addr: &str) -> Key {
        let mut mac =
            <Hmac<Sha256> as Mac>::new_from_slice(&self.key).expect("HMAC takes any key length");
        mac.update(&Self::network(addr));
        mac.finalize().into_bytes().into()
    }

    /// Count one request from `addr` at `now`; `false` when it must be refused: the
    /// network spent this clock hour's budget, or the table is full and does not hold it.
    pub fn admit(&self, addr: &str, now: u64) -> bool {
        let key = self.hash(addr);
        let hour = now / HOUR_SECS;
        let (admitted, previous) = {
            let mut table = self.table.lock().unwrap_or_else(|p| p.into_inner());
            // A new hour: the old table is taken out whole and forgotten below, once the
            // lock is released.
            let previous = if table.hour != hour {
                table.hour = hour;
                Some(std::mem::take(&mut table.entries))
            } else {
                None
            };
            let full = table.entries.len() >= self.capacity;
            let admitted = match table.entries.get_mut(&key) {
                Some(count) if *count >= self.limit => false,
                Some(count) => {
                    *count += 1;
                    true
                }
                None if full => false,
                None => {
                    table.entries.insert(key, 1);
                    true
                }
            };
            (admitted, previous)
        };
        drop(previous);
        admitted
    }

    /// Networks currently held (tests).
    pub fn len(&self) -> usize {
        self.table
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entries
            .len()
    }

    /// Whether the guard holds no network.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A report as the gRPC layer hands it over (the message fields, nothing else).
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub install_week: i32,
    pub browser: i32,
    pub version: String,
    pub steps: Vec<i32>,
}

/// A leaving answer as the HTTP route hands it over.
#[derive(Clone, Debug, Default)]
pub struct Leave {
    pub reason: String,
    pub browser: String,
    pub version: String,
}

fn invalid(i: Invalid) -> AppError {
    AppError::InvalidArgument(i.message().into())
}

fn too_many() -> AppError {
    AppError::ResourceExhausted("too many requests".into())
}

/// The UTC day number of `now_secs`.
fn utc_day(now_secs: u64) -> i32 {
    (now_secs / 86_400) as i32
}

/// Delete every counter whose week is more than 25 months old on the UTC day of
/// `now_secs`, and return the first week kept (design D9). Run once a day by the worker's
/// scheduled `lingua_usage_prune` job — not from the request path, so retention holds
/// whether counting is on or off and whether any report arrives (the review: pruning used
/// to run only after the kill switch's check, on a report).
pub async fn prune_expired(repo: &dyn UsageCountsRepo, now_secs: u64) -> Result<i32> {
    let cutoff = core::retention_cutoff(utc_day(now_secs));
    repo.delete_before(cutoff).await?;
    Ok(cutoff)
}

pub struct UsageCountsModule {
    repo: Arc<dyn UsageCountsRepo>,
    policy: Arc<dyn UsageCountsPolicy>,
    clock: Arc<dyn Clock>,
    guard: FloodGuard,
}

impl UsageCountsModule {
    pub fn new(
        repo: Arc<dyn UsageCountsRepo>,
        policy: Arc<dyn UsageCountsPolicy>,
        clock: Arc<dyn Clock>,
        guard: FloodGuard,
    ) -> Self {
        Self {
            repo,
            policy,
            clock,
            guard,
        }
    }

    fn today(&self) -> i32 {
        utc_day(self.clock.now_secs())
    }

    /// The floods first: a malformed request spends the network's budget too.
    fn admit(&self, addr: &str) -> Result<()> {
        if self.guard.admit(addr, self.clock.now_secs()) {
            Ok(())
        } else {
            Err(too_many())
        }
    }

    /// A request whose body could not even be read: refused as invalid — once the flood
    /// guard has counted it, so malformed requests cannot be sent past the limit either.
    pub fn refuse_malformed(&self, addr: &str) -> AppError {
        match self.admit(addr) {
            Err(e) => e,
            Ok(()) => AppError::InvalidArgument("the answer is not one of the six".into()),
        }
    }

    /// A counter that could not be written is a failure the client retries.
    fn unavailable(e: AppError) -> AppError {
        tracing::warn!(error = %e, "usage counts: counter not written");
        AppError::Unavailable("usage counts".into())
    }

    /// Count a report's steps. `Ok` acknowledges it, counted or not (flag off, week out of
    /// range, ceiling reached); a malformed report is `InvalidArgument`, a flood
    /// `ResourceExhausted`. Its version is filed under its own name only near the latest
    /// release (`other` otherwise), so the versions a week holds stay bounded.
    pub async fn report(&self, addr: &str, r: &Report) -> Result<()> {
        self.admit(addr)?;
        let checked =
            core::check_report(r.install_week, r.browser, &r.version, &r.steps).map_err(invalid)?;
        if !self.policy.enabled() {
            return Ok(());
        }
        if !core::week_in_window(checked.week, self.today()) {
            return Ok(());
        }
        let steps: Vec<&'static str> = checked.steps.iter().map(|s| s.as_str()).collect();
        let version = core::filed_version(checked.version, &self.policy.latest_version());
        self.repo
            .count_steps(
                checked.week,
                checked.browser.as_str(),
                &version,
                &steps,
                self.policy.ceiling(),
            )
            .await
            .map_err(Self::unavailable)
    }

    /// Count a leaving answer under the UTC week it arrived in, with the same flag, guard,
    /// ceiling, version bound and retention as the reports.
    pub async fn leave(&self, addr: &str, l: &Leave) -> Result<()> {
        self.admit(addr)?;
        let checked = core::check_leave(&l.reason, &l.browser, &l.version).map_err(invalid)?;
        if !self.policy.enabled() {
            return Ok(());
        }
        let version = checked.filed_version(&self.policy.latest_version());
        self.repo
            .count_leave(
                core::monday_of(self.today()),
                checked.reason.as_str(),
                checked.browser,
                &version,
                self.policy.ceiling(),
            )
            .await
            .map_err(Self::unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage_counts_core::{monday_of, retention_cutoff};
    use mockall::predicate::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// 2026-10-21, a Wednesday, at noon UTC.
    const NOW: u64 = 20_747 * 86_400 + 43_200;
    const TODAY: i32 = 20_747;
    const ADDR: &str = "203.0.113.5";

    fn monday() -> i32 {
        monday_of(TODAY)
    }

    fn policy(enabled: bool, ceiling: i64) -> MockUsageCountsPolicy {
        let mut p = MockUsageCountsPolicy::new();
        p.expect_enabled().return_const(enabled);
        p.expect_ceiling().return_const(ceiling);
        p.expect_latest_version().return_const("1.10".to_string());
        p
    }

    /// A clock the test moves (an atomic the mock reads).
    fn clock(at: Arc<AtomicU64>) -> MockClock {
        let mut c = MockClock::new();
        c.expect_now_secs()
            .returning(move || at.load(Ordering::SeqCst));
        c
    }

    fn module(repo: MockUsageCountsRepo, policy: MockUsageCountsPolicy) -> UsageCountsModule {
        module_at(repo, policy, Arc::new(AtomicU64::new(NOW)))
    }

    fn module_at(
        repo: MockUsageCountsRepo,
        policy: MockUsageCountsPolicy,
        at: Arc<AtomicU64>,
    ) -> UsageCountsModule {
        UsageCountsModule::new(
            Arc::new(repo),
            Arc::new(policy),
            Arc::new(clock(at)),
            FloodGuard::with_key([7; 32], FLOOD_LIMIT, FLOOD_CAPACITY),
        )
    }

    fn report(steps: &[i32]) -> Report {
        Report {
            install_week: monday(),
            browser: 2,
            version: "1.10".into(),
            steps: steps.to_vec(),
        }
    }

    /// A repo that accepts any write (and is never asked to prune: the request path does
    /// not).
    fn accepting() -> MockUsageCountsRepo {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps().returning(|_, _, _, _, _| Ok(()));
        repo.expect_count_leave().returning(|_, _, _, _, _| Ok(()));
        repo
    }

    #[tokio::test]
    async fn a_report_is_counted_under_its_week() {
        // No `delete_before` expectation: a report never prunes (the worker does).
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .withf(|week, browser, version, steps, ceiling| {
                *week == monday_of(TODAY)
                    && browser == "firefox"
                    && version == "1.10"
                    && steps == ["installed", "set_up"]
                    && *ceiling == 5_000
            })
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let m = module(repo, policy(true, 5_000));
        m.report(ADDR, &report(&[2, 1])).await.unwrap();
    }

    #[tokio::test]
    async fn the_flag_off_acknowledges_and_touches_nothing() {
        // No expectation set: any repo call would panic.
        let m = module(MockUsageCountsRepo::new(), policy(false, 5_000));
        m.report(ADDR, &report(&[1])).await.unwrap();
        m.leave(
            ADDR,
            &Leave {
                reason: "slow".into(),
                browser: "firefox".into(),
                version: "1.10".into(),
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn a_week_out_of_range_is_acknowledged_not_counted() {
        let m = module(MockUsageCountsRepo::new(), policy(true, 5_000));
        let mut old = report(&[1]);
        old.install_week = monday_of(TODAY - 200);
        m.report(ADDR, &old).await.unwrap();
        let mut ahead = report(&[1]);
        ahead.install_week = monday_of(TODAY + 7);
        m.report(ADDR, &ahead).await.unwrap();
    }

    #[tokio::test]
    async fn malformed_reports_are_invalid_and_count_nothing() {
        let m = module(MockUsageCountsRepo::new(), policy(true, 5_000));
        let mut not_monday = report(&[1]);
        not_monday.install_week += 1;
        let mut bad_version = report(&[1]);
        bad_version.version = "1.10.0".into();
        let mut leading_zero = report(&[1]);
        leading_zero.version = "007.0".into();
        for r in [
            report(&[9]),
            report(&[]),
            not_monday,
            bad_version,
            leading_zero,
        ] {
            assert!(matches!(
                m.report(ADDR, &r).await,
                Err(AppError::InvalidArgument(_))
            ));
        }
    }

    /// The review: `is_version` accepted ~1.23M strings, each a row of its own in a
    /// counter's key. A version far from the latest release is now filed under `other`.
    #[tokio::test]
    async fn a_version_far_from_the_latest_is_filed_under_other() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .withf(|_, _, v, _, _| v == "other")
            .times(2)
            .returning(|_, _, _, _, _| Ok(()));
        repo.expect_count_steps()
            .withf(|_, _, v, _, _| v == "1.13")
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let m = module(repo, policy(true, 5_000));
        for version in ["999.999", "1.14", "1.13"] {
            let mut r = report(&[1]);
            r.version = version.into();
            m.report(ADDR, &r).await.unwrap();
        }
    }

    #[tokio::test]
    async fn the_latest_version_comes_from_the_policy() {
        let mut p = MockUsageCountsPolicy::new();
        p.expect_enabled().return_const(true);
        p.expect_ceiling().return_const(5_000i64);
        p.expect_latest_version().return_const("2.0".to_string());
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .withf(|_, _, v, _, _| v == "other")
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        repo.expect_count_leave()
            .withf(|_, _, _, v, _| v == "2.1")
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let m = module(repo, p);
        m.report(ADDR, &report(&[1])).await.unwrap(); // 1.10 is behind a 2.0 latest
        m.leave(
            ADDR,
            &Leave {
                reason: "slow".into(),
                browser: "firefox".into(),
                version: "2.1".into(),
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn the_ceiling_reaches_the_repo() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .withf(|_, _, _, _, ceiling| *ceiling == 3)
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let m = module(repo, policy(true, 3));
        m.report(ADDR, &report(&[1])).await.unwrap();
    }

    #[tokio::test]
    async fn a_failed_write_is_unavailable_so_the_client_retries() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .returning(|_, _, _, _, _| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        let m = module(repo, policy(true, 5_000));
        assert!(matches!(
            m.report(ADDR, &report(&[1])).await,
            Err(AppError::Unavailable(_))
        ));
    }

    #[tokio::test]
    async fn the_61st_request_of_the_hour_is_refused() {
        let m = module(accepting(), policy(true, 5_000));
        for _ in 0..60 {
            m.report(ADDR, &report(&[1])).await.unwrap();
        }
        assert!(matches!(
            m.report(ADDR, &report(&[1])).await,
            Err(AppError::ResourceExhausted(_))
        ));
        // Another address keeps its own budget.
        m.report("198.51.100.7", &report(&[1])).await.unwrap();
    }

    #[tokio::test]
    async fn forty_installations_behind_one_school_address_are_all_counted() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_steps()
            .times(40)
            .returning(|_, _, _, _, _| Ok(()));
        let m = module(repo, policy(true, 5_000));
        for _ in 0..40 {
            m.report(ADDR, &report(&[1])).await.unwrap();
        }
    }

    #[tokio::test]
    async fn one_ipv6_network_shares_one_budget() {
        let m = module(accepting(), policy(true, 5_000));
        for i in 0..60 {
            let addr = format!("2001:db8:1:2::{:x}", i + 1);
            m.report(&addr, &report(&[1])).await.unwrap();
        }
        assert!(matches!(
            m.report("2001:db8:1:2:ffff::1", &report(&[1])).await,
            Err(AppError::ResourceExhausted(_))
        ));
        // Another /64 is another network.
        m.report("2001:db8:1:3::1", &report(&[1])).await.unwrap();
    }

    #[tokio::test]
    async fn the_leaving_route_shares_the_budget() {
        let m = module(accepting(), policy(true, 5_000));
        for _ in 0..60 {
            m.report(ADDR, &report(&[1])).await.unwrap();
        }
        let answer = Leave {
            reason: "slow".into(),
            browser: "firefox".into(),
            version: "1.10".into(),
        };
        assert!(matches!(
            m.leave(ADDR, &answer).await,
            Err(AppError::ResourceExhausted(_))
        ));
    }

    #[test]
    fn a_full_table_refuses_a_new_network_until_entries_expire() {
        let guard = FloodGuard::with_key([1; 32], 60, 3);
        for a in ["192.0.2.1", "192.0.2.2", "192.0.2.3"] {
            assert!(guard.admit(a, NOW));
        }
        assert_eq!(guard.len(), 3);
        // Full: a new network is refused, a known one still admitted.
        assert!(!guard.admit("192.0.2.4", NOW + 10));
        assert!(guard.admit("192.0.2.1", NOW + 10));
        // Once the clock hour is over, the whole table is forgotten and makes room.
        assert!(guard.admit("192.0.2.4", NOW + 3_600));
        assert_eq!(guard.len(), 1);
    }

    #[test]
    fn an_entry_is_forgotten_when_its_clock_hour_ends() {
        // NOW is noon sharp: its clock hour runs to NOW + 3 599.
        let guard = FloodGuard::with_key([1; 32], 2, 10);
        assert!(guard.admit(ADDR, NOW));
        assert!(guard.admit(ADDR, NOW + 1));
        assert!(!guard.admit(ADDR, NOW + 3_599));
        // A new hour, a new budget.
        assert!(guard.admit(ADDR, NOW + 3_600));
        assert!(guard.admit("192.0.2.9", NOW + 3_600));
        assert_eq!(guard.len(), 2);
        // An entry nobody revisits goes with its hour.
        assert!(guard.admit("192.0.2.10", NOW + 7_200));
        assert_eq!(guard.len(), 1);
        assert!(!guard.is_empty());
    }

    /// The review's DoS: with the table full, every new network used to sweep all 50 000
    /// entries under the lock. Now a request costs one HMAC and one lookup whatever the
    /// table holds: refusing new networks against a full table takes about as long as
    /// admitting one known network the same number of times (a ratio, so the test holds on
    /// any machine and under coverage instrumentation; the old sweep was ~10 000× slower).
    #[test]
    fn a_full_table_stays_constant_time_per_request() {
        const REQUESTS: usize = 5_000;
        let full = FloodGuard::with_key([2; 32], FLOOD_LIMIT, FLOOD_CAPACITY);
        for i in 0..FLOOD_CAPACITY {
            let addr = format!("10.{}.{}.{}", i >> 16, (i >> 8) & 0xff, i & 0xff);
            assert!(full.admit(&addr, NOW));
        }
        assert_eq!(full.len(), FLOOD_CAPACITY);
        let new_networks: Vec<String> = (0..REQUESTS)
            .map(|i| format!("172.16.{}.{}", i >> 8, i & 0xff))
            .collect();
        let started = std::time::Instant::now();
        for (i, addr) in new_networks.iter().enumerate() {
            assert!(!full.admit(addr, NOW + 1 + (i as u64 % 3_000)));
        }
        let against_full = started.elapsed();
        assert_eq!(full.len(), FLOOD_CAPACITY);

        let baseline = FloodGuard::with_key([2; 32], u32::MAX, FLOOD_CAPACITY);
        let started = std::time::Instant::now();
        for i in 0..REQUESTS {
            assert!(baseline.admit(ADDR, NOW + 1 + (i as u64 % 3_000)));
        }
        let against_one = started.elapsed();
        assert!(
            against_full < against_one * 20 + std::time::Duration::from_millis(50),
            "a full table cost {against_full:?} for {REQUESTS} requests, one known network {against_one:?}"
        );
    }

    #[test]
    fn the_guard_holds_no_address_in_clear() {
        let guard = FloodGuard::with_key([3; 32], 60, 10);
        guard.admit(ADDR, NOW);
        let table = guard.table.lock().unwrap();
        let (key, _) = table.entries.iter().next().unwrap();
        // The key is a 32-byte MAC; neither the text nor the octets of the address are in it.
        assert!(!key.windows(4).any(|w| w == [203, 0, 113, 5]));
        assert_ne!(key, &guard.hash("198.51.100.7"));
        // An IPv4-mapped IPv6 address is the same network as its IPv4 address.
        assert_eq!(guard.hash("::ffff:203.0.113.5"), guard.hash(ADDR));
        // An unparseable address is still keyed (all such requests share one budget).
        assert_eq!(guard.hash("unknown"), guard.hash("unknown"));
        assert_ne!(guard.hash("unknown"), guard.hash(ADDR));
    }

    #[test]
    fn a_fresh_guard_draws_a_key() {
        let a = FloodGuard::new().unwrap();
        let b = FloodGuard::new().unwrap();
        assert_ne!(a.hash(ADDR), b.hash(ADDR));
        assert!(a.is_empty());
    }

    /// The review: pruning used to run only on a report, after the kill switch's check, so
    /// with counting off (or no traffic) counters outlived their 25 months. Retention is now
    /// its own daily run, which reads neither the switch nor any request.
    #[tokio::test]
    async fn retention_runs_without_the_switch_or_a_report() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_delete_before()
            .with(eq(retention_cutoff(TODAY)))
            .times(1)
            .returning(|_| Ok(()));
        // No policy and no module are involved: nothing can stop it.
        assert_eq!(
            prune_expired(&repo, NOW).await.unwrap(),
            retention_cutoff(TODAY)
        );
        // A failure is the job's to retry.
        let mut failing = MockUsageCountsRepo::new();
        failing
            .expect_delete_before()
            .returning(|_| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        assert!(prune_expired(&failing, NOW).await.is_err());
    }

    #[tokio::test]
    async fn a_report_or_an_answer_never_prunes() {
        // `delete_before` has no expectation: a call would panic.
        let at = Arc::new(AtomicU64::new(NOW));
        let m = module_at(accepting(), policy(true, 5_000), at.clone());
        m.report(ADDR, &report(&[1])).await.unwrap();
        at.store(NOW + 86_400, Ordering::SeqCst);
        m.report(ADDR, &report(&[1])).await.unwrap();
        m.leave(
            ADDR,
            &Leave {
                reason: "other".into(),
                browser: "unknown".into(),
                version: "unknown".into(),
            },
        )
        .await
        .unwrap();
    }

    /// The review's nit: a body that could not be read was refused before the guard, so a
    /// flood of malformed requests was never limited.
    #[test]
    fn malformed_requests_spend_the_budget_too() {
        let m = module(MockUsageCountsRepo::new(), policy(true, 5_000));
        for _ in 0..FLOOD_LIMIT {
            assert!(matches!(
                m.refuse_malformed(ADDR),
                AppError::InvalidArgument(_)
            ));
        }
        assert!(matches!(
            m.refuse_malformed(ADDR),
            AppError::ResourceExhausted(_)
        ));
    }

    #[tokio::test]
    async fn a_leaving_answer_is_counted_under_the_week_it_arrived_in() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .withf(|week, reason, browser, version, ceiling| {
                *week == monday_of(TODAY)
                    && reason == "slow"
                    && browser == "firefox"
                    && version == "1.10"
                    && *ceiling == 5_000
            })
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));
        let m = module(repo, policy(true, 5_000));
        m.leave(
            ADDR,
            &Leave {
                reason: "slow".into(),
                browser: "firefox".into(),
                version: "1.10".into(),
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn free_text_is_refused_as_invalid() {
        let m = module(MockUsageCountsRepo::new(), policy(true, 5_000));
        let r = m
            .leave(
                ADDR,
                &Leave {
                    reason: "please call me on 06 12 34 56 78".into(),
                    browser: "firefox".into(),
                    version: "1.10".into(),
                },
            )
            .await;
        assert!(matches!(r, Err(AppError::InvalidArgument(_))));
    }

    #[tokio::test]
    async fn a_failed_leaving_write_is_unavailable() {
        let mut repo = MockUsageCountsRepo::new();
        repo.expect_count_leave()
            .returning(|_, _, _, _, _| Err(AppError::Internal(anyhow::anyhow!("db down"))));
        let m = module(repo, policy(true, 5_000));
        let r = m
            .leave(
                ADDR,
                &Leave {
                    reason: "slow".into(),
                    browser: "firefox".into(),
                    version: "1.10".into(),
                },
            )
            .await;
        assert!(matches!(r, Err(AppError::Unavailable(_))));
    }

    #[test]
    fn the_system_clock_reads_the_time() {
        assert!(SystemClock.now_secs() > NOW - 86_400 * 365);
    }
}
