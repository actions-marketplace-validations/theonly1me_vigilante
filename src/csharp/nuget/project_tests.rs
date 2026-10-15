use super::project::{ProjectInput, read_project};

#[test]
fn a_project_reads_frameworks_references_properties_and_central_versions() {
    let project = read_project(ProjectInput {
        project: r#"<Project Sdk="Microsoft.NET.Sdk">
          <PropertyGroup><TargetFrameworks>net8.0;netstandard2.0</TargetFrameworks><PollyVersion>8.4.0</PollyVersion></PropertyGroup>
          <ItemGroup>
            <PackageReference Include="Serilog" Version="3.1.1" />
            <PackageReference Include="Polly" Version="$(PollyVersion)" />
            <PackageReference Include="Dapper" />
            <PackageReference Include="Humanizer" VersionOverride="2.14.1" />
            <ProjectReference Include="..\Lib\Lib.csproj" />
          </ItemGroup>
        </Project>"#,
        build_properties: Some("<Project><PropertyGroup><SharedVersion>1.0</SharedVersion></PropertyGroup></Project>"),
        central_versions: Some(r#"<Project><ItemGroup><PackageVersion Include="Dapper" Version="2.1.35" /><PackageVersion Include="Humanizer" Version="2.0.0" /></ItemGroup></Project>"#),
    })
    .expect("project");

    assert_eq!(project.frameworks, vec!["net8.0", "netstandard2.0"]);
    assert_eq!(
        project.references,
        vec![
            ("Serilog".to_string(), "3.1.1".to_string()),
            ("Polly".to_string(), "8.4.0".to_string()),
            ("Dapper".to_string(), "2.1.35".to_string()),
            ("Humanizer".to_string(), "2.14.1".to_string()),
        ]
    );
}

#[test]
fn a_property_that_refers_to_itself_is_expanded_once() {
    let project = read_project(ProjectInput {
        project: r#"<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework><NoWarn>$(NoWarn);CS1591</NoWarn><Version>$(NoWarn)</Version></PropertyGroup>
          <ItemGroup><PackageReference Include="Serilog" Version="$(Version)" /></ItemGroup></Project>"#,
        build_properties: None,
        central_versions: None,
    })
    .expect("project");

    assert_eq!(project.references.len(), 1);
    assert!(project.references[0].1.contains("CS1591"));
}

#[test]
fn central_versions_use_properties_from_their_own_file_and_ids_are_trimmed() {
    let project = read_project(ProjectInput {
        project: r#"<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup>
          <ItemGroup><PackageReference Include="sqlite-net-pcl " /></ItemGroup></Project>"#,
        build_properties: None,
        central_versions: Some(r#"<Project><PropertyGroup><SqliteNetVersion>1.9.172</SqliteNetVersion></PropertyGroup>
          <ItemGroup><PackageVersion Include="sqlite-net-pcl" Version="$(SqliteNetVersion)" /></ItemGroup></Project>"#),
    })
    .expect("project");

    assert_eq!(
        project.references,
        vec![("sqlite-net-pcl".to_string(), "1.9.172".to_string())]
    );
}
