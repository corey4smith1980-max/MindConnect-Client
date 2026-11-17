# BetterDesk Client

<div align="center">

<img src="res/betterdesk.png" alt="BetterDesk" width="280">

<br><br>

![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)
![Flutter](https://img.shields.io/badge/Flutter-desktop-02569B.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)

[![Sponsor](https://img.shields.io/badge/GitHub-Sponsor-181717?logo=github&logoColor=white&style=flat)](https://github.com/sponsors/UNITRONIX)
[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white&style=flat)](https://discord.gg/MPp9hyyG97)

**Official UNITRONIX desktop remote client** — a fork of [RustDesk](https://github.com/rustdesk/rustdesk), licensed under **AGPL-3.0**.

[Build](docs/BUILD_DESKTOP.md) · [Official client](docs/OFFICIAL_CLIENT.md) · [Connectivity](docs/CONNECTIVITY.md) · [Client API](docs/CLIENT_API.md) · [Upstream](https://github.com/rustdesk/rustdesk)

</div>

---

> [!Caution]
> **Misuse Disclaimer:** The developers of this software do not condone or support any unethical or illegal use. Misuse, such as unauthorized access, control or invasion of privacy, is strictly against our guidelines. The authors are not responsible for any misuse of the application.

This repository is the **official BetterDesk desktop client** ([UNITRONIX/BetterDesk-Client](https://github.com/UNITRONIX/BetterDesk-Client)). Active work targets **Flutter desktop on Windows and Linux**. Mobile and web trees remain in-repo but are not the current focus.

> **AI-assisted development:** BetterDesk Client is created and maintained with the assistance of artificial intelligence tools.

The client **does not** connect to public `*.rustdesk.com` infrastructure. A normal build of this fork already contains the MindConnect BetterDesk server, so a clean install registers and shows an ID without a `custom.txt` and without anyone typing servers into Settings. See [Built-in server](#built-in-server).

Pair it with the self-hosted BetterDesk server and web console: [UNITRONIX/BetterDesk](https://github.com/UNITRONIX/BetterDesk).

---

## Product SKUs

| SKU | Role |
|-----|------|
| **BetterDesk Client** | Full desktop client (outbound + inbound) |
| **BetterDesk Support Agent** | Incoming-only agent (Generator bake-in; no outbound remote control) |

Both identify to the API as `betterdesk-desktop` with a distinct `product_sku`. Details: [docs/OFFICIAL_CLIENT.md](docs/OFFICIAL_CLIENT.md).

---

## Documentation

| Topic | Doc |
|-------|-----|
| Official client behaviour (no public cloud, branding, Generator, Support Agent) | [docs/OFFICIAL_CLIENT.md](docs/OFFICIAL_CLIENT.md) |
| Desktop build (Windows / Linux Flutter) | [docs/BUILD_DESKTOP.md](docs/BUILD_DESKTOP.md) |
| How connections work (P2P / LAN / relay / API) | [docs/CONNECTIVITY.md](docs/CONNECTIVITY.md) |
| Client API layers (HTTP / FFI / IPC) | [docs/CLIENT_API.md](docs/CLIENT_API.md) |
| BetterDesk server wiki | [BetterDesk Wiki](https://github.com/UNITRONIX/BetterDesk/wiki) |

---

## Built-in server

One file is the source for Windows, Linux, macOS, Android, and iOS. Every full-client build compiles it into the shared Rust library.

| Setting | Value |
|---|---|
| File | [`res/mindconnect/server.json`](res/mindconnect/server.json) |
| ID / rendezvous | `104.207.90.103` (hbbs port 21116) |
| Relay | `104.207.90.103` (hbbr port 21117) |
| API | `http://104.207.90.103:21121` |
| Public key | `ddU+/IrwdxX+Y/CGwJK7Ap3/lGNvYrLMTXcizekHlWo=` |

The server's device enrollment mode is open, so a new install is accepted when it registers.

To point a build at a different server, either edit that file or set these environment variables before compiling. A non-empty variable replaces the matching field. An empty variable leaves the file value.

| Variable | Field |
|---|---|
| `MINDCONNECT_ID_SERVER` | `id_server` |
| `MINDCONNECT_RELAY_SERVER` | `relay_server` |
| `MINDCONNECT_API_SERVER` | `api_server` |
| `MINDCONNECT_KEY` | `key` |

On GitHub Actions the same names are repository **variables** (Settings → Secrets and variables → Actions → Variables). They are passed into every platform job, including the Linux build container. Leave a variable unset to keep the committed value. The build rejects a host on `rustdesk.com`.

The values are applied as RustDesk `override-settings`, with `hide-server-settings` set to `Y`. Settings → Network hides the ID/Relay Server fields on desktop and on mobile. A saved config, a deploy-string import, or a server heartbeat cannot replace the ID server, relay, API server, or key. Proxy and WebSocket settings stay editable. `RENDEZVOUS_SERVERS` and `RS_PUB_KEY` stay empty, so there is no public RustDesk fallback.

`BETTERDESK_SUPPORT_CONFIG_B64` and `BETTERDESK_GENERIC_SUPPORT=1` keep their own Support Agent behavior and do not use this file.

## Build

Desktop tool versions and local Windows/Linux commands: **[docs/BUILD_DESKTOP.md](docs/BUILD_DESKTOP.md)**.

```sh
git submodule update --init --recursive
# optional: export MINDCONNECT_ID_SERVER=... MINDCONNECT_RELAY_SERVER=... MINDCONNECT_API_SERVER=... MINDCONNECT_KEY=...
```

CI workflow [`.github/workflows/mindconnect-client-build.yml`](.github/workflows/mindconnect-client-build.yml) runs on pull requests and **workflow_dispatch**. It calls [`.github/workflows/flutter-build.yml`](.github/workflows/flutter-build.yml). A push to `master` (and `desktop/*` tags) also builds through [`.github/workflows/betterdesk-desktop-release.yml`](.github/workflows/betterdesk-desktop-release.yml) and publishes a GitHub Release.

| Platform | CI artifact | Install |
|---|---|---|
| Windows x86_64 and ARM64 | portable EXE and MSI | Run the EXE or install the MSI |
| Linux x86_64 and ARM64 | `.deb` and `.rpm` | Install the package |
| macOS x86_64 and ARM64 | `.dmg` | Open the DMG |
| Android arm64, armv7, x86_64 | `mindconnect-*-android-*.apk` | Sideload the APK |
| iOS arm64 | `mindconnect-*-ios-unsigned.ipa` | Not installable on a device until signed |

Android signing: set repository secrets `ANDROID_SIGNING_KEY` (base64 keystore), `ANDROID_ALIAS`, `ANDROID_KEY_STORE_PASSWORD`, and `ANDROID_KEY_PASSWORD` to upload a release-signed APK. Without those secrets the workflow signs the APK with the Android debug keystore. That APK sideloads; it is not a Play Store signature. The 32-bit x86 APK is not built in CI; the stock Flutter SDK has no matching x86 engine.

iOS: the job runs `flutter build ipa --release --no-codesign` and does not fail the rest of the workflow if that build fails. The Xcode project is still upstream team `HZF9JMC8YN` and bundle id `com.carriez.flutterHbb`. This repository has no Apple distribution certificate or provisioning profile, so CI cannot produce an IPA that installs on an iPhone. Sign the archive with an Apple Developer team, or re-sign the unsigned IPA, before installing it on hardware.

Local Android (arm64), after the NDK and vcpkg setup in `flutter/build_android_deps.sh`:

```sh
export ANDROID_NDK_HOME=/path/to/ndk
export VCPKG_ROOT=/path/to/vcpkg
bash flutter/build_android_deps.sh arm64-v8a
bash flutter/ndk_arm64.sh
# copy target/aarch64-linux-android/release/liblibrustdesk.so and libc++_shared.so
# into flutter/android/app/src/main/jniLibs/arm64-v8a/, then:
cd flutter && flutter build apk --release --target-platform android-arm64 --split-per-abi
```

Local iOS (unsigned):

```sh
bash flutter/ios_arm64.sh
cd flutter && flutter build ipa --release --no-codesign
```

## Build (desktop)

Full steps, tool versions, and smoke-build commands: **[docs/BUILD_DESKTOP.md](docs/BUILD_DESKTOP.md)**.

Short checklist:

1. Init the required submodule:
   ```sh
   git submodule update --init --recursive
   ```
2. Install the toolchain pinned in that guide (Flutter **3.24.5**, Rust **1.75**, vcpkg, LLVM/Clang as needed).
3. On Windows, optionally activate/check the env:
   ```powershell
   . .\scripts\activate_desktop_env_windows.ps1
   powershell -ExecutionPolicy Bypass -File .\scripts\check_desktop_env_windows.ps1
   ```
4. Build the Flutter desktop app for Windows or Linux per `BUILD_DESKTOP.md`.

---

## Repository layout

- **`src/`** — Rust application (server services, client connection, platform code)
- **`src/server/`** — audio / clipboard / input / video / network services
- **`src/platform/`** — platform-specific code
- **`flutter/`** — Flutter UI (desktop under `flutter/lib/desktop/`)
- **`libs/hbb_common/`** — config, protobuf, shared utilities (git submodule)
- **`libs/scrap/`** — screen capture
- **`libs/enigo/`** — input control
- **`libs/clipboard/`** — clipboard / file copy-paste
- **`res/betterdesk/`** — Generator signing keys and notes for `custom.txt`

---

## Attribution and license

BetterDesk Client is a fork of [RustDesk](https://github.com/rustdesk/rustdesk). The About UI and sysinfo keep upstream attribution and source links as required by the project policy ([docs/OFFICIAL_CLIENT.md](docs/OFFICIAL_CLIENT.md)).

[AGPL-3.0](LICENSE) · Source: [UNITRONIX/BetterDesk-Client](https://github.com/UNITRONIX/BetterDesk-Client) · Upstream: [rustdesk/rustdesk](https://github.com/rustdesk/rustdesk)

---

## Support

- [Discord](https://discord.gg/MPp9hyyG97)
- [BetterDesk Issues](https://github.com/UNITRONIX/BetterDesk/issues)
- [BetterDesk-Client Issues](https://github.com/UNITRONIX/BetterDesk-Client/issues)
