# NovyDeck — gestion des plugins et blocs (chantier 0.11)

## Périmètre approuvé

Demandes de Hatim, mémoire `novydeck-plugin-management-next` (20261025012900-f3z9grw), captures des 24–25 octobre 2026. Base de travail : main/v0.10.1, 379689b517476cbae7f2d1743b417458c8c5195d. Ne pas modifier l'installation active du PC, lancer une seconde instance connectée au D6, remplacer ses images, exécuter ses macros ou se connecter à Google pendant le développement. Aucun tag/release/déploiement sur le PC sans nouvelle validation du résultat.

Rust natif + egui/eframe =0.31.1, sans nouvelle interface web pour l'application ni montée de dépendances. Les inspecteurs HTML existants des plugins restent hébergés dans leur processus isolé. Textes FR/EN avec tr/trf et clés françaises exactes, préserver CRLF/BOM, ne pas reformater globalement.

## Décisions d'architecture

- Réutiliser l'importeur confiné et sa vérification SHA-256, mais garder son import idempotent : une mise à jour est une opération distincte, explicitement confirmée, qui valide UUID/version, prépare une archive en staging et réalise un remplacement transactionnel avec sauvegarde/rollback. Ne pas considérer la version du catalogue comme la version locale.
- Désinstallation : lister les usages (touches, appuis longs, blocs imbriqués), confirmer, ne jamais supprimer silencieusement les profils/réglages ni les secrets DPAPI. Sauvegarder le dossier avant retrait et rendre visibles les références devenues indisponibles.
- Arrêter et reprendre uniquement les processus des plugins concernés via une requête acquittée du gestionnaire runtime. Ne pas geler/tuer l'application pour ses opérations normales ; empêcher les redémarrages automatiques concurrents pendant le remplacement.
- Les plugins installés sont affichés dans des fiches compactes repliables ; le sélecteur de blocs regroupe d'abord par plugin puis catégorie/action et reste défilable dans la hauteur disponible. Inventaire dynamique réel, aucun cas spécial SignalRGB.
- Un bloc possède un contexte virtuel d'inspecteur stable lié à la clé/profil/page/chemin du bloc. Réutiliser le routage existant des réglages virtuels, avec rejet des événements périmés et isolation entre blocs, même si un message n'inclut pas le champ action. Un clic et un bouton explicite ouvrent l'inspecteur ; copier une touche reste facultatif.
- Pour Tester pour de vrai, distinguer action plugin directe et macro vide ; la simulation n'envoie aucune commande réelle. Réutiliser le transport générique et ses garde-fous, pas une seconde application ni un accès matériel parallèle.
- Réutiliser StateDisplay/StateFace pour les visuels ON/OFF : source état du plugin, mapping d'états explicite, aucune image imposée, aperçu et deck identiques. Ne jamais basculer avant un succès ou à partir d'une erreur. État inconnu => visuel de base/indication inconnue ; distinguer état rapporté par le plugin et état local/dernière commande. Les plugins ne rapportant pas d'état ne deviennent pas artificiellement des capteurs physiques.

## Ordre et responsabilités

1. Transactions et cycle de vie runtime (worker Sol) : plugin_import/community + nouveau module de gestion + plugin_runtime/host si nécessaire. Tests RED/GREEN, rollback et arrêt ciblé. Pas d'édition de ui.rs/ui_extras/ui_plugin_store.
2. Navigation des plugins (worker Luna) : ui_extras, ui_plugin_store, tests UI ciblés. Liste compacte et sélecteur hiérarchique défilable. Pas d'édition du runtime ni des opérations disque.
3. Intégration contrôlée 1+2, puis UI Mettre à jour/Désinstaller dans le magasin et les fiches (worker Luna). Vérifier usages et confirmations, conserver le consentement réseau actuel.
4. Inspecteur natif des blocs et test direct des plugins (worker Sol), après intégration : ui_extras/ui.rs/plugin_runtime et tests d'isolation/preview.
5. Visuels ON/OFF des plugins (worker Sol ou Luna selon les changements nécessaires) : modèle compatible, moteur de rendu, événements, configuration UI et tests. Si les plugins maison doivent émettre setState, modifier leurs sources dans un worktree dédié et versionner séparément ; pas de mise à jour du PC.
6. Revue, régressions et finition : compilation Windows --locked, tests ciblés puis suite UI/i18n/thèmes, captures FR/EN sombre/clair, petites fenêtres/HiDPI. README et notes 0.11.0 cohérents avec les fonctions réellement livrées.

Chaque worker a son worktree et ses fichiers bornés. Aucun fichier logique partagé en écriture simultanée : les opérations runtime et les inspecteurs sont séquencés ; la bibliothèque/sélecteur ne touchent pas au runtime. Les traductions parallèles seront fusionnées par union des clés connues, sans écrasement. Le coordinateur garde les arbitrages, la version finale, l'intégration et la validation visuelle. Tests VM sous le verrou commun `/tmp/novydeck-vm.lock`, pas de tests simultanés ni de reset de travaux inconnus.

## Boucles de validation

- VM de labo désignée : 230 deckforge-win, SSH deckforge@192.168.0.115 avec identité existante. Vérifier état/machine avant utilisation ; ne pas toucher au PC utilisateur.
- `bash /root/.hermes/novy-bench/vmtest.sh '<filtres>' [-Ui]` depuis une branche feat/* ; seuls BUILD=0, TESTBUILD=0 et T_<filtre>=0 avec les vrais résultats font foi. Alternativement CI Windows de la branche exacte. Pas de Rust local supposé.
- Démontrer RED avant correctif, puis GREEN et non-régression. Fixtures de plugin contrôlées, événements WebSocket réels en labo, aucun service/compte/appareil tiers.
- La validation de sécurité des mises à jour/désinstallations et le refus d'archives UUID incorrect, version inférieure, lien/chemin sortant ou opération sur un autre plugin sont bloquants.
- Livrable attendu : branche intégrée réellement compilée/testée, captures relues et aperçu présenté à Hatim. Publication et installation sur son PC ne font pas partie de cette autorisation de démarrage.

## Risques / points à confirmer

- Verrous d'exécutables Windows et redémarrage automatique des plugins : arrêt ciblé acquitté avant toute mutation.
- Contrats de réglages des inspecteurs tiers : tester deux blocs d'un même plugin et des messages sans action, ainsi qu'un profil/page changés pendant l'ouverture.
- Les sources maison consultées dans plugins-release104 (Google Home et Wallpaper Engine) envoient actuellement showOk/showAlert et des titres, mais pas setState ; le SDK n'expose pas encore d'helper d'état. T6 doit donc inclure une émission explicite d'état pour les actions maison concernées, pas seulement une option d'images dans NovyDeck. Pour Google Home, ne jamais remplacer la lecture réelle GetTraits par un compteur local ; pour Wallpaper Engine, qualifier clairement l'état Pause/Lecture comme local. Tester le flux protocolaire de succès ET d'échec, sans compte ni logiciel tiers réel, dans un worktree de plugins distinct.
- setState peut signifier autre chose que ON/OFF selon le plugin : mapping explicite requis ; absence de signal ne doit jamais être présentée comme un état réel.
- Les captures déjà reçues montrent un menu tronqué, mais ne prouvent pas à elles seules qu'il ne contient que SignalRGB : corriger l'accessibilité et vérifier l'inventaire.
