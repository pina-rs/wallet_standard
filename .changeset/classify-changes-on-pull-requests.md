---
wallet_standard: none
wallet_standard_browser: none
---

# Classify pull request changes before they reach a release

Add a `changeset-policy` workflow that runs two gates on every pull request.

The changeset policy fails a pull request when changed release-owned paths are not covered by a changeset and posts one remediation comment. It now derives the changed packages from git history with `from: origin/main`, so it also compares each attached changeset bump against the classified change type.

The change-classification step posts the compatibility evidence for the pull request: public API, dependency, and metadata changes with a proposed bump per package. The report separates the current pull request from everything accumulated since the last release, so a pull request that adds an API is not mistaken for one that breaks it.

Classification runs at `--detection-level semantic` against a cargo-semver-checks matrix covering the default feature set and all features, so a break behind `solana` or `browser` is proven rather than inferred from syntax.
