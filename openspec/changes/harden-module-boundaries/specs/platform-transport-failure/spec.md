## ADDED Requirements

### Requirement: A transport failure is expressible and distinct from a domain error

The platform error type SHALL be able to represent a call that did not reach its callee or
did not answer in time, distinctly from a domain outcome such as "not found", "denied", or
"invalid". Converting a remote status into the platform error type SHALL preserve that
distinction, and SHALL NOT collapse a transport failure into the generic internal error.

A caller SHALL be able to decide, from the error alone, whether the answer is *no* or
whether there was *no answer*.

#### Scenario: A timeout is not a domain error

- **WHEN** a call to another backend component exceeds its deadline
- **THEN** the caller receives an error identifying a transport failure, not "not found"
  and not a generic internal error

#### Scenario: An unreachable callee is identified as such

- **WHEN** the callee is unreachable
- **THEN** the caller receives an error identifying unavailability

#### Scenario: A domain error survives the round trip

- **WHEN** the callee answers that a resource does not exist
- **THEN** the caller receives a not-found domain error, distinguishable from
  unavailability

### Requirement: An optional enrichment never fails silently

An enrichment lookup that fails for a reason other than a domain outcome SHALL emit a
diagnostic record naming the failed lookup and its subject. The enriched field MAY be
omitted from the response so a private or absent resource still yields a complete answer —
but it SHALL NOT be omitted silently.

A response that omits enrichment because a dependency failed SHALL therefore be traceable
after the fact, even though the caller received a success.

#### Scenario: A private profile yields no credit and no noise

- **WHEN** an enrichment lookup reports that the subject is private or absent
- **THEN** the field is omitted and no failure is recorded — this is the expected outcome

#### Scenario: An unreachable dependency leaves a trace

- **WHEN** an enrichment lookup fails because the dependency is unreachable or timed out
- **THEN** the field is omitted, the response still succeeds, and a diagnostic record
  names the lookup and the subject

#### Scenario: The visible behaviour is unchanged

- **WHEN** an enrichment lookup fails for any reason
- **THEN** the client receives the same successful response shape it received before this
  requirement existed
