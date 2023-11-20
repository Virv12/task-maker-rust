use std::path::Path;

use task_maker_dag::*;

use crate::language::{
    CompilationSettings, CompiledLanguageBuilder, Language, SimpleCompiledLanguageBuilder,
};

#[derive(Debug)]
pub struct LanguageJava;

impl LanguageJava {
    pub fn new() -> LanguageJava {
        LanguageJava
    }
}

impl Language for LanguageJava {
    fn name(&self) -> &'static str {
        "Java / JDK"
    }

    fn extensions(&self) -> Vec<&'static str> {
        vec!["java"]
    }

    fn need_compilation(&self) -> bool {
        true
    }

    fn inline_comment_prefix(&self) -> Option<&'static str> {
        Some("//")
    }

    fn compilation_builder(
        &self,
        source: &Path,
        settings: CompilationSettings,
    ) -> Option<Box<dyn CompiledLanguageBuilder + '_>> {
        let mut metadata = SimpleCompiledLanguageBuilder::new(
            self,
            source,
            settings,
            ExecutionCommand::system("sh"),
        );
        metadata.binary_name = source
            .with_extension("class")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let base_name = source.file_stem().unwrap().to_string_lossy().to_string();
        let source_lossy = source.file_name().unwrap().to_string_lossy().to_string();

        metadata.add_arg("-c").add_arg(format!(r#"
/usr/bin/sed -Ez 's/package\s([a-zA-Z0-9_$.]|\s)*;//;s/public\s+class\s+[a-zA-Z0-9_$]*/public class {base_name}/' -i {source_lossy}
javac {source_lossy} #
"#));

        metadata.callback(move |comp| {
            comp.limits_mut()
                .add_extra_readable_dir("/etc/java-openjdk")
                .mount_proc(true)
                .allow_multiprocess();
        });

        Some(Box::new(metadata))
    }

    fn runtime_command(&self, _path: &Path, _write_to: Option<&Path>) -> ExecutionCommand {
        ExecutionCommand::system("bash")
    }

    fn runtime_args(&self, path: &Path, _: Option<&Path>, mut args: Vec<String>) -> Vec<String> {
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        args.push("-exc".into());
        args.push(format!(r#"
chmod +w .
cp {file_name} {file_stem}.class
java -Deval=true -Xmx512M -Xss64M {file_stem}
"#));
        args
    }

    fn custom_limits(&self, limits: &mut ExecutionLimits) {
        limits
            .add_extra_readable_dir("/etc/java-openjdk")
            .mount_proc(true)
            .allow_multiprocess();
    }
}
