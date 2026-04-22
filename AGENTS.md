# AGENTS.md

## Release Procedure

Follow the existing release history when preparing a new crate release.

1. Start from the latest `main`.

   ```shell
   git switch main
   git pull
   git switch -c release/vX.Y.Z
   ```

2. Update the crate version in:

   - `Cargo.toml`
   - `README.md`

3. Refresh `Cargo.lock`.

   ```shell
   cargo update
   ```

   Confirm that the `gh-config` package entry in `Cargo.lock` has the new
   release version.

4. Verify the release build.

   ```shell
   cargo check --locked --all-features
   git diff --check
   ```

5. Review the diff. A typical release diff includes:

   - package version update in `Cargo.toml`
   - dependency example update in `README.md`
   - lockfile update from `cargo update`

6. Commit using the existing release commit format.

   ```shell
   git add Cargo.toml Cargo.lock README.md
   git commit -m "chore: Release vX.Y.Z"
   ```

