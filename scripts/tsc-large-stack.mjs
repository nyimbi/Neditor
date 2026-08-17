#!/usr/bin/env node
/**
 * Runs tsc with an enlarged Node.js call stack.
 *
 * happy-dom's Window type has circular self-references that cause TypeScript
 * 5.6's flow-type resolver to blow the default 8 MB stack.  Increasing to
 * 64 MB allows the compilation to complete.  This shim resolves `tsc.js`
 * via the local `typescript` package so it tracks whatever version is pinned
 * in package.json without requiring a hard-coded path.
 *
 * Usage: node scripts/tsc-large-stack.mjs [tsc options...]
 */
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";

const _require = createRequire(import.meta.url);
const tscPath  = _require.resolve("typescript/lib/tsc");

execFileSync(
  process.execPath,
  ["--stack-size=65536", tscPath, ...process.argv.slice(2)],
  { stdio: "inherit" },
);
