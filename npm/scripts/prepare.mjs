#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, "..", "..");
const NPM_DIR = path.resolve(ROOT_DIR, "npm");
const PACKAGES_DIR = path.resolve(NPM_DIR, "packages");

// 1. Read version from Cargo.toml
const cargoTomlContent = fs.readFileSync(path.join(ROOT_DIR, "Cargo.toml"), "utf-8");
const versionMatch = cargoTomlContent.match(/version\s*=\s*"([^"]+)"/);
if (!versionMatch) {
  console.error("Error: Could not determine version from Cargo.toml");
  process.exit(1);
}
const VERSION = versionMatch[1];
console.log(`[npm prepare] Synchronizing packages to version: ${VERSION}`);

const PLATFORMS = [
  {
    name: "traffic-status-win32-x64",
    scopedName: "@its-jani/traffic-status-win32-x64",
    os: ["win32"],
    cpu: ["x64"],
    bin: "traffic-status.exe",
    platformId: "win32-x64",
  },
  {
    name: "traffic-status-linux-x64",
    scopedName: "@its-jani/traffic-status-linux-x64",
    os: ["linux"],
    cpu: ["x64"],
    bin: "traffic-status",
    platformId: "linux-x64",
  },
  {
    name: "traffic-status-darwin-arm64",
    scopedName: "@its-jani/traffic-status-darwin-arm64",
    os: ["darwin"],
    cpu: ["arm64"],
    bin: "traffic-status",
    platformId: "darwin-arm64",
  },
  {
    name: "traffic-status-darwin-x64",
    scopedName: "@its-jani/traffic-status-darwin-x64",
    os: ["darwin"],
    cpu: ["x64"],
    bin: "traffic-status",
    platformId: "darwin-x64",
  },
];

// Check CLI args for --bin-dir
const args = process.argv.slice(2);
const binDirIdx = args.indexOf("--bin-dir");
const binDir = binDirIdx !== -1 ? path.resolve(process.cwd(), args[binDirIdx + 1]) : null;

// 2. Prepare Platform Packages
fs.mkdirSync(PACKAGES_DIR, { recursive: true });

for (const plat of PLATFORMS) {
  const pkgDir = path.join(PACKAGES_DIR, plat.name);
  fs.mkdirSync(pkgDir, { recursive: true });

  const pkgJson = {
    name: plat.scopedName,
    version: VERSION,
    description: `Native prebuilt binary for traffic-status on ${plat.os[0]} (${plat.cpu[0]})`,
    os: plat.os,
    cpu: plat.cpu,
    files: [plat.bin, "LICENSE", "README.md"],
    license: "MIT",
    author: "Traffic Status Team",
    repository: {
      type: "git",
      url: "https://github.com/its-jani/traffic-status.git",
    },
  };

  fs.writeFileSync(
    path.join(pkgDir, "package.json"),
    JSON.stringify(pkgJson, null, 2) + "\n"
  );

  // Copy LICENSE and minimal README
  const licensePath = path.join(ROOT_DIR, "LICENSE");
  if (fs.existsSync(licensePath)) {
    fs.copyFileSync(licensePath, path.join(pkgDir, "LICENSE"));
  }

  fs.writeFileSync(
    path.join(pkgDir, "README.md"),
    `# ${plat.scopedName}\n\nPrebuilt native binary for \`traffic-status\` on ${plat.os[0]} (${plat.cpu[0]}).\n`
  );

  // Copy binary if available
  if (binDir) {
    const srcBin = path.join(binDir, plat.platformId, plat.bin);
    const destBin = path.join(pkgDir, plat.bin);
    if (fs.existsSync(srcBin)) {
      fs.copyFileSync(srcBin, destBin);
      if (plat.os[0] !== "win32") {
        fs.chmodSync(destBin, 0o755);
      }
      console.log(`  ✅ Copied ${srcBin} -> ${destBin}`);
    } else {
      console.warn(`  ⚠️ Binary not found at ${srcBin}`);
    }
  }
}

// 3. Update main package.json
const mainPkgJsonPath = path.join(NPM_DIR, "traffic-status", "package.json");
const mainPkgJson = JSON.parse(fs.readFileSync(mainPkgJsonPath, "utf-8"));
mainPkgJson.version = VERSION;

const optionalDeps = {};
for (const plat of PLATFORMS) {
  optionalDeps[plat.scopedName] = VERSION;
}
mainPkgJson.optionalDependencies = optionalDeps;

fs.writeFileSync(mainPkgJsonPath, JSON.stringify(mainPkgJson, null, 2) + "\n");
console.log(`[npm prepare] Updated ${mainPkgJsonPath}`);
