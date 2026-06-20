# macOS Support Plan

## Goal

Support both Windows and macOS while keeping the core app logic shared:

- Windows: current tray app behavior remains intact.
- macOS: menu-bar app with League Client LCU discovery, auto-accept, launch-at-login support, and release artifacts for Intel and Apple Silicon.

Supported targets:

- `x86_64-pc-windows-msvc`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

---

## Current Branch Status

Branch: `macos-support`

Last checked on: 2026-06-17

The platform split has started. Windows behavior now sits mostly behind `src/platform/windows`, and the shared application code calls `platform::startup` and `platform::lcu_auth`.

Completed:

- `main.rs` uses `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`.
- `main.rs` has an unsupported-platform compile guard for non-Windows and non-macOS builds.
- `src/platform/mod.rs` conditionally exposes `windows::{startup, lcu_auth}` and `macos::{startup, lcu_auth}`.
- Windows startup registry logic lives in `src/platform/windows/startup.rs`.
- Windows LCU auth discovery lives in `src/platform/windows/lcu_auth.rs`.
- Core modules use platform-neutral imports:
  - `app.rs` uses `crate::platform::startup`.
  - `acceptor.rs` uses `crate::platform::lcu_auth`.
  - `lcu.rs` depends on `crate::platform::lcu_auth::LcuAuth`.
- Windows-only crates are isolated to Windows target sections in `Cargo.toml`:
  - `winreg`
  - `powershell_script`
  - `winres`
- `build.rs` guards the `winres` import and Windows resource compilation with `#[cfg(windows)]`.
- Windows `cargo check` passes.
- `cargo fmt --all -- --check` passes.

Partially done:

- `src/platform/macos/mod.rs` exists but has no implementation.
- The platform API exists, but naming is still Windows-shaped in one place: `get_lcu_auth()` should become a platform-neutral discovery API.

Still Windows-specific:

- `tray.rs` loads only `assets/icon.ico`.
- `tray.rs` always labels startup as `Run on Startup`.
- GitHub Actions builds only Windows.

---

## Target Architecture

Current target layout:

```text
src/
  main.rs
  app.rs
  acceptor.rs
  lcu.rs
  tray.rs
  platform/
    mod.rs
    windows/
      mod.rs
      startup.rs
      lcu_auth.rs
    macos/
      mod.rs
      startup.rs
      lcu_auth.rs
    lcu_auth_parser.rs
```

Core app modules should call platform-neutral functions, not Windows-specific modules.

Example public platform API:

```rust
platform::startup::is_enabled()
platform::startup::enable()
platform::startup::disable()
platform::startup::cleanup_stale_entry()

platform::lcu_auth::discover()
```

Shared parsing should not live under either platform:

```rust
platform::lcu_auth_parser::parse_lockfile(...)
platform::lcu_auth_parser::parse_process_args(...)
```

---

## Platform-Agnostic Work Remaining

### Current Scope: Windows Release Checkpoint

Before filling in real macOS behavior, first finish the Windows-side platform boundary and make sure the Windows release path still works.

In scope for this checkpoint:

- Keep Windows behavior unchanged for users.
- Keep Windows startup logic behind `platform::startup`.
- Keep Windows LCU discovery behind `platform::lcu_auth`.
- Isolate Windows-only dependencies and build resources.
- Normalize LCU discovery so missing auth is explicit.
- Confirm GitHub Actions still builds/releases the Windows artifact correctly.
- Commit this as a stable Windows release-prep change before continuing with macOS implementation.

Out of scope for this checkpoint:

- macOS tray/menu-bar icon loading.
- macOS `Launch at Login` menu wording.
- macOS `startup.rs`.
- macOS `lcu_auth.rs`.
- macOS `.app` packaging.
- macOS CI/release artifacts.

Acceptance criteria:

- `cargo check` passes on Windows.
- `cargo fmt --all -- --check` passes.
- Shared code no longer calls a fake/default LCU auth value when discovery fails.
- GitHub Actions Windows build/release workflow is reviewed and still matches the current Windows artifact needs.

Recommended commit contents:

- `Cargo.toml`
- `build.rs`
- `src/acceptor.rs`
- `src/platform/windows/lcu_auth.rs`
- `docs/macos-support-plan.md`

### 1. Normalize the LCU auth API

Rename the public platform API from `get_lcu_auth()` to `discover()` or `discover_lcu_auth()`.

Recommended API:

```rust
pub fn discover() -> Option<LcuAuth>;
```

Keep a compatibility wrapper only if it makes the refactor smaller:

```rust
pub fn get_lcu_auth() -> LcuAuth {
    discover().unwrap_or_default()
}
```

Update `acceptor.rs` so failed discovery is explicit instead of represented as `https://127.0.0.1:0` with an empty token.

Acceptance criteria:

- Shared code no longer calls a Windows-flavored `get_lcu_auth()` function.
- Missing League Client auth is represented as `None` or `Result::Err`, not a fake `LcuAuth`.

### 2. Add shared LCU parsers

Add a shared parser module that both Windows and macOS can use.

Lockfile parser:

```text
process_name:pid:port:password:protocol
```

Process argument parser should extract:

```text
--app-port=...
--remoting-auth-token=...
```

Parser output:

```rust
LcuAuth {
    base_url: format!("{protocol}://127.0.0.1:{port}"),
    token: password.to_string(),
}
```

Acceptance criteria:

- Unit tests cover valid lockfile input.
- Unit tests cover malformed lockfile input.
- Unit tests cover process command lines with quoted and escaped token values.
- Windows LCU discovery uses the shared process-argument parser instead of owning separate regex matching.

### 3. Add Windows lockfile-first discovery

Improve Windows LCU discovery order:

1. Try known League lockfile locations.
2. Fall back to the existing PowerShell/CIM process command line query.
3. Parse the fallback command line with the shared process-argument parser.

Likely Windows lockfile locations:

```text
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
League install directory lockfile, if discoverable
```

Keep the current PowerShell behavior until lockfile discovery is validated.

Acceptance criteria:

- Existing Windows users keep the same behavior.
- If a lockfile is available, the app does not need PowerShell to discover auth.
- Fallback still works if the lockfile is missing.

### 4. Implement macOS LCU discovery

Add:

```text
src/platform/macos/lcu_auth.rs
```

Discovery order:

1. Try macOS League lockfile locations.
2. Fall back to `/bin/ps ax -o command=`.
3. Parse fallback command lines with the shared process-argument parser.

Likely macOS search paths:

```text
/Applications/League of Legends.app/Contents/LoL/lockfile
~/Applications/League of Legends.app/Contents/LoL/lockfile
```

Process fallback should search command lines containing League/Riot client arguments and the required auth keys:

```text
--app-port=
--remoting-auth-token=
```

Acceptance criteria:

- macOS builds have a real `platform::lcu_auth` module.
- Discovery works with lockfile content.
- Discovery has a process fallback for clients that expose the auth args but do not leave an expected lockfile path.

### 5. Implement macOS startup support

Add:

```text
src/platform/macos/startup.rs
```

Use a LaunchAgent plist at:

```text
~/Library/LaunchAgents/com.iholston.lol-accept.plist
```

Public API should match Windows:

```rust
pub fn is_enabled() -> io::Result<bool>;
pub fn enable() -> io::Result<()>;
pub fn disable() -> io::Result<()>;
```

The plist should launch the current executable:

```xml
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.iholston.lol-accept</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Applications/LoL-Accept.app/Contents/MacOS/lol-accept</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
```

Stale entry behavior should mirror Windows:

- If the stored executable path differs from `std::env::current_exe()`, remove the stale plist.
- If startup was enabled with a stale path, rewrite it with the current path.

Acceptance criteria:

- `platform::startup` compiles and works on macOS.
- Unit tests cover plist generation and stale-path detection without writing to real `~/Library/LaunchAgents`.

### 6. Make tray assets and labels platform-aware

Keep the `tao` + `tray-icon` approach unless it proves incompatible on macOS.

Needed changes:

- Windows runtime tray icon: `assets/icon.ico`.
- macOS runtime tray icon: `assets/icon.png`.
- macOS bundle icon: `assets/icon.icns`.
- Windows menu label: `Run on Startup`.
- macOS menu label: `Launch at Login`.

Acceptance criteria:

- Windows still loads the `.ico`.
- macOS can load a PNG runtime tray/menu-bar icon.
- Startup menu text matches the platform.

### 7. Add macOS package output

Create a `.app` bundle:

```text
LoL-Accept.app/
  Contents/
    Info.plist
    MacOS/lol-accept
    Resources/icon.icns
```

Important `Info.plist` keys:

```xml
<key>CFBundleExecutable</key>
<string>lol-accept</string>
<key>CFBundleIdentifier</key>
<string>com.iholston.lol-accept</string>
<key>CFBundleName</key>
<string>LoL-Accept</string>
<key>CFBundleIconFile</key>
<string>icon</string>
<key>LSUIElement</key>
<true/>
```

Add:

```text
scripts/package-macos.sh
```

Responsibilities:

1. Build `.app` directory.
2. Copy release binary.
3. Copy `icon.icns`.
4. Generate `Info.plist`.
5. Zip the app.

Acceptance criteria:

- A local macOS release build creates a launchable `.app`.
- App runs as a menu-bar/background app without a Dock icon.

### 8. Update CI and release artifacts

Split release workflow into:

```text
quality
build-windows
build-macos
release
```

Quality job should run on Windows and macOS:

```yaml
- run: cargo fmt --all -- --check
- run: cargo check
- run: cargo clippy --all-targets --all-features -- -D warnings
- run: cargo test
```

Artifacts:

```text
lol-accept-windows-x86_64.zip
lol-accept-macos-x86_64.zip
lol-accept-macos-aarch64.zip
SHA256SUMS.txt
```

Acceptance criteria:

- PRs validate Windows and macOS compilation.
- Tags publish Windows and both macOS architecture artifacts.

### Done: Dependency isolation

Move Windows-only crates out of global dependency sections:

```toml
[dependencies]
reqwest = { version = "0.12.5", default-features = false, features = ["rustls-tls", "blocking"] }
image = { version = "0.25.2", default-features = false, features = ["ico", "png"] }
tray-icon = "0.24.0"
tao = "0.35.2"
regex = "1.10.6"

[target.'cfg(target_os = "windows")'.dependencies]
powershell_script = "1.1.0"
winreg = "0.52.0"

[target.'cfg(target_os = "macos")'.dependencies]
plist = "1"
dirs = "5"

[target.'cfg(target_os = "windows")'.build-dependencies]
winres = "0.1.12"
```

`build.rs` should keep the Windows-only import guarded:

```rust
#[cfg(windows)]
use winres::WindowsResource;
```

Acceptance criteria:

- `cargo check` still passes on Windows.
- A macOS target does not attempt to resolve or compile `winreg`, `powershell_script`, or `winres`.

---

## Immediate Next PRs

### PR 1: Finish the platform boundary

- Move Windows-only dependencies to target-specific sections.
- Guard `winres` import in `build.rs`.
- Rename LCU discovery API to `discover()` or `discover_lcu_auth()`.
- Add `src/platform/macos/startup.rs` and `src/platform/macos/lcu_auth.rs` stubs that compile on macOS.
- Keep Windows runtime behavior unchanged.

### PR 2: Shared LCU parser

- Add shared lockfile parser.
- Add shared process-argument parser.
- Add parser tests.
- Change Windows fallback to reuse shared parser.

### PR 3: macOS discovery and startup

- Implement macOS lockfile discovery.
- Implement macOS `/bin/ps` fallback.
- Implement LaunchAgent startup support.
- Add tests for plist generation and stale-entry handling.

### PR 4: Assets, packaging, and CI

- Add PNG and ICNS assets.
- Make tray icon loading platform-aware.
- Add macOS `.app` packaging script.
- Add CI matrix and release artifacts.

---

## Historical Notes

The following sections capture the original implementation notes. Some items are already complete on `macos-support`; use the status and immediate PR sections above as the source of truth.

### Conditional Windows subsystem

Change:

```rust
#![windows_subsystem = "windows"]
```

to:

```rust
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
```

### Startup abstraction

Move `reg.rs` behind a platform startup API.

Windows implementation:

- Keep using `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- Preserve stale-entry cleanup behavior.

macOS implementation:

- Use a LaunchAgent plist at:

```text
~/Library/LaunchAgents/com.iholston.lol-accept.plist
```

- Write/remove plist for launch-at-login.
- Compare stored executable path to `std::env::current_exe()` for stale cleanup.

Minimal plist shape:

```xml
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.iholston.lol-accept</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Applications/LoL-Accept.app/Contents/MacOS/lol-accept</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
```

### LCU auth discovery abstraction

Rename `cmd.rs` conceptually to LCU auth discovery.

Recommended model:

```rust
pub struct LcuAuth {
    pub base_url: String,
    pub token: String,
}
```

Recommended API:

```rust
pub fn discover_lcu_auth() -> Result<LcuAuth, LcuAuthError>;
```

Discovery strategy:

1. Try League lockfile.
2. Fall back to process argument parsing.

Lockfile parser should be shared. Typical contents:

```text
process_name:pid:port:password:protocol
```

Parser output:

```rust
LcuAuth {
    base_url: format!("{protocol}://127.0.0.1:{port}"),
    token: password.to_string(),
}
```

Windows fallback:

- Keep PowerShell/CIM initially.
- Reuse the shared process-argument parser.

macOS fallback:

- Use `/bin/ps ax -o command=`.
- Search for League client args:
  - `--app-port=...`
  - `--remoting-auth-token=...`

### Tray/menu changes

The existing `tao` + `tray-icon` approach can likely stay.

Needed updates:

- Replace `crate::reg` usage with `platform::startup`.
- Load platform-specific icon assets.
- Consider platform label:
  - Windows: `Run on Startup`
  - macOS: `Launch at Login`

### Assets

Add:

```text
assets/icon.png
assets/icon.icns
```

Use:

- `.ico` for Windows executable resource and tray icon.
- `.png` for macOS runtime tray icon, if needed.
- `.icns` for macOS `.app` bundle icon.

Update `image` features if loading PNG at runtime:

```toml
image = { version = "0.25.2", default-features = false, features = ["ico", "png"] }
```

### Dependencies

Move platform-specific dependencies into target sections:

```toml
[dependencies]
reqwest = { version = "0.12.5", default-features = false, features = ["rustls-tls", "blocking"] }
image = { version = "0.25.2", default-features = false, features = ["ico", "png"] }
tray-icon = "0.24.0"
tao = "0.35.2"
regex = "1.10.6"

[target.'cfg(target_os = "windows")'.dependencies]
powershell_script = "1.1.0"
winreg = "0.52.0"

[target.'cfg(target_os = "macos")'.dependencies]
plist = "1"
```

### Build script

Make `winres` import conditional if moving it to target-specific build dependencies:

```rust
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()?;
    }

    Ok(())
}
```

---

## macOS Packaging Notes

Create a `.app` bundle:

```text
LoL-Accept.app/
  Contents/
    Info.plist
    MacOS/lol-accept
    Resources/icon.icns
```

Important `Info.plist` keys:

```xml
<key>CFBundleExecutable</key>
<string>lol-accept</string>
<key>CFBundleIdentifier</key>
<string>com.iholston.lol-accept</string>
<key>CFBundleName</key>
<string>LoL-Accept</string>
<key>CFBundleIconFile</key>
<string>icon</string>
<key>LSUIElement</key>
<true/>
```

`LSUIElement=true` makes it a menu-bar/background app without a Dock icon.

Add script:

```text
scripts/package-macos.sh
```

Responsibilities:

1. Build `.app` directory.
2. Copy release binary.
3. Copy `icon.icns`.
4. Generate `Info.plist`.
5. Zip the app.

---

## GitHub Actions Notes

Split release workflow into:

```text
quality
build-windows
build-macos
release
```

### Quality job

Run on Windows and macOS:

```yaml
- run: cargo fmt --all -- --check
- run: cargo check
- run: cargo clippy --all-targets --all-features -- -D warnings
- run: cargo test
```

### Windows artifact

```text
lol-accept-windows-x86_64.zip
```

### macOS artifacts

Build matrix:

```text
x86_64-apple-darwin
aarch64-apple-darwin
```

Artifacts:

```text
lol-accept-macos-x86_64.zip
lol-accept-macos-aarch64.zip
```

### Release job

Download all artifacts and publish them to the GitHub release.

Also generate checksums:

```text
SHA256SUMS.txt
```

---

## Optional Release Hardening

Later, add:

- Apple Developer ID signing.
- Apple notarization.
- Stapling notarization ticket.
- Windows code signing.
- Dependency audit tooling, such as `cargo audit` or `cargo deny`.

Unsigned macOS apps will trigger Gatekeeper warnings, so signing/notarization is recommended before broad distribution.

---

## Testing Plan

Add unit tests for:

- lockfile parser
- process argument parser
- gameflow phase parser
- LaunchAgent plist generation
- startup stale-entry detection logic

Avoid writing to real registry or real `~/Library/LaunchAgents` in unit tests. Use temp paths or injectable path providers.

---

## Original Phased Roadmap

### Phase 1: Cross-platform foundation

- Add conditional Windows subsystem attribute.
- Add `platform` module.
- Move Windows startup logic behind platform API.
- Move Windows dependencies to target-specific sections.
- Add `cargo fmt --check` to CI.
- Preserve Windows behavior.

### Phase 2: LCU auth refactor

- Move LCU auth discovery out of `cmd.rs` conceptually.
- Add shared lockfile parser.
- Add shared process-argument parser.
- Change auth discovery to return `Result` or `Option`.
- Add parser tests.

### Phase 3: macOS LCU discovery

- Add macOS lockfile search.
- Add `/bin/ps` fallback.
- Validate against running macOS League client.

### Phase 4: macOS startup support

- Add LaunchAgent plist write/remove/read.
- Connect menu toggle to macOS startup API.
- Add stale plist cleanup.

### Phase 5: macOS tray assets and bundle

- Add `.png` and `.icns` assets.
- Package `.app` with `LSUIElement=true`.
- Verify menu-bar behavior.

### Phase 6: CI/release matrix

- Add Windows + macOS quality jobs.
- Add Windows artifact.
- Add macOS Intel and Apple Silicon artifacts.
- Publish all artifacts and checksums on tag release.

### Phase 7: Signing and notarization

- Add Apple signing/notarization when ready.
- Optionally add Windows signing.

---

## Recommended First PR

Keep the first PR small and low risk:

1. Add `platform` module.
2. Move Windows startup behind platform API.
3. Make `main.rs` platform-safe.
4. Move Windows-only dependencies to target sections.
5. Add CI format check.
6. Keep Windows behavior unchanged.
