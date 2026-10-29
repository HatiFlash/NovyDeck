# Site NovyDeck 0.16.1

Site vitrine Next.js dans `website/launch/`, exporté statiquement sur GitHub Pages. Le code de l'application Rust n'est pas modifié.

## Développement et vérification

Node.js ≥ 24 : `npm ci && npm run typecheck && npm run build && npm run test:e2e`. Les tests Playwright couvrent 320, 390, 768, 1024 et 1440 px, le clavier, l'accessibilité axe, FR/EN et la réduction des animations. Pour la mesure locale de performance, servir `out/` sur le port 8766 puis lancer `node tests/perf.mjs`. Les résultats locaux ne représentent pas les performances réseau public.

L'export sous le sous-chemin GitHub Pages se construit avec `NEXT_PUBLIC_SITE_BASE_PATH=/NovyDeck npx next build --webpack`. Copier `out/` sous `NovyDeck/` pour le test local de `node tests/pages-smoke.mjs http://127.0.0.1:8767/NovyDeck/`. Sur `gh-pages`, publier le contenu de `out/` à la racine, avec `.nojekyll` indispensable aux ressources `_next/`.

## Composition et véracité

Le héros présente une illustration 3D inspirée du D6, interactive au pointeur et au défilement sur écran ≥ 900 px, y compris sous `prefers-reduced-motion: reduce` (où l'oscillation automatique est désactivée). Sur mobile/tablette, fallback CSS statique. Le modèle et les textures de touches proviennent du projet, pas d'une photographie du périphérique. Diagnostic de la version précédente : sur la page publique en Chromium 1440 px, le canvas existait mais son conteneur mesurait 12 977 px de large (intrinsic sizing WebGL non contenu), hors champ visible ; sous `reduce`, la condition `no-preference` retirait délibérément le canvas. La boucle `frameloop="demand"` ne produisait pas d'animation autonome et l'IntersectionObserver ajoutait un délai de montage. La v2 borne le conteneur, monte le canvas dès l'affichage, anime en douceur hors reduce, et conserve sous reduce seulement la réaction au pointeur.

Les cinq chapitres présentent touches et profils, macros/ON-OFF, plugins, OLED Saver et assistant facultatif. Les captures 0.16.1 françaises `v0161-onoff-cards-fr.webp` et `v0161-plugins-tab-fr.webp` sont des conversions WebP sans perte des PNG sous `docs/images/` vérifiées pixel à pixel avec Pillow. Elles sont montrées également en EN avec alt indiquant explicitement leur langue. Ces captures viennent du harnais Windows hors écran : aucun deck connecté n'a servi à les produire. Les vues historiques sont identifiées par leurs descriptions. Le plugin OLED Saver s'installe séparément ; l'IA locale est facultative et éteinte par défaut. Les plugins chiffrés Elgato ne sont pas pris en charge. Les autres matériels Mirabox ne sont pas déclarés testés. Source : `README.fr.md`, `docs/RELEASE-0.16.1.md`.

Lien direct de l'installateur : `https://github.com/HatiFlash/NovyDeck/releases/download/v0.16.1/NovyDeck-0.16.1-Setup.exe` ; autres fichiers : `https://github.com/HatiFlash/NovyDeck/releases/latest`.
