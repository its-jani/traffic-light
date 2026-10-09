#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, "..", "..");
const NPM_DIR = path.resolve(ROOT_DIR, "npm");
const PACKAGES_DIR = path.resolve(NPM_DIR, "packages");

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
];

function machArchName(cputype) {
  if (cputype === 0x01000007) return "x86_64";
  if (cputype === 0x0100000c) return "arm64";
  if (cputype === 7) return "i386";
  if (cputype === 12) return "arm";
  return `cputype-0x${cputype.toString(16)}`;
}

export function inspectBinary(filePath) {
  if (!fs.existsSync(filePath)) {
    return { format: "missing", size: 0, valid: false, error: "File not found" };
  }
  const stat = fs.statSync(filePath);
  if (stat.size === 0) {
    return { format: "empty (0 bytes)", size: 0, valid: false, error: "File is empty" };
  }

  // 64 bytes covers PE/Mach-O fields and ELF e_machine at offset 18
  const buf = Buffer.alloc(64);
  const fd = fs.openSync(filePath, "r");
  fs.readSync(fd, buf, 0, 64, 0);
  fs.closeSync(fd);

  // PE (Windows): 0x4D 0x5A ('MZ')
  if (buf[0] === 0x4d && buf[1] === 0x5a) {
    return { format: "PE32+ (Windows)", arch: "x64", size: stat.size, valid: true, type: "win32" };
  }

  // ELF (Linux): 0x7F 'E' 'L' 'F'; e_machine at offset 18 (little-endian)
  if (buf[0] === 0x7f && buf[1] === 0x45 && buf[2] === 0x4c && buf[3] === 0x46) {
    const is64 = buf[4] === 2;
    const machine = buf.readUInt16LE(18);
    const arch =
      machine === 0x3e ? "x86_64" : machine === 0xb7 ? "aarch64" : `machine-0x${machine.toString(16)}`;
    return {
      format: `ELF ${is64 ? "64-bit" : "32-bit"} (Linux)`,
      arch,
      size: stat.size,
      valid: true,
      type: "linux",
    };
  }

  // Mach-O (macOS)
  const magic32le = buf.readUInt32LE(0);
  const magic32be = buf.readUInt32BE(0);
  if (magic32le === 0xfeedfacf || magic32be === 0xfeedfacf) {
    const cputype =
      magic32le === 0xfeedfacf ? buf.readUInt32LE(4) : buf.readUInt32BE(4);
    return {
      format: "Mach-O 64-bit (macOS)",
      arch: machArchName(cputype),
      size: stat.size,
      valid: true,
      type: "darwin",
    };
  }
  if (magic32le === 0xfeedface || magic32be === 0xfeedface) {
    const cputype =
      magic32le === 0xfeedface ? buf.readUInt32LE(4) : buf.readUInt32BE(4);
    return {
      format: "Mach-O 32-bit (macOS)",
      arch: machArchName(cputype),
      size: stat.size,
      valid: true,
      type: "darwin",
    };
  }
  if (
    magic32be === 0xcafebabe ||
    magic32le === 0xcafebabe ||
    magic32be === 0xbebafeca ||
    magic32le === 0xbebafeca
  ) {
    return { format: "Mach-O Universal/FAT (macOS)", arch: "universal", size: stat.size, valid: true, type: "darwin" };
  }

  return {
    format: `Unknown header (${buf.subarray(0, 4).toString("hex")})`,
    size: stat.size,
    valid: false,
    error: `Invalid magic bytes: ${buf.subarray(0, 4).toString("hex")}`,
  };
}

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

function main() {
  // 1. Start from a clean staging dir — never mix binaries from a previous run
  fs.rmSync(PACKAGES_DIR, { recursive: true, force: true });

  // 2. Read version from Cargo.toml
  const cargoTomlContent = fs.readFileSync(path.join(ROOT_DIR, "Cargo.toml"), "utf-8");
  const versionMatch = cargoTomlContent.match(/version\s*=\s*"([^"]+)"/);
  if (!versionMatch) {
    console.error("❌ Error: Could not determine version from Cargo.toml");
    process.exit(1);
  }
  const VERSION = versionMatch[1];
  console.log(`[npm prepare] Synchronizing packages to version: ${VERSION}`);

  // 3. Parse CLI arguments
  const args = process.argv.slice(2);
  const binDirIdx = args.indexOf("--bin-dir");
  const binDir = binDirIdx !== -1 ? path.resolve(process.cwd(), args[binDirIdx + 1]) : null;

  // 4. Prepare Platform Packages
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

    if (plat.os[0] !== "win32") {
      // npm sets the executable bit on `bin` targets at install time
      // (bin-links/fixBin), so the binary runs even when the tarball was
      // packed on Windows, where files carry no exec bit.
      pkgJson.bin = { "traffic-status-native": `./${plat.bin}` };
    }

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
      console.log(`  ✅ [${plat.name}] Staged ${info.format}${info.arch ? ` ${info.arch}` : ""} (${(info.size / 1024 / 1024).toFixed(2)} MB)`);
    }
  }

  // 5. Update main package.json
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
}

// Run only when executed directly (verify-packages.mjs imports PLATFORMS/inspectBinary
// from this file and must not re-run the staging side effects).
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
