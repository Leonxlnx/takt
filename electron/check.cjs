const fs = require('node:fs');
const path = require('node:path');

const required = [
  'electron/main.cjs',
  'electron/preload.cjs',
  'electron/renderer/index.html',
  'electron/renderer/styles.css',
  'electron/renderer/app.js'
];

for (const file of required) {
  const full = path.resolve(__dirname, '..', file);
  if (!fs.existsSync(full)) {
    console.error(`Missing ${file}`);
    process.exit(1);
  }
}

console.log('Electron app files ok');
