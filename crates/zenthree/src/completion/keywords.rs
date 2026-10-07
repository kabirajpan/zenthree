use std::path::Path;

pub struct KeywordItem {
    pub label: &'static str,
    pub detail: &'static str,
}

pub fn get_keywords_for_path(path: Option<&Path>) -> &'static [KeywordItem] {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" => &RUST_KEYWORDS,
        "ts" | "tsx" | "js" | "jsx" => &JS_TS_KEYWORDS,
        "py" => &PYTHON_KEYWORDS,
        "go" => &GO_KEYWORDS,
        "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => &CPP_KEYWORDS,
        _ => &COMMON_KEYWORDS,
    }
}

pub static RUST_KEYWORDS: [KeywordItem; 28] = [
    KeywordItem { label: "fn", detail: "keyword fn name() { ... }" },
    KeywordItem { label: "let", detail: "keyword let binding" },
    KeywordItem { label: "mut", detail: "keyword mutable modifier" },
    KeywordItem { label: "pub", detail: "keyword public visibility" },
    KeywordItem { label: "struct", detail: "keyword struct definition" },
    KeywordItem { label: "enum", detail: "keyword enum definition" },
    KeywordItem { label: "impl", detail: "keyword implementation block" },
    KeywordItem { label: "trait", detail: "keyword trait definition" },
    KeywordItem { label: "match", detail: "keyword pattern matching" },
    KeywordItem { label: "if", detail: "keyword conditional branch" },
    KeywordItem { label: "else", detail: "keyword alternative branch" },
    KeywordItem { label: "loop", detail: "keyword infinite loop" },
    KeywordItem { label: "while", detail: "keyword conditional loop" },
    KeywordItem { label: "for", detail: "keyword for in iterator loop" },
    KeywordItem { label: "in", detail: "keyword in iterator binding" },
    KeywordItem { label: "return", detail: "keyword return expression" },
    KeywordItem { label: "break", detail: "keyword break loop" },
    KeywordItem { label: "continue", detail: "keyword continue next iteration" },
    KeywordItem { label: "use", detail: "keyword use path import" },
    KeywordItem { label: "mod", detail: "keyword module declaration" },
    KeywordItem { label: "const", detail: "keyword constant value" },
    KeywordItem { label: "static", detail: "keyword static global variable" },
    KeywordItem { label: "async", detail: "keyword asynchronous function/block" },
    KeywordItem { label: "await", detail: "keyword await future completion" },
    KeywordItem { label: "where", detail: "keyword generic clause constraints" },
    KeywordItem { label: "type", detail: "keyword type alias" },
    KeywordItem { label: "self", detail: "keyword current instance value" },
    KeywordItem { label: "Self", detail: "keyword implementing type" },
];

pub static JS_TS_KEYWORDS: [KeywordItem; 26] = [
    KeywordItem { label: "function", detail: "keyword function declaration" },
    KeywordItem { label: "const", detail: "keyword constant declaration" },
    KeywordItem { label: "let", detail: "keyword mutable block-scoped variable" },
    KeywordItem { label: "var", detail: "keyword function-scoped variable" },
    KeywordItem { label: "return", detail: "keyword return statement" },
    KeywordItem { label: "if", detail: "keyword conditional branch" },
    KeywordItem { label: "else", detail: "keyword alternative branch" },
    KeywordItem { label: "for", detail: "keyword for loop" },
    KeywordItem { label: "while", detail: "keyword while loop" },
    KeywordItem { label: "switch", detail: "keyword switch statement" },
    KeywordItem { label: "case", detail: "keyword switch case" },
    KeywordItem { label: "break", detail: "keyword break loop/case" },
    KeywordItem { label: "continue", detail: "keyword continue loop" },
    KeywordItem { label: "class", detail: "keyword class definition" },
    KeywordItem { label: "interface", detail: "keyword TypeScript interface" },
    KeywordItem { label: "type", detail: "keyword TypeScript type alias" },
    KeywordItem { label: "import", detail: "keyword module import" },
    KeywordItem { label: "export", detail: "keyword module export" },
    KeywordItem { label: "from", detail: "keyword import source" },
    KeywordItem { label: "default", detail: "keyword default export/case" },
    KeywordItem { label: "async", detail: "keyword async function" },
    KeywordItem { label: "await", detail: "keyword await promise resolution" },
    KeywordItem { label: "try", detail: "keyword try error-handling block" },
    KeywordItem { label: "catch", detail: "keyword catch error block" },
    KeywordItem { label: "finally", detail: "keyword finally clean-up block" },
    KeywordItem { label: "throw", detail: "keyword throw error" },
];

pub static PYTHON_KEYWORDS: [KeywordItem; 24] = [
    KeywordItem { label: "def", detail: "keyword function definition" },
    KeywordItem { label: "class", detail: "keyword class definition" },
    KeywordItem { label: "import", detail: "keyword import module" },
    KeywordItem { label: "from", detail: "keyword from module import" },
    KeywordItem { label: "return", detail: "keyword return value" },
    KeywordItem { label: "yield", detail: "keyword generator yield" },
    KeywordItem { label: "if", detail: "keyword conditional branch" },
    KeywordItem { label: "elif", detail: "keyword else-if branch" },
    KeywordItem { label: "else", detail: "keyword else branch" },
    KeywordItem { label: "for", detail: "keyword for loop" },
    KeywordItem { label: "while", detail: "keyword while loop" },
    KeywordItem { label: "in", detail: "keyword membership operator" },
    KeywordItem { label: "not", detail: "keyword logical negation" },
    KeywordItem { label: "and", detail: "keyword logical conjunction" },
    KeywordItem { label: "or", detail: "keyword logical disjunction" },
    KeywordItem { label: "is", detail: "keyword identity comparison" },
    KeywordItem { label: "try", detail: "keyword try block" },
    KeywordItem { label: "except", detail: "keyword exception handler" },
    KeywordItem { label: "finally", detail: "keyword finally block" },
    KeywordItem { label: "with", detail: "keyword context manager" },
    KeywordItem { label: "as", detail: "keyword alias / binding" },
    KeywordItem { label: "lambda", detail: "keyword anonymous function" },
    KeywordItem { label: "async", detail: "keyword async coroutine definition" },
    KeywordItem { label: "await", detail: "keyword await coroutine execution" },
];

pub static GO_KEYWORDS: [KeywordItem; 20] = [
    KeywordItem { label: "func", detail: "keyword function definition" },
    KeywordItem { label: "package", detail: "keyword package declaration" },
    KeywordItem { label: "import", detail: "keyword package import" },
    KeywordItem { label: "type", detail: "keyword type definition" },
    KeywordItem { label: "struct", detail: "keyword struct definition" },
    KeywordItem { label: "interface", detail: "keyword interface definition" },
    KeywordItem { label: "var", detail: "keyword variable declaration" },
    KeywordItem { label: "const", detail: "keyword constant declaration" },
    KeywordItem { label: "return", detail: "keyword return statement" },
    KeywordItem { label: "if", detail: "keyword conditional branch" },
    KeywordItem { label: "else", detail: "keyword alternative branch" },
    KeywordItem { label: "for", detail: "keyword loop statement" },
    KeywordItem { label: "range", detail: "keyword range iterator" },
    KeywordItem { label: "go", detail: "keyword goroutine spawn" },
    KeywordItem { label: "chan", detail: "keyword channel type" },
    KeywordItem { label: "select", detail: "keyword channel multiplexer" },
    KeywordItem { label: "defer", detail: "keyword deferred function call" },
    KeywordItem { label: "switch", detail: "keyword switch statement" },
    KeywordItem { label: "case", detail: "keyword switch case" },
    KeywordItem { label: "break", detail: "keyword break loop/switch" },
];

pub static CPP_KEYWORDS: [KeywordItem; 20] = [
    KeywordItem { label: "auto", detail: "keyword type deduction" },
    KeywordItem { label: "template", detail: "keyword template definition" },
    KeywordItem { label: "typename", detail: "keyword template type parameter" },
    KeywordItem { label: "class", detail: "keyword class definition" },
    KeywordItem { label: "struct", detail: "keyword struct definition" },
    KeywordItem { label: "namespace", detail: "keyword namespace declaration" },
    KeywordItem { label: "using", detail: "keyword using declaration/alias" },
    KeywordItem { label: "constexpr", detail: "keyword compile-time constant" },
    KeywordItem { label: "const", detail: "keyword const qualifier" },
    KeywordItem { label: "return", detail: "keyword return statement" },
    KeywordItem { label: "if", detail: "keyword conditional branch" },
    KeywordItem { label: "else", detail: "keyword alternative branch" },
    KeywordItem { label: "for", detail: "keyword for loop" },
    KeywordItem { label: "while", detail: "keyword while loop" },
    KeywordItem { label: "public", detail: "keyword public access specifier" },
    KeywordItem { label: "private", detail: "keyword private access specifier" },
    KeywordItem { label: "protected", detail: "keyword protected access specifier" },
    KeywordItem { label: "virtual", detail: "keyword virtual method" },
    KeywordItem { label: "override", detail: "keyword virtual override specifier" },
    KeywordItem { label: "nullptr", detail: "keyword null pointer literal" },
];

pub static COMMON_KEYWORDS: [KeywordItem; 10] = [
    KeywordItem { label: "if", detail: "keyword conditional" },
    KeywordItem { label: "else", detail: "keyword alternative" },
    KeywordItem { label: "for", detail: "keyword loop" },
    KeywordItem { label: "while", detail: "keyword loop" },
    KeywordItem { label: "return", detail: "keyword return" },
    KeywordItem { label: "function", detail: "keyword function" },
    KeywordItem { label: "class", detail: "keyword class" },
    KeywordItem { label: "import", detail: "keyword import" },
    KeywordItem { label: "export", detail: "keyword export" },
    KeywordItem { label: "const", detail: "keyword constant" },
];
