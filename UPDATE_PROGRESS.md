# ZeWIF-Zingo Progress Notes

## Workspace context
- `zewif` defines the Zcash wallet interchange format atop `bc-envelope` and `dcbor` (see `zewif/src/lib.rs` for the format surface built on those primitives).
- `zewif-zcashd` consumes `zewif` to understand `zcashd` wallet internals, pinning Electric Coin Company crates (`orchard 0.10`, `zcash_primitives 0.19`, etc.).
- `zmigrate` is the CLI that glues the format adapters together; fixtures live under `zmigrate/tests/fixtures` with helper scripts such as `zmigrate/run_zcashd_dumps.sh`.
- `zewif-zingo` is meant to parallel the `zcashd` adapter using Zingolabs’ stack (`zingolib`, `zcash_client_backend`, etc.), but it has been idle and is now out of sync with its upstream dependencies.

## Build attempt
- Command: `cargo check -p zewif-zingo`
- Result: fails while compiling `incrementalmerkletree v0.7.1` before reaching any `zewif-zingo` sources.
- Key error (first blocker):
  ```
  error[E0277]: the trait bound `R: proptest::prelude::Rng` is not satisfied
     --> incrementalmerkletree-0.7.1/src/frontier.rs:719:22
      |
  719 |         fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TestNode {
      |                      ^^^ the trait `DerefMut` is not implemented for `R`
  ```
  The compiler points out that the trait bound mirrors `proptest::prelude::Rng`, not `rand::Rng`, and the same span also triggers `E0277` complaining about `R: Sized`.

## Root cause analysis
- `zingolib` forces `incrementalmerkletree` features `test-dependencies` and `legacy-api` (see `zingolib/Cargo.toml`). The `test-dependencies` feature switches on optional deps `proptest` and `rand` inside `incrementalmerkletree`.
- Cargo has resolved `proptest` to `1.7.0` (confirmed in the workspace `Cargo.lock`). This release upgraded to the `rand 0.9` ecosystem and re-exports its own `Rng` trait in the prelude. When `incrementalmerkletree` pulls `proptest::prelude::*`, that re-export shadows the intended `rand::Rng` import within `frontier.rs`.
- `incrementalmerkletree 0.7.1` itself still depends on `rand 0.8`. Because the feature mix now sees both `rand 0.8` (direct) and `rand 0.9` (via `proptest 1.7`), the `Distribution<TestNode>` impl ends up type-checking against the wrong `Rng` trait, producing the unsatisfied bound errors above.
- This means the breakage is in a transitive dependency rather than `zewif-zingo`’s own code. The crate cannot currently be checked or built without first reconciling the dependency versions or feature flags in the Zingolib stack.

## Related observations
- `zingolib` is pinned to commit `965e8122…` which still expects the older `incrementalmerkletree` API. Newer `orchard` releases (>=0.11) moved to `incrementalmerkletree 0.8`, but this Zingolib rev keeps `orchard 0.10`, so simply bumping the tree crate likely requires a coordinated upstream update.
- `zmigrate/run_zingo_dumps.sh` is the historical harness for the fixtures under `zmigrate/tests/fixtures/zingo`. Unlike the currently maintained `run_zcashd_dumps.sh`, it lacks hardening (no `set -euo pipefail`, no target directory creation, no path sanity checks) and assumes a `cargo run -- zingo <input>` CLI that now fails to build. Once the compilation issues are solved, this script will need the same modernization the `zcashd` dump script received.

## Options to move forward
1. Short-term unblock: patch the workspace to use a fork of `incrementalmerkletree 0.7.x` that either pins `proptest` to `<1.7` or disambiguates the `Rng` import. This is the minimal change to get `cargo check` running with the current Zingolib revision.
2. Medium-term: drop the `test-dependencies` feature when pulling `incrementalmerkletree` through Zingolib if the test-only APIs are not needed for migration. That would avoid the problematic optional deps entirely.
3. Longer-term alignment: update Zingolib (and its pinned `librustzcash` components) to versions that already ride on `incrementalmerkletree 0.8`/`orchard 0.11`, then adapt `zewif-zingo` to any API changes. This should eliminate the `proptest`/`rand` mismatch but will require more invasive changes across the adapter layer.

Until one of these dependency strategies is adopted, `zewif-zingo` stays unbuildable, and downstream tools such as `zmigrate` cannot exercise the Zingo migration path or refresh the fixture dumps.

## Upstream Zingolib status (Feb 2025)
- Checked the current `main` branch (`660938b684f89aef91a091f4e0594d26e4328a48`) of `zingolib` by cloning `https://github.com/zingolabs/zingolib` into `target/tmp/zingolib-main`.
- Workspace manifest now targets `incrementalmerkletree 0.8.2`, `orchard 0.11.0`, `sapling-crypto 0.5.0`, and `shardtree 0.6.1` (`target/tmp/zingolib-main/Cargo.toml:16`). These align with the modern ECC stacks and eliminate the `rand`/`proptest` mismatch we hit with `0.7.x`.
- The same manifest downgrades `proptest` usage to `1.6.0` and keeps `rand 0.8`, avoiding the `rand 0.9` transitive pull that triggered our current build failure.
- `zingolib`’s crate manifest no longer enables the `incrementalmerkletree` `test-dependencies` feature (`target/tmp/zingolib-main/zingolib/Cargo.toml:29`), so even if `proptest` is present elsewhere, that feature set is not forced into library consumers.
- The library moved to Rust 2024 edition and introduced new workspace crates such as `pepper-sync`, `zingo-price`, and `zcash_transparent` while dropping the old `zingo-sync` dependency (`target/tmp/zingolib-main/zingolib/Cargo.toml:1-50`). Any upgrade will need to account for renamed modules and relocated wallet code.
- Zingo’s upstream `zcash_*` crates now point at ECC’s `librustzcash` repo rev `d387aed7e04e881dbe30c6ff8b26a96c834c094b` rather than the older Zingolabs fork (`target/tmp/zingolib-main/Cargo.toml:20-40`), so a dependency refresh here will cascade through the serialization code we rely on.

### Implication for ZeWIF integration
- To resolve our compiler errors without local patching, we should plan to bump `zewif-zingo` to the newer `zingolib` (and transitive crates) stack, then reconcile API shifts caused by the `zingolib` reorg. This gives us `incrementalmerkletree 0.8.x` plus a consistent `rand 0.8` universe.
- If we cannot move the whole dependency chain yet, selectively backporting the upstream `Cargo.toml` changes (drop `test-dependencies`, pin `proptest` to 1.6.0) into our forked commit may be an interim path, but that drifts from upstream and should be documented.
