import fs from 'node:fs';
import path from 'node:path';
import {root} from './run.mjs';

const shots = ['v017-deck-white-fr', 'v017-deck-color-free-fr', 'v017-activity-collapsed-fr', 'v016-workshop-cards-fr', 'v016-workshop-library-fr', 'v015-assistant-fr', 'v015-appearance-fr', 'v0161-onoff-cards-fr', 'v0161-plugins-tab-fr', 'v014-update-fr'];
const flipbooks = ['drag', 'tabs', 'atelier'].flatMap((n) => Array.from({length:n === 'tabs' ? 13 : 14}, (_, i) => `ui018r-${n}-${String(i).padStart(2, '0')}`));
const required = ['logo.png', 'soundtrack.wav', ...[...shots, ...flipbooks].map(n => `shots/${n}.png`), ...Array.from({length: 15}, (_, n) => `keys/key-${String(n).padStart(2, '0')}.png`), ...Array.from({length: 12}, (_, n) => `keys/key-09-f${String(n).padStart(2, '0')}.png`)];
const missing = required.filter(n => !fs.existsSync(path.join(root, 'public', n)));
if (missing.length) {
  console.error('Ressources absentes (captures NovyDeck indispensables):\n' + missing.map(n => `  public/${n}`).join('\n'));
  process.exit(1);
}
console.log(`Ressources validées : ${required.length} fichiers présents.`);
