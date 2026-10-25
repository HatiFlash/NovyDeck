# NovyDeck 0.11.1 — Plugin Store maintenance

## English

- **Updates work with official packages.** Version 0.11.0 required a root-level `UUID` field that official plugin manifests omit, so an update could be rejected while the old version and blue **Update** button remained. A package without that field is now accepted only when its folder and installed folder are exactly `{plugin-id}.sdPlugin` and its verified identity/version match. SHA-256 verification, targeted pause, backup/rollback and local-data protection remain.
- **Verified status.** After a successful replacement, NovyDeck reads the installed manifest back before updating the store's inventory and runtime catalog. A failure keeps the upgrade available and reports the error. Old store messages are cleared when a new transaction starts.
- **Readable store.** Responsive card columns and separate version/status lines prevent badges and update labels from collapsing into vertical letters.

On the Windows lab VM, published Home Assistant 1.0.3 and Google Home 1.0.5 archives upgraded temporary 1.0.2 and 1.0.4 fixtures, with backups and reloaded inventories checked. UI layout tests covered a wide screen and 125% HiDPI in English/French. Neither the user's PC/plugins nor the physical D6, live Google Home or Wallpaper Engine were exercised. Existing profiles, custom key images and plugins are not automatically replaced; install NovyDeck first, then choose plugin updates explicitly.

## Français

- **Mise à jour des plugins officiels.** La version 0.11.0 exigeait un champ `UUID` absent des manifestes officiels : la mise à jour pouvait échouer, laissant l'ancienne version et le bouton bleu **Mettre à jour**. Sans ce champ, l'archive n'est désormais acceptée que si son dossier et le dossier installé portent exactement le nom `{identifiant}.sdPlugin` et si l'identité et la version vérifiées correspondent. Contrôle SHA-256, pause ciblée, sauvegarde/retour arrière et préservation des données locales restent actifs.
- **Statut vérifié.** Après remplacement réussi, NovyDeck relit le manifeste installé avant de rafraîchir le magasin et le catalogue du runtime. Un échec conserve la possibilité de mise à jour et affiche l'erreur. Les anciens messages du magasin sont effacés au début d'une transaction.
- **Magasin lisible.** Les cartes adaptent leur nombre de colonnes à la fenêtre ; versions et statuts sur des lignes distinctes ne s'écrasent plus en lettres verticales.

Sur la VM Windows de labo, les archives publiques Home Assistant 1.0.3 et Google Home 1.0.5 ont mis à jour des installations témoins 1.0.2 et 1.0.4 dans des dossiers temporaires, avec contrôle des sauvegardes et des inventaires relus. Les tests d'interface couvrent grand écran et HiDPI 125 % en français/anglais. Ni le PC et les plugins de l'utilisateur, ni le D6 physique, ni les services Google Home/Wallpaper Engine réels n'ont été utilisés. Profils, images personnalisées et plugins ne sont pas remplacés automatiquement : installez d'abord NovyDeck, puis choisissez les mises à jour des plugins.
