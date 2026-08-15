use crate::model::{
    CodeRef, CodeRole, ContextSpec, Document, Frontmatter, FrontmatterState, ResourceRef,
};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn parse_document(root: &Path, path: PathBuf) -> io::Result<Document> {
    let text = fs::read_to_string(&path)?;
    Ok(parse_document_text(root, path, &text, true))
}

pub fn parse_document_text(root: &Path, path: PathBuf, text: &str, include_body: bool) -> Document {
    let (frontmatter, body) = split_frontmatter(text);
    let headings = extract_headings(body);
    let directory = path.parent().unwrap_or(root).to_path_buf();

    Document {
        path,
        directory,
        body: if include_body {
            body.to_string()
        } else {
            String::new()
        },
        headings,
        frontmatter: match frontmatter {
            Some(block) => match parse_frontmatter(block) {
                Ok(parsed) => FrontmatterState::Parsed(parsed),
                Err(err) => FrontmatterState::Invalid(err),
            },
            None => FrontmatterState::Absent,
        },
    }
}

pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    if !text.starts_with("---") {
        return (None, text);
    }

    let mut lines = text.split('\n');
    let first = lines.next().unwrap();
    if first.trim_end_matches('\r') != "---" {
        return (None, text);
    }

    let mut current_offset = first.len() + 1;
    let mut found_end_offset = None;

    for line in lines {
        let line_len_with_nl = line.len() + 1;
        if line.trim_end_matches('\r') == "---" {
            found_end_offset = Some(current_offset);
            break;
        }
        current_offset += line_len_with_nl;
    }

    match found_end_offset {
        Some(end_offset) => {
            let frontmatter = &text[first.len() + 1..end_offset];
            // frontmatter block trim trailing \n or \r\n
            let frontmatter = frontmatter.trim_end_matches('\r').trim_end_matches('\n');

            // body starts after the "---" line we just found
            let body_start = end_offset + 3; // "---" has length 3
            let body = if body_start < text.len() {
                let mut b = &text[body_start..];
                if b.starts_with('\r') {
                    b = &b[1..];
                }
                if b.starts_with('\n') {
                    b = &b[1..];
                }
                b
            } else {
                ""
            };
            (Some(frontmatter), body)
        }
        None => (Some(&text[first.len() + 1..]), ""),
    }
}

pub fn extract_headings(body: &str) -> Vec<String> {
    extract_heading_groups(body)
        .into_iter()
        .filter_map(|group| group.into_iter().next())
        .collect()
}

pub fn extract_heading_groups(body: &str) -> Vec<Vec<String>> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if let Some(h) = trimmed.strip_prefix("## ") {
                Some(h.trim())
            } else if let Some(h) = trimmed.strip_prefix("### ") {
                Some(h.trim())
            } else {
                None
            }
        })
        .filter(|heading| !heading.is_empty())
        .map(parse_heading_group)
        .collect()
}

pub fn split_markdown_link_target(text: &str) -> Option<String> {
    let start = text.find("](")? + 2;
    let rest = text.get(start..)?;
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

pub fn document_id(root: &Path, path: &Path, frontmatter: Option<&Frontmatter>) -> String {
    if let Some(id) = frontmatter.and_then(|fm| fm.id.as_ref()) {
        return id.replace("\\", "/").to_lowercase();
    }

    let relative = path.strip_prefix(root).unwrap_or(path);
    let without_ext = relative.with_extension("");
    without_ext
        .iter()
        .map(|component| component.to_string_lossy().to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join("/")
}

fn parse_frontmatter(block: &str) -> Result<Frontmatter, String> {
    let lines = block.lines().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut frontmatter = Frontmatter::default();

    while let Some(raw_line) = lines.get(index) {
        let line = raw_line.trim();
        index += 1;

        if line.is_empty() {
            continue;
        }

        let Some((key, rest)) = line.split_once(':') else {
            return Err(format!("invalid frontmatter line: {line}"));
        };

        let key = key.trim();
        let rest = rest.trim();
        frontmatter.present_keys.insert(key.to_lowercase());

        if key == "title" {
            frontmatter.title = scalar_value(rest);
        }

        match key {
            "profile" => frontmatter.profile = scalar_value(rest).map(|s| s.to_lowercase()),
            "status" => frontmatter.status = scalar_value(rest).map(|s| s.to_lowercase()),
            "created" | "created_at" | "date" => frontmatter.created = scalar_value(rest),
            "updated" | "last_updated" | "updated_at" => frontmatter.updated = scalar_value(rest),
            "share" => frontmatter.share = scalar_value(rest).map(|s| s.to_lowercase()),
            "description" => frontmatter.description = scalar_value(rest),
            "id" => {
                frontmatter.id = scalar_value(rest).map(|s| s.replace('\\', "/").to_lowercase())
            }
            "owner" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                if !items.is_empty() {
                    frontmatter.owner = Some(items.join(", "));
                    index = next;
                } else {
                    frontmatter.owner = scalar_value(rest);
                }
            }
            "ods" => {
                if !rest.is_empty() {
                    frontmatter.ods = scalar_value(rest).map(|s| s.to_lowercase());
                } else {
                    // Parse nested ods: map block
                    let (nested_fm, next) = parse_nested_ods_map(&lines, index, 2)?;
                    if nested_fm.profile.is_some() {
                        frontmatter.profile = nested_fm.profile;
                    }
                    if nested_fm.status.is_some() {
                        frontmatter.status = nested_fm.status;
                    }
                    if nested_fm.created.is_some() {
                        frontmatter.created = nested_fm.created;
                    }
                    if nested_fm.updated.is_some() {
                        frontmatter.updated = nested_fm.updated;
                    }
                    if nested_fm.share.is_some() {
                        frontmatter.share = nested_fm.share;
                    }
                    if nested_fm.id.is_some() {
                        frontmatter.id = nested_fm.id;
                    }
                    if !nested_fm.depends.is_empty() {
                        frontmatter.depends.extend(nested_fm.depends);
                    }
                    if !nested_fm.related.is_empty() {
                        frontmatter.related.extend(nested_fm.related);
                    }
                    if !nested_fm.resources.is_empty() {
                        frontmatter.resources.extend(nested_fm.resources);
                    }
                    if !nested_fm.code.is_empty() {
                        frontmatter.code.extend(nested_fm.code);
                    }
                    if nested_fm.context.is_some() {
                        frontmatter.context = nested_fm.context;
                    }
                    if nested_fm.custom_profile.is_some() {
                        frontmatter.custom_profile = nested_fm.custom_profile;
                    }
                    // Nested tags under ods: are invalid (root-only contract). Merge into the
                    // model so lint/find can surface them; migrate must hoist to top-level.
                    if !nested_fm.tags.is_empty() || nested_fm.tags_misplaced {
                        frontmatter.tags.extend(nested_fm.tags);
                        frontmatter.tags_misplaced = true;
                    }
                    index = next;
                }
            }
            "profiles" | "custom-profiles" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.profiles.extend(items);
                mark_non_null_key(&mut frontmatter, "profiles", rest, index, next);
                index = next;
            }
            "packs" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.packs.extend(items);
                mark_non_null_key(&mut frontmatter, "packs", rest, index, next);
                index = next;
            }
            "name" => frontmatter.name = scalar_value(rest),
            "ignore" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.ignore.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace('\\', "/").trim_end_matches('/').to_string())
                        .filter(|s| !s.is_empty()),
                );
                mark_non_null_key(&mut frontmatter, "ignore", rest, index, next);
                index = next;
            }
            "depends" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.depends.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace("\\", "/").to_lowercase()),
                );
                mark_non_null_key(&mut frontmatter, "depends", rest, index, next);
                index = next;
            }
            "related" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.related.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace("\\", "/").to_lowercase()),
                );
                mark_non_null_key(&mut frontmatter, "related", rest, index, next);
                index = next;
            }
            "tags" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                // Normalize each entry; keep duplicates so lint can warn.
                for item in items {
                    if let Some(n) = crate::tags::normalize_tag(&item) {
                        frontmatter.tags.push(n);
                    }
                }
                mark_non_null_key(&mut frontmatter, "tags", rest, index, next);
                index = next;
            }
            "resources" => {
                let (items, next) = parse_resources(&lines, index, 2)?;
                frontmatter.resources.extend(items);
                mark_non_null_key(&mut frontmatter, "resources", rest, index, next);
                index = next;
            }
            "code" => {
                let (items, next) = parse_code_refs(&lines, index, 2)?;
                frontmatter.code.extend(items);
                mark_non_null_key(&mut frontmatter, "code", rest, index, next);
                index = next;
            }
            "context" => {
                let (context, next) = parse_context(&lines, index, 2)?;
                frontmatter.context = Some(context);
                mark_non_null_key(&mut frontmatter, "context", rest, index, next);
                index = next;
            }
            "aliases" => {
                let (aliases, next) = parse_aliases(&lines, index, 2);
                frontmatter.aliases.extend(aliases);
                mark_non_null_key(&mut frontmatter, "aliases", rest, index, next);
                index = next;
            }
            "specs" => {
                let (specs, next) = parse_specs_config(&lines, index, 2);
                frontmatter.specs = specs;
                mark_non_null_key(&mut frontmatter, "specs", rest, index, next);
                index = next;
            }
            "okf_lint" | "okf-lint" => {
                let (cfg, next) = parse_spec_lint_config(&lines, index, 2);
                frontmatter.specs.okf = cfg;
                mark_non_null_key(&mut frontmatter, "okf_lint", rest, index, next);
                index = next;
            }
            "skills_lint" | "skills-lint" => {
                let (cfg, next) = parse_spec_lint_config(&lines, index, 2);
                frontmatter.specs.skills = cfg;
                mark_non_null_key(&mut frontmatter, "skills_lint", rest, index, next);
                index = next;
            }
            _ => {
                let (val, next) = parse_custom_value(&lines, index, 2, rest);
                // Case-fold keys so queries are case-insensitive; last write wins.
                frontmatter
                    .custom_keys
                    .insert(key.trim().to_lowercase(), val);
                index = next;
            }
        }
    }

    Ok(frontmatter)
}

include!("frontmatter_helpers.rs");
