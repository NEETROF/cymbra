// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Shared gRPC helpers for the lingua services (thin transport glue).

#![allow(clippy::result_large_err)]

use std::time::{SystemTime, UNIX_EPOCH};

use cymbra_platform::AuthIdentity;
use tonic::{Request, Status};

/// The authenticated caller's user id, taken ONLY from the interceptor-injected
/// identity — never from the request body.
pub fn caller<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

/// The server receipt time in epoch milliseconds — the clamp reference for client ops.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
