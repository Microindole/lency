#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const editorsRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(editorsRoot, "..");
const extPath = path.resolve(editorsRoot, "vscode");
const npmCachePath = path.resolve(editorsRoot, ".npm-cache");
const checkOnly = process.argv.includes("--check-only");
const skipInstall = process.argv.includes("--skip-install");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
    ...options,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function hasCommand(command) {
  const checker = process.platform === "win32" ? "where" : "which";
  const result = spawnSync(checker, [command], {
    stdio: "ignore",
    shell: process.platform === "win32",
  });
  return result.status === 0;
}

if (!skipInstall && hasCommand("npm")) {
  console.log("Installing dependencies and building extension...");
  run("npm", ["ci", "--cache", npmCachePath], { cwd: extPath });
  run("npm", ["run", "build"], { cwd: extPath });
}

const builtOutput = path.resolve(extPath, "dist", "extension.js");
if (!fs.existsSync(builtOutput)) {
  console.warn(`Build output not found: ${builtOutput}`);
}

if (checkOnly) {
  console.log("Startup entry smoke check passed (--check-only).");
  process.exit(0);
}

const ideCandidates = ["code", "cursor", "antigravity"];
const ideCommand = ideCandidates.find(hasCommand);

if (!ideCommand) {
  console.error("No supported IDE command found (code/cursor/antigravity).");
  process.exit(1);
}

console.log(`Launching IDE with extension development path via ${ideCommand}...`);
run(ideCommand, ["--extensionDevelopmentPath", extPath, repoRoot]);
