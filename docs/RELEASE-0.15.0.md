# NovyDeck 0.15.0 — Clearer controls and assistant proposals

## English

- **Deck header and library:** the main screen labels key execution as running or paused, and the library names its Templates and Saved sections.
- **Key inspector:** the layout keeps initial action steps visible at compact and HiDPI sizes; Appearance controls remain accessible by scrolling.
- **Assistant entry and conversation:** an empty key offers Ask the assistant, opening Workshop → Assistant. Requests and replies appear in conversation order. Recognized proposals show a block count and structured preview, including nested branches, before the user chooses Add or Apply.
- **Deterministic rules and optional local AI:** built-in deterministic rules work without an AI model. An optional local model can handle requests beyond those rules; it does not imply universal plugin comprehension. Plugin actions/settings are validated, not assumed.

## Français

- **En-tête du deck et bibliothèque :** l'écran principal indique si l'exécution des touches est active ou en pause ; la bibliothèque nomme ses sections Modèles et Enregistrés.
- **Inspecteur de touche :** la disposition garde les premières étapes d'action visibles en petite fenêtre et en HiDPI ; les réglages Apparence restent accessibles par défilement.
- **Accès à l'assistant et conversation :** une touche vide propose Demander à l’IA et ouvre Atelier → Assistant. Demandes et réponses apparaissent dans l'ordre. Les propositions reconnues affichent leur nombre de blocs et un aperçu structuré, avec branches imbriquées, avant le choix Ajouter ou Appliquer.
- **Règles déterministes et IA locale facultative :** les règles déterministes intégrées fonctionnent sans modèle IA. Un modèle local facultatif peut traiter les demandes au-delà de ces règles ; cela ne signifie pas qu'il comprend tous les plugins. Actions et réglages des plugins sont validés, pas supposés.

## Qualification

The READMEs include eight 0.15.0 screenshots generated with the off-screen Windows UI harness at 1366 × 768, four dark-theme views in each language. Additional light-theme views were generated and reviewed. Targeted checks reported BUILD=0 and TESTBUILD=0; `T_main015_captures=0`, `T_main015_` ran 7 tests, `T_assistant015_` ran 3 tests, and `T_i18n::` ran 3 tests. These results cover the named harness checks only. The English proposal screenshot contains a French example request from an offline fixture; the clarification screenshots use an offline SoundPad fixture, not an installed plugin. Neither D6 hardware nor a particular local AI model was exercised.

Les README comprennent huit captures 0.15.0 produites avec le harnais UI Windows hors écran à 1366 × 768 : quatre vues en thème sombre dans chaque langue. D'autres vues en thème clair ont été produites et relues. Les contrôles ciblés ont indiqué BUILD=0 et TESTBUILD=0 ; `T_main015_captures=0`, `T_main015_` a exécuté 7 tests, `T_assistant015_` en a exécuté 3 et `T_i18n::` en a exécuté 3. Ces résultats ne couvrent que les contrôles nommés du harnais. La capture de proposition anglaise contient une demande d'exemple en français issue d'une fixture hors ligne ; les captures de clarification utilisent une fixture SoundPad hors ligne, pas un plugin installé. Ni le D6 physique ni un modèle IA local particulier n'ont été exercés.
