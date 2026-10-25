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
- Captures issues de `docs/images/novydeck-main.png`, `redesign-light-fr.png`, `novydeck-block-editor.png`, `v0111-store-fr.png` ; affiche issue de `docs/media/novydeck-demo-poster.png`.
- Fonctions et limites matérielles : `README.fr.md` de la version 0.11.1. Les captures sont des rendus de l'application hors écran ; aucun périphérique réel n'était nécessaire.
- Bouton principal : installeur officiel 0.11.1 sur GitHub ; bouton secondaire : page `releases/latest` pour choisir notamment le ZIP portable. La vidéo s'ouvre depuis l'asset public de la même version.

## Pour une publication ultérieure

Ne pas publier sans décision sur l'URL/le dépôt d'hébergement. Pour GitHub Pages, choisir la racine `website/` via un workflow Pages dédié ou copier son contenu dans la branche/dossier configuré ; la page utilise des chemins relatifs pour ses images. Tester le chemin réel de publication et le comportement mobile avant d'activer un domaine personnalisé.

Lors de la sortie d'une nouvelle version, mettre à jour **ensemble** la version affichée, le lien direct de l'installateur, le lien vidéo si elle change, les captures et les mentions de compatibilité. `releases/latest` reste le chemin sans numéro de version.
