const { transformSync } = require('@swc/core');
const path = require('path');

const pluginPath = path.resolve(__dirname, './target/wasm32-wasi/release/swc_plugin_preeval.wasm');

const code = `
import {preeval} from 'swc-plugin-preeval/preeval';

const x = preeval\`1+1\`;
const y = preeval\`"hello " + "world"\`;
`;

const result = transformSync(code, {
  jsc: {
    experimental: {
      plugins: [
        [pluginPath, {}]
      ]
    }
  }
});

console.log(result);