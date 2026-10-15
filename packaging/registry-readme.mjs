import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const REPOSITORY = "theonly1me/vigilante";
const RELATIVE_IMAGE = /src="(?!https?:)([^"]+)"/g;
const RELATIVE_LINK = /\]\((?!https?:|#)([^)]+)\)/g;

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export function registryReadme() {
  const cargoManifest = readFileSync(join(repositoryRoot, "Cargo.toml"), "utf8");
  const [, version] = /^version = "(.+)"$/m.exec(cargoManifest) ?? [];
  if (!version) {
    throw new Error("Cargo.toml has no version");
  }
  const readme = readFileSync(join(repositoryRoot, "README.md"), "utf8");
  if (readme.search(RELATIVE_IMAGE) === -1) {
    throw new Error("README.md has no relative image to rewrite");
  }
  const tag = `v${version}`;
  return readme
    .replace(RELATIVE_IMAGE, `src="https://raw.githubusercontent.com/${REPOSITORY}/${tag}/$1"`)
    .replace(RELATIVE_LINK, `](https://github.com/${REPOSITORY}/blob/${tag}/$1)`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.stdout.write(registryReadme());
}
