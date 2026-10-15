use std::collections::HashMap;

use super::{
    Coordinates, EffectiveRequest, Pom, PomSource, ResolveRequest, effective, parse_pom, resolve,
};
use crate::Error;

const ROOT: &str = r#"<project xmlns="http://maven.apache.org/POM/4.0.0">
  <parent><groupId>com.acme</groupId><artifactId>parent</artifactId><version>1</version></parent>
  <artifactId>app</artifactId>
  <dependencyManagement><dependencies>
    <dependency><groupId>org.bom</groupId><artifactId>bom</artifactId><version>1</version><type>pom</type><scope>import</scope></dependency>
  </dependencies></dependencyManagement>
  <dependencies>
    <dependency><groupId>org.lib</groupId><artifactId>core</artifactId>
      <exclusions><exclusion><groupId>org.drop</groupId><artifactId>me</artifactId></exclusion></exclusions>
    </dependency>
    <dependency><groupId>org.extra</groupId><artifactId>util</artifactId></dependency>
    <dependency><groupId>org.bom</groupId><artifactId>client</artifactId></dependency>
    <dependency><groupId>org.test</groupId><artifactId>junit</artifactId><version>4</version><scope>test</scope></dependency>
    <dependency><groupId>org.shared</groupId><artifactId>common</artifactId><version>1.0</version></dependency>
    <dependency><groupId>com.acme</groupId><artifactId>tools</artifactId><version>${project.version}</version></dependency>
  </dependencies>
</project>"#;

const POMS: &[(&str, &str)] = &[
    (
        "com.acme:parent:1",
        r#"<project><groupId>com.acme</groupId><artifactId>parent</artifactId><version>1</version><packaging>pom</packaging>
      <properties><lib.version>2.0</lib.version></properties>
      <dependencyManagement><dependencies><dependency><groupId>org.lib</groupId><artifactId>core</artifactId><version>${lib.version}</version></dependency></dependencies></dependencyManagement>
    </project>"#,
    ),
    (
        "org.bom:bom:1",
        r#"<project><groupId>org.bom</groupId><artifactId>bom</artifactId><version>1</version><packaging>pom</packaging>
      <dependencyManagement><dependencies>
        <dependency><groupId>org.extra</groupId><artifactId>util</artifactId><version>3.0</version></dependency>
        <dependency><groupId>org.bom</groupId><artifactId>client</artifactId><version>${project.version}</version></dependency>
      </dependencies></dependencyManagement>
    </project>"#,
    ),
    (
        "org.lib:core:2.0",
        r#"<project><groupId>org.lib</groupId><artifactId>core</artifactId><version>2.0</version>
      <dependencies>
        <dependency><groupId>org.shared</groupId><artifactId>common</artifactId><version>2.0</version></dependency>
        <dependency><groupId>org.opt</groupId><artifactId>thing</artifactId><version>1</version><optional>true</optional></dependency>
        <dependency><groupId>org.drop</groupId><artifactId>me</artifactId><version>1</version></dependency>
        <dependency><groupId>org.deep</groupId><artifactId>leaf</artifactId><version>1</version></dependency>
        <dependency><groupId>org.test</groupId><artifactId>mock</artifactId><version>1</version><scope>test</scope></dependency>
      </dependencies>
    </project>"#,
    ),
    (
        "org.extra:util:3.0",
        "<project><groupId>org.extra</groupId><artifactId>util</artifactId><version>3.0</version></project>",
    ),
    (
        "org.bom:client:1",
        "<project><groupId>org.bom</groupId><artifactId>client</artifactId><version>1</version></project>",
    ),
    (
        "org.shared:common:1.0",
        "<project><groupId>org.shared</groupId><artifactId>common</artifactId><version>1.0</version></project>",
    ),
    (
        "org.deep:leaf:1",
        "<project><groupId>org.deep</groupId><artifactId>leaf</artifactId><version>1</version></project>",
    ),
];

struct Poms(HashMap<String, String>);

impl PomSource for Poms {
    fn pom(&mut self, coordinates: &Coordinates) -> Result<Option<Pom>, Error> {
        let key = format!(
            "{}:{}:{}",
            coordinates.group, coordinates.artifact, coordinates.version
        );
        Ok(self.0.get(&key).and_then(|text| parse_pom(text)))
    }
}

#[test]
fn resolution_applies_parents_boms_scopes_exclusions_and_nearest_versions() {
    let mut source = Poms(
        POMS.iter()
            .map(|(key, text)| (key.to_string(), text.to_string()))
            .collect(),
    );
    let root = effective(EffectiveRequest {
        pom: parse_pom(ROOT).expect("root"),
        source: &mut source,
        depth: 0,
    })
    .expect("effective");

    let resolution = resolve(ResolveRequest {
        root: &root,
        source: &mut source,
        is_local: &|(group, artifact)| group == "com.acme" && artifact == "tools",
    })
    .expect("resolution");

    let found: Vec<String> = resolution
        .artifacts
        .iter()
        .map(|item| format!("{}:{}:{}", item.group, item.artifact, item.version))
        .collect();
    assert_eq!(root.coordinates.version, "1");
    assert_eq!(
        found,
        vec![
            "org.lib:core:2.0",
            "org.extra:util:3.0",
            "org.bom:client:1",
            "org.shared:common:1.0",
            "org.deep:leaf:1"
        ]
    );
    assert!(resolution.skipped.is_empty(), "{:?}", resolution.skipped);
}
