import { execFileSync } from "node:child_process";
import { copyFileSync, cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const PLATFORMS = [
  { target: "aarch64-apple-darwin", os: "darwin", cpu: "arm64" },
  { target: "x86_64-apple-darwin", os: "darwin", cpu: "x64" },
  { target: "aarch64-unknown-linux-musl", os: "linux", cpu: "arm64" },
  { target: "x86_64-unknown-linux-musl", os: "linux", cpu: "x64" },
];
const LICENSE_FILES = ["LICENSE-MIT", "LICENSE-APACHE"];
const BINARY = "vigilante";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const mainPackageFolder = join(repositoryRoot, "npm", "vigilante");
const mainManifest = JSON.parse(readFileSync(join(mainPackageFolder, "package.json"), "utf8"));

const [archiveFolder, outputFolder] = process.argv.slice(2).map((folder) => resolve(folder));
if (!archiveFolder || !outputFolder) {
  console.error("usage: node npm/pack.mjs <archive folder> <output folder>");
  process.exit(2);
}

const platformPackageNames = PLATFORMS.map(platformPackageName).sort();
const optionalDependencyNames = Object.keys(mainManifest.optionalDependencies).sort();
if (platformPackageNames.join() !== optionalDependencyNames.join()) {
  console.error(`optionalDependencies must be exactly: ${platformPackageNames.join(", ")}`);
  process.exit(1);
}

const stagingFolder = join(outputFolder, "staging");
rmSync(stagingFolder, { recursive: true, force: true });
mkdirSync(stagingFolder, { recursive: true });
PLATFORMS.forEach(packPlatform);
packMainPackage();
rmSync(stagingFolder, { recursive: true });

function platformPackageName(platform) {
  return `${mainManifest.name}-${platform.os}-${platform.cpu}`;
}

function packPlatform(platform) {
  const folder = join(stagingFolder, `${platform.os}-${platform.cpu}`);
  mkdirSync(folder);
  const archiveRoot = `vigilante-${platform.target}`;
  execFileSync("tar", [
    "-xzf",
    join(archiveFolder, `${archiveRoot}.tar.gz`),
    "-C",
    folder,
    "--strip-components=1",
    `${archiveRoot}/${BINARY}`,
  ]);
  const manifest = {
    name: platformPackageName(platform),
    version: mainManifest.version,
    description: `The ${platform.os} ${platform.cpu} binary for ${mainManifest.name}`,
    license: mainManifest.license,
    repository: mainManifest.repository,
    os: [platform.os],
    cpu: [platform.cpu],
    files: [BINARY, ...LICENSE_FILES],
  };
  writeFileSync(join(folder, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  pack(folder);
}

function packMainPackage() {
  const folder = join(stagingFolder, "main");
  cpSync(mainPackageFolder, folder, { recursive: true });
  pack(folder);
}

function pack(folder) {
  for (const license of LICENSE_FILES) {
    copyFileSync(join(repositoryRoot, license), join(folder, license));
  }
  execFileSync("npm", ["pack", "--pack-destination", outputFolder], {
    cwd: folder,
    stdio: ["ignore", "inherit", "inherit"],
  });
}
