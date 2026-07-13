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

//! robots.txt enforcement.
//!
//! Wraps [`texting_robots`] with our descriptive User-Agent so the web-crawl
//! adapters can ask "may I fetch this URL?" before every request. Parsing is
//! offline-testable from robots.txt content; the fetch/cache of robots.txt per
//! host is wired with the HTTP client in the web-adapter slice.

use anyhow::{Result, anyhow};
use texting_robots::Robot;

/// A parsed robots.txt policy for one host + one User-Agent.
pub struct RobotsPolicy {
    robot: Robot,
}

impl RobotsPolicy {
    /// Parses robots.txt `content` for `user_agent`.
    pub fn parse(user_agent: &str, content: &[u8]) -> Result<Self> {
        let robot =
            Robot::new(user_agent, content).map_err(|e| anyhow!("parse robots.txt: {e}"))?;
        Ok(Self { robot })
    }

    /// A permissive policy (used when a host has no robots.txt / it 404s).
    pub fn allow_all(user_agent: &str) -> Result<Self> {
        Self::parse(user_agent, b"")
    }

    /// Whether `url` may be fetched under this policy.
    pub fn allowed(&self, url: &str) -> bool {
        self.robot.allowed(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROBOTS: &[u8] = b"User-agent: *\n\
        Disallow: /private/\n\
        Disallow: /tmp\n\
        Allow: /\n";

    #[test]
    fn allows_public_paths() {
        let p = RobotsPolicy::parse("cymbra-score-crawler", ROBOTS).unwrap();
        assert!(p.allowed("https://example.org/scores/piece.musicxml"));
        assert!(p.allowed("https://example.org/"));
    }

    #[test]
    fn blocks_disallowed_paths() {
        let p = RobotsPolicy::parse("cymbra-score-crawler", ROBOTS).unwrap();
        assert!(!p.allowed("https://example.org/private/secret"));
        assert!(!p.allowed("https://example.org/tmp/x"));
    }

    #[test]
    fn empty_robots_allows_everything() {
        let p = RobotsPolicy::allow_all("cymbra-score-crawler").unwrap();
        assert!(p.allowed("https://example.org/anything/at/all"));
    }
}
