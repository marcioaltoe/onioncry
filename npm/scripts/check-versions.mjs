import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptDir, "..", "..");
const expectedVersion = process.argv[2];
const cargoToml = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const cargoVersion = cargoToml.match(/^version = "([^"]+)"$/m)?.[1];

if (!cargoVersion) {
  fail("could not read Cargo.toml package version");
}

if (expectedVersion && expectedVersion !== cargoVersion) {
  fail(`tag version ${expectedVersion} does not match Cargo.toml version ${cargoVersion}`);
}

const packagePaths = [
  "npm/package.json",
  "npm/platforms/darwin-arm64/package.json",
  "npm/platforms/darwin-x64/package.json",
  "npm/platforms/linux-arm64/package.json",
  "npm/platforms/linux-x64/package.json",
  "npm/platforms/win32-x64/package.json",
];

for (const packagePath of packagePaths) {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, packagePath), "utf8"));
  if (manifest.version !== cargoVersion) {
    fail(`${packagePath} version ${manifest.version} does not match Cargo.toml version ${cargoVersion}`);
  }
}

const mainManifest = JSON.parse(fs.readFileSync(path.join(root, "npm/package.json"), "utf8"));
for (const [name, version] of Object.entries(mainManifest.optionalDependencies ?? {})) {
  if (version !== cargoVersion) {
    fail(`optional dependency ${name}@${version} does not match Cargo.toml version ${cargoVersion}`);
  }
}

function fail(message) {
  console.error(`version check failed: ${message}`);
  process.exit(1);
}
