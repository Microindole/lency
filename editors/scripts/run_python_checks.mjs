#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
    env: {
      ...process.env,
      PYTHONUTF8: "1",
    },
    ...options,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function canRun(command, args) {
  const result = spawnSync(command, args, {
    stdio: "ignore",
    shell: process.platform === "win32",
  });
  return result.status === 0;
}

const python = canRun("python", ["--version"])
  ? "python"
  : canRun("python3", ["--version"])
    ? "python3"
    : null;

if (!python) {
  console.error("Python not found. Please install python or python3.");
  process.exit(1);
}

const checks = [
  path.resolve(__dirname, "check_todos.py"),
  path.resolve(__dirname, "check_file_size.py"),
  path.resolve(__dirname, "check_banned_patterns.py"),
];

for (const check of checks) {
  run(python, [check], { cwd: path.resolve(__dirname, "..") });
}
