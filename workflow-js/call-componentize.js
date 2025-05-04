import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { componentize } from '@bytecodealliance/componentize-js';

const jsSource = await readFile('src/workflow.js', 'utf8');

const { component } = await componentize(jsSource, {
  debugBindings: false,
  witPath: resolve('wit'),
  enableAot: false,
  disableFeatures: ['stdio', 'clocks', 'http', 'random'],
  enableWizerLogging: true,
  wizerBin: '/nix/store/q3rv4iakgqsvy30jx2qj7mybkczsw2dx-wizer-8.0.0/bin/wizer' // FIXME
});

await writeFile('dist/workflow-js.wasm', component);
