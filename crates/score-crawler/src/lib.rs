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

//! Score crawler engine: harvest only redistributable scores, convert to
//! validated `.mxl`, and (once the backend score store lands) ingest them.
//!
//! The pure, offline-testable core — licence gate, conversion,
//! metadata/difficulty extraction, and the adapter/orchestration contract — is
//! the heart; the git/dataset adapters and Postgres ingestion sit on top. The
//! operator interface is the CLI and the Docker fan-out (`docker-compose.yml`).

pub mod catalog;
pub mod cli;
pub mod config;
pub mod convert;
pub mod crawl;
pub mod difficulty;
pub mod fingerprint;
pub mod license;
pub mod manifest;
pub mod metadata;
pub mod output;
pub mod registry;
pub mod run;
pub mod sources;
pub mod state;
