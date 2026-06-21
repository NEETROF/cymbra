# Security Policy

## Reporting a vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report them privately through GitHub Security Advisories:
<https://github.com/NEETROF/cymbra/security/advisories/new>

If you cannot use that, contact the maintainers and we will open a private
advisory on your behalf.

Please include, as far as possible:
- a description of the issue and its impact;
- steps to reproduce or a proof of concept;
- affected component (e.g. `apps/music`, `crates/*`, `backend`) and platform;
- any suggested mitigation.

## Response

This is a community project maintained on a best-effort basis. We aim to:
- acknowledge your report within **5 business days**;
- keep you informed of progress;
- coordinate a fix and a disclosure timeline with you before going public.

We support **coordinated disclosure**: please give us a reasonable window to
release a fix before any public disclosure.

## Supported versions

Security fixes target the **latest release** of each artifact (and the `main`
branch). Older releases are not maintained.

## Scope

In scope: vulnerabilities in this repository's source code (Rust engine/FFI,
Flutter apps, backend) and CI/release configuration.

Out of scope: issues in third-party dependencies (report those upstream;
tell us if Cymbra is affected so we can bump the dependency), and the security
of self-hosted forks or modified builds.
