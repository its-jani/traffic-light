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
  console.error("❌ Error: Could not determine version from Cargo.toml");
  process.exit(1);
}
const VERSION = versionMatch[1];
console.log(`[npm prepare] Synchronizing packages to version: ${VERSION}`);

export const PLATFORMS = [
  {
    name: "traffic-status-win32-x64",
    scopedName: "@its-jani/traffic-status-win32-x64",
    os: ["win32"],
    cpu: ["x64"],
    bin: "traffic-status.exe",
    platformId: "win32-x64",
    targetTriple: "x86_64-pc-windows-msvc",
    expectedType: "win32",
  },
  {
    name: "traffic-status-linux-x64",
    scopedName: "@its-jani/traffic-status-linux-x64",
    os: ["linux"],
    cpu: ["x64"],
    bin: "traffic-status",
    platformId: "linux-x64",
    targetTriple: "x86_64-unknown-linux-gnu",
    expectedType: "linux",
  },
  {
    name: "traffic-status-darwin-arm64",
    scopedName: "@its-jani/traffic-status-darwin-arm64",
    os: ["darwin"],
    cpu: ["arm64"],
    bin: "traffic-status",
    platformId: "darwin-arm64",
    targetTriple: "aarch64-apple-darwin",
    expectedType: "darwin",
  },
  {
    name: "traffic-status-darwin-x64",
    scopedName: "@its-jani/traffic-status-darwin-x64",
    os: ["darwin"],
    cpu: ["x64"],
    bin: "traffic-status",
    platformId: "darwin-x64",
    targetTriple: "x86_64-apple-darwin",
    expectedType: "darwin",
  },
];

export function inspectBinary(filePath) {
  if (!fs.existsSync(filePath)) {
    return { format: "missing", size: 0, valid: false, error: "File not found" };
  }
  const stat = fs.statSync(filePath);
  if (stat.size === 0) {
    return { format: "empty (0 bytes)", size: 0, valid: false, error: "File is empty" };
  }

  const buf = Buffer.alloc(16);
  const fd = fs.openSync(filePath, "r");
  fs.readSync(fd, buf, 0, 16, 0);
  fs.closeSync(fd);

  // PE (Windows): 0x4D 0x5A ('MZ')
  if (buf[0] === 0x4d && buf[1] === 0x5a) {
    return { format: "PE32+ (Windows)", size: stat.size, valid: true, type: "win32" };
  }

  // ELF (Linux): 0x7F 'E' 'L' 'F'
  if (buf[0] === 0x7f && buf[1] === 0x45 && buf[2] === 0x4c && buf[3] === 0x46) {
    const is64 = buf[4] === 2;
    return { format: `ELF ${is64 ? "64-bit" : "32-bit"} (Linux)`, size: stat.size, valid: true, type: "linux" };
  }

  // Mach-O (macOS)
  const magic32le = buf.readUInt32LE(0);
  const magic32be = buf.readUInt32BE(0);
  if (magic32le === 0xfeedfacf || magic32be === 0xfeedfacf) {
    return { format: "Mach-O 64-bit (macOS)", size: stat.size, valid: true, type: "darwin" };
  }
  if (magic32le === 0xfeedface || magic32be === 0xfeedface) {
    return { format: "Mach-O 32-bit (macOS)", size: stat.size, valid: true, type: "darwin" };
  }
  if (
    magic32be === 0xcafebabe ||
    magic32le === 0xcafebabe ||
    magic32be === 0xbebafeca ||
    magic32le === 0xbebafeca
  ) {
    return { format: "Mach-O Universal/FAT (macOS)", size: stat.size, valid: true, type: "darwin" };
  }

  return {
    format: `Unknown header (${buf.subarray(0, 4).toString("hex")})`,
    size: stat.size,
    valid: false,
    error: `Invalid magic bytes: ${buf.subarray(0, 4).toString("hex")}`,
  };
}

// 2. Parse CLI arguments
const args = process.argv.slice(2);
const binDirIdx = args.indexOf("--bin-dir");
const binDir = binDirIdx !== -1 ? path.resolve(process.cwd(), args[binDirIdx + 1]) : null;

// Helper to locate candidate source binary in provided binDir
function findSourceBinary(baseDir, plat) {
  if (!baseDir) return null;

  const candidates = [
    // 1. Folder by platformId: e.g. binDir/win32-x64/traffic-status.exe
    path.join(baseDir, plat.platformId, plat.bin),
    // 2. Folder by targetTriple: e.g. binDir/x86_64-pc-windows-msvc/traffic-status.exe
    path.join(baseDir, plat.targetTriple, plat.bin),
    // 3. Extracted archive folder: e.g. binDir/traffic-status-x86_64-pc-windows-msvc/traffic-status.exe
    path.join(baseDir, `traffic-status-${plat.targetTriple}`, plat.bin),
    // 4. Suffix naming: e.g. binDir/traffic-status-win32-x64.exe
    path.join(baseDir, `traffic-status-${plat.platformId}${plat.os[0] === "win32" ? ".exe" : ""}`),
    // 5. Direct file if only single platform directory
    path.join(baseDir, plat.bin),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

// 3. Prepare Platform Packages
fs.mkdirSync(PACKAGES_DIR, { recursive: true });

let errors = 0;

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

  // Copy LICENSE and README
  const licensePath = path.join(ROOT_DIR, "LICENSE");
  if (fs.existsSync(licensePath)) {
    fs.copyFileSync(licensePath, path.join(pkgDir, "LICENSE"));
  }

  fs.writeFileSync(
    path.join(pkgDir, "README.md"),
    `# ${plat.scopedName}\n\nPrebuilt native binary for \`traffic-status\` on ${plat.os[0]} (${plat.cpu[0]}).\n`
  );

  if (binDir) {
    const srcBin = findSourceBinary(binDir, plat);
    const destBin = path.join(pkgDir, plat.bin);

    if (!srcBin) {
      console.error(`❌ [${plat.name}] Binary not found in ${binDir}`);
      errors++;
      continue;
    }

    const info = inspectBinary(srcBin);
    if (!info.valid) {
      console.error(`❌ [${plat.name}] Invalid binary at ${srcBin}: ${info.error} (${info.format})`);
      errors++;
      continue;
    }

    if (info.type !== plat.expectedType) {
      console.error(
        `❌ [${plat.name}] Format mismatch at ${srcBin}: expected ${plat.expectedType}, got ${info.type} (${info.format})`
      );
      errors++;
      continue;
    }

    fs.copyFileSync(srcBin, destBin);
    if (plat.os[0] !== "win32") {
      fs.chmodSync(destBin, 0o755);
    }
    console.log(`  ✅ [${plat.name}] Staged ${info.format} (${(info.size / 1024 / 1024).toFixed(2)} MB)`);
  }
}

// 4. Update main package.json
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

if (binDir && errors > 0) {
  console.error(`\n❌ Failed to prepare platform packages: ${errors} error(s) encountered.`);
  process.exit(1);
}

console.log("✨ Prepare completed successfully.");
