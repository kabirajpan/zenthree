[
  "break" "case" "chan" "const" "continue" "default" "defer" "else" "fallthrough"
  "for" "func" "go" "goto" "if" "import" "interface" "map" "package" "range"
  "return" "select" "struct" "switch" "type" "var"
] @keyword

(function_declaration name: (identifier) @function)
(call_expression function: (identifier) @function.call)
(type_identifier) @type
(interpreted_string_literal) @string
(raw_string_literal) @string
(int_literal) @number
(float_literal) @number
(comment) @comment
