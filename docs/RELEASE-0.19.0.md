# NovyDeck 0.19.0 — Real-time Home Assistant triggers

## English

- New **Home Assistant** connection in Settings: address and long-lived access token, WebSocket connection with automatic reconnection. The token is stored encrypted with Windows DPAPI. The address and token of the Home Assistant plugin can be reused without modifying the plugin.
- New automatic trigger **Home Assistant state**: entity, optional attribute, comparison and value, with a "For (seconds)" duration (0 = immediate) and an option to fire at startup if the condition is already true.
- Optional **return action**: another key is triggered when the condition becomes false again.
- The key can show a **badge** with the state (countdown, triggered, disconnected).
- New **Home Assistant condition** in the If / else and Wait until blocks.

Validated by targeted tests (Home Assistant logic, conditions, translations, desktop triggers, key rendering) and CI. Not yet verified against a live Home Assistant server or on a physical deck.

## Français

- Nouvelle connexion **Home Assistant** dans les Réglages : adresse et jeton d'accès longue durée, connexion WebSocket avec reconnexion automatique. Le jeton est stocké chiffré avec DPAPI de Windows. L'adresse et le jeton du plugin Home Assistant peuvent être réutilisés sans modifier le plugin.
- Nouveau déclencheur automatique **État Home Assistant** : entité, attribut optionnel, comparaison et valeur, avec une durée « Depuis (secondes) » (0 = immédiat) et une option pour se déclencher au démarrage si la condition est déjà vraie.
- **Action de retour** optionnelle : une autre touche est déclenchée quand la condition redevient fausse.
- La touche peut afficher une **pastille** d'état (compte à rebours, déclenché, déconnecté).
- Nouvelle **condition Home Assistant** dans les blocs Si… sinon et Attendre que….

Validé par des tests ciblés (logique Home Assistant, conditions, traductions, déclencheurs, rendu des touches) et la CI. Pas encore vérifié avec un vrai serveur Home Assistant ni sur un deck physique.
