---
name: dotxpander-unsafe-guard
description: Enforces strict safety boundaries and HRESULT checking for windows-rs API calls.
---
MANDATORY RULE: All Win32 API calls via `windows-rs` inside `unsafe` blocks must be strictly bounded. You must explicitly check and handle all `HRESULT` or Win32 error codes. Do not use `.unwrap()` on FFI boundaries. Ensure any state accessed inside the low-level keyboard hook callback is panic-safe.
