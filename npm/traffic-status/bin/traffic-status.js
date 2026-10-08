#!/usr/bin/env node
import { spawn } from "node:child_process";
import path from "node:path";
import fs from "node:fs";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

const PLATFORMS = {
  "win32-x64": {
    pkg: "@its-jani/traffic-status-win32-x64",
    bin: "traffic-status.exe",
  },
  "linux-x64": {
    pkg: "@its-jani/traffic-status-linux-x64",
    bin: "traffic-status",
  },
  "darwin-arm64": {
    pkg: "@its-jani/traffic-status-darwin-arm64",
    bin: "traffic-status",
  },
  "darwin-x64": {
    pkg: "@its-jani/traffic-status-darwin-x64",
    bin: "traffic-status",
  },
};

function getBinaryPath() {
  const platformKey = `${process.platform}-${process.arch}`;
  const target = PLATFORMS[platformKey];

  if (!target) {
    console.error(
      `[traffic-status] Error: Unsupported platform/architecture (${platformKey}).`
    );
    console.error(
      `Supported platforms: ${Object.keys(PLATFORMS).join(", ")}`
    );
    process.exit(1);
  }

  // 1. Try resolving via installed platform package
  try {
    const pkgJsonPath = require.resolve(`${target.pkg}/package.json`);
    const binPath = path.join(path.dirname(pkgJsonPath), target.bin);
    if (fs.existsSync(binPath)) {
      return binPath;
    }
  } catch {
    // Package resolution failed
  }

  // 2. Check local fallback (e.g. running from repo or local link)
  const localCandidates = [
    path.join(path.dirname(new URL(import.meta.url).pathname), "..", "..", "target", "release", target.bin),
    path.join(path.dirname(new URL(import.meta.url).pathname), "..", "..", "target", "debug", target.bin),
  ];

  for (const cand of localCandidates) {
    const cleanPath = process.platform === "win32" && cand.startsWith("\\") ? cand.slice(1) : cand;
    if (fs.existsSync(cleanPath)) {
      return cleanPath;
    }
  }

  console.error(
    `[traffic-status] Error: Could not locate native binary for ${platformKey}.`
  );
  console.error(
    `The optional dependency '${target.pkg}' may not have been installed.`
  );
  console.error(
    `If you installed with '--no-optional' or an unsupported package manager, reinstall with optional dependencies enabled:`
  );
  console.error(`  npm install -g traffic-status`);
  process.exit(1);
}

function main() {
  const binaryPath = getBinaryPath();
  const args = process.argv.slice(2);

  // Backstop for tarballs packed on Windows (no exec bit in the archive):
  // ensure the native binary is executable before spawning. The platform
  // package's `bin` field makes npm set this at install time too.
  if (process.platform !== "win32") {
    try {
      fs.chmodSync(binaryPath, 0o755);
    } catch {
      // best effort — spawn below reports a clear error if it still can't run
    }
  }

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
  process.on("SIGINT", () => child.kill("SIGINT"));
  process.on("SIGTERM", () => child.kill("SIGTERM"));
}

main();
