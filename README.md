## swc-plugin-preeval

A plugin for SWC that evaluates codes at a compile time. This is not for production use; more of a proof of concept or a demonstration purpose. For those reason this is not published to npm and crates.io.

Say you have a code like this:

```js
import {preeval} from 'swc-plugin-preeval/preeval';

const x = preeval\`1+1\`;
const y = preeval\`"hello " + "world"\`;
```

Transformed codes with this plugin will be:

```js
var x = 2;
var y = "hello world";
```

By plugin runs, evaluate codes inside of `preeval` template tag. It is backed by [boa](https://github.com/boa-dev/boa) engine, which is a JavaScript interpreter written in Rust.

Since plugin only embeds JS interpreter but not the runtime, this is pretty much useless. For example, you can't have module imports, or any runtime apis from node.js or browsers.
