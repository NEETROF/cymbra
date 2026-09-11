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

//! Cymbra Lingua — the Claude Code plugin (`lingua` binary).
//!
//! Ingests AI-agent session vocabulary locally: the same `lingua-core` brain as the
//! browser extension (same `analyzer_version`), compiled natively, wired to Claude Code
//! through a `Stop` hook (ingestion), a statusline, a `/vocab` skill and an MCP server.
//! Local-only by construction (`add-lingua-agent`, design D2): no transcript content is
//! persisted, and the binary opens no network connection.

pub mod engine;
pub mod ingest;
pub mod mcp;
pub mod source;
pub mod statusline;
pub mod store;
pub mod vocab;
