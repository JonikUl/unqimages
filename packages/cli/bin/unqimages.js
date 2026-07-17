#!/usr/bin/env node

import { run } from '../dist/index.js';

run().catch((error) => {
  console.error(error);
  process.exit(2);
});
