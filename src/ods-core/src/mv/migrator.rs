/// Canonical order of ODS 2.0 top-level engine keys (specs/ods/keys.md).
///
/// Keep in lockstep with [`crate::spec::SpecSchema::canonical_engine_key_order`]
/// (asserted in schema unit tests).
const CANONICAL_ODS_KEY_ORDER: [&str; 9] = [
    "profile",
    "status",
    "id",
    "share",
    "depends",
    "related",
    "resources",
    "code",
    "load",
];

/// Universal top-level keys that must never live under `ods:`.
/// When found nested, migrate hoists them to root (SSG / multi-tool interop).
const UNIVERSAL_TOP_LEVEL_KEYS: [&str; 1] = ["tags"];

/// Migrate one document's raw frontmatter text into ODS 2.0 flat top-level layout:
/// engine keys (`profile`, `status`, `id`, `share`, `depends`, `related`, `resources`, `code`, `load`)
/// at the root (no nested `ods:` wrapper), with universal keys (`tags`, `description`, `owner`, …) preserved.
///
/// Legacy nested `ods:` maps are hoisted to flat keys; nested `context.load` becomes top-level `load`.
///
/// Operates on raw text/lines, never on the parsed [`crate::model::Frontmatter`]
/// struct, because that struct is lossy for `owner` and `code[].symbol`
/// (both collapse YAML list-vs-scalar form into a single joined string) —
/// re-emitting from the struct would silently corrupt those fields' original
/// shape. Idempotent: returns `None` if nothing changes.
///
/// Skips (returns `None` for) documents that use a scalar `ods: <version>`
/// marker line (the root `index.md` workspace-marker form) rather than a
/// nested `ods:` map, and documents with no frontmatter block at all.
pub fn migrate_frontmatter_to_canonical(text: &str) -> Option<String> {
    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let frontmatter = frontmatter?;

    if has_scalar_ods_marker(frontmatter) {
        return None;
    }

    let blocks = group_top_level_blocks(frontmatter);
    if blocks.is_empty() {
        return None;
    }

    let mut engine: std::collections::BTreeMap<&str, (usize, Vec<String>)> =
        std::collections::BTreeMap::new();
    // Universal keys found under nested `ods:` (e.g. tags) — hoist to root.
    let mut hoisted_universal: Vec<(String, Vec<String>)> = Vec::new();
    // Unknown nested keys under `ods:` — preserve as opaque blocks (non-destructive policy).
    let mut unknown_nested: Vec<Vec<String>> = Vec::new();
    let mut had_nested_ods = false;
    let mut had_flat_engine = false;

    for (position, block) in blocks.iter().enumerate() {
        if block.key == "context" {
            had_flat_engine = true;
            if let Some(load_lines) = extract_context_load_lines(&block.lines) {
                engine.insert("load", (position, load_lines));
            }
        } else if let Some(&canonical_key) = CANONICAL_ODS_KEY_ORDER.iter().find(|k| **k == block.key)
        {
            had_flat_engine = true;
            engine.insert(canonical_key, (position, block.lines.clone()));
        } else if block.key == "ods" {
            had_nested_ods = true;
            for sub in group_sub_blocks(&block.lines[1..], 2) {
                if sub.key == "context" {
                    if let Some(load_lines) = extract_context_load_lines(&sub.lines) {
                        engine.insert("load", (position, load_lines));
                    }
                } else if let Some(&canonical_key) =
                    CANONICAL_ODS_KEY_ORDER.iter().find(|k| **k == sub.key)
                {
                    let candidate_wins = match engine.get(canonical_key) {
                        Some((existing_position, _)) => position >= *existing_position,
                        None => true,
                    };
                    if candidate_wins {
                        engine.insert(canonical_key, (position, sub.lines));
                    }
                } else if UNIVERSAL_TOP_LEVEL_KEYS.contains(&sub.key.as_str()) {
                    // Hoist nested universal keys (tags) to root; de-indent from ods nesting.
                    let root_lines = deindent(&sub.lines, 2);
                    hoisted_universal.push((sub.key.clone(), root_lines));
                } else {
                    // Preserve foreign / unknown keys nested under ods: (do not drop).
                    unknown_nested.push(sub.lines);
                }
            }
        }
    }

    // Need engine keys and/or nested tags to hoist; otherwise nothing to do.
    if engine.is_empty() && hoisted_universal.is_empty() {
        return None;
    }
    // Pure hoist of nested tags without any engine keys still rewrites.
    if engine.is_empty() && !had_nested_ods && !had_flat_engine {
        return None;
    }

    let mut new_frontmatter_lines: Vec<String> = Vec::new();
    let mut root_tags_emitted = false;
    for block in &blocks {
        let is_engine_key = CANONICAL_ODS_KEY_ORDER.contains(&block.key.as_str())
            || block.key == "context";
        if is_engine_key || block.key == "ods" {
            continue;
        }
        if block.key == "tags" {
            // Merge root tags with any nested tags being hoisted.
            let merged = merge_tag_blocks(&block.lines, &hoisted_universal);
            new_frontmatter_lines.extend(merged);
            root_tags_emitted = true;
            continue;
        }
        new_frontmatter_lines.extend(block.lines.iter().cloned());
    }

    // No root tags block yet: emit hoisted tags before the ods: engine map.
    if !root_tags_emitted {
        for (key, lines) in &hoisted_universal {
            if key == "tags" {
                new_frontmatter_lines.extend(lines.iter().cloned());
            }
        }
    }

    if !engine.is_empty() || !unknown_nested.is_empty() {
        // ODS 2.0: flat top-level engine keys (no `ods:` wrapper).
        for key in CANONICAL_ODS_KEY_ORDER {
            if let Some((_, lines)) = engine.get(key) {
                let indent = if had_nested_ods && lines.first().is_some_and(|l| l.starts_with("  ")) {
                    2
                } else {
                    0
                };
                new_frontmatter_lines.extend(deindent(lines, indent));
            }
        }
        for lines in &unknown_nested {
            new_frontmatter_lines.extend(deindent(lines, 2));
        }
    }

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let new_frontmatter = new_frontmatter_lines.join(ending);
    let out = if body.is_empty() {
        format!("---{ending}{new_frontmatter}{ending}---{ending}")
    } else {
        format!("---{ending}{new_frontmatter}{ending}---{ending}{body}")
    };

    if out == text {
        None
    } else {
        Some(out)
    }
}

/// Pull `load:` list lines from a legacy `context:` block for 2.0 migration.
fn extract_context_load_lines(ctx_lines: &[String]) -> Option<Vec<String>> {
    let mut in_load = false;
    let mut out = Vec::new();
    for line in ctx_lines {
        let trimmed = line.trim();
        if trimmed.starts_with("load:") {
            in_load = true;
            if let Some(rest) = trimmed.strip_prefix("load:").map(str::trim) {
                if !rest.is_empty() {
                    out.push(format!("load: {rest}"));
                    in_load = false;
                }
            }
            continue;
        }
        if in_load {
            if trimmed.starts_with("- ") {
                let item = deindent(std::slice::from_ref(line), 4);
                if let Some(first) = item.into_iter().next() {
                    out.push(first);
                }
            } else if !trimmed.is_empty() && !trimmed.ends_with(':') {
                in_load = false;
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        let mut block = vec!["load:".to_string()];
        block.extend(out);
        Some(block)
    }
}

/// Remove up to `spaces` leading spaces from each line (for hoisting nested blocks).
fn deindent(lines: &[String], spaces: usize) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let mut drop = 0usize;
            for ch in line.chars() {
                if ch == ' ' && drop < spaces {
                    drop += 1;
                } else {
                    break;
                }
            }
            line[drop..].to_string()
        })
        .collect()
}

/// Merge an existing root `tags:` block with hoisted nested tag blocks.
/// Prefers list form; appends unique normalized values from nested lists.
fn merge_tag_blocks(
    root_tags_lines: &[String],
    hoisted: &[(String, Vec<String>)],
) -> Vec<String> {
    let nested_tag_lines: Vec<&[String]> = hoisted
        .iter()
        .filter(|(k, _)| k == "tags")
        .map(|(_, lines)| lines.as_slice())
        .collect();
    if nested_tag_lines.is_empty() {
        return root_tags_lines.to_vec();
    }

    // Collect all tag values from root + nested (list items or inline).
    let mut values: Vec<String> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for lines in std::iter::once(root_tags_lines).chain(nested_tag_lines) {
        for v in extract_tag_values_from_block(lines) {
            if let Some(n) = crate::tags::normalize_tag(&v)
                && seen.insert(n.clone())
            {
                values.push(n);
            }
        }
    }

    if values.is_empty() {
        return root_tags_lines.to_vec();
    }

    // Emit canonical list form at root.
    let mut out = vec!["tags:".to_string()];
    for v in values {
        out.push(format!("  - {v}"));
    }
    out
}

fn extract_tag_values_from_block(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if lines.is_empty() {
        return out;
    }
    let first = lines[0].trim_start();
    if let Some(rest) = first.strip_prefix("tags:") {
        let rest = rest.trim();
        if !rest.is_empty() && !rest.starts_with('#') {
            if rest.starts_with('[') && rest.ends_with(']') {
                let inner = &rest[1..rest.len() - 1];
                for p in inner.split(',') {
                    let p = p.trim().trim_matches(|c| c == '"' || c == '\'');
                    if !p.is_empty() {
                        out.push(p.to_string());
                    }
                }
            } else {
                let p = rest.trim_matches(|c| c == '"' || c == '\'');
                if !p.is_empty() {
                    out.push(p.to_string());
                }
            }
            return out;
        }
    }
    for line in lines.iter().skip(1) {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix('-') {
            let rest = rest.trim().trim_matches(|c| c == '"' || c == '\'');
            if !rest.is_empty() {
                out.push(rest.to_string());
            }
        }
    }
    out
}

/// Rewrite every document under `root` into canonical Pattern B frontmatter
/// shape via [`migrate_frontmatter_to_canonical`]. Skips the workspace root
/// `index.md` (the scalar `ods: <version>` marker file) and any document
/// whose frontmatter failed to parse or is absent.
pub fn migrate_workspace_frontmatter(root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let workspace = load_workspace(root.as_ref())?;
    migrate_workspace_frontmatter_with_workspace(&workspace)
}

/// Same as [`migrate_workspace_frontmatter`], but takes an already-loaded
/// `Workspace` instead of reloading — each document's text is still
/// re-read fresh from disk, so this is safe to run after
/// [`normalize_workspace_frontmatter_spacing_with_workspace`] and
/// [`canonicalize_workspace_document_refs_with_workspace`] against the same
/// workspace.
pub fn migrate_workspace_frontmatter_with_workspace(
    workspace: &crate::model::Workspace,
) -> io::Result<Vec<PathBuf>> {
    let root_index = workspace.root.join("index.md");
    let mut changed = Vec::new();

    for document in &workspace.documents {
        if document.path == root_index {
            continue;
        }
        if !matches!(
            document.frontmatter,
            crate::model::FrontmatterState::Parsed(_)
        ) {
            continue;
        }

        let text = match fs::read_to_string(&document.path) {
            Ok(text) => text,
            Err(_) => continue,
        };

        if let Some(next) = migrate_frontmatter_to_canonical(&text) {
            fs::write(&document.path, &next)?;
            changed.push(document.path.clone());
        }
    }

    Ok(changed)
}

struct Block {
    key: String,
    lines: Vec<String>,
}

/// True if `frontmatter` contains a top-level `ods: <value>` line (the root
/// workspace scalar version marker, e.g. `ods: 0.1`), as opposed to a bare
/// `ods:` line that introduces a nested map.
fn has_scalar_ods_marker(frontmatter: &str) -> bool {
    frontmatter.lines().any(|line| {
        indent(line) == 0
            && line
                .trim_start()
                .strip_prefix("ods:")
                .is_some_and(|rest| !rest.trim().is_empty())
    })
}

/// Group frontmatter lines into top-level (indent == 0) key blocks: a block
/// is its key line plus every following line up to (not including) the next
/// indent-0 line. Blank lines are dropped — they carry no semantic weight in
/// YAML frontmatter and `normalize_frontmatter_body_spacing` separately owns
/// edge-of-block spacing.
fn group_top_level_blocks(frontmatter: &str) -> Vec<Block> {
    group_blocks(frontmatter.lines(), 0)
}

/// Same grouping one level down: `min_indent`-indented key lines plus
/// deeper-indented continuation lines belong to the preceding key's block.
fn group_sub_blocks(lines: &[String], min_indent: usize) -> Vec<Block> {
    group_blocks(lines.iter().map(|s| s.as_str()), min_indent)
}

fn group_blocks<'a>(lines: impl Iterator<Item = &'a str>, key_indent: usize) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if indent(line) == key_indent {
            let key = line
                .trim_start()
                .split_once(':')
                .map(|(k, _)| k.trim().to_string())
                .unwrap_or_else(|| line.trim().to_string());
            blocks.push(Block {
                key,
                lines: vec![line.to_string()],
            });
        } else if let Some(last) = blocks.last_mut() {
            last.lines.push(line.to_string());
        }
    }
    blocks
}

fn indent(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}
