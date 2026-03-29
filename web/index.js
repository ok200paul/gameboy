import init, { mount } from '../pkg/gameboy.js';

async function main() {
  await init();
  mount('game');
}

main().catch(console.error);
