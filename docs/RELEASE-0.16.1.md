# NovyDeck 0.16.1 — No green plugin badge, tidier Plugins tab and ON/OFF editor

## English

- **No more green badge after a successful plugin action:** the `showOk` event sent by plugins (for example RivaTuner OSD) is still accepted but no longer draws a green check badge on the key, on the physical deck or in the preview. `showAlert` error badges are unchanged.
- **Tidier Plugins tab:** prominent **Plugin store** button at the top, installed plugins below, imports (FIFINE / StreamDock / Elgato, `.streamDeckPlugin` file, `.sdPlugin` folder) and *Refresh* grouped in one **Import…** menu; technical status text removed and status messages no longer truncated.
- **Clearer ON/OFF key editor:** **First press** and **Second press** cards side by side (stacked on very narrow inspectors), each with Add / Edit / Remove / Replace buttons above its block list; the selected block editor shows below and scrolls with the cards. Same data model, existing keys unchanged.

Tested with the Windows CI suite and the off-screen UI harness on the test VM. Not yet tested on a connected deck.

## Français

- **Plus de pastille verte après une action de plugin réussie :** l'événement `showOk` envoyé par les plugins (par exemple RivaTuner OSD) reste accepté mais ne dessine plus de pastille verte avec coche sur la touche, ni sur le deck physique ni dans l'aperçu. Les pastilles d'alerte `showAlert` sont inchangées.
- **Onglet Plugins rangé :** bouton **Magasin de plugins** bien visible en haut, plugins installés dessous, imports (FIFINE / StreamDock / Elgato, fichier `.streamDeckPlugin`, dossier `.sdPlugin`) et « Actualiser » regroupés dans un seul menu **Importer…** ; textes techniques retirés et messages d'état plus tronqués.
- **Touche ON/OFF plus claire :** cartes **1er appui** et **2e appui** côte à côte (empilées si l'inspecteur est très étroit), chacune avec ses boutons Ajouter / Modifier / Retirer / Remplacer au-dessus de sa liste de blocs ; l'éditeur du bloc sélectionné s'affiche dessous et défile avec les cartes. Même modèle de données, touches existantes inchangées.

Validé par la suite de tests Windows de la CI et par le harnais d'interface hors écran sur la VM de test. Pas encore essayé sur un deck connecté.
