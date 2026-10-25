# Site vitrine NovyDeck

Page statique autonome pour présenter NovyDeck, inspirée par la **structure** du site Nusa (aperçu, fonctions, compatibilité, téléchargement), mais réalisée avec l'identité visuelle et les captures réelles de NovyDeck. Aucun code, texte, logo ou image de Nusa n'est repris.

## Voir le site en local

Depuis `website/` :

```sh
python3 -m http.server 8765
```

Ouvrir `http://127.0.0.1:8765/`. Le site utilise uniquement `index.html` et les images WebP sous `assets/`. Il fonctionne aussi en ouvrant directement `index.html` dans un navigateur moderne. Aucun build ni dépendance JS n'est nécessaire.

## Contenu et sources

- Palette et typographie : `docs/DESIGN.md` ; logo : `res/branding/novydeck-logo.png`.
- Captures issues de `docs/images/novydeck-main.png`, `novydeck-block-editor.png`, `v0111-store-fr.png`, `novydeck-assistant.png`, `redesign-light-fr.png`, `v011-plugin-state-on-fr.png`, `novydeck-media-crop.png` ; les quatre plans rapprochés de « Toute la palette » sont recadrés depuis ces captures réelles, sans nouvelle interface inventée. Affiche issue de `docs/media/novydeck-demo-poster.png`.
- La séquence locale `assets/novydeck-motion.mp4` reprend les secondes 1 à 12, sans son, directement depuis le film source 1920 × 1080 `docs/media/novydeck-demo.mp4`, encodées en H.264 1080p/30 i/s (CRF 18, environ 2,8 Mo). L’ancien extrait 960 × 540 à 123 kb/s provoquait les gros pixels en plein écran ; ne pas l’agrandir ni le réencoder à partir de lui. Le film se charge et se lit à l’entrée de la section, se met en pause hors champ et en onglet masqué. Avec « réduire les animations », lecture automatique désactivée et poster statique, lecture manuelle possible.
- Les fenêtres illustrées ont une profondeur 3D modérée au pointeur et révèlent une information factuelle supplémentaire au survol **et au focus clavier**. Sur écran tactile, cette information apparaît sans survol ; sous « réduire les animations », la perspective est supprimée. Quatre illustrations agrandies appuient les 22 fonctions. Les chapitres apparaissent au défilement ; le module « Dans les détails » comporte trois onglets accessibles au clavier. Ces effets sont décoratifs : contenu et liens restent accessibles si JavaScript ou les animations sont désactivés.
- L'inventaire de 22 fonctions décrit séparément touches, macros, écosystème et contrôles. Les intégrations domotiques sont des **plugins** : elles peuvent dépendre de comptes/services tiers ; les autres modèles de deck ne sont pas déclarés testés.
- Fonctions et limites matérielles : `README.fr.md` de la version 0.11.1. Les captures sont des rendus de l'application hors écran ; aucun périphérique réel n'était nécessaire.
- Bouton principal : installeur officiel 0.11.1 sur GitHub ; bouton secondaire : page `releases/latest` pour choisir notamment le ZIP portable. La vidéo s'ouvre depuis l'asset public de la même version.

## Pour une publication ultérieure

Ne pas publier sans décision sur l'URL/le dépôt d'hébergement. Pour GitHub Pages, choisir la racine `website/` via un workflow Pages dédié ou copier son contenu dans la branche/dossier configuré ; la page utilise des chemins relatifs pour ses images. Tester le chemin réel de publication et le comportement mobile avant d'activer un domaine personnalisé.

Lors de la sortie d'une nouvelle version, mettre à jour **ensemble** la version affichée, le lien direct de l'installateur, le lien vidéo si elle change, les captures et les mentions de compatibilité. `releases/latest` reste le chemin sans numéro de version.
