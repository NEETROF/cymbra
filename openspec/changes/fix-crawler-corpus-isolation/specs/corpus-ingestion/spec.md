## ADDED Requirements

### Requirement: Crawler working files live outside the served corpus

The crawler SHALL write its working files — source checkouts and any other material kept for
its own operation rather than for serving — to a work location configured independently of
the corpus root, never derived from it. The corpus root SHALL contain only servable objects
under the corpus prefixes.

The crawler SHALL refuse to run when the resolved work location is inside the corpus root,
failing at startup rather than writing there, so a misconfigured deployment is reported
instead of silently polluting the served corpus.

#### Scenario: Source checkouts stay out of the corpus

- **WHEN** the crawler prepares its sources during a run
- **THEN** the checkouts are created under the configured work location, and the corpus root
  gains no entry other than servable objects under the corpus prefixes

#### Scenario: Work location inside the corpus root is refused

- **WHEN** the crawler is configured with a work location that resolves inside the corpus root
- **THEN** it exits with an error before crawling, and writes nothing to the corpus root

#### Scenario: Work location is configurable independently

- **WHEN** an operator points the corpus root and the work location at two different
  directories
- **THEN** servable objects are written to the former and working files to the latter, with no
  path of one nested in the other

### Requirement: The off-box mirror carries only servable objects

The off-box mirror of the corpus SHALL transfer only objects under the servable corpus
prefixes, selected by an allow-list rather than by excluding known-unwanted paths, so that any
non-servable content present at the corpus root is not mirrored. The mirrored key SHALL remain
equal to the catalog `object_key`, so the mirror stays readable as the storage origin.

#### Scenario: Non-servable content at the corpus root is not mirrored

- **WHEN** the corpus root contains an entry that is not under a servable corpus prefix and the
  mirror runs
- **THEN** that entry is not transferred, while the servable objects are

#### Scenario: Mirrored key equals the object key

- **WHEN** a servable object is mirrored off-box
- **THEN** its key in the mirror is exactly the `object_key` recorded on its catalog row

### Requirement: Unreferenced corpus objects are reconcilable

The system SHALL provide a maintenance operation that reconciles the corpus against the
catalog: it reports every corpus object, local and off-box, that no `catalog_scores` row
references. The operation SHALL report without writing by default, and remove the unreferenced
objects only when explicitly asked to apply.

The operation SHALL reason over the *set* of referenced `object_key` values, never per row, so
an object referenced by any remaining row is never removed. It SHALL abort without writing
when the referenced set is empty or when the proportion of objects it would remove exceeds a
configured safety threshold, so a failed or mis-scoped catalog query cannot empty the corpus.
Removal SHALL be reversible until an explicit, separate purge step.

#### Scenario: Dry run reports without writing

- **WHEN** the reconciliation runs without being asked to apply
- **THEN** it reports the unreferenced objects it found and neither removes nor moves anything

#### Scenario: Referenced objects are never removed

- **WHEN** the reconciliation applies its removals and an object is referenced by at least one
  catalog row
- **THEN** that object remains in place and servable, including when another row that also
  referenced it has been deleted

#### Scenario: Implausible reference set aborts the run

- **WHEN** the referenced-key set is empty, or the share of objects to remove exceeds the
  safety threshold
- **THEN** the operation aborts without removing anything and reports why

#### Scenario: Removal is reversible until purged

- **WHEN** the reconciliation has applied its removals and no purge has been run
- **THEN** the removed objects can still be restored

## MODIFIED Requirements

### Requirement: Idempotent ingestion

The system SHALL make ingestion idempotent: re-running the crawler SHALL NOT
create duplicate object-store objects or duplicate `catalog_scores` rows for the
same content. Dedup SHALL use the SHA-256 content hash, checked against both the
in-run set and existing `catalog_scores` rows.

Object-level idempotence SHALL hold **at write time and without consulting the catalog**,
because the crawler writes its objects before — and independently of — any catalog
connection. The object key of a retained score SHALL therefore be derived from its content
hash, so that re-crawling unchanged content resolves to the same key and rewrites the same
object instead of creating a second one. A crawl that retains only already-known content
SHALL leave the number of corpus objects unchanged.

Deriving the object key from content SHALL NOT change how a catalog row is identified: the
row's own identifier stays independent of the key, and readers SHALL resolve an object only
through the `object_key` recorded on the row, never by rebuilding it from the identifier.

#### Scenario: Re-ingesting existing content is a no-op
- **WHEN** the crawler encounters content whose SHA-256 already exists in
  `catalog_scores`
- **THEN** no new object is written and no new row is inserted for that content

#### Scenario: Re-crawling unchanged content does not grow the corpus
- **WHEN** a crawl retains only content that was already ingested by an earlier run
- **THEN** the corpus contains no object it did not contain before the run, the earlier
  objects remaining referenced by their existing rows

#### Scenario: Identical content resolves to one row and one object
- **WHEN** the same content is retained from two different sources, or by two successive runs
- **THEN** a single `catalog_scores` row and a single corpus object exist for it, dedup having
  collapsed the duplicate before a second row or object could be created

#### Scenario: Row identity is unaffected by the content-derived key
- **WHEN** a score is ingested with a content-derived `object_key`
- **THEN** its catalog row keeps its own identifier, and the object is resolved from the
  stored `object_key` rather than from that identifier
