[
  "abstract" "as" "async" "await" "break" "case" "catch" "class" "const"
  "continue" "debugger" "default" "delete" "do" "else" "enum"
  "export" "extends" "finally" "for" "from" "function" "if"
  "implements" "import" "in" "instanceof" "interface" "let" "new"
  "of" "private" "protected" "public" "return" "static" "switch"
  "throw" "try" "typeof" "var" "void" "while" "yield"
] @keyword

(false) @keyword
(super) @keyword
(this) @keyword
(true) @keyword
(function_declaration name: (identifier) @function)
(call_expression function: (identifier) @function.call)
(type_identifier) @type
(identifier) @variable
(string) @string
(template_string) @string
(number) @number
(comment) @comment
