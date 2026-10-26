[
  "abstract" "assert" "break" "case" "catch" "class"
  "continue" "default" "do" "else" "enum" "extends" "final"
  "finally" "for" "if" "implements" "import" "instanceof"
  "interface" "native" "new" "package" "private" "protected" "public"
  "return" "static" "strictfp" "switch" "synchronized"
  "throw" "throws" "transient" "try" "volatile" "while"
] @keyword

(false) @keyword
(boolean_type) @type
(floating_point_type) @type
(integral_type) @type
(super) @keyword
(this) @keyword
(true) @keyword
(void_type) @type
(class_declaration name: (identifier) @type)
(method_declaration name: (identifier) @function)
(type_identifier) @type
(string_literal) @string
(decimal_integer_literal) @number
(decimal_floating_point_literal) @number
(line_comment) @comment
(block_comment) @comment
