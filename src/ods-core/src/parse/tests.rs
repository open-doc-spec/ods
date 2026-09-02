include!("helpers.rs");

fn parse_passthrough_block(lines: &[&str], start: usize, min_indent: usize) -> ((), usize) {
    let mut index = start;
    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }
        if indent(raw_line) < min_indent {
            break;
        }
        index += 1;
    }
    ((), index)
}

fn parse_resource_kv(
    text: &str,
    path: &mut Option<PathBuf>,
    url: &mut Option<String>,
) -> Result<(), String> {
    let Some((key, rest)) = text.split_once(':') else {
        return Err(format!("invalid resource entry: {text}"));
    };

    match key.trim() {
        "path" => *path = Some(PathBuf::from(unquote(rest.trim()))),
        // `url:` values contain their own `://`, so re-join the split remainder.
        "url" => *url = Some(unquote(rest.trim()).to_string()),
        _ => {}
    }

    Ok(())
}

fn parse_code_kv(
    text: &str,
    path: &mut Option<PathBuf>,
    symbol: &mut Option<String>,
    role: &mut Option<CodeRole>,
) -> Result<(), String> {
    if let Some(item) = text.strip_prefix("- ") {
        let val = unquote(item.trim());
        if !val.is_empty() {
            if let Some(s) = symbol {
                s.push_str(", ");
                s.push_str(&val);
            } else {
                *symbol = Some(val);
            }
        }
        return Ok(());
    }

    let Some((key, rest)) = text.split_once(':') else {
        return Err(format!("invalid code entry: {text}"));
    };

    match key.trim() {
        "path" => {
            *path = Some(PathBuf::from(unquote(rest.trim()).replace('\\', "/")));
        }
        "symbol" => {
            let trimmed = rest.trim();
            if trimmed.starts_with('[') {
                let items = parse_inline_list(trimmed);
                if !items.is_empty() {
                    *symbol = Some(items.join(", "));
                }
            } else {
                let value = unquote(trimmed);
                if !value.is_empty() {
                    *symbol = Some(value);
                }
            }
        }
        "role" => {
            let value = unquote(rest.trim()).to_lowercase();
            let Some(parsed) = CodeRole::parse(&value) else {
                return Err(format!("invalid code role: {value}"));
            };
            *role = Some(parsed);
        }
        _ => {}
    }

    Ok(())
}

fn parse_inline_list(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    let inner = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']'));
    inner
        .map(|items| {
            items
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(unquote)
                .collect()
        })
        .unwrap_or_else(|| vec![unquote(trimmed)])
}

fn indent(text: &str) -> usize {
    text.chars().take_while(|ch| *ch == ' ').count()
}

fn unquote(text: &str) -> String {
    let trimmed = text.trim();
    let maybe_stripped = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"'));
    maybe_stripped
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(trimmed)
        .to_string()
}

#[cfg(test)]
include!("v02_tests.rs");
