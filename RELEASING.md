# Release checklist

With flatgeobuf as test requirement we have a circular dependency on geozero,
which causes build conflicts when doing major version updates.

## geozero

* `cd geozero`
* Make sure local branch is up-to-date (`git pull --rebase`)
* Check for compatible major updates with `cargo outdated`
* `cargo test --all-features`
* `export DATABASE_URL="postgresql://$USER@localhost/postgistest?sslmode=disable"` (see `tests/data/postgis.sql`)
* `cargo test --all-features -- --ignored postgis --test-threads 1`
* Bump version in `Cargo.toml`
* `cargo publish` (possibly `cargo publish --no-verify`)
* Set release date in `CHANGELOG.md`
* `git commit -a -m "Release geozero x.x.x"`
* `git tag -m v0.x.x v0.x.x`
* Bump to next minor version in `Cargo.toml` (without `-dev` postfix)
* `git commit -a -m "Bump version"`

Major updates:
* Create branch of flatgeobuf with updated geozero dependency
* Change flatgeobuf to git version until flatgeobuf crate is released
