import init, { activate } from '../pkg/gameboy.js';

async function main() {
  await init();

  // License JSON would be provided by the hosting platform
  const licenseJson = window.__RUNLICENSE_JSON || '';
  activate(licenseJson, 'game');
}

main().catch(console.error);
