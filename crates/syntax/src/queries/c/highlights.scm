[
  "break" "case" "const" "continue" "default" "do" "else" "enum" "extern"
  "for" "goto" "if" "inline" "register" "return" "sizeof" "static" "struct"
  "switch" "typedef" "union" "volatile" "while"
] @keyword

(type_identifier) @type
(primitive_type) @type
(identifier) @variable
(string_literal) @string
(char_literal) @string
(number_literal) @number
(comment) @comment
