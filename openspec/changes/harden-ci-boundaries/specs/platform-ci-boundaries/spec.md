## ADDED Requirements

### Requirement: No buildable unit ships without CI

Every app under `apps/` and every package under `packages/` or `crates/` SHALL be covered
by at least one workflow that checks it. Adding a unit that matches no workflow SHALL fail
CI with a message naming the uncovered path — silence SHALL NOT be a valid outcome.

The check SHALL run on pull requests, so the gap is reported before the unit reaches the
default branch rather than after.

#### Scenario: A new app with no workflow fails the build

- **WHEN** a pull request adds an app under `apps/` that no workflow selects
- **THEN** CI fails and names the uncovered path

#### Scenario: A new app with a workflow passes

- **WHEN** a pull request adds an app together with a workflow that selects it
- **THEN** CI passes

#### Scenario: The gap is caught before merge

- **WHEN** the uncovered unit is added
- **THEN** the failure appears on the pull request, not only after merging

### Requirement: A workflow's trigger matches what its job does

A workflow's path filters SHALL select exactly the units its job verifies. A workflow
whose trigger uses a wildcard across units SHALL either handle every unit that wildcard
selects, or narrow the trigger to the units it actually handles.

#### Scenario: A wildcard trigger handles every unit it selects

- **WHEN** a workflow triggers on a path glob spanning several units
- **THEN** its job verifies each unit the glob selects

#### Scenario: A single-unit job does not claim a wildcard

- **WHEN** a job is written for one specific unit
- **THEN** its trigger names that unit rather than a glob over its siblings

#### Scenario: An unrelated unit does not fire it

- **WHEN** a change touches a unit the workflow does not verify
- **THEN** that workflow does not run

### Requirement: A workflow's name says whether it is product-scoped

A workflow that covers one target SHALL be named `<target>-<verb>`. A workflow with no
target prefix SHALL be repo-wide, covering every product. The prefix SHALL be the
deployable or app it serves, not the language or toolchain it happens to use — a stack
name stops being true as soon as a second product uses the same stack.

Reading the workflow list SHALL therefore answer, without opening any file, which
workflows a given product owns.

#### Scenario: A product-scoped workflow carries its target

- **WHEN** a workflow checks or builds exactly one app or deployable
- **THEN** its name begins with that target

#### Scenario: A repo-wide workflow carries no target

- **WHEN** a workflow applies to every product (commit linting, security scanning,
  release automation)
- **THEN** its name is a bare verb with no target prefix

#### Scenario: A stack name is not a target

- **WHEN** a workflow covers one app written in a given language
- **THEN** it is named after the app, not the language, so a second app in that language
  does not make the name false
