# NovyDeck 0.11.1

Maintenance release for the plugin store, built from 0.11.0 without resetting profiles or custom key images.

- **Plugin upgrades:** Official `.sdPlugin` packages without a root-level `UUID` can now be upgraded when the package folder is exactly `{plugin-id}.sdPlugin` and the verified identity/version match. A noncanonical installed folder is rejected without changing it. Archive SHA-256 verification, pause/backup/rollback, and preservation of local data remain unchanged.
- **Accurate status:** The committed plugin is read back before its verified version is supplied to the store and runtime. A failed transaction retains the update option and displays an error rather than claiming success.
- **Plugin store layout:** Adaptive card columns and separated version/status lines prevent narrow, letter-by-letter pills. Headless FR/EN captures cover a wide window and 125% HiDPI.
- **VM verification:** A regression test reproduced the refusal with published archives before the fix. Afterward, Home Assistant 1.0.2 → 1.0.3 and Google Home 1.0.4 → 1.0.5 succeeded inside temporary plugin roots on the Windows lab VM, including backup and reloaded inventory checks. No physical D6 or user installation was used.

Existing user profiles, key images, macros and plugins are not automatically replaced. Install the new app version first; then explicitly update plugins in the store.
