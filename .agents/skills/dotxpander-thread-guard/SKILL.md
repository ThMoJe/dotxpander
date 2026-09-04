---
name: dotxpander-thread-guard
description: Enforces strict thread separation between the Slint UI and Win32 hook message loop.
---
MANDATORY RULE: You must maintain strict separation between the Slint UI thread and the Win32 hook message loop thread. NEVER lock a std::sync::Mutex inside the low-level keyboard hook callback. Always use arc-swap for wait-free reads of the AppConfig. When passing state from the Win32 background thread to the UI, you MUST use slint::invoke_from_event_loop.
