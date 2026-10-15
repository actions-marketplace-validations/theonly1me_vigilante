use super::{GemSource, read_lockfile};

const LOCKFILE: &str = "GIT
  remote: https://github.com/acme/runner.git
  revision: 0123456789abcdef0123456789abcdef01234567
  branch: main
  specs:
    runner (1.2.0)
      spawner

PATH
  remote: engines/billing
  specs:
    billing (0.1.0)

GEM
  remote: https://rubygems.org/
  specs:
    faraday (2.9.0)
      faraday-net_http (>= 2.0, < 3.2)
    nokogiri (1.16.0)
      racc (~> 1.4)
    nokogiri (1.16.0-x86_64-linux)
      racc (~> 1.4)
    spawner (3.0.0)

PLATFORMS
  ruby
  x86_64-linux

DEPENDENCIES
  billing!
  faraday
  runner!

CHECKSUMS
  faraday (2.9.0) sha256=5f9d8c8e4b6a7f9b8f1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a
  nokogiri (1.16.0-x86_64-linux) sha256=0000000000000000000000000000000000000000000000000000000000000000

BUNDLED WITH
   2.5.4
";

#[test]
fn a_lockfile_lists_each_gem_with_its_source_platform_and_checksum() {
    let gems = read_lockfile(LOCKFILE);

    let entries: Vec<String> = gems
        .iter()
        .map(|gem| {
            format!(
                "{} {} {:?} {:?} {}",
                gem.name,
                gem.version,
                gem.platform,
                gem.source,
                gem.checksum
                    .as_deref()
                    .unwrap_or("-")
                    .get(..4)
                    .unwrap_or("-")
            )
        })
        .collect();
    assert_eq!(
        entries,
        vec![
            "runner 1.2.0 None Git { remote: \"https://github.com/acme/runner.git\", revision: \"0123456789abcdef0123456789abcdef01234567\" } -",
            "billing 0.1.0 None Path { folder: \"engines/billing\" } -",
            "faraday 2.9.0 None Registry { remote: \"https://rubygems.org/\" } 5f9d",
            "nokogiri 1.16.0 None Registry { remote: \"https://rubygems.org/\" } -",
            "spawner 3.0.0 None Registry { remote: \"https://rubygems.org/\" } -",
        ]
    );
    assert_eq!(gems[0].dependencies, vec!["spawner"]);
    let _ = GemSource::Path {
        folder: String::new(),
    };
}

#[test]
fn a_plain_gem_wins_over_platform_builds_and_otherwise_the_first_platform_is_kept() {
    let gems = read_lockfile(
        "GEM\n  remote: https://rubygems.org/\n  specs:\n    sqlite3 (1.7.2-arm64-darwin)\n    sqlite3 (1.7.2-x86_64-linux)\n    ffi (1.17.0-x86_64-linux)\n    ffi (1.17.0)\n",
    );

    assert_eq!(gems.len(), 2);
    assert_eq!(gems[0].platform.as_deref(), Some("arm64-darwin"));
    assert_eq!(gems[1].platform, None);
}
