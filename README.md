# TP LAUNCHER · TP LABS

[Español](README.es.md) · [Source code](https://github.com/tomaap07/MinecraftTPS) · [Contact and support](https://github.com/tomaap07/MinecraftTPS/issues)

TP LAUNCHER is an independent Windows desktop launcher for **Minecraft: Java Edition**, built with Rust, Tauri 2, Svelte 5 and TypeScript. It manages game installations, compatible mod content, accounts and player creation tools. The current application version is **0.1.0**.

## Microsoft / Minecraft Services review summary

| Item | Implementation |
| --- | --- |
| Application | TP LAUNCHER, maintained by TP LABS |
| Maintainer contact | [tplabs.contacto@gmail.com](mailto:tplabs.contacto@gmail.com); [tomaap07 on GitHub](https://github.com/tomaap07); [project issues](https://github.com/tomaap07/MinecraftTPS/issues) |
| Public application ID | `6a4f8732-6bf7-49c3-a749-16607f97281d` |
| Authentication | Microsoft identity platform **OAuth 2.0 v2.0 Authorization Code Flow with PKCE S256** in the user's system browser |
| Microsoft scopes | `XboxLive.signin offline_access` |
| Registered redirect | `http://localhost:8080`; a loopback callback validates method, Host, path and random `state` |
| Minecraft authorization | Microsoft → Xbox Live → XSTS → Minecraft Services; Java entitlement and official profile are checked before completing sign-in |
| Credential storage | Refresh tokens encrypted with **Windows DPAPI** for the current Windows user; Minecraft access tokens kept in memory. Passwords entered only on Microsoft's website |
| Offline Mode | Available only after successful verification of a licensed Microsoft Minecraft Java account on this device, with saved protected credentials and a valid verification record |
| First run | Microsoft sign-in and Minecraft Java ownership verification before language and optional Java setup |

TP LAUNCHER uses its own application registration. A client secret and another launcher's application ID are not distributed. The application does not provide an unverified account path to launch Minecraft. Creating or selecting a local gameplay profile is subject to the verified-account requirement in Rust and in the interface. A skin preview by username does not authenticate that username.

**Service access review is in progress; approval has not been confirmed.** Microsoft sign-in and Xbox authentication are separate from permission to use Minecraft Services. The observed Minecraft Services HTTP 403 is documented in the [authentication guide](docs/microsoft-client-configuration.md). Failed authorization is reported as a failure, not a connected account. [Application-review technical details](docs/minecraft-services-request.md).

**TP LAUNCHER is not an official Minecraft product. It is not approved by Mojang or associated with Microsoft.**

## Sign-in and Offline Mode

1. Sign in through the official Microsoft page in your browser.
2. The launcher obtains Xbox/XSTS authorization, checks the Minecraft Java entitlement and loads the official Minecraft profile.
3. Successful verification and encrypted local storage enable Offline Mode and local gameplay profiles on that device. The launcher's verification cache lasts **30 days**, after which an online recheck is required. This duration is a launcher policy, not a claim about a Microsoft-mandated interval.
4. Without eligible account evidence, local profiles remain disabled with this notice: “Para utilizar perfiles locales, debes iniciar sesión con una cuenta Microsoft al menos una vez en este equipo”. Existing local profiles do not bypass the requirement.

Offline Mode does not provide a valid session for servers requiring official online authentication. Account credentials and verification records are local to the device and are not shipped in the installer or repository.

Need Minecraft? Purchase a license from the [official Minecraft store](https://www.minecraft.net/es-es/store/minecraft-deluxe-collection-pc?tabs=%7B%22details%22%3A0%7D).

## Features

- **Play and installations:** Vanilla and loader catalogs, compatible Java selection, memory settings, recent installations and playtime. The background character uses the connected skin or a skin preview by username.
- **Mods, modpacks, shaders and resource packs:** Modrinth and CurseForge catalogs, search, categories, project images, file details and dependencies. Installation selects a recent compatible file and asks for a compatible installed destination when needed.
- **Isolated instances:** Separate modpack directories, import/export tools, installation progress, streaming downloads, hash checks and retry caches. Content redistribution remains subject to author and provider terms.
- **Account and skins:** Microsoft profile, official skins and capes, local skin library and username-based appearance previews.
- **3D studio:** Pose editing, camera, lighting and transparent PNG rendering, built with [skinview3d](https://github.com/bs-community/skinview3d) and [Three.js](https://threejs.org/), with their MIT notices preserved.
- **Library and HUD:** Screenshots, worlds, resource/shader folders and persistent logs. Internal game metrics require compatible adapters; see [Universal HUD](docs/universal-hud.md) and [TP Companion](tp-companion/README.md).
- **Languages:** Local dictionaries for ten primary languages, with additional variants and partial catalogs identified in the interface. MyMemory translation is optional and limited to public content summaries; [details](docs/translations.md).

Game clients, libraries, Java runtimes, loaders and content are obtained from their providers during installation. The repository and launcher installer do not bundle Minecraft game clients or OptiFine installers. Installed third-party content retains its applicable terms and licenses.

## Development

Use Node.js 22 or later, Rust stable with MSVC, Visual Studio C++ build tools and Windows SDK. Building TP Companion requires a JDK 21 or later through `JAVA_HOME`. The Windows installer includes the WebView2 offline runtime installer.

```sh
npm ci
npm run tauri dev
```

```sh
npm run check
npm run check:ui
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

Configure your own Microsoft registration using the [authentication guide](docs/microsoft-client-configuration.md). A public client ID identifies an application; it is neither a client secret nor proof of service approval. Forks must use their own registration.

Main entry points: `src/routes/+page.svelte`, `src-tauri/src/microsoft_oauth.rs`, `microsoft_services.rs`, `account_licensing.rs` and `launcher_settings.rs`. The content catalog uses `src/lib/ModStudio.svelte` and `src-tauri/src/mod_studio.rs`.

## Provider integrations

CurseForge credentials are read at runtime from the environment, a development `.env` file or the device's `integrations.env`. They are excluded from Git and the bundle; no developer key is distributed. Provider authorization and download restrictions are reported. If an author disables third-party downloads, the launcher requests the exact file from its official page and verifies its size and hash. [Content installation details](docs/modpack-installation.md).

Services include [Modrinth](https://docs.modrinth.com/api/), [CurseForge](https://docs.curseforge.com/rest-api/), [Fabric](https://meta.fabricmc.net/), [Quilt](https://meta.quiltmc.org/), [Forge](https://files.minecraftforge.net/), [OptiFine](https://optifine.net/downloads) and the [TP server catalog](https://tp-api-hazel.vercel.app/api/v1/servers). Names and logos identify their services and do not imply endorsement.

## User files

Preferences and the account library reside in `%APPDATA%/LauncherTPS/`. Game data defaults there for new installations; an existing `%APPDATA%/MinecraftTPS/` directory is retained. Settings can select another game-data directory. The table distinguishes configuration storage from the selected game-data root.

| Location | Contents |
| --- | --- |
| Configuration: `launcher-settings.json` | Preferences, language and game-data directory |
| Configuration: `accounts.json` | Account names, identifiers and selection; no plaintext tokens |
| Configuration: `accounts/*.dat` | DPAPI-encrypted Microsoft refresh tokens |
| Configuration: `minecraft-license-cache.dat` | DPAPI-encrypted Minecraft Java verification records |
| Configuration: `translation-cache.json` | Cached translations of public descriptions |
| Game data: `account.dat` | DPAPI-encrypted active Microsoft refresh token |
| Game data: `versions`, `libraries`, `assets`, `runtime` | Downloaded dependencies and Java |
| Game data: `instances/...` | Isolated mods, configuration, worlds and logs |
| Game data: `skins_library` | Skin PNG files and library index |
| Game data: `custom_poses.json`, `renders` | Custom poses and PNG exports |
| Game data: `playtime.json`, `logs` | Playtime and diagnostics |
| Game data: `cache/content`, `cache/modpacks` | Verified download and import caches |

Do not commit or distribute local credentials, account data, verification caches, worlds or private configuration. Username appearance queries and screenshots are distinct from authentication.

## License, assets and distribution

Original launcher code is licensed under **GPL-3.0-only**; [LICENSE](LICENSE). This license does not grant trademark permission to present a modified distribution as official TP LAUNCHER. Project branding remains with its owner. Dependencies, provider marks, fonts and user-supplied artwork retain their licenses and attribution in [third-party notices](static/legal/THIRD-PARTY-NOTICES.txt).

Some optional visuals can read resources from the user's installed game JAR. Local access does not grant redistribution permission or make them original TP artwork. [Asset policy](docs/asset-policy-audit.md).

The Windows build has **no Authenticode signing identity configured**. A signing workflow is available for a valid certificate; service approval and executable signing are separate requirements. See [Windows distribution](docs/distribution-windows.md). Automated checks do not certify every Minecraft version, adapter or third-party modpack.

This 0.1.0 release is an explicit version renumbering accompanying the Microsoft-first account policy; it does not represent the absence of earlier development versions. Existing 0.1.1–0.1.3 installations require manual installation because 0.1.0 is lower under semantic versioning. Removing obsolete release downloads does not change the Git development history.
