use super::super::Sources;

#[test]
fn the_default_download_hosts_of_each_ecosystem_are_public() {
    let sources = Sources::new(&[]).expect("sources");

    for url in [
        "https://registry.npmjs.org/left-pad/-/left-pad-1.3.0.tgz",
        "https://files.pythonhosted.org/packages/aa/bb/requests-2.32.3-py3-none-any.whl",
        "https://repo1.maven.org/maven2/com/acme/app/1/app-1.jar",
        "https://static.crates.io/crates/serde/serde-1.0.219.crate",
        "https://api.nuget.org/v3-flatcontainer/serilog/3.1.1/serilog.3.1.1.nupkg",
        "https://rubygems.org/downloads/faraday-2.9.0.gem",
    ] {
        assert!(sources.allows(url), "{url}");
    }
    assert!(!sources.allows("https://crates.example.com/serde-1.0.219.crate"));
}
