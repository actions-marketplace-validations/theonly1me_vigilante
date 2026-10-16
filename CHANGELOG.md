# Changelog

## [0.1.1](https://github.com/theonly1me/vigilante/compare/v0.1.0...v0.1.1) (2026-10-16)


### Fixes

* **deps:** update zip to 9.0.3, because 9.0.0 to 9.0.2 are yanked ([7ce70b3](https://github.com/theonly1me/vigilante/commit/7ce70b325c0f928c5f508a8a4d59b822c860031e))

## 0.1.0 (2026-10-15)


### Features

* **action:** cache downloaded packages between runs ([2b7e6f5](https://github.com/theonly1me/vigilante/commit/2b7e6f50232277908f6812c0f131c9be4ce362d5))
* **action:** install Go and Capslock for repositories with Go modules ([a796ca1](https://github.com/theonly1me/vigilante/commit/a796ca17fffc8e98288052e35f42741fa110cee1))
* **action:** run vigilante from a GitHub Action ([db8cc9a](https://github.com/theonly1me/vigilante/commit/db8cc9a6c2b2fd30f7f58ff1266b51429a9564b4))
* **agent:** find powers in Claude, MCP, and Codex config ([c6a17c3](https://github.com/theonly1me/vigilante/commit/c6a17c3b460ef4ba7d44d4c89203a886371236d2))
* **agent:** report only allow rules that run any code or read open content ([602d50c](https://github.com/theonly1me/vigilante/commit/602d50cba187430c9e88daca90353de5d3ed7ab0))
* **build:** find install hooks and git hooks ([a2cfe33](https://github.com/theonly1me/vigilante/commit/a2cfe3396f2320bf2941bfc8a68b74d8bd2baea3))
* **build:** skip hooks that only run plain local commands ([b2be0eb](https://github.com/theonly1me/vigilante/commit/b2be0eb77e177fe4d61f6f6115d366d559c255fd))
* **ci:** find new powers in GitHub workflows ([5b4080c](https://github.com/theonly1me/vigilante/commit/5b4080c4e6fe425392e1e44e2d09723c5d37a612))
* **ci:** scope actions to all workflows and trust configured owners ([32d329d](https://github.com/theonly1me/vigilante/commit/32d329dda03e7c66a3ebcb18facc3c0f1a6605f3))
* **cli:** add scan, lock, diff, and check commands ([0cc79a5](https://github.com/theonly1me/vigilante/commit/0cc79a5efc5adb11359ec3f79faacf6a070d2a4f))
* **code:** add the ast-grep engine and 7 language packs ([8260388](https://github.com/theonly1me/vigilante/commit/82603889c06dbc59934af4df294de940d40fc0aa))
* **code:** apply rule file globs and skip native in generated protobuf Go ([64f9aa4](https://github.com/theonly1me/vigilante/commit/64f9aa437f57592baa6bd70afe991d7c891610cb))
* **code:** find file access, system state changes, and system calls in every pack ([3a8ceb3](https://github.com/theonly1me/vigilante/commit/3a8ceb3b90c271080a5763c2a218ab2e43515388))
* **config:** read vigilante.toml and find package-root scopes ([d2dee14](https://github.com/theonly1me/vigilante/commit/d2dee1409e744fd4d4d6da9d2315f5506cfeb055))
* **config:** skip test project folders such as App.Tests ([6cec0de](https://github.com/theonly1me/vigilante/commit/6cec0de131e7a7daa7ac2a988e530b4d4a2abadf))
* **csharp:** dispatch calls through the class hierarchy ([0ff9248](https://github.com/theonly1me/vigilante/commit/0ff92481330676dda5fadaa1252dceb8449c6c2c))
* **csharp:** follow calls through own code and NuGet packages ([8b1654c](https://github.com/theonly1me/vigilante/commit/8b1654c5644ea39cdbfbf28b0545eebdb048a751))
* **deep:** report a power reached through another own scope as a note ([9b6eb44](https://github.com/theonly1me/vigilante/commit/9b6eb44d57074084b5c36f360dcd4620fe7970da))
* **go:** block only on exec, network, and native in Go ([4361576](https://github.com/theonly1me/vigilante/commit/43615762df526364d5e3b886b11735729a4fd738))
* **go:** block only on powers that a package reaches through its own code ([5f5a368](https://github.com/theonly1me/vigilante/commit/5f5a3687588bcf49c761d5b8928ff4b166a8bb91))
* **go:** find Go capabilities through dependencies with Capslock ([8b397b8](https://github.com/theonly1me/vigilante/commit/8b397b84275d162dc6b27df6d726eb2fbfc55071))
* **java:** dispatch calls through the class hierarchy ([6dcfd16](https://github.com/theonly1me/vigilante/commit/6dcfd161905dbd9816bddd3effc27280b57e97ec))
* **java:** follow calls through own code and Maven and Gradle packages ([83b16e3](https://github.com/theonly1me/vigilante/commit/83b16e3cfcfa0bd1063e47de58551ca28b4b4661))
* **javascript:** follow calls through own code and npm packages ([b52d9f9](https://github.com/theonly1me/vigilante/commit/b52d9f98653ce294ae8769d3a4fc40ae9fbebcf9))
* **javascript:** read pnpm-lock.yaml and load packages in parallel ([d3dbb0c](https://github.com/theonly1me/vigilante/commit/d3dbb0c425d845ffaffe8eb01fc7e2ffaf6a4a3c))
* **javascript:** read yarn.lock from yarn 1 and yarn 2 or later ([a65302e](https://github.com/theonly1me/vigilante/commit/a65302eaacf9dd22e5e2fdec3a466b2b0d1a94c2))
* **javascript:** resolve calls with the TypeScript checker ([ffb17c7](https://github.com/theonly1me/vigilante/commit/ffb17c7d6b9745ac50b8fc68d323590e80a07bb9))
* **lock:** write and read vigilante.lock ([fdb230e](https://github.com/theonly1me/vigilante/commit/fdb230e439488e47ebe20072ef34aea055c82ba9))
* **model:** add every capability that Capslock reports ([2854bfa](https://github.com/theonly1me/vigilante/commit/2854bfac78135ff7b282656e16042a9ad68d9f31))
* **python:** follow calls through own code and Python packages ([42b4623](https://github.com/theonly1me/vigilante/commit/42b46230bf9ccf850dfab60761a08a5ff2c17522))
* **python:** resolve calls with Pyright through scip-python ([c2facbe](https://github.com/theonly1me/vigilante/commit/c2facbee84f72d3d4304191bf3d5c472e22c9a76))
* report the new powers that a pull request gives to code, CI, builds, and AI agents ([40ee615](https://github.com/theonly1me/vigilante/commit/40ee6154e2c4eb229868d5f4b2b28502bd09fb2e))
* **report:** render Markdown, JSON, and SARIF reports ([77402d5](https://github.com/theonly1me/vigilante/commit/77402d5a82fd2ce4f58e8cd6d2aac37d493efe1f))
* resolve calls by type in JavaScript, TypeScript, Python, Rust, Java, and C# ([98f33fa](https://github.com/theonly1me/vigilante/commit/98f33faafb4be57dec1adfbca987166988a021eb))
* **review:** compare base and head powers against the lock ([8863bab](https://github.com/theonly1me/vigilante/commit/8863babe762c3aadef8e1effdfdd7a8c1de82a12))
* **review:** note new packages on Go call paths and show call paths in reports ([dd26555](https://github.com/theonly1me/vigilante/commit/dd26555bfab7e3cfbf2fc6af8b21a322d3a8ef3d))
* **ruby:** follow calls through own code and gems ([cb4d8dc](https://github.com/theonly1me/vigilante/commit/cb4d8dc7d8206dc2ddce5954215942560ddcb12d))
* **rust:** follow calls through own code and Cargo packages ([71fb0f6](https://github.com/theonly1me/vigilante/commit/71fb0f669d66091817d85f67a906309a5298a668))
* **rust:** resolve calls with rust-analyzer in a build with no network ([9f955fb](https://github.com/theonly1me/vigilante/commit/9f955fb9da893f0e3db9f393ef2135fe74a886d0))
* **tree:** read files at a git commit and in a directory ([5e452e2](https://github.com/theonly1me/vigilante/commit/5e452e213d63ea298f4353ee097355f8255f92ce))


### Fixes

* **action:** verify archives from the main branch build at the commit of the version tag ([931b9b6](https://github.com/theonly1me/vigilante/commit/931b9b64294828cb82a63ee399e15184f4ea2ae4))
* **agent:** see risky commands behind options and command wrappers ([4d4d30e](https://github.com/theonly1me/vigilante/commit/4d4d30e2a8717123981d1ea272dabd717eec8e24))
* **build:** check the scripts that npm runs before and after a script ([2165578](https://github.com/theonly1me/vigilante/commit/2165578d9811699341931058f1cd08638370c032))
* **ci:** read GitHub expressions inside YAML flow lists ([38a03db](https://github.com/theonly1me/vigilante/commit/38a03db340b9ea37e6306b46d564e20ef2b40168))
* **ci:** read secret names only inside GitHub expressions ([5ea30ad](https://github.com/theonly1me/vigilante/commit/5ea30adcde5e4ee686821e91f3251175034d8f67))
* **cli:** name a folder that is not a git repository ([f61f90e](https://github.com/theonly1me/vigilante/commit/f61f90ec7b7729f7c39a4660f8958791ad01f095))
* **cli:** use singular nouns in the lock summary ([943e19b](https://github.com/theonly1me/vigilante/commit/943e19bcc18ae0306c0fc156a057cc2968e1aed6))
* **container:** download the image inside the time limit of the build ([2b4458c](https://github.com/theonly1me/vigilante/commit/2b4458cbb047acc7dcc0c4055fcaa84b61359b7c))
* **container:** download the image inside the time limit of the build ([5127db4](https://github.com/theonly1me/vigilante/commit/5127db4fb5255112715de2adfd02e603cb3759f9))
* **container:** keep the container alive past the deadline of its steps ([a0d2099](https://github.com/theonly1me/vigilante/commit/a0d2099c72d0f1a8309ee1f88c1c5b8baf858ea4))
* **container:** keep the container alive past the deadline of its steps ([75cb052](https://github.com/theonly1me/vigilante/commit/75cb05223fc4ce2313bfb0a9e5b44a0e67e934ae))
* **csharp:** pass the Rust 1.99 clippy question_mark lint ([c8c4a2e](https://github.com/theonly1me/vigilante/commit/c8c4a2e1c4c5eb2075ebd587cadc3936faf4b29e))
* **deep:** keep top-level code out of calls by method name ([e20ea9f](https://github.com/theonly1me/vigilante/commit/e20ea9fcc739a660ad7e135ba910a9b986b85525))
* **deep:** stop a type check at type_check_seconds and resolve calls by name ([6b40eb1](https://github.com/theonly1me/vigilante/commit/6b40eb1d5a14b9cf88a482c4671c6fe88543b501))
* **deps:** update rustls to 0.23.46 for four TLS advisories ([055dc1b](https://github.com/theonly1me/vigilante/commit/055dc1b02834a1a14c03c84b5e498c37a360d6a9))
* **docs:** load the README logo from the repository ([ad625c9](https://github.com/theonly1me/vigilante/commit/ad625c98c91c447100876219d8000cf5946d24e7))
* **java:** list a pom.xml that is not valid instead of stopping ([d273b75](https://github.com/theonly1me/vigilante/commit/d273b75e55b6ba224736f0ba3f0d3c46b7e63714))
* **packs:** count only okhttp clients and calls as Java network ([54cff3c](https://github.com/theonly1me/vigilante/commit/54cff3c4bafb9f3648f2757bb48eec75e9c9fec1))
* **packs:** leave marker trait impls and wasm_bindgen imports out of Rust native ([d00d0f5](https://github.com/theonly1me/vigilante/commit/d00d0f57f7f5da8621c8317b2bb4ae2f36ae8d07))
* **packs:** match nested use trees in the Rust exec and network rules ([9035bcd](https://github.com/theonly1me/vigilante/commit/9035bcdb6853ee7b52fe0176afa46194952f94ae))
* **packs:** match the module name in Python import rules ([c2ff7de](https://github.com/theonly1me/vigilante/commit/c2ff7dec72d5526806f8542fbe19720e7c464700))
* **python:** skip type annotations and caught exception types ([a1ad118](https://github.com/theonly1me/vigilante/commit/a1ad1183e8f1c7788f1cbb54a2737716e3d07e9b))
* **registry:** key the package cache by source and checksum ([7eef451](https://github.com/theonly1me/vigilante/commit/7eef45129134a2fe91a69beeb0168685ebe0775e))
* **registry:** retry brief download failures and report missing packages ([7f7d749](https://github.com/theonly1me/vigilante/commit/7f7d749c1a3a2c2eec955ac600cb6d96d27924ff))
* **release:** build macOS wheels without Linux platform tags ([b8fa7c9](https://github.com/theonly1me/vigilante/commit/b8fa7c95ef209b90e20b327fb59e75f6ee86c8ca))
* **release:** change only the version text in action.yml and package.json ([198e9b6](https://github.com/theonly1me/vigilante/commit/198e9b65cd2495b6cccddb3615e0dedb567e0581))
* **release:** change only the version text in action.yml and package.json ([e70e470](https://github.com/theonly1me/vigilante/commit/e70e4702ce67f37e3ab72024738f10b133d0a8f8))
* **release:** read the release pull request branch only when one exists ([8ac116c](https://github.com/theonly1me/vigilante/commit/8ac116c0f29f3f5fa54708ea6133d3a568efc4da))
* **release:** read the release pull request branch only when one exists ([f4ec900](https://github.com/theonly1me/vigilante/commit/f4ec90081c1bfb038e02b8563d94ea7df294c6f4))
* **report:** show at most ten lines that were not analyzed ([11f4abe](https://github.com/theonly1me/vigilante/commit/11f4abe266b0f7c980bf911cf70d8c6214228efd))
* **rust:** leave out module files that only tests declare ([ab95d4e](https://github.com/theonly1me/vigilante/commit/ab95d4e60b1b53f4dad724a8cd419b4c3b7be150))
* **rust:** mark only the use tree names that a rule matches ([3bad758](https://github.com/theonly1me/vigilante/commit/3bad758bb3467736513b89455b85bbc0f4e51de2))
* **scan:** leave files that an engine owns to that engine ([8ae9016](https://github.com/theonly1me/vigilante/commit/8ae90167a727be3be2fe538ace21c8f54b8c55f2))
* **yaml:** read flow lists that GitHub accepts with less indentation ([e7f42d5](https://github.com/theonly1me/vigilante/commit/e7f42d58d2255c741be429eaf227ec3c4ae7be73))


### Performance

* **deep:** find a function in its file by name instead of a scan ([97097ae](https://github.com/theonly1me/vigilante/commit/97097ae4d0761e2ca2bd57e056a07b77c2616fca))
* **javascript:** type-check own files and read package files on demand ([d6d04c0](https://github.com/theonly1me/vigilante/commit/d6d04c0701ab9ade75d9febb84cb5f35737ecc06))
* **python:** index only own code and map package symbols by module ([98b8c29](https://github.com/theonly1me/vigilante/commit/98b8c298887ab8e43b06781dd6136108193461c9))


### Documentation

* add agent instructions and the shadowclone workflow skills ([a6c4b0f](https://github.com/theonly1me/vigilante/commit/a6c4b0fc3fb510765d3b6e16477e98d2bfb97f12))
* add contributing, security, and conduct guides ([37bb8df](https://github.com/theonly1me/vigilante/commit/37bb8df7436efe48dd423035485a6428603b4057))
* add the architecture page and the design records ([2a4055a](https://github.com/theonly1me/vigilante/commit/2a4055afc5b2d7db08225ce71a51dc25bb066232))
* add the README and the setup guides ([6e2929f](https://github.com/theonly1me/vigilante/commit/6e2929fb6e57b9d09da7bc8d2783b18de2696fc2))
* center the README header and add status badges ([edbd470](https://github.com/theonly1me/vigilante/commit/edbd470654bb5d40ebe18450510f9b54d6435fec))
* describe typed call resolution and its limits ([75073b4](https://github.com/theonly1me/vigilante/commit/75073b42408bd47db96deaf2b090ed0bc0becdca))
* explain the call paths through packages for every language ([a061f2e](https://github.com/theonly1me/vigilante/commit/a061f2e79eb801acbd0041598d55027112557728))
* explain the Capslock engine for Go and the new powers ([1d2a72d](https://github.com/theonly1me/vigilante/commit/1d2a72d2b5371515ef1f70f8b2ce0b607431d738))
* render the CI badge before the first workflow run ([5d92baf](https://github.com/theonly1me/vigilante/commit/5d92baf6cc36908a75108afcc0b5c0268778be76))
* rewrite the guides in Simplified Technical English ([948ed2a](https://github.com/theonly1me/vigilante/commit/948ed2a466a7aed0ac5a6f7118c8bf6a13cd853f))
* show the white banner logo and name each logo by its content ([a9d3018](https://github.com/theonly1me/vigilante/commit/a9d301871bbedbc3e4fbfd23fe5b0221825cf669))
