---
name: dotxpander-binary-optimizer
description: Optimizes Cargo.toml release profiles for minimal background memory footprint and binary size.
---
MANDATORY RULE: When modifying `Cargo.toml`, ensure the `[profile.release]` is optimized for a background utility. Enforce `lto = true`, `codegen-units = 1`, `opt-level = 'z'`, and `strip = true`. Favor lightweight crates over heavy dependencies to keep the idle memory footprint low.
