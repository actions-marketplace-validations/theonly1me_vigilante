# Changelog

## [0.3.2](https://github.com/theonly1me/vigilante/compare/v0.3.1...v0.3.2) (2026-10-18)


### Fixes

* **config:** treat benchmark folders as test code ([d15d407](https://github.com/theonly1me/vigilante/commit/d15d407bb0f5fd9c54717801c3888298b21da88c))
* **deep:** look up assignments, functions, and call sites by key, not by scan ([37fe38f](https://github.com/theonly1me/vigilante/commit/37fe38fb38708da48f262ca32044066e2c540d2a))
* **deep:** look up assignments, functions, and call sites by key, not by scan ([52691c8](https://github.com/theonly1me/vigilante/commit/52691c84ea09754dedb50e3baa140c8e3ea22d12))
* **go:** leave test paths out of Capslock evidence ([ba7c9b3](https://github.com/theonly1me/vigilante/commit/ba7c9b333b157fdb6f8320feae2a67df7dd54002))
* **go:** skip a go.mod folder that has no Go files ([aa40d51](https://github.com/theonly1me/vigilante/commit/aa40d51e34f54c2e92583947cec3f6868f43cd1c))
* **go:** skip a go.mod folder that has no Go files ([3ab9bb0](https://github.com/theonly1me/vigilante/commit/3ab9bb0f92d63af9c00df10b401251ade91af7ed))
* **scan:** show examples from more than one folder ([#40](https://github.com/theonly1me/vigilante/issues/40)) ([78abc76](https://github.com/theonly1me/vigilante/commit/78abc76f005ec19f35d1eabedd441603fbe1aca2))

## [0.3.1](https://github.com/theonly1me/vigilante/compare/v0.3.0...v0.3.1) (2026-10-17)


### Fixes

* **deep:** read the argument facts of each call, not of each line ([fde300c](https://github.com/theonly1me/vigilante/commit/fde300c5a3aae41f64ca6d09f46472f554da8fa8))
* **deep:** read the argument facts of each call, not of each line ([bc7030c](https://github.com/theonly1me/vigilante/commit/bc7030c79204d6547f8fdd8cceae25f3e92f4062))
* **deps:** bump the crates group across 1 directory with 3 updates ([77773f2](https://github.com/theonly1me/vigilante/commit/77773f27958f9aad5a1fb519633f2a5ca7702aa2))
* **deps:** bump the crates group across 1 directory with 3 updates ([063b20f](https://github.com/theonly1me/vigilante/commit/063b20f3221a2887ceeb07f26f487e38c873b11e))
* **doctor:** fail when a protected folder cannot be read ([e0daf61](https://github.com/theonly1me/vigilante/commit/e0daf612795acd2a365b48a247ede8cbfd1f948b))
* **doctor:** fail when a protected folder cannot be read ([cd1d9c7](https://github.com/theonly1me/vigilante/commit/cd1d9c788f1093c98003dce7f079654d9752bc76))

## [0.3.0](https://github.com/theonly1me/vigilante/compare/v0.2.1...v0.3.0) (2026-10-17)


### Features

* **javascript:** read bun.lock ([80c302f](https://github.com/theonly1me/vigilante/commit/80c302ff4e06f5d350572a8d4ba3659962c6ecb5))
* **javascript:** read bun.lock ([7e333d8](https://github.com/theonly1me/vigilante/commit/7e333d8fac16c30eae4be0ba752734e4620bffa5))

## [0.2.1](https://github.com/theonly1me/vigilante/compare/v0.2.0...v0.2.1) (2026-10-16)


### Fixes

* **deep:** class untrusted input by the input arguments, not by options ([d6b14c9](https://github.com/theonly1me/vigilante/commit/d6b14c95928a1de2c79cb9910810290add9d4f64))
* **deep:** class untrusted input by the input arguments, not by options ([f8ee81b](https://github.com/theonly1me/vigilante/commit/f8ee81b457eecd561d53860ef3d13670a99f7b31))
* **doctor:** check the configured rule folders and each file in a protected folder ([990eebf](https://github.com/theonly1me/vigilante/commit/990eebfd0e305360c874151193785a6bdf172125))
* **doctor:** check the configured rule folders and each file in a protected folder ([e647503](https://github.com/theonly1me/vigilante/commit/e647503f31092c3a0a6b41d2612283919eafb536))

## [0.2.0](https://github.com/theonly1me/vigilante/compare/v0.1.1...v0.2.0) (2026-10-16)


### Features

* **action:** write the report as an opt-in pull request comment ([9b5a2ae](https://github.com/theonly1me/vigilante/commit/9b5a2ae8e5211f72cb9392ae9cea5c324c006f25))
* **agent:** read skill frontmatter, open code options, deny rules, and unknown install programs ([a6235ea](https://github.com/theonly1me/vigilante/commit/a6235ea3a17f36cf004c64e6aa9af0ea86302555))
* **ci:** report script injection, head checkouts, and weaker action pins as risks ([320dfd3](https://github.com/theonly1me/vigilante/commit/320dfd3782f6504c051c6a36626cc5d60309e380))
* **ci:** report script injection, head checkouts, and weaker action pins as risks ([d7cc1f2](https://github.com/theonly1me/vigilante/commit/d7cc1f285a36751cfebe4b0ef75df818f0809ec3))
* **deep:** find privileged calls through aliases, constants, and global objects, and skip local shadows ([2137e2f](https://github.com/theonly1me/vigilante/commit/2137e2ff892f648b4ba43d683dbe413523de81b6))
* **deep:** find the input class of each privileged call in own code ([4bb33e5](https://github.com/theonly1me/vigilante/commit/4bb33e5e28c69b070353a00e0be443c8d2940507))
* **deep:** skip module members that a rule marks as inert ([37746d9](https://github.com/theonly1me/vigilante/commit/37746d9ce73e215fa5ef89360e2c7dbe0ba529d0))
* **doctor:** check that code owners and branch rules protect the lock ([283abfa](https://github.com/theonly1me/vigilante/commit/283abfaef67d61e93606b2713801c9689ba90d81))
* **doctor:** check that code owners and branch rules protect the lock ([7441667](https://github.com/theonly1me/vigilante/commit/74416675c1f349c07d71d765f6b548258be8ae17))
* enforce deny rules on every power, trust only base rules, and report analysis gaps ([d010110](https://github.com/theonly1me/vigilante/commit/d010110ee0a13b15e5fed6f380101d41e72b329c))
* find privileged calls by identity, not only by spelling ([85596a1](https://github.com/theonly1me/vigilante/commit/85596a176acfa847250f69349d86a24d0370d84f))
* **javascript:** report a module load whose name is not a literal as an analysis gap ([b249894](https://github.com/theonly1me/vigilante/commit/b24989409601b5e611327af5bd1ad87d8f14deea))
* report test code and unreadable module loads ([bf0da67](https://github.com/theonly1me/vigilante/commit/bf0da672f0a7acec79d9de7069ac7db9563e54ee))
* **report:** list analysis gaps in every format and let a policy block new gaps ([e58c3ca](https://github.com/theonly1me/vigilante/commit/e58c3cacce7dba61d3d60c6009e9ed02e44c3458))
* **report:** put the result and the needed approval at the top of the report ([0c18357](https://github.com/theonly1me/vigilante/commit/0c183573a1297f3fc757b79f14d4f8814143d549))
* **review:** report changed risks and approve them in the lock ([0f4d951](https://github.com/theonly1me/vigilante/commit/0f4d951d709c09e2cbde2523c2b7228625368333))
* **review:** report changed risks and approve them in the lock ([0d093e7](https://github.com/theonly1me/vigilante/commit/0d093e74437dabb1956634c90b65a39a8b82e8b3))
* **scan:** report the powers of tests and fixtures as test code ([9c443e9](https://github.com/theonly1me/vigilante/commit/9c443e94b710861fb215316e8eda35948228101f))


### Fixes

* **deep:** class a command by its command arguments, not by its options or callback ([a6a6da3](https://github.com/theonly1me/vigilante/commit/a6a6da308db277f7cf50f3e2e02a5f210587c54a))
* **deep:** class a command by its command arguments, not by its options or callback ([ad727c1](https://github.com/theonly1me/vigilante/commit/ad727c1da959bcec7c12063ff7f13023dde36196))
* **deep:** keep object and class methods from hiding a global call with the same name ([9e36cc1](https://github.com/theonly1me/vigilante/commit/9e36cc152c745a878194c6172d538a8b59ab74e9))
* **deep:** keep object and class methods from hiding a global call with the same name ([69f1540](https://github.com/theonly1me/vigilante/commit/69f15403221fe9e1bf4c0d6db6924c84d2661939))
* **deep:** make risks only at sinks, not at each call into a package ([58b4b82](https://github.com/theonly1me/vigilante/commit/58b4b824a6abae2dcd9b05b6553f21b0b6557a67))
* **deep:** make risks only at sinks, not at each call into a package ([c9eb892](https://github.com/theonly1me/vigilante/commit/c9eb89257060a6ce935bf22143b4d2f5f72d3626))
* **packs:** match fully qualified Java sockets and asyncio subprocesses ([095b13f](https://github.com/theonly1me/vigilante/commit/095b13f39a24b76a8b00c0bb2bfd4d22fb3da8d2))
* **review:** block a denied power also when the report makes it a note ([85966ec](https://github.com/theonly1me/vigilante/commit/85966eca6492d6515b0d47f834539f9e2f8fa5b7))
* **scan:** read custom rules from the merge base in a diff ([fff0587](https://github.com/theonly1me/vigilante/commit/fff05873e03a6a05bbc592230eef0e2a71352d78))


### Documentation

* add the benchmark results and the comparison with capslock-git-diff ([371db5d](https://github.com/theonly1me/vigilante/commit/371db5d93ab286b90a471807cbdd0bdf3c5e5da8))
* describe CI and agent risks, and the new surface powers ([c5c790f](https://github.com/theonly1me/vigilante/commit/c5c790f28dc102356edf467e08cf617b51b10ee6))
* describe risks, their input classes, and their approval ([8aef62d](https://github.com/theonly1me/vigilante/commit/8aef62d5b65f2cef7b7f20fd4b287d15f67eefa1))
* describe the result summary, vigilante doctor, and the pull request comment ([69a7d5f](https://github.com/theonly1me/vigilante/commit/69a7d5f79b7199aa2f0c99ce5888d27051e49db2))
* split a long sentence in the CI risk record ([a650b88](https://github.com/theonly1me/vigilante/commit/a650b8814a91454115e0aabbb30dc1ef61e75eaf))

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
