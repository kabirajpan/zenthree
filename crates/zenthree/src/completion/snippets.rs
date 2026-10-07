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
