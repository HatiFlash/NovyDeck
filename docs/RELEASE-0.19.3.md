# NovyDeck 0.19.3 — Signed updates and plugin catalogue

## English

- **Signed updates (audit S-01)**: each release publishes `NovyDeck-{version}-Setup.exe.sig`, an Ed25519 signature of `NovyDeck update v1\n{version}\n{sha256}\n`. NovyDeck checks it after reading the `.sha256` file and before downloading the installer. Two public keys are pinned in the application (a main key and an offline backup key); a signature from either is accepted. A missing, unreadable or invalid signature stops the update. The release is first created as a draft by CI, signed with a key that is not stored on GitHub, then published.
- **Signed catalogue (S-02)**: the store downloads `registry.json.sig` next to `registry.json` and checks the signature over the exact catalogue bytes. A missing or invalid signature makes the store unavailable with a clear message, without crashing. Antivirus scan results remain informative and unsigned. The plugin repository CI checks the signature on every change.
- **Formatting and lint (C-02, C-05)**: the whole repository is formatted with `cargo fmt`; CI checks formatting and runs Clippy (errors fail the build, style warnings are reported).

Older versions (0.19.2 and earlier) do not check signatures; updating to 0.19.3 brings the protection for the following updates.

Validated by targeted Windows tests (signatures, update, catalogue, store, plugin state, translations) and CI.

## Français

- **Mises à jour signées (audit S-01)** : chaque release publie `NovyDeck-{version}-Setup.exe.sig`, une signature Ed25519 de `NovyDeck update v1\n{version}\n{sha256}\n`. NovyDeck la vérifie après lecture du fichier `.sha256` et avant de télécharger l'installeur. Deux clés publiques sont épinglées dans l'application (une principale et une de secours gardée hors ligne) ; une signature de l'une ou l'autre est acceptée. Une signature absente, illisible ou invalide arrête la mise à jour. La release est d'abord créée en brouillon par la CI, signée avec une clé qui n'est pas sur GitHub, puis publiée.
- **Catalogue signé (S-02)** : le magasin télécharge `registry.json.sig` à côté de `registry.json` et vérifie la signature sur les octets exacts du catalogue. Une signature absente ou invalide rend le magasin indisponible avec un message clair, sans plantage. Les résultats d'analyse antivirus restent informatifs et non signés. La CI du dépôt des plugins vérifie la signature à chaque modification.
- **Formatage et analyse (C-02, C-05)** : tout le dépôt est formaté avec `cargo fmt` ; la CI vérifie le formatage et lance Clippy (les erreurs bloquent, les avertissements de style sont signalés).

Les anciennes versions (0.19.2 et avant) ne vérifient pas les signatures ; passer à la 0.19.3 apporte la protection pour les mises à jour suivantes.

Validé par des tests Windows ciblés (signatures, mise à jour, catalogue, magasin, état des plugins, traductions) et la CI.
