# Release checklist

geozero-cli and geozero-bench are dependent on geozero and flatgeobuf.
This implies the following update sequence for major version updates:

	geozero -> flatgeobuf -> geozero-cli/geozero-bench

## geozero

* Make sure GitHub CI status is green.
* Make sure local branch is up-to-date (`git pull --rebase`)
* `cd geozero`
* Check for compatible major updates with `cargo outdated`
* Bump version in `Cargo.toml`
* Set release date in `CHANGELOG.md`
* `git commit -a -m "Release geozero x.x.x"`
* `cargo publish` (possibly `cargo publish --no-verify`)
* `git tag -m v0.x.x v0.x.x`
* Bump to next minor version in `Cargo.toml` (without `-dev` postfix)
* `git commit -a -m "Bump version"`

Major updates:
* Release new version of flatgeobuf with updated geozero dependency
* Update geozero version in top-level Cargo.toml
