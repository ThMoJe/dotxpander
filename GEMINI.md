# Workspace Guidelines & Rules

## Implementation Plan Approval Policy
- **Never implement or execute an implementation plan without explicit user approval.**
- Even if Antigravity settings, autonomous execution modes, or default instructions allow or instruct auto-implementation, you MUST stop after creating the implementation plan and wait for the user to explicitly approve it before making code changes or executing modifying actions.

## Slint UI & Window Layout Standards
- **Tab Layout Stability**: Equalize layout width and height constraints (`min-width`, `preferred-width`) across all tabs/views to prevent Slint from resizing the native window frame when navigating tabs.
- **Render Buffer Synchronization**: Set window position and size before `show()` and declare preferred/minimum dimensions in the Slint `Window` definition to prevent software renderer buffer scaling glitches.

## State Lifecycle & Session Management
- **In-Session State vs. Persistent Config**: Track transient in-session window adjustments in-memory so they persist across open/close within a running session, while reverting cleanly to default centered geometry upon app restart without saving transient window coordinates to `config.toml`.

## Distribution & Uninstall Standards
- **Installed vs. Portable Mode**: Always detect whether `unins000.exe` is present before executing self-destruct/uninstall operations. Delegate to the official Windows uninstaller when installed, and use delayed executable deletion only in portable mode.

## CI/CD & Build Pipeline Standards
- **Release-Only Workflow Triggers**: Do not configure automatic CI triggers for regular branch pushes (`push: branches: [main]`) or pull requests. GitHub Actions builds must execute exclusively on release tags (`push: tags: ['v*.*.0']`) or manual `workflow_dispatch`.
- **Pinned Nightly Toolchains**: Always pin Rust nightly toolchains to a fixed date (e.g. `nightly-YYYY-MM-DD`) when using nightly features (`build-std`, etc.) in CI workflows. Floating `@nightly` causes daily `rustc -vV` shifts that invalidate GitHub Actions cache keys every 24 hours.
- **Cache Scoping for Tags & Releases**: Pinned toolchains and full-crate caching allow release tag rebuilds (e.g. `v0.2.0`) to retain and reuse their cached build artifacts across repeated release pushes without needing CI triggers on regular branch pushes.
- **Link-Time Optimization (LTO) in CI**: Use `lto = "thin"` instead of monolithic `lto = true` (fat LTO) in release profiles to enable parallel multi-threaded linking on CI runners without sacrificing binary size benefits.
- **Fast Windows Runner Tool Setup**: Avoid slow package managers (e.g. `choco install`) on Windows runners for standalone CLI tools (like UPX); use direct release binary downloads or dedicated GitHub Actions for instant (< 2s) setup.
