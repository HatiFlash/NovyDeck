# NovyDeck 0.13.0 — D6 interaction and key appearance

## English

- **Folder navigation:** physical press tracking survives page synchronization, so repeated reports from the navigation gesture are not treated as a new press at the same position in the destination. A distinct new press remains usable.
- **Key feedback:** HID key events explicitly request an interface repaint instead of waiting for unrelated activity. The preview cache includes the saved title-overlay setting. These changes address tested software paths; no physical-device latency measurement is claimed.
- **Text without a background:** in a key's **Appearance** tab, clear **Background and shadow behind the title**. This keeps the custom image and text while removing the added gradient and shadow. The setting is saved per key and applies to both preview and device rendering. Existing profiles keep the previous appearance by default.
- **Scrollable Add menu:** the key inspector's action menu is bounded and scrollable, including compact windows and HiDPI. Tests reach and select its final entry.
- **Wallpaper Engine Stop / Play:** the separate plugin **1.0.6** adds a combined action for all screens, alongside explicit Stop, Play and Pause / Play. Update the plugin separately in the store. Its state is local to successful plugin commands, initially unknown, and does not follow changes made outside the plugin. The first press sends Stop; failures do not flip its state.

Qualification uses the Windows laboratory and offline/inert fixtures. No physical D6, real Wallpaper Engine installation, OLED display, connected light or user PC was operated by these checks. Updating NovyDeck does not automatically install or update its plugins. Profiles and custom media are retained.

## Français

- **Navigation dans les dossiers :** le suivi de l'appui physique est conservé pendant la synchronisation de page ; les rapports répétés du même geste de navigation ne sont plus traités comme un nouvel appui à la même position dans l'écran de destination. Un nouvel appui distinct reste utilisable.
- **Retour visuel des touches :** les événements HID demandent explicitement un rafraîchissement de l'interface sans attendre une autre activité. Le cache d'aperçu inclut le réglage du fond du titre. Ces changements corrigent les chemins logiciels testés, sans annoncer de mesure de latence sur le périphérique physique.
- **Texte sans fond :** dans **Apparence**, décochez **Fond et ombre derrière le titre**. L'image personnalisée et le texte restent ; seuls le dégradé et l'ombre ajoutés sont retirés. Le choix est enregistré par touche et utilisé pour l'aperçu comme pour l'image envoyée au deck. Les profils existants gardent le rendu précédent par défaut.
- **Menu Ajouter défilable :** la liste des actions de l'inspecteur est bornée et défilable, y compris en petite fenêtre et en HiDPI. Les tests atteignent et sélectionnent sa dernière entrée.
- **Wallpaper Engine Arrêt / Lecture :** le plugin séparé **1.0.6** ajoute une action combinée sur tous les écrans, à côté d'Arrêt, Lecture et Pause / Lecture. Mettez ce plugin à jour séparément dans le magasin. Son état est local aux commandes réussies du plugin, initialement inconnu, et ne suit pas les changements effectués ailleurs. Le premier appui envoie Arrêt ; un échec ne fait pas basculer l'état.

La qualification repose sur le laboratoire Windows et des témoins hors ligne ou inertes. Aucun D6 physique, installation Wallpaper Engine réelle, écran OLED, lumière connectée ni PC utilisateur n'a été piloté par ces contrôles. Mettre à jour NovyDeck n'installe ni ne met à jour automatiquement ses plugins. Profils et médias personnalisés sont conservés.
