# Omera storage evolution

Status: design direction; optimizations require measurements and append-only migrations.

## Decision

Retain embedded SQLite with WAL as the authoritative local library, renamed to `omera.db`. The current application already relies on relational transactions for files, folders, tags, albums, stacks, and recovery. Renaming or replacing the engine alone does not make those workloads faster. Remote MySQL/PostgreSQL support is not implemented end to end and must not be advertised as available.

Keep original media on disk, large model weights outside the database, and thumbnails in a disposable bounded disk cache. Preserve existing IDs, relationships, metadata, embeddings, and schema version during identity migration. Do not rebuild a user's library from a filesystem scan as a substitute for migration.

## Workload-driven changes

| Workload | Proposed change | Evidence required |
| --- | --- | --- |
| Gallery pages | Small typed projections, stable keyset cursors with ID tie-breaks, indexes matching scope and ordering | EXPLAIN QUERY PLAN and first/deep-page p50/p95 latency |
| Metadata filters | Extract frequently queried fields into typed indexed columns or a one-to-one metadata table; keep full source JSON separately | Lower page/filter latency and IPC size without lost search semantics |
| Full-text search | Evaluate SQLite FTS5 alongside existing behavior | Multilingual tokenization, substring expectations, update cost, index size and correctness |
| Sidebar totals | Aggregate counts in bounded query groups and invalidate by mutation category | Startup query count and refresh latency, with accurate counts |
| Thumbnail bookkeeping | Reuse worker connections, coalesce access/budget writes, bound eviction batches | Decode throughput, lock wait and write amplification |
| Semantic search | Bounded top-k selection and bulk hydration first; evaluate optional vector indexes later | Exact-search baseline, recall@k, memory, latency and packaging cost |
| Background indexing | Short bounded write transactions and explicit busy handling; separate background reads | UI latency while scan/inference/cache jobs run together |

Do not add indexes for every column or introduce a second authoritative database without measurements. Separating cache metadata into another SQLite database may reduce contention, but also loses cross-database foreign-key guarantees and complicates recovery; retain the existing layout until contention measurements justify it. Optional vector indexes must be rebuildable derivatives of versioned authoritative embeddings.

## Schema and migration rules

- Applied migrations remain unchanged and ordered. Append new migrations in the storage crate; crate renaming moves the file without rewriting its history.
- Backfill large new structures in bounded resumable batches. Do not expose incomplete indexes as complete search results. Define schema compatibility and recovery before switching readers.
- Use foreign keys, uniqueness constraints, and explicit version/revision fields where required by actual ownership. Verify `integrity_check` and `foreign_key_check` on migrated copies.
- Store migration receipts outside the source database so failure never requires modifying the only good copy. Legacy discovery and cleanup follow [OMERA_MIGRATION.md](OMERA_MIGRATION.md).
- Keep authoritative databases on local storage; media may reside on network shares. A shared multi-user service is a separate future architecture, not a connection-string toggle.

## Benchmark and acceptance plan

Benchmark production query and cache functions on reproducible 1k, 10k, 50k and 100k fixtures; add 500k where resources permit. Include metadata-heavy rows, sparse fields, multiple tags/albums, stacked files, cold/warm caches, concurrent indexing, and deep pagination. Record hardware, commit, dataset generation, query plans, database/index bytes, peak memory, startup-to-first-page time, IPC bytes and p50/p95 latency.

Compare against the same baseline and publish results in `docs/benchmarks/`. A redesign is accepted only when correctness tests pass and measured target workloads improve without material startup, write, memory, or recovery regressions. Synthetic loops that do not exercise production functions are not evidence. No engine change or unmeasured speedup is promised by this plan.
