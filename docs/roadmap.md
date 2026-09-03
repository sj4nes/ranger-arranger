# Roadmap

## SQLx / external client support

**Status:** blocked on VillageSQL server-side prepared-statement compatibility.

**What we want:** the ability to write Rust client programs using `sqlx` (or another async MySQL client) against a running VillageSQL server, so developers can use this extension from application code without dropping to the `mysql` CLI.

**What blocks it:** in the current VillageSQL build (`8.4.11-villagesql-0.0.7-dev-a5d49e67f99`), sqlx’s prepared-statement workflow fails with `Prepared statement needs to be re-prepared` (error `1615`). This is a server-side limitation, not an extension bug. The same failure occurs with `MySqlPool::connect` and any query that goes through sqlx’s prepare/execute path.

**What we’ll do when unblocked:**
1. Revisit sqlx compatibility once the VillageSQL server exposes stable prepared-statement support.
2. Convert `examples/booking_demo.rs` to sqlx once the blocker is resolved.

**Interim workaround:** `examples/booking_demo.rs` uses the `mysql` crate (version `28`) against a live server. The extension’s Rust unit tests, ABI tests, proptest suite, and fuzz harness already cover correctness without a live server.
