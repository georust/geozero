# About the release dependency cycle

(Written by Claude with input from @urschrei)

It explains why a breaking geozero release breaks the build of `geozero-cli` and `geozero-bench`, how the repository reached its current state, and what the options are.

The sources are the git history, the crates.io index, and the release tags of flatgeobuf and geoarrow-rs.
Statements about what does or does not compile were checked by building the code; where something is an inference, I say so.

## Summary

`geozero-cli` and `geozero-bench` are in the geozero workspace, and build against the local geozero.
They also use `flatgeobuf` and `geoarrow-array`, which depend on geozero from crates.io.

A `[patch.crates-io]` entry makes those crates use the local geozero as well, but only while the local version satisfies their version requirements.
A breaking geozero release always breaks that condition.
The downstream crates can't publish a compatible version until geozero has been published, so the workspace can't build in the interval between the two releases.

## The dependency cycle

```text
             crates.io                         this workspace
        +-------------------+            +---------------------------+
        | flatgeobuf        |--depends-->| geozero (library)         |
        | geoarrow-array    |    on      |                           |
        | geoarrow-schema   |            |                           |
        +-------------------+            +---------------------------+
                  ^                                  ^
                  |                                  |
              depends on                         depends on
                  |                                  |
        +---------------------------------------------------------+
        | geozero-cli (flatgeobuf, geoarrow-array, geoparquet)    |
        | geozero-bench (flatgeobuf)                              |
        +---------------------------------------------------------+
```

The `geozero` library crate depends on none of these crates.
The cycle runs through the two consumers in the workspace:

- `geozero-cli` uses `flatgeobuf` (`FgbReader`, `FgbWriter`, `HttpFgbReader`) and, since #247, `geoarrow-array`, `geoarrow-schema` and `geoparquet` for GeoParquet input.
- `geozero-bench` uses `flatgeobuf` (`FgbReader`, `HttpFgbReader`) in `benches/geobench.rs`.

## How the patch works, and when it stops working

The root `Cargo.toml` contains this entry:

```toml
[patch.crates-io]
geozero = { path = "./geozero" }
```

A [patch entry](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section) replaces a crates.io package with another source, but only for dependents whose version requirement the replacement satisfies.
For a 0.x crate, a requirement such as `0.14` means [`>=0.14.0, <0.15.0`](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#default-requirements), so every 0.x minor release is a breaking release as far as Cargo is concerned.

The downstream crates currently require:

| Crate | Version | Requires geozero |
|---|---|---|
| `flatgeobuf` | 5.0.0 (in use) | `^0.14.0` |
| `flatgeobuf` | 6.0.1 (latest) | `^0.15.1` |
| `geoarrow-array` | 0.8.0 to 0.10.0, and geoarrow-rs `main` | `^0.14` |

While the workspace version is 0.14.x, the patch applies to all of them, and the workspace contains a single geozero.

If the workspace version changes to one that a downstream crate doesn't accept, Cargo resolves that crate to geozero from crates.io instead.
The workspace then contains two geozero packages, whose traits and types are unrelated as far as the compiler is concerned.
Code that passes `flatgeobuf` or `geoarrow-array` values to the local geozero then fails to compile.

I observed the following effects:

- `geozero-cli` fails with errors such as ``the trait bound `FgbWriter<'_>: geozero::FeatureProcessor` is not satisfied`` and ``no method named `process` found for struct `GeozeroRecordBatchReader` ``.
- `cargo test -p geozero` and `cargo doc -p geozero` fail with ``specification `geozero` is ambiguous``, because two packages have that name.
- CI fails in the lint, test, docs and coverage jobs, all of which build the workspace.

The `geozero` library is not affected.
`cargo publish` from `geozero/` builds and verifies the crate on its own, and the crate depends on none of the downstream crates.

## How the repository reached its current state

| Date | Event |
|---|---|
| 2020-05 | `c5b7a49` adds the `[patch.crates-io]` entry for geozero, with the comment `# Patch flatgeobuf dependency (dev only)`. The cycle with `flatgeobuf` exists from this point. |
| 2021-02 to 2024-08 | The `geozero` crate itself depends on `flatgeobuf` (at least from `ec0e48b`, which merges geozero-core into geozero), until #235 removes that dependency. After #235, only the CLI and the bench are part of the cycle. |
| 2025-07-31 | `flatgeobuf` 5.0.0 is released, requiring geozero `^0.14.0`. |
| 2025-12-11 | geozero 0.15.0 and 0.15.1 are published from `dd76819` and `6889c5d`, which branch from `5f1a5b1` (#275) and were never merged into `main`. At `6889c5d` the workspace version is 0.15.1 but `flatgeobuf` is still 5.0.0, so `geozero-cli` does not compile at the release commit. |
| 2025-12-28 | `flatgeobuf` 6.0.0 and 6.0.1 are released, requiring geozero `^0.15.1`. From this date, `main` could have changed to 0.15.1 with `flatgeobuf` 6; that did not happen. |
| 2026-03-27 | geoarrow-rs 0.8.0 is released; `geoarrow-array` requires geozero `^0.14`. |
| 2026-07-19 | #247 adds GeoParquet input to `geozero-cli`, using `geoarrow-array` 0.8. The cycle now includes a second downstream project, and `main` can only build at 0.14.x. |
| 2026-11 | `main` is at 0.14.0, with many breaking changes under `UNRELEASED`. #387 merges the 0.15 release commits into `main` and corrects the changelog; the version stays at 0.14.0, with a comment explaining why. |

The reason for releasing 0.15 from a separate branch is not recorded.
The CLI failure at `6889c5d` is consistent with one explanation: merging into `main` would have broken CI.
That is an inference, not something the history confirms.

## Effect on the 0.16.0 release

0.16.0 can be published from `geozero/` at any time.
The problem is the workspace: once `main` says 0.16.0, the CLI and the bench won't build until *both* of the following are true:

- `flatgeobuf` has a release that requires geozero 0.16.
- `geoarrow-array` and `geoarrow-schema` have a release that requires geozero 0.16.

Neither project can release such a version before geozero 0.16.0 is on crates.io.

#367 adds a complication.
After #367, `geoarrow-array` also needs a one-line source change (`GeoWriter` to `GeoWriter<f64>`), and that change does not compile against a geozero without #367.

## Options

### Option 1: Patch the downstream crates for each breaking release

Patch `flatgeobuf`, `geoarrow-array` and `geoarrow-schema` from git commits that require the new geozero version, pinned by `rev`, and remove the patches once the downstream releases exist.
`geoarrow-schema` has to come from the same commit as `geoarrow-array`; otherwise `geoparquet` from crates.io sees two copies of `geoarrow-schema`.

A draft of this exists for 0.16.0.
With path patches to the same commits, it builds the full workspace and passes the local tests.

- Advantages: the CLI and the benches stay in CI throughout, and the downstream changes are prepared alongside the geozero change.
- Disadvantages: the patch commits have to be hosted somewhere, for example in forks. Breaking API changes such as #367 can require changes to the downstream commits, and a new pinned `rev`. The same work recurs at every breaking release, and `geozero-cli` still can't be published until the downstream releases exist.

### Option 2: Exclude the consumers during a breaking release

Add `geozero-cli` to `[workspace] exclude`, remove it from the CI package lists, and disable the `flatgeobuf` benchmarks until the downstream releases exist; then reverse these changes.

- Advantages: no forks or patches.
- Disadvantages: the CLI and the `flatgeobuf` benchmarks aren't built or tested in the meantime, and the length of that period depends on the release schedules of two other projects. The exclusion also has to be reversed by hand.

### Option 3: Build the consumers against published geozero

Put `geozero-cli` and the benchmarks that use `flatgeobuf` in a separate workspace, either in this repository or in a different one.
That workspace depends on the latest published geozero, has no `[patch]` entry for it, and updates to a new geozero version once `flatgeobuf` and geoarrow-rs have released compatible versions.

- Advantages: this removes the cycle. A breaking geozero release needs no coordination with other projects, and the workspace version can always match the version that is about to be published.
- Disadvantages: the CLI can't use geozero features before they are published, and CI needs a job for the second workspace. Benchmarks in that workspace measure the published geozero rather than the code on `main`; benchmarks that use only geozero can stay in the main workspace.

Across a breaking release, the CLI can't use unpublished geozero without Option 1 or Option 2 anyway, so the first disadvantage mainly affects non-breaking features between releases.

### Option 4: Remove the downstream formats from the CLI

Remove FlatGeobuf and GeoParquet support from `geozero-cli`, or move it into tools in the downstream projects.

- Advantages: this removes the cycle at its source.
- Disadvantages: users lose features, and moving them requires agreement with the downstream projects.

## Open questions

- Who uses `geozero-cli`, and does it need to be published at the same time as the library?
- Would the flatgeobuf and geoarrow-rs maintainers prefer to receive pull requests that update geozero before each geozero release, or to update after it?
- In my opinion, Option 1 for 0.16.0, followed by Option 3, could be the most practical combination: Option 1 keeps CI complete for the release that is waiting, and Option 3 avoids repeating the same work at the next breaking release. That is a suggestion for discussion, not a decision.

## Further reading

- [Release checklist](../RELEASING.md), which documents the update sequence `geozero -> flatgeobuf -> geozero-cli/geozero-bench`
- [Overriding dependencies: the `[patch]` section](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section) in the Cargo reference
- [Default version requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#default-requirements) in the Cargo reference
