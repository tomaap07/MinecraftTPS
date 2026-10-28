# TP LAUNCHER 0.1.0 · Verified-account release

Date: 28 October 2026. This release renumbers the application to 0.1.0 and replaces the previous downloadable test packages. It does not erase the project's development history. Existing installations reporting 0.1.1–0.1.3 require a manual install because semantic-version comparison does not automatically offer a lower version.

## Account policy

The first-run wizard now starts with Microsoft sign-in and Minecraft Java ownership verification. Language selection and optional Java setup follow the account step. A returning user with valid saved licence evidence can also complete setup with an already authorized local profile while offline.

Creating, activating or launching a local gameplay profile requires at least one saved Microsoft account with a matching successful Java entitlement check, a valid encrypted local verification record and a usable Windows DPAPI-protected refresh token. Local profiles from older versions do not bypass this requirement. Skin previews by username do not grant an authenticated identity.

Without eligible evidence, local actions are disabled and display the requested notice:

> Para utilizar perfiles locales, debes iniciar sesión con una cuenta Microsoft al menos una vez en este equipo

The existing verification window is 30 days, as a launcher policy. Removing the qualifying account or its saved token, corrupting the evidence, expiration, or a confirmed authorization/licence rejection prevents further offline use until verification succeeds again. Network failures and application HTTP 403 responses are not treated as proof that a previously verified owner has lost the game.

Account, settings and first-run screens link to the official Minecraft purchase page with a green action. No password is entered into TP LAUNCHER. Microsoft OAuth 2.0 v2.0 Authorization Code Flow with PKCE, Xbox/XSTS, Minecraft entitlements/profile and native DPAPI storage remain in use. The public contact is tplabs.contacto@gmail.com.

## Documentation and distribution

- The main README is in English, with a complete Spanish README and valid Markdown file tables.
- Current guides describe the implemented account requirement. Dated reports and Git history are retained as historical records rather than rewritten as descriptions of the current build.
- The five obsolete release downloads were withdrawn as part of this transition. The repository was made public after the source/history security checks; Git history and historical tags were retained.
- No Microsoft/Mojang approval has been confirmed. The current Client ID still needs its Minecraft Services access review; this release does not bypass an HTTP 403 or guarantee approval.
- The installer remains unsigned because no trusted signing identity is configured. Signature preparation tooling is available, but an Authenticode signature is not claimed.

## Validation

- `npm run check`: zero errors and zero warnings.
- `npm run check:ui`: 1466 entries; primary language dictionaries and English variants have no missing keys. Additional partial catalogs remain identified as partial.
- `cargo test --quiet`: 111 passed, 19 ignored, zero failed.
- Native DPAPI and account-evidence tests cover missing/corrupt/empty/deleted tokens, missing/expired/future verification, removed Microsoft accounts, legacy local-only libraries and targeted cache revocation.
- HTTP authentication fixtures retain the successful chain and rejected Microsoft/Minecraft scenarios without logging private response bodies.

The tests do not establish successful live Microsoft/Minecraft authorization for the pending application registration, universal Minecraft/HUD compatibility, or legal certification. The API provider and applicable terms remain authoritative.

## Installer verification

- File: `TP-LAUNCHER-0.1.0-Windows-x64-Setup.exe`, 220704168 bytes.
- Installer SHA-256: `cbc8f7e2b03e291dde85578546760c926434a3c768304aa436fe102d9489c07f`.
- Bundled executable SHA-256: `c345c8d38a67b2a42eaa71a8accaca9370b27446134899640a50fdb4820aef12`.
- The NSIS payload and companion JARs were extracted for inspection. All 19 current frontend chunks were confirmed in the executable; 52 embedded assets were decompressed for inspection. Only Tauri's expected three-byte bundle-type marker differs from the standalone executable.
- Source/history and extracted-payload audit: 567 text files scanned, 476 historical blobs, 786 files compared against four local secrets; no matches and no high-severity findings. Seven informational findings identify public OAuth application IDs. A heuristic audit is not an absolute guarantee that secrets cannot exist.
- The final installer's local Microsoft Defender custom scan completed without detecting threats. No protections, exclusions or trusted certificates were changed. This is not a security certification or a SmartScreen reputation result.
- Authenticode status: `NotSigned`. A trusted signing identity is not configured.
- Final production browser tests used isolated Edge and simulated IPC: first-run licence blocking, 403 failure/retry, licensed unlock, local profile creation, cached licensed repeat setup without another online login, ten language choices, optional Java skip, official store links and relocking after evidence revocation. No browser runtime errors were observed. This is not a live Microsoft/Minecraft sign-in test.
