use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpListener;
use serde_json::{Value, json};

pub(crate) fn run_lsp_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("lsp");
        return Ok(ExitCode::from(0));
    }
    let port = parse_port_flag(args);
    if let Some(port) = port {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .map_err(|e| fail_msg(ods_core::io_failed("bind LSP socket", e)))?;
        eprintln!("ods lsp: listening for JSON-RPC connections on 127.0.0.1:{port}");
        for stream in listener.incoming().flatten() {
            let reader = stream.try_clone().map_err(|e| fail_io("operation", e))?;
            let writer = stream;
            let mut session = LspSession::new(BufReader::new(reader), writer);
            let _ = session.run_loop();
        }
    } else {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut session = LspSession::new(BufReader::new(stdin.lock()), stdout.lock());
        let _ = session.run_loop();
    }
    Ok(ExitCode::from(0))
}

fn parse_port_flag(args: &[String]) -> Option<u16> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--port" {
            if let Some(val) = args.get(i + 1) {
                return val.parse().ok();
            }
        }
        i += 1;
    }
    None
}

struct LspSession<R, W> {
    reader: R,
    writer: W,
    workspace_root: Option<PathBuf>,
    documents: HashMap<String, String>,
}

impl<R: BufRead, W: Write> LspSession<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            workspace_root: None,
            documents: HashMap::new(),
        }
    }

    fn run_loop(&mut self) -> Result<(), io::Error> {
        while let Some(req) = self.read_message()? {
            let id = req.get("id").cloned();
            let method = req.get("method").and_then(Value::as_str).unwrap_or("");

            match method {
                "initialize" => {
                    if let Some(params) = req.get("params") {
                        if let Some(root_uri) = params.get("rootUri").and_then(Value::as_str) {
                            self.workspace_root = uri_to_path(root_uri);
                        } else if let Some(root_path) = params.get("rootPath").and_then(Value::as_str) {
                            self.workspace_root = Some(PathBuf::from(root_path));
                        }
                    }
                    if let Some(id) = id {
                        self.send_response(
                            &id,
                            json!({
                                "capabilities": {
                                    "textDocumentSync": {
                                        "openClose": true,
                                        "change": 2,
                                        "save": { "includeText": true }
                                    },
                                    "hoverProvider": true,
                                    "definitionProvider": true,
                                    "completionProvider": {
                                        "triggerCharacters": [":", " ", "/"]
                                    },
                                    "workspace": {
                                        "workspaceFolders": {
                                            "supported": true,
                                            "changeNotifications": true
                                        }
                                    }
                                }
                            }),
                        )?;
                    }
                }
                "initialized" => {}
                "workspace/didChangeWatchedFiles" | "workspace/didChangeWorkspaceFolders" => {
                    // Re-diagnose open documents against updated workspace state.
                    let uris: Vec<String> = self.documents.keys().cloned().collect();
                    for uri in uris {
                        let _ = self.publish_diagnostics_for_uri(&uri);
                    }
                }
                "$/cancelRequest" | "$/setTrace" => {}
                "textDocument/didOpen" => {
                    if let Some(params) = req.get("params") {
                        if let Some(doc) = params.get("textDocument") {
                            let uri = doc.get("uri").and_then(Value::as_str).unwrap_or("");
                            let text = doc.get("text").and_then(Value::as_str).unwrap_or("");
                            self.documents.insert(uri.to_string(), text.to_string());
                            self.publish_diagnostics_for_uri(uri)?;
                        }
                    }
                }
                "textDocument/didChange" => {
                    if let Some(params) = req.get("params") {
                        let uri = params
                            .get("textDocument")
                            .and_then(|t| t.get("uri"))
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        if let Some(changes) = params.get("contentChanges").and_then(Value::as_array)
                        {
                            let current = self
                                .documents
                                .get(uri)
                                .cloned()
                                .unwrap_or_default();
                            let updated = apply_content_changes(&current, changes);
                            self.documents.insert(uri.to_string(), updated);
                            self.publish_diagnostics_for_uri(uri)?;
                        }
                    }
                }
                "textDocument/didSave" => {
                    if let Some(params) = req.get("params") {
                        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
                        self.publish_diagnostics_for_uri(uri)?;
                    }
                }
                "textDocument/didClose" => {
                    if let Some(params) = req.get("params") {
                        let uri = params
                            .get("textDocument")
                            .and_then(|t| t.get("uri"))
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        self.documents.remove(uri);
                        // Clear diagnostics for closed doc
                        self.send_notification(
                            "textDocument/publishDiagnostics",
                            json!({ "uri": uri, "diagnostics": [] }),
                        )?;
                    }
                }
                "textDocument/hover" => {
                    if let Some(id) = id {
                        let result = self.handle_hover(req.get("params"));
                        self.send_response(&id, result)?;
                    }
                }
                "textDocument/definition" => {
                    if let Some(id) = id {
                        let result = self.handle_definition(req.get("params"));
                        self.send_response(&id, result)?;
                    }
                }
                "textDocument/completion" => {
                    if let Some(id) = id {
                        let result = self.handle_completion(req.get("params"));
                        self.send_response(&id, result)?;
                    }
                }
                "shutdown" => {
                    if let Some(id) = id {
                        self.send_response(&id, Value::Null)?;
                    }
                }
                "exit" => {
                    break;
                }
                _ => {
                    if let Some(id) = id {
                        self.send_response(&id, Value::Null)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn read_message(&mut self) -> io::Result<Option<Value>> {
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(None);
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(val) = trimmed.strip_prefix("Content-Length:") {
                content_length = val.trim().parse().ok();
            }
        }

        let Some(len) = content_length else {
            return Ok(None);
        };

        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;
        let json_val: Value = serde_json::from_slice(&buf)?;
        Ok(Some(json_val))
    }

    fn send_response(&mut self, id: &Value, result: Value) -> io::Result<()> {
        let resp = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        });
        self.write_message(&resp)
    }

    fn send_notification(&mut self, method: &str, params: Value) -> io::Result<()> {
        let notif = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.write_message(&notif)
    }

    fn write_message(&mut self, val: &Value) -> io::Result<()> {
        let payload = serde_json::to_string(val)?;
        let header = format!("Content-Length: {}\r\n\r\n", payload.len());
        self.writer.write_all(header.as_bytes())?;
        self.writer.write_all(payload.as_bytes())?;
        self.writer.flush()
    }

    fn publish_diagnostics_for_uri(&mut self, uri: &str) -> io::Result<()> {
        let Some(path) = uri_to_path(uri) else {
            return Ok(());
        };

        let root = self.workspace_root.clone().unwrap_or_else(|| {
            path.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
        });

        let buffer = self.documents.get(uri).cloned();
        let mut lsp_diagnostics = Vec::new();

        // Prefer buffer text for frontmatter key range hints
        let range_for_msg = |msg: &str| -> Value {
            if let Some(ref text) = buffer {
                if let Some((line, col, end_col)) = find_diagnostic_range(text, msg) {
                    return json!({
                        "start": { "line": line, "character": col },
                        "end": { "line": line, "character": end_col }
                    });
                }
            }
            json!({
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 80 }
            })
        };

        if ods_core::ods_enabled(&root) {
            if let Ok(ws) = load_workspace(&root) {
                let diags = lint_workspace_with_level(&ws, LintLevel::Full);
                for diag in diags {
                    if diag.path == path || paths_equal(&diag.path, &path) {
                        lsp_diagnostics.push(json!({
                            "range": range_for_msg(&diag.message),
                            "severity": match diag.severity {
                                Severity::Error => 1,
                                Severity::Warning => 2,
                            },
                            "source": "ods",
                            "message": diag.message
                        }));
                    }
                }
            }
        }

        if ods_core::okf_enabled(&root) {
            if let Ok(bundle) = ods_core::load_okf_bundle(&root) {
                for diag in ods_core::lint_okf_bundle(&bundle) {
                    if diag.path == path || paths_equal(&diag.path, &path) {
                        lsp_diagnostics.push(json!({
                            "range": range_for_msg(&diag.message),
                            "severity": match diag.severity {
                                Severity::Error => 1,
                                Severity::Warning => 2,
                            },
                            "source": "okf",
                            "message": diag.message
                        }));
                    }
                }
            }
        }

        // Agent Skills package for SKILL.md buffers
        if path.file_name().and_then(|s| s.to_str()) == Some("SKILL.md") {
            if let Some(pkg_root) = path.parent() {
                if let Ok(pkg) = ods_core::parse_skill_package(pkg_root) {
                    for diag in ods_core::lint_skill_package(&pkg) {
                        lsp_diagnostics.push(json!({
                            "range": range_for_msg(&diag.message),
                            "severity": match diag.severity {
                                Severity::Error => 1,
                                Severity::Warning => 2,
                            },
                            "source": "skills",
                            "message": diag.message
                        }));
                    }
                }
            }
        }

        self.send_notification(
            "textDocument/publishDiagnostics",
            json!({
                "uri": uri,
                "diagnostics": lsp_diagnostics
            }),
        )
    }

    fn handle_hover(&self, params: Option<&Value>) -> Value {
        let Some(params) = params else { return Value::Null };
        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
        let line_num = params.get("position").and_then(|p| p.get("line")).and_then(Value::as_u64).unwrap_or(0) as usize;

        let Some(text) = self.documents.get(uri) else { return Value::Null };
        let lines: Vec<&str> = text.lines().collect();
        let Some(line) = lines.get(line_num) else { return Value::Null };

        let hover_text = if line.contains("okf_version:") {
            "**okf_version**: OKF bundle version marker (root index only; use `ods lint --okf`)."
        } else if line.contains("type:") && !line.contains("parameters") {
            "**type** (OKF): Required concept kind (e.g. Metric, Attested Computation)."
        } else if line.contains("stale_after:") {
            "**stale_after** (OKF): Absolute freshness deadline (YYYY-MM-DD)."
        } else if line.contains("allowed-tools:") {
            "**allowed-tools** (Agent Skills): Space-separated pre-approved tools."
        } else if line.contains("compatibility:") {
            "**compatibility** (Agent Skills): Environment requirements (max 500 chars)."
        } else if line.trim_start().starts_with("name:") {
            "**name** (Agent Skills): Required skill id; must match parent directory."
        } else if line.contains("status:") {
            "**status**: Lifecycle state (`draft`, `stable`, `deprecated`, `archived`)."
        } else if line.contains("profile:") {
            "**profile**: Document profile template (`index`, `rfc`, `api`, `note`)."
        } else if line.contains("depends:") {
            "**depends**: Hard graph dependency documents required by this document."
        } else if line.contains("related:") {
            "**related**: Soft contextual relation documents associated with this document."
        } else if line.contains("share:") {
            "**share**: Visibility filter (`public`, `org`, `private`)."
        } else if line.trim_start().starts_with("tags:") || line.contains("tags:") {
            "**tags**: Free-form taxonomy facets. MUST be top-level frontmatter (not under `ods:`) so SSGs and other tools can read them."
        } else if line.contains("custom_profiles") || line.contains("custom-profiles:") {
            "**custom_profiles**: Workspace-wide array of custom profile schema paths in `ods.toml` (not document frontmatter)."
        } else if line.contains("ods:") {
            "**ods**: Open Document Spec nested engine key block or root version marker. Engine keys only — never put `tags` here."
        } else {
            return Value::Null;
        };

        json!({
            "contents": {
                "kind": "markdown",
                "value": hover_text
            }
        })
    }

    fn handle_definition(&self, params: Option<&Value>) -> Value {
        let Some(params) = params else { return Value::Null };
        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
        let line_num = params.get("position").and_then(|p| p.get("line")).and_then(Value::as_u64).unwrap_or(0) as usize;

        let Some(text) = self.documents.get(uri) else { return Value::Null };
        let lines: Vec<&str> = text.lines().collect();
        let Some(line) = lines.get(line_num) else { return Value::Null };

        let doc_path = uri_to_path(uri);
        let doc_dir = doc_path.as_ref().and_then(|p| p.parent()).unwrap_or_else(|| Path::new("."));

        if let Some(target) = extract_path_from_line(line) {
            let target_path = doc_dir.join(target);
            if target_path.exists() {
                let target_uri = format!("file://{}", target_path.canonicalize().unwrap_or(target_path).display());
                return json!({
                    "uri": target_uri,
                    "range": {
                        "start": { "line": 0, "character": 0 },
                        "end": { "line": 0, "character": 0 }
                    }
                });
            }
        }

        Value::Null
    }

    fn handle_completion(&self, _params: Option<&Value>) -> Value {
        json!([
            { "label": "ods", "kind": 14, "detail": "ODS Engine Key Block" },
            { "label": "profile: rfc", "kind": 12, "detail": "RFC Specification Profile" },
            { "label": "profile: api", "kind": 12, "detail": "API Specification Profile" },
            { "label": "profile: note", "kind": 12, "detail": "General Note Profile" },
            { "label": "status: draft", "kind": 12, "detail": "Draft status" },
            { "label": "status: stable", "kind": 12, "detail": "Stable status" },
            { "label": "status: deprecated", "kind": 12, "detail": "Deprecated status" },
            { "label": "status: archived", "kind": 12, "detail": "Archived status" },
            { "label": "depends:", "kind": 14, "detail": "Graph dependencies array" },
            { "label": "related:", "kind": 14, "detail": "Related documents array" },
            { "label": "share: public", "kind": 12, "detail": "Public visibility" },
            { "label": "share: org", "kind": 12, "detail": "Organization visibility" },
            { "label": "share: private", "kind": 12, "detail": "Private visibility" },
            { "label": "tags:", "kind": 14, "detail": "Top-level taxonomy tags (never under ods:)" },
            { "label": "description:", "kind": 14, "detail": "Universal top-level description" },
            { "label": "okf_version: \"0.2\"", "kind": 12, "detail": "OKF root marker" },
            { "label": "type:", "kind": 14, "detail": "OKF required concept type" },
            { "label": "name:", "kind": 14, "detail": "Agent Skills required name" },
            { "label": "allowed-tools:", "kind": 14, "detail": "Agent Skills tools" }
        ])
    }
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    if let Some(stripped) = uri.strip_prefix("file://") {
        Some(PathBuf::from(stripped))
    } else {
        Some(PathBuf::from(uri))
    }
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

/// Apply LSP content changes: full replace (no range) or incremental (UTF-16-ish cols approximated as chars).
fn apply_content_changes(text: &str, changes: &[Value]) -> String {
    let mut content = text.to_string();
    for change in changes {
        let Some(new_text) = change.get("text").and_then(Value::as_str) else {
            continue;
        };
        let Some(range) = change.get("range") else {
            // Full document sync
            content = new_text.to_string();
            continue;
        };
        let start = range.get("start");
        let end = range.get("end");
        let (Some(start), Some(end)) = (start, end) else {
            content = new_text.to_string();
            continue;
        };
        let sl = start.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
        let sc = start
            .get("character")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let el = end.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
        let ec = end.get("character").and_then(Value::as_u64).unwrap_or(0) as usize;
        let start_off = char_offset_at(&content, sl, sc);
        let end_off = char_offset_at(&content, el, ec).max(start_off);
        if start_off <= content.len() && end_off <= content.len() {
            content.replace_range(start_off..end_off, new_text);
        } else {
            content = new_text.to_string();
        }
    }
    content
}

fn char_offset_at(text: &str, line: usize, character: usize) -> usize {
    let mut current_line = 0usize;
    for (i, ch) in text.char_indices() {
        if current_line == line {
            // Count UTF-16 code units on this line until `character`.
            let rest = &text[i..];
            let line_text = rest.split('\n').next().unwrap_or(rest);
            let mut u16_units = 0usize;
            for (byte_idx, c) in line_text.char_indices() {
                if u16_units >= character {
                    return i + byte_idx;
                }
                u16_units += c.len_utf16();
            }
            return i + line_text.len();
        }
        if ch == '\n' {
            current_line += 1;
        }
    }
    text.len()
}

/// Best-effort map of diagnostic message → frontmatter line range in buffer.
fn find_diagnostic_range(text: &str, message: &str) -> Option<(u32, u32, u32)> {
    let keys = [
        "profile", "status", "depends", "related", "share", "type", "name", "description",
        "okf_version", "runtime", "stale_after", "ods",
    ];
    let lower = message.to_lowercase();
    for (i, line) in text.lines().enumerate() {
        for key in keys {
            if lower.contains(key) && line.contains(&format!("{key}:")) {
                let col = line.find(key).unwrap_or(0) as u32;
                return Some((i as u32, col, (col + key.len() as u32).max(col + 1)));
            }
        }
    }
    // Frontmatter start
    if text.starts_with("---") {
        Some((0, 0, 3))
    } else {
        None
    }
}

fn extract_path_from_line(line: &str) -> Option<String> {
    if let Some(start) = line.find('(') {
        if let Some(end) = line[start..].find(')') {
            let target = &line[start + 1..start + end];
            if target.ends_with(".md") {
                return Some(target.to_string());
            }
        }
    }
    for word in line.split_whitespace() {
        let clean = word.trim_matches(|c| c == '-' || c == '"' || c == '\'' || c == '`');
        if clean.ends_with(".md") {
            return Some(clean.to_string());
        }
    }
    None
}
