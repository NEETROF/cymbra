#!/usr/bin/env python3
"""Enforce the modular-monolith dependency rule (task 7.1 / design D0).

Normal (non-dev) dependencies only:
  - an implementation crate (cymbra-auth, cymbra-user) MUST NOT depend on another
    implementation crate — only on `<other>-port` + platform;
  - a `-port` contract crate MUST NOT depend on any implementation or other port
    crate — only on platform (+ third-party).

Dev-dependencies are exempt (tests may use the impl crates / fakes).
"""
import json
import subprocess
import sys

IMPL = {"cymbra-auth", "cymbra-user"}
PORTS = {"cymbra-auth-port", "cymbra-user-port"}

md = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    )
)
pkgs = {p["name"]: p for p in md["packages"]}


def normal_deps(name):
    return {
        d["name"]
        for d in pkgs[name]["dependencies"]
        if d.get("kind") in (None, "normal")
    }


violations = []

for impl in IMPL & pkgs.keys():
    for dep in normal_deps(impl) & IMPL:
        if dep != impl:
            violations.append(f"{impl} depends on impl crate {dep} (use {dep}-port)")

for port in PORTS & pkgs.keys():
    for dep in normal_deps(port) & (IMPL | PORTS):
        if dep != port:
            violations.append(f"{port} depends on {dep} (ports depend on platform only)")

if violations:
    print("Module boundary violations:")
    for v in violations:
        print(f"  - {v}")
    sys.exit(1)

print("Module boundaries OK (no impl->impl or port->impl dependencies).")
