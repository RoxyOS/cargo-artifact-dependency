# cargo-artifact-dependency

Stable crate alternative for cargo artifact dependency.

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
