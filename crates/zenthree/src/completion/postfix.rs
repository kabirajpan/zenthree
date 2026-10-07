use std::path::Path;

pub struct PostfixTemplate {
    pub label: &'static str,
    pub detail: &'static str,
    pub template: &'static str, // "{expr}" will be replaced with previous expression
}

pub fn get_postfix_templates(path: Option<&Path>) -> &'static [PostfixTemplate] {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" => &RUST_POSTFIX,
        "ts" | "tsx" | "js" | "jsx" => &JS_TS_POSTFIX,
        "py" => &PYTHON_POSTFIX,
        _ => &COMMON_POSTFIX,
    }
}

pub static RUST_POSTFIX: [PostfixTemplate; 10] = [
    PostfixTemplate {
        label: "match",
        detail: "postfix: match expr { ... }",
        template: "match {expr} {\n    Ok(val) => { val },\n    Err(err) => { return Err(err.into()); },\n}",
    },
    PostfixTemplate {
        label: "if",
        detail: "postfix: if expr { ... }",
        template: "if {expr} {\n    \n}",
    },
    PostfixTemplate {
        label: "dbg",
        detail: "postfix: dbg!(expr)",
        template: "dbg!({expr})",
    },
    PostfixTemplate {
        label: "unwrap",
        detail: "postfix: expr.unwrap()",
        template: "{expr}.unwrap()",
    },
    PostfixTemplate {
        label: "not",
        detail: "postfix: !expr",
        template: "!{expr}",
    },
    PostfixTemplate {
        label: "ref",
        detail: "postfix: &expr",
        template: "&{expr}",
    },
    PostfixTemplate {
        label: "some",
        detail: "postfix: Some(expr)",
        template: "Some({expr})",
    },
    PostfixTemplate {
        label: "ok",
        detail: "postfix: Ok(expr)",
        template: "Ok({expr})",
    },
    PostfixTemplate {
        label: "err",
        detail: "postfix: Err(expr)",
        template: "Err({expr})",
    },
    PostfixTemplate {
        label: "ret",
        detail: "postfix: return expr;",
        template: "return {expr};",
    },
];

pub static JS_TS_POSTFIX: [PostfixTemplate; 6] = [
    PostfixTemplate {
        label: "log",
        detail: "postfix: console.log(expr)",
        template: "console.log({expr});",
    },
    PostfixTemplate {
        label: "if",
        detail: "postfix: if (expr) { ... }",
        template: "if ({expr}) {\n    \n}",
    },
    PostfixTemplate {
        label: "not",
        detail: "postfix: !expr",
        template: "!{expr}",
    },
    PostfixTemplate {
        label: "ret",
        detail: "postfix: return expr;",
        template: "return {expr};",
    },
    PostfixTemplate {
        label: "await",
        detail: "postfix: await expr",
        template: "await {expr}",
    },
    PostfixTemplate {
        label: "const",
        detail: "postfix: const name = expr;",
        template: "const val = {expr};",
    },
];

pub static PYTHON_POSTFIX: [PostfixTemplate; 8] = [
    PostfixTemplate {
        label: "pprint",
        detail: "Pretty print",
        template: "__import__('pprint').pprint({expr})",
    },
    PostfixTemplate {
        label: "property",
        detail: "Property decorator",
        template: "@property\ndef {expr}(self):\n    \"\"\"The {expr} property.\"\"\"\n    return self._{expr}",
    },
    PostfixTemplate {
        label: "import",
        detail: "Import module",
        template: "import {expr}",
    },
    PostfixTemplate {
        label: "print",
        detail: "Print expression",
        template: "print({expr})",
    },
    PostfixTemplate {
        label: "if",
        detail: "If condition block",
        template: "if {expr}:\n    pass",
    },
    PostfixTemplate {
        label: "not",
        detail: "Negate expression",
        template: "not {expr}",
    },
    PostfixTemplate {
        label: "ret",
        detail: "Return expression",
        template: "return {expr}",
    },
    PostfixTemplate {
        label: "len",
        detail: "Length of expression",
        template: "len({expr})",
    },
];

pub static COMMON_POSTFIX: [PostfixTemplate; 3] = [
    PostfixTemplate {
        label: "if",
        detail: "postfix: if (expr)",
        template: "if ({expr}) {\n}",
    },
    PostfixTemplate {
        label: "ret",
        detail: "postfix: return expr",
        template: "return {expr};",
    },
    PostfixTemplate {
        label: "not",
        detail: "postfix: !expr",
        template: "!{expr}",
    },
];
