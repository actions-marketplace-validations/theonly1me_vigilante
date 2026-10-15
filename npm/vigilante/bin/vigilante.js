#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");

const PLATFORM_PACKAGES = {
  "darwin arm64": "@theonly1me/vigilante-darwin-arm64",
  "darwin x64": "@theonly1me/vigilante-darwin-x64",
  "linux arm64": "@theonly1me/vigilante-linux-arm64",
  "linux x64": "@theonly1me/vigilante-linux-x64",
};
const ERROR_EXIT_CODE = 2;

function findBinary() {
  const platform = `${process.platform} ${process.arch}`;
  const platformPackage = PLATFORM_PACKAGES[platform];
  if (!platformPackage) {
    return { error: `there is no Vigilante binary for ${platform}` };
  }
  try {
    return { path: require.resolve(`${platformPackage}/vigilante`) };
  } catch {
    return {
      error: `${platformPackage} is not installed. Install @theonly1me/vigilante again, with optional dependencies.`,
    };
  }
}

function exitWithError(message) {
  console.error(`vigilante: ${message}`);
  process.exit(ERROR_EXIT_CODE);
}

const binary = findBinary();
if (binary.error) {
  exitWithError(binary.error);
}
const run = spawnSync(binary.path, process.argv.slice(2), { stdio: "inherit" });
if (run.error) {
  exitWithError(run.error.message);
}
process.exit(run.status ?? ERROR_EXIT_CODE);
