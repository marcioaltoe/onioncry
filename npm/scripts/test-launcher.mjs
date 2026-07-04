import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptDir, "..", "..");
const platformPackage = platformPackageName();
const packageRoot = path.join(root, "npm", "node_modules", ...platformPackage.split("/"));
const binDir = path.join(packageRoot, "bin");
const binaryName = process.platform === "win32" ? "onioncry.exe" : "onioncry";
const binaryPath = path.join(binDir, binaryName);

fs.rmSync(path.join(root, "npm", "node_modules"), { recursive: true, force: true });
fs.mkdirSync(binDir, { recursive: true });
fs.writeFileSync(
  path.join(packageRoot, "package.json"),
  JSON.stringify({ name: platformPackage, version: "0.0.0" }),
);
fs.writeFileSync(binaryPath, shimSource());
if (process.platform !== "win32") {
  fs.chmodSync(binaryPath, 0o755);
}

const result = spawnSync(process.execPath, [path.join(root, "npm", "bin", "onioncry.js"), "--sentinel"], {
  encoding: "utf8",
});

fs.rmSync(path.join(root, "npm", "node_modules"), { recursive: true, force: true });

if (result.status !== 7) {
  console.error(result.stdout);
  console.error(result.stderr);
  throw new Error(`expected launcher to preserve exit code 7, got ${result.status}`);
}

if (!result.stdout.includes("args:--sentinel")) {
  console.error(result.stdout);
  throw new Error("expected launcher to forward arguments and stdout");
}

const missingBinary = spawnSync(
  process.execPath,
  [path.join(root, "npm", "bin", "onioncry.js"), "--help"],
  { encoding: "utf8" },
);

if (missingBinary.status !== 2) {
  console.error(missingBinary.stderr);
  throw new Error(
    `expected launcher to exit 2 when the platform package is missing, got ${missingBinary.status}`,
  );
}

if (!missingBinary.stderr.includes(platformPackage)) {
  console.error(missingBinary.stderr);
  throw new Error("expected launcher error to name the missing platform package");
}

function platformPackageName() {
  const packages = {
    "darwin-arm64": "@onioncry/cli-darwin-arm64",
    "darwin-x64": "@onioncry/cli-darwin-x64",
    "linux-arm64": "@onioncry/cli-linux-arm64",
    "linux-x64": "@onioncry/cli-linux-x64",
    "win32-x64": "@onioncry/cli-win32-x64",
  };
  const key = `${process.platform}-${process.arch}`;
  const packageName = packages[key];
  if (!packageName) {
    throw new Error(`test host ${key} is not supported`);
  }
  return packageName;
}

function shimSource() {
  if (process.platform === "win32") {
    return `@echo off\r\necho args:%*\r\nexit /b 7\r\n`;
  }
  return `#!/bin/sh\necho "args:$*"\nexit 7\n`;
}
