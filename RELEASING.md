## Releasing

### Bump version number

 - Fill the `CHANGELOG.md`
 - Edit `Cargo.toml`
 - Edit `src/commands/mod.rs`
 - Edit `Cargo.lock`

### Commit the changes

 - `git commit -am "Release <version number>"`
 - `git tag <version number>`

### Publish to crates.io

 - `cargo publish`

### Push changes on github

 -  `git push`
 -  `git push origin <version number>`

