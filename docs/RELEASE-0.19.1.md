# NovyDeck 0.19.1 — Home Assistant in the plugin, safer plugins

## English

- **Home Assistant moves to the Home Assistant plugin (1.1.1 or later).** Its new **Automatic state** action follows an entity live and drives the key image and the key itself. The built-in Home Assistant connection, the "Home Assistant state" trigger, the "Key image follows the mode" option and the Home Assistant condition introduced in 0.19.0 are removed. Preferences from 0.19.0 still load: the old triggers are dropped, the stored connection is no longer written, and keys that still contain the old condition are listed once in a non-blocking notice. Nothing is converted automatically.
- **Native plugin authentication (Windows)**: `registerPlugin` is only accepted from a connection owned by a process inside the plugin's Job, i.e. launched by NovyDeck. Any other local process is refused, disconnected and logged; if the owner cannot be determined, the connection is refused.
- **`novydeckTrigger`**: each refusal is logged with its reason (at most one line per context and reason every 5 s), and a key launched by a plugin trigger cannot be re-triggered while it runs or within 2 s (loop guard).
- **`plugin-settings.json`**: atomic write (temporary file, sync, `.bak` copy, rename); an unreadable or invalid file is reported and not overwritten, also when reading global settings.

Validated by targeted Windows tests (plugin host, plugin triggers, plugin state and blocks, preferences, translations) and CI. IPv6 peer lookup is implemented but not exercised by a test. Not tested against a live Home Assistant server or on a physical deck.

## Français

- **Home Assistant passe dans le plugin Home Assistant (1.1.1 ou plus récent).** Sa nouvelle action **État automatique** suit une entité en direct et pilote l'image de la touche et la touche elle-même. La connexion Home Assistant intégrée, le déclencheur « État Home Assistant », l'option « Image de la touche selon le mode » et la condition Home Assistant ajoutés en 0.19.0 sont retirés. Les préférences de la 0.19.0 se chargent toujours : les anciens déclencheurs sont supprimés, la connexion enregistrée n'est plus réécrite, et les touches qui contiennent encore l'ancienne condition sont listées une fois dans une notification non bloquante. Rien n'est converti automatiquement.
- **Authentification des plugins natifs (Windows)** : `registerPlugin` n'est accepté que d'une connexion appartenant à un processus du Job du plugin, donc lancé par NovyDeck. Tout autre processus local est refusé, déconnecté et journalisé ; si le propriétaire ne peut pas être déterminé, la connexion est refusée.
- **`novydeckTrigger`** : chaque refus est journalisé avec sa raison (au plus une ligne par contexte et par raison toutes les 5 s), et une touche lancée par un déclencheur de plugin ne peut pas être relancée pendant son exécution ni dans les 2 s (garde anti-boucle).
- **`plugin-settings.json`** : écriture atomique (fichier temporaire, sync, copie `.bak`, renommage) ; un fichier illisible ou invalide est signalé et n'est pas écrasé, y compris à la lecture des réglages globaux.

Validé par des tests Windows ciblés (hôte des plugins, déclencheurs, état et blocs de plugin, préférences, traductions) et la CI. La recherche du pair en IPv6 est implémentée mais pas couverte par un test. Non testé avec un vrai serveur Home Assistant ni sur un deck physique.
