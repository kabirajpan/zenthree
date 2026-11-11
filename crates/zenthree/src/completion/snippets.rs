use std::path::Path;

#[derive(Debug, Clone)]
pub struct SnippetTemplate {
    pub label: &'static str,
    pub detail: &'static str,
    pub body: &'static str,
}

pub fn get_snippets_for_path(path: Option<&Path>) -> &'static [SnippetTemplate] {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "py" => &PYTHON_SNIPPETS,
        "rs" => &RUST_SNIPPETS,
        "ts" | "tsx" | "js" | "jsx" => &JS_TS_SNIPPETS,
        _ => &COMMON_SNIPPETS,
    }
}

pub static PYTHON_SNIPPETS: [SnippetTemplate; 22] = [
    SnippetTemplate {
        label: "if",
        detail: "if statement",
        body: "if condition:\n    pass",
    },
    SnippetTemplate {
        label: "ifelse",
        detail: "if statement with else",
        body: "if condition:\n    pass\nelse:\n    pass",
    },
    SnippetTemplate {
        label: "ifmain",
        detail: "if __name__ == '__main__'",
        body: "if __name__ == \"__main__\":\n    main()",
    },
    SnippetTemplate {
        label: "elif",
        detail: "elif branch",
        body: "elif condition:\n    pass",
    },
    SnippetTemplate {
        label: "for",
        detail: "for loop",
        body: "for value in iterable:\n    pass",
    },
    SnippetTemplate {
        label: "forr",
        detail: "for loop that iterates over range of integers",
        body: "for value in range():\n    pass",
    },
    SnippetTemplate {
        label: "afor",
        detail: "async for loop",
        body: "async for value in iterable:\n    pass",
    },
    SnippetTemplate {
        label: "def",
        detail: "function definition",
        body: "def func_name(args):\n    pass",
    },
    SnippetTemplate {
        label: "adef",
        detail: "async function definition",
        body: "async def func_name(args):\n    pass",
    },
    SnippetTemplate {
        label: "class",
        detail: "class definition",
        body: "class ClassName:\n    def __init__(self):\n        pass",
    },
    SnippetTemplate {
        label: "property",
        detail: "New property: get and set via decorator",
        body: "@property\ndef pname(self):\n    \"\"\"The property.\"\"\"\n    return self._pname\n\n@pname.setter\ndef pname(self, value):\n    self._pname = value",
    },
    SnippetTemplate {
        label: "pprint",
        detail: "Pretty print",
        body: "import pprint\npprint.pprint()",
    },
    SnippetTemplate {
        label: "pudb",
        detail: "Code snippet for pudb debug",
        body: "import pudb; pudb.set_trace()",
    },
    SnippetTemplate {
        label: "pdb",
        detail: "PDB debugger breakpoint",
        body: "import pdb; pdb.set_trace()",
    },
    SnippetTemplate {
        label: "ipdb",
        detail: "IPDB debugger breakpoint",
        body: "import ipdb; ipdb.set_trace()",
    },
    SnippetTemplate {
        label: "try",
        detail: "try / except block",
        body: "try:\n    pass\nexcept Exception as e:\n    pass",
    },
    SnippetTemplate {
        label: "tryf",
        detail: "try / except / finally",
        body: "try:\n    pass\nexcept Exception as e:\n    pass\nfinally:\n    pass",
    },
    SnippetTemplate {
        label: "except",
        detail: "except block",
        body: "except Exception as e:\n    pass",
    },
    SnippetTemplate {
        label: "exceptas",
        detail: "except Exception as e",
        body: "except Exception as e:\n    pass",
    },
    SnippetTemplate {
        label: "with",
        detail: "with context manager",
        body: "with open(filename, \"r\") as f:\n    pass",
    },
    SnippetTemplate {
        label: "import",
        detail: "import statement",
        body: "import module_name",
    },
    SnippetTemplate {
        label: "copyright",
        detail: "copyright header",
        body: "# Copyright (c) 2026. All rights reserved.",
    },
];

pub static RUST_SNIPPETS: [SnippetTemplate; 13] = [
    SnippetTemplate {
        label: "fn",
        detail: "function definition",
        body: "fn name() {\n    \n}",
    },
    SnippetTemplate {
        label: "pfn",
        detail: "public function",
        body: "pub fn name() {\n    \n}",
    },
    SnippetTemplate {
        label: "struct",
        detail: "struct definition",
        body: "pub struct Name {\n    \n}",
    },
    SnippetTemplate {
        label: "enum",
        detail: "enum definition",
        body: "pub enum Name {\n    \n}",
    },
    SnippetTemplate {
        label: "impl",
        detail: "impl block",
        body: "impl TypeName {\n    \n}",
    },
    SnippetTemplate {
        label: "match",
        detail: "match pattern",
        body: "match expr {\n    Some(val) => {},\n    None => {},\n}",
    },
    SnippetTemplate {
        label: "iflet",
        detail: "if let Some(val)",
        body: "if let Some(val) = expr {\n    \n}",
    },
    SnippetTemplate {
        label: "whilelet",
        detail: "while let Some(val)",
        body: "while let Some(val) = expr {\n    \n}",
    },
    SnippetTemplate {
        label: "for",
        detail: "for in iterator loop",
        body: "for item in iter {\n    \n}",
    },
    SnippetTemplate {
        label: "test",
        detail: "unit test function",
        body: "#[test]\nfn test_name() {\n    assert_eq!(1, 1);\n}",
    },
    SnippetTemplate {
        label: "modtest",
        detail: "test module",
        body: "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_basic() {\n        \n    }\n}",
    },
    SnippetTemplate {
        label: "derive",
        detail: "derive attribute",
        body: "#[derive(Debug, Clone, PartialEq)]",
    },
    SnippetTemplate {
        label: "loop",
        detail: "infinite loop",
        body: "loop {\n    \n}",
    },
];

pub static JS_TS_SNIPPETS: [SnippetTemplate; 12] = [
    SnippetTemplate {
        label: "clg",
        detail: "console.log",
        body: "console.log();",
    },
    SnippetTemplate {
        label: "fn",
        detail: "function declaration",
        body: "function name() {\n    \n}",
    },
    SnippetTemplate {
        label: "afn",
        detail: "arrow function",
        body: "const name = () => {\n    \n};",
    },
    SnippetTemplate {
        label: "if",
        detail: "if statement",
        body: "if (condition) {\n    \n}",
    },
    SnippetTemplate {
        label: "ifelse",
        detail: "if statement with else",
        body: "if (condition) {\n    \n} else {\n    \n}",
    },
    SnippetTemplate {
        label: "for",
        detail: "for loop",
        body: "for (let i = 0; i < len; i++) {\n    \n}",
    },
    SnippetTemplate {
        label: "forof",
        detail: "for...of loop",
        body: "for (const item of items) {\n    \n}",
    },
    SnippetTemplate {
        label: "forin",
        detail: "for...in loop",
        body: "for (const key in obj) {\n    \n}",
    },
    SnippetTemplate {
        label: "try",
        detail: "try / catch block",
        body: "try {\n    \n} catch (err) {\n    \n}",
    },
    SnippetTemplate {
        label: "async",
        detail: "async function declaration",
        body: "async function name() {\n    \n}",
    },
    SnippetTemplate {
        label: "prom",
        detail: "new Promise",
        body: "new Promise((resolve, reject) => {\n    \n});",
    },
    SnippetTemplate {
        label: "import",
        detail: "import declaration",
        body: "import { } from \"./\";",
    },
];

pub static COMMON_SNIPPETS: [SnippetTemplate; 4] = [
    SnippetTemplate {
        label: "if",
        detail: "if statement",
        body: "if (condition) {\n    \n}",
    },
    SnippetTemplate {
        label: "ifelse",
        detail: "if statement with else",
        body: "if (condition) {\n    \n} else {\n    \n}",
    },
    SnippetTemplate {
        label: "for",
        detail: "for loop",
        body: "for (int i = 0; i < n; i++) {\n    \n}",
    },
    SnippetTemplate {
        label: "ret",
        detail: "return statement",
        body: "return;",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSnippet {
    pub clean_text: String,
    pub placeholder_col_range: Option<(usize, usize)>,
    pub body_row_offset: Option<usize>,
    pub body_col: Option<usize>,
}

pub fn parse_snippet_template(template: &str) -> ParsedSnippet {
    // 1. Check for explicit tabstops like ${1:name} and $0 / $2
    if template.contains("${1:") || template.contains("$0") || template.contains("$1") {
        let mut clean = String::new();
        let mut placeholder_range = None;
        let mut body_row_offset = None;
        let mut body_col = None;

        let mut chars = template.chars().peekable();
        let mut current_row = 0;
        let mut current_col = 0;

        while let Some(ch) = chars.next() {
            if ch == '$' {
                if chars.peek() == Some(&'0') {
                    chars.next();
                    body_row_offset = Some(current_row);
                    body_col = Some(current_col);
                    continue;
                } else if chars.peek() == Some(&'{') {
                    // ${1:placeholder}
                    let mut snippet_var = String::new();
                    chars.next(); // consume '{'
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        }
                        snippet_var.push(c);
                    }
                    if let Some(colon_pos) = snippet_var.find(':') {
                        let default_val = &snippet_var[colon_pos + 1..];
                        let start_col = current_col;
                        let end_col = current_col + default_val.chars().count();
                        if placeholder_range.is_none() {
                            placeholder_range = Some((start_col, end_col));
                        }
                        for c in default_val.chars() {
                            clean.push(c);
                            current_col += 1;
                        }
                    }
                    continue;
                }
            }

            clean.push(ch);
            if ch == '\n' {
                current_row += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
        }

        return ParsedSnippet {
            clean_text: clean,
            placeholder_col_range: placeholder_range,
            body_row_offset,
            body_col,
        };
    }

    // 2. Standard block pattern heuristic:
    let lines: Vec<&str> = template.lines().collect();
    let mut placeholder_col_range = None;
    let mut body_row_offset = None;
    let mut body_col = None;

    if !lines.is_empty() {
        let first_line = lines[0];
        if let Some(idx) = first_line.find("name") {
            placeholder_col_range = Some((idx, idx + 4));
        } else if let Some(idx) = first_line.find("Name") {
            placeholder_col_range = Some((idx, idx + 4));
        } else if let Some(idx) = first_line.find("func_name") {
            placeholder_col_range = Some((idx, idx + 9));
        } else if let Some(idx) = first_line.find("ClassName") {
            placeholder_col_range = Some((idx, idx + 9));
        } else if let Some(idx) = first_line.find("TypeName") {
            placeholder_col_range = Some((idx, idx + 8));
        } else if let Some(idx) = first_line.find("condition") {
            placeholder_col_range = Some((idx, idx + 9));
        }
    }

    if lines.len() >= 3 {
        // e.g. fn name() {\n    \n}
        body_row_offset = Some(1);
        let second_line_indent = lines[1].chars().take_while(|c| c.is_whitespace()).count();
        body_col = Some(second_line_indent.max(4));
    } else if lines.len() == 2 && lines[1].trim().is_empty() {
        body_row_offset = Some(1);
        body_col = Some(4);
    }

    ParsedSnippet {
        clean_text: template.to_string(),
        placeholder_col_range,
        body_row_offset,
        body_col,
    }
}
