# Release checklist

With flatgeobuf as test requirement we have a circular dependency on geozero,
which causes build conflicts when doing major version updates.

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
* Create branch of flatgeobuf with updated geozero dependency
* Change flatgeobuf to git version until flatgeobuf crate is released
