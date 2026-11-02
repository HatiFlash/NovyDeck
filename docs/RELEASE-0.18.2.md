# NovyDeck 0.18.2 — Plugin icons and clean Windows shutdown

## English

- Custom icons configured for plugin keys now remain visible on the deck; previously the plugin's default icon could replace them.
- During application shutdown or Windows sign-out/shutdown, NovyDeck no longer launches or restarts plugins, avoiding the Windows “0xc0000142” dialogs associated with late plugin launches.
- Plugin restarts are now bounded and use progressively longer delays instead of retrying indefinitely.
- A crashing plugin no longer raises a system error dialog; the failure is recorded in the log.

Validated by the Windows test suite and CI. Still to verify on a connected physical deck and during an actual Windows shutdown on the user's PC.

## Français

- Les icônes personnalisées des touches de plugin restent désormais affichées sur le deck ; auparavant, l'icône par défaut du plugin pouvait les remplacer.
- Pendant la fermeture de l'application ou l'arrêt/la déconnexion de Windows, NovyDeck ne lance ni ne relance plus de plugins, évitant les fenêtres Windows « 0xc0000142 » liées aux lancements tardifs.
- Les relances des plugins sont limitées et espacées par des délais progressifs, sans réessais infinis.
- Un plugin qui plante n'ouvre plus de boîte d'erreur système ; l'erreur est inscrite au journal.

Validé par la suite de tests Windows et la CI. Il reste à vérifier le comportement sur un vrai deck connecté et lors d'un véritable arrêt de Windows sur le PC de l'utilisateur.
