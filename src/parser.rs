
#[derive(Debug, PartialEq)]
pub struct Export {
    pub name: String,
    pub value: String,
}

pub fn parse_posix_exports(input: &str) -> Vec<Export> {
    let mut exports = Vec::new();

    // handle multiline input
    for line in input.lines() {
        // split by && to handle multiple commands on same line
        for segment in line.split("&&") {
            let trimmed = segment.trim();

            // check if this is an export command
            if trimmed.starts_with("export ") {
                let export_content = &trimmed[7..].trim(); // remove "export "
                parse_export_content(export_content, &mut exports);
            } else if trimmed.starts_with("export") && trimmed.len() > 6 {
                // handle cases like "export VAR=value" without space
                let export_content = &trimmed[6..].trim();
                parse_export_content(export_content, &mut exports);
            }
        }
    }

    exports
}

fn parse_export_content(content: &str, exports: &mut Vec<Export>) {
    let mut chars = content.chars().peekable();
    let mut current_var = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
                current_var.push(ch);
            }
            '"' | '\'' if in_quotes && ch == quote_char => {
                // check if escaped
                if current_var.ends_with('\\') {
                    current_var.push(ch);
                } else {
                    in_quotes = false;
                    current_var.push(ch);
                }
            }
            ' ' | '\t' if !in_quotes => {
                // end of current variable
                if !current_var.is_empty() {
                    if let Some(eq_pos) = current_var.find('=') {
                        let name = current_var[..eq_pos].to_string();
                        let value = parse_value(&current_var[eq_pos + 1..]);
                        exports.push(Export { name, value });
                    }
                    current_var.clear();
                }
            }
            _ => {
                current_var.push(ch);
            }
        }
    }

    // handle any remaining variable
    if !current_var.is_empty() {
        if let Some(eq_pos) = current_var.find('=') {
            let name = current_var[..eq_pos].to_string();
            let value = parse_value(&current_var[eq_pos + 1..]);
            exports.push(Export { name, value });
        }
    }
}

fn parse_value(value_str: &str) -> String {
    let trimmed = value_str.trim();

    // handle quoted values
    if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
       (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        // remove quotes and handle escaped characters
        let unquoted = &trimmed[1..trimmed.len()-1];

        if trimmed.starts_with('"') {
            // in double quotes, handle escape sequences
            unquoted.replace("\\\"", "\"")
                   .replace("\\\\", "\\")
                   .replace("\\n", "\n")
                   .replace("\\t", "\t")
                   .replace("\\r", "\r")
        } else {
            // single quotes preserve everything literally
            unquoted.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

pub fn exports_to_nushell(exports: Vec<Export>) -> String {
    exports.into_iter()
        .map(|export| {
            // escape the value for Nushell if needed
            let value = if export.value.contains(' ') ||
                          export.value.contains('"') ||
                          export.value.contains('\'') ||
                          export.value.contains('$') ||
                          export.value.contains('\\') {
                format!("\"{}\"", export.value.replace('\\', "\\\\").replace('"', "\\\""))
            } else if export.value.is_empty() {
                "\"\"".to_string()
            } else {
                export.value
            };

            format!("$env.{} = {}", export.name, value)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_export() {
        let input = "export FOO=bar";
        let exports = parse_posix_exports(input);
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "FOO");
        assert_eq!(exports[0].value, "bar");
    }

    #[test]
    fn test_multiple_exports_same_line() {
        let input = "export FOO=bar && export BAZ=qux";
        let exports = parse_posix_exports(input);
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "FOO");
        assert_eq!(exports[0].value, "bar");
        assert_eq!(exports[1].name, "BAZ");
        assert_eq!(exports[1].value, "qux");
    }

    #[test]
    fn test_multiple_vars_one_export() {
        let input = "export FOO=bar BAZ=qux";
        let exports = parse_posix_exports(input);
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "FOO");
        assert_eq!(exports[0].value, "bar");
        assert_eq!(exports[1].name, "BAZ");
        assert_eq!(exports[1].value, "qux");
    }

    #[test]
    fn test_quoted_values() {
        let input = r#"export FOO="hello world" && export BAR='single quotes'"#;
        let exports = parse_posix_exports(input);
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "FOO");
        assert_eq!(exports[0].value, "hello world");
        assert_eq!(exports[1].name, "BAR");
        assert_eq!(exports[1].value, "single quotes");
    }

    #[test]
    fn test_escaped_quotes() {
        let input = r#"export FOO="hello \"world\"""#;
        let exports = parse_posix_exports(input);
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].name, "FOO");
        assert_eq!(exports[0].value, "hello \"world\"");
    }

    #[test]
    fn test_multiline_input() {
        let input = "export FOO=bar\nexport BAZ=qux";
        let exports = parse_posix_exports(input);
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].name, "FOO");
        assert_eq!(exports[0].value, "bar");
        assert_eq!(exports[1].name, "BAZ");
        assert_eq!(exports[1].value, "qux");
    }

    #[test]
    fn test_to_nushell() {
        let exports = vec![
            Export { name: "FOO".to_string(), value: "bar".to_string() },
            Export { name: "PATH".to_string(), value: "/usr/bin:/bin".to_string() },
            Export { name: "MESSAGE".to_string(), value: "hello world".to_string() },
        ];

        let nushell = exports_to_nushell(exports);
        let expected = "$env.FOO = bar\n$env.PATH = /usr/bin:/bin\n$env.MESSAGE = \"hello world\"";
        assert_eq!(nushell, expected);
    }
}