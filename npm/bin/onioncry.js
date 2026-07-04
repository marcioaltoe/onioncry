#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const path = require("node:path");

const supportedPackages = {
  "darwin-arm64": "@onioncry/cli-darwin-arm64",
  "darwin-x64": "@onioncry/cli-darwin-x64",
  "linux-arm64": "@onioncry/cli-linux-arm64",
  "linux-x64": "@onioncry/cli-linux-x64",
  "win32-x64": "@onioncry/cli-win32-x64",
};

const platformKey = `${process.platform}-${process.arch}`;
const packageName = supportedPackages[platformKey];

if (!packageName) {
  fail(
    `No OnionCry prebuilt binary is available for ${platformKey}.\n` +
      `Supported platforms: ${Object.keys(supportedPackages).join(", ")}`
  );
}

const binaryName = process.platform === "win32" ? "onioncry.exe" : "onioncry";
let binaryPath;

try {
  const packageJsonPath = require.resolve(`${packageName}/package.json`);
  binaryPath = path.join(path.dirname(packageJsonPath), "bin", binaryName);
} catch (error) {
  fail(
    `OnionCry package ${packageName} is not installed for ${platformKey}.\n` +
      `Supported platforms: ${Object.keys(supportedPackages).join(", ")}\n` +
      "Reinstall onioncry so npm can install its optional platform dependency."
  );
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  fail(`Failed to execute OnionCry binary at ${binaryPath}: ${result.error.message}`);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 1);

function fail(message) {
  console.error(`onioncry: ${message}`);
  process.exit(1);
}
