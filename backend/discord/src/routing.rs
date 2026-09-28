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

//! `(product, category) → channel` routing (change: add-discord-notifications,
//! design D1).
//!
//! Channels are named as in `scripts/discord/server.json`, whose provisioning
//! writes one webhook per channel as `DISCORD_WEBHOOK_<CHANNEL>`. An unmapped
//! category resolves to **no channel** — a no-op, never a fallback to some default
//! channel, which would let one product post in another's section.

use std::collections::BTreeMap;

use crate::event::{Category, Product};

/// The channels of each product's section, as provisioned. A route may only
/// point inside its own product's section.
pub fn section(product: Product) -> &'static [&'static str] {
    match product {
        Product::Music => &[
            "music-announcements",
            "scores-and-soundfonts",
            "music-stats",
            "music-leaderboards",
        ],
    }
}

/// The routing table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Routing {
    routes: BTreeMap<Category, &'static str>,
}

impl Default for Routing {
    /// The production routes: accepted catalog items in `#scores-and-soundfonts`,
    /// season records in `#music-leaderboards`.
    fn default() -> Self {
        Self::from_routes([
            (Category::MusicCatalog, "scores-and-soundfonts"),
            (Category::MusicRecords, "music-leaderboards"),
        ])
    }
}

impl Routing {
    /// Build a table from explicit routes. A route leaving its product's section
    /// is dropped (and logged): it would cross products.
    pub fn from_routes(routes: impl IntoIterator<Item = (Category, &'static str)>) -> Self {
        let routes = routes
            .into_iter()
            .filter(|(category, channel)| {
                let inside = section(category.product()).contains(channel);
                if !inside {
                    tracing::error!(
                        category = category.key(),
                        channel,
                        "discord route dropped: channel outside the product's section"
                    );
                }
                inside
            })
            .collect();
        Self { routes }
    }

    /// The channel for `category`, or `None` — publish nothing.
    pub fn resolve(&self, category: Category) -> Option<&'static str> {
        self.routes.get(&category).copied()
    }
}

/// Prefix of the environment variables holding the webhooks, as written by
/// `scripts/discord/provision.sh`: `DISCORD_WEBHOOK_SCORES_AND_SOUNDFONTS` is the
/// webhook of `#scores-and-soundfonts`.
pub const WEBHOOK_ENV_PREFIX: &str = "DISCORD_WEBHOOK_";

/// Read the channel → webhook map from environment variables. Empty values are
/// unset; a value that is not a Discord webhook URL is ignored (and logged by
/// variable name only — the value may be a secret pasted in the wrong place).
pub fn webhooks_from_env<'a>(
    vars: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> BTreeMap<String, String> {
    vars.into_iter()
        .filter_map(|(name, value)| {
            let channel = name.strip_prefix(WEBHOOK_ENV_PREFIX)?;
            let value = value.trim();
            if value.is_empty() || channel.is_empty() {
                return None;
            }
            let valid = [
                "https://discord.com/api/webhooks/",
                "https://discordapp.com/api/webhooks/",
            ]
            .iter()
            .any(|p| value.starts_with(p));
            if !valid {
                tracing::warn!(variable = name, "ignored: not a Discord webhook URL");
                return None;
            }
            Some((
                channel.to_ascii_lowercase().replace('_', "-"),
                value.to_string(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhooks_are_read_from_their_variables() {
        let hooks = webhooks_from_env([
            (
                "DISCORD_WEBHOOK_SCORES_AND_SOUNDFONTS",
                " https://discord.com/api/webhooks/1/abc ",
            ),
            (
                "DISCORD_WEBHOOK_MUSIC_LEADERBOARDS",
                "https://discordapp.com/api/webhooks/2/d",
            ),
            ("DISCORD_WEBHOOK_MUSIC_STATS", ""),
            ("DISCORD_WEBHOOK_", "https://discord.com/api/webhooks/3/e"),
            ("DISCORD_WEBHOOK_ID_STATS", "http://example.com/hook"),
            ("CYMBRA_SMTP_URL", "https://discord.com/api/webhooks/4/f"),
        ]);
        assert_eq!(
            hooks.into_iter().collect::<Vec<_>>(),
            vec![
                (
                    "music-leaderboards".to_string(),
                    "https://discordapp.com/api/webhooks/2/d".to_string()
                ),
                (
                    "scores-and-soundfonts".to_string(),
                    "https://discord.com/api/webhooks/1/abc".to_string()
                ),
            ]
        );
    }

    #[test]
    fn production_routes_stay_inside_their_product_section() {
        let routing = Routing::default();
        for category in Category::ALL {
            let channel = routing.resolve(category).expect("every category is routed");
            assert!(section(category.product()).contains(&channel));
        }
        assert_eq!(
            routing.resolve(Category::MusicCatalog),
            Some("scores-and-soundfonts")
        );
        assert_eq!(
            routing.resolve(Category::MusicRecords),
            Some("music-leaderboards")
        );
    }

    #[test]
    fn an_unmapped_category_resolves_to_no_channel() {
        let routing = Routing::from_routes([(Category::MusicCatalog, "scores-and-soundfonts")]);
        assert_eq!(routing.resolve(Category::MusicRecords), None);
    }

    #[test]
    fn a_route_into_another_section_is_refused() {
        // `lingua-stats` belongs to Cymbra Lingua's section.
        let routing = Routing::from_routes([(Category::MusicRecords, "lingua-stats")]);
        assert_eq!(routing.resolve(Category::MusicRecords), None);
    }
}
