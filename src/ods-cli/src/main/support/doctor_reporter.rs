fn print_diagnostics(diagnostics: &[Diagnostic], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for diagnostic in diagnostics {
                let severity = match diagnostic.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };
                println!(
                    "{severity}: {}: {}",
                    diagnostic.path.display(),
                    diagnostic.message
                );
            }
        }
        OutputFormat::Json => {
            let items: Vec<String> = diagnostics
                .iter()
                .map(|d| {
                    let severity = match d.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                    };
                    format!(
                        r#"{{"severity":"{severity}","path":{},"message":{}}}"#,
                        json_escape(&d.path.display().to_string()),
                        json_escape(&d.message)
                    )
                })
                .collect();
            println!("[{}]", items.join(","));
        }
        OutputFormat::Sarif => {
            let results: Vec<String> = diagnostics
                .iter()
                .map(|d| {
                    let level = match d.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                    };
                    format!(
                        r#"{{"ruleId":"ods-lint","level":"{level}","message":{{"text":{}}},"locations":[{{"physicalLocation":{{"artifactLocation":{{"uri":{}}}}}}}]}}"#,
                        json_escape(&d.message),
                        json_escape(&d.path.display().to_string())
                    )
                })
                .collect();
            println!(
                r#"{{"$schema":"https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json","version":"2.1.0","runs":[{{"tool":{{"driver":{{"name":"ODS","informationUri":"https://github.com/open-doc-spec/ods","version":"{}"}}}},"results":[{}]}}]}}"#,
                env!("CARGO_PKG_VERSION"),
                results.join(",")
            );
        }
    }
}

fn json_escape(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn resolve_root_path(path: PathBuf) -> PathBuf {
    ods_core::find_workspace_root(&path).unwrap_or(path)
}

struct DoctorReport {
    text: String,
    json: String,
    has_error: bool,
}
