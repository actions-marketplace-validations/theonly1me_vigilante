pub const BUN_1_4_LOCKFILE: &str = r#"{
  "lockfileVersion": 2,
  "configVersion": 1,
  "workspaces": {
    "": {
      "name": "demo",
      "dependencies": {
        "@sindresorhus/is": "4.6.0",
        "is-even": "https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz",
        "is-number": "7.0.0",
        "kind-of": "github:jonschlinkert/kind-of",
        "local-tool": "file:./tools/local-tool",
        "my-alias": "npm:is-odd@3.0.1",
      },
    },
    "packages/app": {
      "name": "app",
      "dependencies": {
        "is-number": "6.0.0",
        "shared": "workspace:*",
      },
    },
    "packages/shared": {
      "name": "shared",
      "version": "1.0.0",
    },
  },
  "packages": {
    "@sindresorhus/is": ["@sindresorhus/is@4.6.0", "", {}, "sha512-t09vSN3MdfsyCHoFcTRCH/iUtG7OJ0CsjzB8cjAmKc/va/kIgeDI/TxsigdncE/4be734m0cvIYwNaV4i2XqAw=="],

    "app": ["app@workspace:packages/app"],

    "is-buffer": ["is-buffer@1.1.6", "", {}, "sha512-NcdALwpXkTm5Zvvbk7owOUSvVvBKDgKP5/ewfXEznmQFfs4ZRmanOeKBTjRVjka3QFoN6XJ+9F3USqfHqTaU5w=="],

    "is-even": ["is-even@https://registry.npmjs.org/is-even/-/is-even-1.0.0.tgz", { "dependencies": { "is-odd": "^0.1.2" } }, "sha512-LEhnkAdJqic4Dbqn58A0y52IXoHWlsueqQkKfMfdEnIYG8A1sm/GHidKkS6yvXlMoRrkM34csHnXQtOqcb+Jzg=="],

    "is-number": ["is-number@7.0.0", "", {}, "sha512-41Cifkg6e8TylSpdtTpeLVMqvSBEVzTttHvERD741+pnZ8ANv0004MRL43QKPDlK9cGvNp6NZWZUBlbGXYxxng=="],

    "is-odd": ["is-odd@0.1.2", "", { "dependencies": { "is-number": "^3.0.0" } }, "sha512-Ri7C2K7o5IrUU9UEI8losXJCCD/UtsaIrkR5sxIcFg4xQ9cRJXlWA5DQvTE0yDc0krvSNLsRGXN11UPS6KyfBw=="],

    "kind-of": ["kind-of@github:jonschlinkert/kind-of#3c7e8fd", {}, "jonschlinkert-kind-of-3c7e8fd", "sha512-8kEP2NPVfNlJDn81KnJurpsb8HLtF9X+bfzz2VB/13KBX+eAuIka7tg3/W8P6FOlEi7VpdaK5roiFyfb5jmUPw=="],

    "local-tool": ["local-tool@file:tools/local-tool", { "dependencies": { "is-number": "6.0.0" } }],

    "my-alias": ["is-odd@3.0.1", "", { "dependencies": { "is-number": "^6.0.0" } }, "sha512-CQpnWPrDwmP1+SMHXZhtLtJv90yiyVfluGsX5iNCVkrhQtU3TQHsUWPG9wkdk9Lgd5yNpAg9jQEo90CBaXgWMA=="],

    "shared": ["shared@workspace:packages/shared"],

    "app/is-number": ["is-number@6.0.0", "", {}, "sha512-Wu1VHeILBK8KAWJUAiSZQX94GmOE45Rg6/538fKwiloUu21KncEkYGPqob2oSZ5mUT73vLGrHQjKw3KMPwfDzg=="],

    "is-odd/is-number": ["is-number@3.0.0", "", { "dependencies": { "kind-of": "^3.0.2" } }, "sha512-4cboCqIpliH+mAvFNegjZQ4kgKc3ZUhQVr3HvWbSh5q3WH2v82ct+T2Y1hdU5Gdtorx/cLifQjqCbL7bpznLTg=="],

    "local-tool/is-number": ["is-number@6.0.0", "", {}, "sha512-Wu1VHeILBK8KAWJUAiSZQX94GmOE45Rg6/538fKwiloUu21KncEkYGPqob2oSZ5mUT73vLGrHQjKw3KMPwfDzg=="],

    "my-alias/is-number": ["is-number@6.0.0", "", {}, "sha512-Wu1VHeILBK8KAWJUAiSZQX94GmOE45Rg6/538fKwiloUu21KncEkYGPqob2oSZ5mUT73vLGrHQjKw3KMPwfDzg=="],

    "is-odd/is-number/kind-of": ["kind-of@3.2.2", "", { "dependencies": { "is-buffer": "^1.1.5" } }, "sha512-NOW9QQXMoZGg/oqnVNoNTTIFEIid1627WCffUBJEdMxYApq7mNE7CpzucIPc+ZQg25Phej7IJSmX3hO+oblOtQ=="],
  }
}
"#;
