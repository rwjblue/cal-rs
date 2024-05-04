# Release Process

1. Update `CHANGELOG.md` with the new version number and release date.
2. Update `cargo.toml` with the new version number.
3. Run `cargo build` (or any other command that will update `Cargo.lock`).
4. Commit the changes (e.g. `git commit -m "Release vX.Y.Z")
5. Tag the commit with the version number (e.g. `git tag vX.Y.Z`)
6. Push the commit and tag to the repository (e.g. `git push origin main --tags`)
7. Update the release notes on GitHub with the changes from `CHANGELOG.md`
8. Update `rwjblue/homebrew-tap` to use the updated version and assets.
  - Update the version in `Formula/cal.rb`
  - Download the new release assets from GitHub, and calculate the `sha256` for each (`shasum -a 256 path/to/download`).
  - Update the `url` and `sha256` for each variant the new version in `Formula/cal.rb`
