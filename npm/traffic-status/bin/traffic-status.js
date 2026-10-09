#!/usr/bin/env node
import { spawn } from "node:child_process";
import path from "node:path";
import fs from "node:fs";
import { createRequire } from "node:module";

if (process.platform !== "win32" || process.arch !== "x64") {
  console.error("traffic-status currently supports Windows x64 only");
  process.exit(1);
}

const require = createRequire(import.meta.url);
const DEBUG = process.env.TRAFFIC_STATUS_DEBUG === "1";
const DEV   = process.env.TRAFFIC_STATUS_DEV   === "1";

const PLATFORM_PKG = "@its-jani/traffic-status-win32-x64";
const PLATFORM_BIN = "traffic-status.exe";

function getBinaryPath() {
  // 1. Try resolving via the installed platform package (always attempted first).
  try {
    const pkgJsonPath = require.resolve(`${PLATFORM_PKG}/package.json`);
    const binPath = path.join(path.dirname(pkgJsonPath), PLATFORM_BIN);
    if (fs.existsSync(binPath)) {
      if (DEBUG) console.error(`[traffic-status] resolved via npm package: ${binPath}`);
      return binPath;
    }
  } catch {
    // Platform package not installed — fall through.
  }

  // 2. Local dev fallback: only active when TRAFFIC_STATUS_DEV=1.
  //    Never set for end-users; guards against accidentally shipping a
  //    repo-relative path that resolves to a developer's Cargo build.
  if (DEV) {
    // Script lives at npm/traffic-status/bin/ — three levels up is the repo root.
    const base = path.dirname(new URL(import.meta.url).pathname);
    const devCandidates = [
      path.join(base, "..", "..", "..", "target", "release", PLATFORM_BIN),
      path.join(base, "..", "..", "..", "target", "debug",   PLATFORM_BIN),
    ];
    for (const cand of devCandidates) {
      // On Windows, URL.pathname starts with a leading slash: /D:/...
      const cleanPath = cand.startsWith("\\") ? cand.slice(1) : cand;
      if (fs.existsSync(cleanPath)) {
        if (DEBUG) console.error(`[traffic-status] resolved via TRAFFIC_STATUS_DEV fallback: ${cleanPath}`);
        return cleanPath;
      }
    }
  }

  console.error(`[traffic-status] Error: Could not locate native binary (win32-x64).`);
  console.error(`The optional dependency '${PLATFORM_PKG}' may not have been installed.`);
  console.error(`Reinstall with optional dependencies enabled:`);
  console.error(`  npm install -g traffic-status`);
  process.exit(1);
}

function main() {
  const binaryPath = getBinaryPath();
  const args = process.argv.slice(2);

  const child = spawn(binaryPath, args, {
    stdio: "inherit",
    windowsHide: false,
  });

  child.on("error", (err) => {
    console.error(`[traffic-status] Failed to start native process:`, err);
    process.exit(1);
  });

  child.on("close", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 0);
    }
  });

  // Forward termination signals
  process.on("SIGINT",  () => child.kill("SIGINT"));
  process.on("SIGTERM", () => child.kill("SIGTERM"));
}

main();
