use super::maven::Coordinates;

const CLASSPATHS: &[&str] = &["compileclasspath", "runtimeclasspath"];
const TEST: &str = "test";

pub fn locked_artifacts(text: &str) -> Vec<Coordinates> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#') && !line.starts_with("empty="))
        .filter_map(|line| {
            let (artifact, configurations) = line.split_once('=')?;
            let is_used = configurations.split(',').any(|configuration| {
                let configuration = configuration.to_lowercase();
                !configuration.contains(TEST)
                    && CLASSPATHS
                        .iter()
                        .any(|classpath| configuration.ends_with(classpath))
            });
            let mut parts = artifact.split(':');
            let coordinates = Coordinates {
                group: parts.next()?.to_string(),
                artifact: parts.next()?.to_string(),
                version: parts.next()?.to_string(),
            };
            is_used.then_some(coordinates)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::locked_artifacts;

    #[test]
    fn a_gradle_lockfile_lists_compile_and_runtime_artifacts() {
        let text = "# This is a Gradle generated file\ncom.squareup.okhttp3:okhttp:4.12.0=compileClasspath,runtimeClasspath\njunit:junit:4.13.2=testCompileClasspath,testRuntimeClasspath\norg.slf4j:slf4j-api:2.0.9=releaseRuntimeClasspath\nempty=annotationProcessor\n";

        let found: Vec<String> = locked_artifacts(text)
            .iter()
            .map(|item| format!("{}:{}:{}", item.group, item.artifact, item.version))
            .collect();

        assert_eq!(
            found,
            vec![
                "com.squareup.okhttp3:okhttp:4.12.0",
                "org.slf4j:slf4j-api:2.0.9"
            ]
        );
    }
}
