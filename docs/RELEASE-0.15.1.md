# NovyDeck 0.15.1 — ON/OFF actions in the main inspector and Workshop block menus

## English

- **ON/OFF without the Workshop:** select a key, open **Action**, then enable **ON/OFF actions**. The key's existing short-press steps become **Action 1**; select **Action 2** and choose its shortcut or other action in the same inspector. Alternate presses run Action 1 then Action 2. Advanced sequences remain available in the Workshop. Appearance's two-state display controls only what is shown on the key, not the actions.
- **Preserved configuration:** conversion retains the original steps and long-press behavior, and updates plugin-state references when blocks move. Root navigation and attached plugins cannot be converted to this simplified ON/OFF form; navigation and nested toggles are excluded from its action picker.
- **Workshop block menus:** the **…** menu on root and nested blocks now opens reliably, including **Delete** for the selected block. Selection no longer intercepts clicks on the menu or drag handle.

## Français

- **Actions ON/OFF sans passer par l'Atelier :** sélectionner une touche, ouvrir **Action**, puis activer **Actions ON/OFF**. Les étapes actuelles de l'appui court deviennent **Action 1** ; choisir **Action 2** et son raccourci ou une autre action dans le même inspecteur. Les appuis alternent ensuite entre les deux actions. L'Atelier reste disponible pour les séquences avancées. L'affichage à deux états dans Apparence ne règle que le visuel de la touche, pas ses actions.
- **Configuration préservée :** la conversion conserve les étapes d'origine et l'appui long, et adapte les références d'état de plugin quand les blocs changent de place. La navigation racine et les plugins associés ne sont pas convertibles sous cette forme simplifiée ; la navigation et les bascules imbriquées sont exclues de son catalogue.
- **Menus des blocs de l'Atelier :** le menu **…** des blocs racine et imbriqués s'ouvre désormais correctement, y compris **Supprimer** pour le bloc choisi. La sélection n'intercepte plus les clics sur le menu ou la poignée de déplacement.

## Qualification / Validation

These fixes were exercised with automated egui mouse-event tests in the Windows off-screen harness; the screenshots are simulated UI captures. They do not constitute a test on a physical FIFINE D6 or of any third-party application triggered by a shortcut.

Ces corrections ont été vérifiées par des tests de clics simulés dans le harnais egui hors écran sur Windows. Les captures représentent cette interface simulée : elles ne prouvent pas un essai sur un FIFINE D6 physique ni sur une application tierce commandée par raccourci.
