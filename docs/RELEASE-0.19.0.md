# NovyDeck 0.19.0 — Real-time Home Assistant triggers

## English

- New **Home Assistant** connection in Settings: address and long-lived access token, WebSocket connection with automatic reconnection. The token is stored encrypted with Windows DPAPI. The address and token of the Home Assistant plugin can be reused without modifying the plugin.
- New automatic trigger **Home Assistant state**: entity, optional attribute, comparison and value, with a "For (seconds)" duration (0 = immediate) and an option to fire at startup if the condition is already true.
- New option **Key image follows the mode**: the key (or another chosen key) shows its "active" image while the mode is active (between the firing and the return action) and its "inactive" image otherwise, also during the countdown. At startup and after reconnection the image reflects the real state without running the action; while disconnected the last image is kept with the "HA ×" badge. Global pause blocks actions but the image keeps following the real state. In-memory only, nothing is rewritten on disk. OLED scenario: put active/inactive images on the OLED key (two-state option), add a Home Assistant state trigger on the key that starts the saver (presence sensor, duration, return key), then tick the option.
- Optional **return action**: another key is triggered when the condition becomes false again.
- The key can show a **badge** with the state (countdown, triggered, disconnected).
- New **Home Assistant condition** in the If / else and Wait until blocks.

Validated by targeted tests (Home Assistant logic, conditions, translations, desktop triggers, key rendering) and CI. Not yet verified against a live Home Assistant server or on a physical deck.

## Français

- Nouvelle connexion **Home Assistant** dans les Réglages : adresse et jeton d'accès longue durée, connexion WebSocket avec reconnexion automatique. Le jeton est stocké chiffré avec DPAPI de Windows. L'adresse et le jeton du plugin Home Assistant peuvent être réutilisés sans modifier le plugin.
- Nouveau déclencheur automatique **État Home Assistant** : entité, attribut optionnel, comparaison et valeur, avec une durée « Depuis (secondes) » (0 = immédiat) et une option pour se déclencher au démarrage si la condition est déjà vraie.
- Nouvelle option **Image de la touche selon le mode** : la touche (ou une autre touche choisie) affiche son image « actif » tant que le mode est actif (entre le déclenchement et l'action de retour) et son image « inactif » sinon, y compris pendant le compte à rebours. Au démarrage et après reconnexion l'image reflète l'état réel sans exécuter l'action ; déconnecté, la dernière image reste avec la pastille « HA × ». La pause globale bloque les actions, mais l'image continue de suivre l'état réel. Uniquement en mémoire : rien n'est réécrit sur le disque. Scénario OLED : images actif/inactif sur la touche OLED (option deux états), déclencheur État Home Assistant sur la touche qui lance l'économiseur (capteur de présence, durée, touche de retour), puis cocher l'option.
- **Action de retour** optionnelle : une autre touche est déclenchée quand la condition redevient fausse.
- La touche peut afficher une **pastille** d'état (compte à rebours, déclenché, déconnecté).
- Nouvelle **condition Home Assistant** dans les blocs Si… sinon et Attendre que….

Validé par des tests ciblés (logique Home Assistant, conditions, traductions, déclencheurs, rendu des touches) et la CI. Pas encore vérifié avec un vrai serveur Home Assistant ni sur un deck physique.
