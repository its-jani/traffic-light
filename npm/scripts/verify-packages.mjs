#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { PLATFORMS, inspectBinary } from "./prepare.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, "..", "..");
const NPM_DIR = path.resolve(ROOT_DIR, "npm");
const PACKAGES_DIR = path.resolve(NPM_DIR, "packages");

console.log("🔍 Verifying npm Platform & Root Packages");
console.log("=========================================");

let errors = 0;

// 1. Verify Platform Packages
for (const plat of PLATFORMS) {
  const pkgDir = path.join(PACKAGES_DIR, plat.name);
  const pkgJsonPath = path.join(pkgDir, "package.json");
  const binPath = path.join(pkgDir, plat.bin);

  console.log(`\n📦 Package: ${plat.scopedName}`);

  if (!fs.existsSync(pkgJsonPath)) {
    console.error(`  ❌ package.json missing in ${pkgDir}`);
    errors++;
    continue;
  }

  const pkgJson = JSON.parse(fs.readFileSync(pkgJsonPath, "utf-8"));
  console.log(`  • Version:       ${pkgJson.version}`);
  console.log(`  • Target OS:     ${JSON.stringify(pkgJson.os)}`);
  console.log(`  • Target CPU:    ${JSON.stringify(pkgJson.cpu)}`);

  if (!fs.existsSync(binPath)) {
    console.warn(`  ⚠️ Binary missing: ${binPath} (run prepare.mjs --bin-dir <dir>)`);
    errors++;
    continue;
  }

  const info = inspectBinary(binPath);
  console.log(`  • Binary Name:   ${plat.bin}`);
  console.log(`  • Binary Size:   ${(info.size / 1024 / 1024).toFixed(2)} MB (${info.size.toLocaleString()} bytes)`);
  console.log(`  • Binary Format: ${info.format}`);

  if (!info.valid) {
    console.error(`  ❌ Invalid binary: ${info.error}`);
    errors++;
  } else if (info.type !== plat.expectedType) {
    console.error(`  ❌ Platform mismatch: expected ${plat.expectedType}, detected ${info.type}`);
    errors++;
  } else {
    console.log(`  ✅ Binary verification passed.`);
  }
}

// 2. Verify Main Package
console.log(`\n📦 Root Package: traffic-status`);
const mainPkgJsonPath = path.join(NPM_DIR, "traffic-status", "package.json");
if (!fs.existsSync(mainPkgJsonPath)) {
  console.error(`  ❌ package.json missing in ${path.dirname(mainPkgJsonPath)}`);
  errors++;
} else {
  const mainPkgJson = JSON.parse(fs.readFileSync(mainPkgJsonPath, "utf-8"));
  console.log(`  • Version:       ${mainPkgJson.version}`);
  console.log(`  • Launcher Bin:  ${JSON.stringify(mainPkgJson.bin)}`);
  console.log(`  • Optional Dependencies:`);
  for (const [dep, ver] of Object.entries(mainPkgJson.optionalDependencies || {})) {
    console.log(`    - ${dep}: ${ver}`);
  }
  const launcherPath = path.join(NPM_DIR, "traffic-status", "bin", "traffic-status.js");
  if (fs.existsSync(launcherPath)) {
    console.log(`  ✅ Launcher script found (${launcherPath})`);
  } else {
    console.error(`  ❌ Launcher script missing: ${launcherPath}`);
    errors++;
  }
}

console.log("\n=========================================");
if (errors > 0) {
  console.error(`❌ Verification failed with ${errors} error(s).`);
  process.exit(1);
} else {
  console.log("✨ All 5 packages verified successfully!");
}
