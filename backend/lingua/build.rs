// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// build_client(false): nothing in the workspace calls this module over gRPC — the
// boundary next door is a Rust trait (a port), and the .proto is only the external
// contract the clients (extension, Apple app) generate their own stubs from. All
// three files share one package, so they compile in a single call.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile_protos(
            &[
                "proto/known_words.proto",
                "proto/deck.proto",
                "proto/stats.proto",
                "proto/lingua_admin.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
