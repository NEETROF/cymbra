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

//! The back-office flags this crate reads (change: add-discord-notifications,
//! design D8). Registered in `cymbra-feature-flags`; a test there pins the
//! literals to these builders.

use crate::event::Category;

/// Global kill-switch. **Default off**: the code deploys dark, and turning it
/// off also silences announcements already enqueued.
pub const DISCORD_ENABLED: &str = "discord.enabled";

/// The per-category flag — `discord.<product>.<category>` — so one product's
/// feed, or one kind of message, can be muted alone.
pub fn category_flag(category: Category) -> String {
    format!("discord.{}", category.key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_flags_are_product_namespaced() {
        assert_eq!(
            category_flag(Category::MusicCatalog),
            "discord.music.catalog"
        );
        assert_eq!(
            category_flag(Category::MusicRecords),
            "discord.music.records"
        );
    }
}
