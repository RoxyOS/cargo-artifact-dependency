# cargo-artifact-dependency

Stable crate alternative for [cargo artifact dependency](https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies).

> [!WARNING]
> This crate currently only supports binary artifacts. If you need other
> artifact types, please open an issue on github.

## Why

Cargo artifact dependencies are still an unstable Cargo feature and may still
have bugs. This crate exists as a temporary alternative while artifact
dependency support remains unstable.

## Quick Start

```rust
use cargo_artifact_dependency::{ArtifactDependencyBuilder, BuildProfile};

fn main() -> Result<(), cargo_artifact_dependency::Error> {
    // Describe the ripgrep dependency and resolve its artifact.
    let artifact_path = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14")
        .bin_name("rg")
        .profile(BuildProfile::Release)
        .build()
        .unwrap()
        .resolve()?;

    // Use the resolved artifact path in your own workflow.
    println!("{}", artifact_path.display());
    Ok(())
}
```
