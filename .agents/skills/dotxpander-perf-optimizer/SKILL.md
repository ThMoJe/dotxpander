---
name: dotxpander-perf-optimizer
description: Prevents dynamic memory allocations in the mission-critical KeyBuffer ring buffer.
---
MANDATORY RULE: The KeyBuffer ring buffer is on a mission-critical hot path (~1.04 ns latency). When modifying keystroke matching logic, you are STRICTLY PROHIBITED from introducing dynamic memory allocations (e.g., String::new(), .to_string(), format!()) or complex iterators. All hotkey evaluations must remain zero-allocation and run in constant time. For text injection, strictly use clipboard-based Win32 APIs (Ctrl+V via SendInput) instead of per-character injection.
