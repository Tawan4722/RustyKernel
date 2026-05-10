# Open Source License Recommendation

## Current State

This repo currently contains the **GNU AGPL-3.0** license text at the root (`LICENSE`).

## Recommended Default for This Project

For an OS-kernel research project intended to maximize reuse and contributions, I recommend:

- **Dual license: MIT OR Apache-2.0**

Why:

- Permissive and widely accepted in the Rust ecosystem
- Low friction for contributors and adopters
- Apache-2.0 adds explicit patent language; MIT keeps adoption simple
- Matches common Rust crate practice and downstream integration needs

## When to Keep AGPL Instead

Keep AGPL if your main goal is strong copyleft, especially if you want network-deployed derivatives to publish source changes.

## Practical Decision

- If your priority is ecosystem adoption and easier commercial/academic reuse: choose **MIT OR Apache-2.0**.
- If your priority is enforcing source-sharing obligations on derivatives: keep **AGPL-3.0**.
