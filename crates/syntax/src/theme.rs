use zenthra_core::Color;

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub function: Color,
    pub function_call: Color,
    pub type_name: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub operator: Color,
    pub punctuation: Color,
    pub variable: Color,
    pub constant: Color,
    pub default: Color,
}

impl SyntaxTheme {
    /// Dark theme (inspired by One Dark)
    pub fn dark() -> Self {
        Self {
            keyword: Color::rgb(198.0 / 255.0, 120.0 / 255.0, 221.0 / 255.0),      // Purple
            function: Color::rgb(97.0 / 255.0, 175.0 / 255.0, 239.0 / 255.0),      // Blue
            function_call: Color::rgb(97.0 / 255.0, 175.0 / 255.0, 239.0 / 255.0), // Blue
            type_name: Color::rgb(229.0 / 255.0, 192.0 / 255.0, 123.0 / 255.0),    // Yellow
            string: Color::rgb(152.0 / 255.0, 195.0 / 255.0, 121.0 / 255.0),       // Green
            number: Color::rgb(209.0 / 255.0, 154.0 / 255.0, 102.0 / 255.0),       // Orange
            comment: Color::rgb(92.0 / 255.0, 99.0 / 255.0, 112.0 / 255.0),        // Gray
            operator: Color::rgb(86.0 / 255.0, 182.0 / 255.0, 194.0 / 255.0),      // Cyan
            punctuation: Color::rgb(171.0 / 255.0, 178.0 / 255.0, 191.0 / 255.0),  // Light gray
            variable: Color::rgb(224.0 / 255.0, 108.0 / 255.0, 117.0 / 255.0),     // Red
            constant: Color::rgb(209.0 / 255.0, 154.0 / 255.0, 102.0 / 255.0),     // Orange
            default: Color::rgb(171.0 / 255.0, 178.0 / 255.0, 191.0 / 255.0),      // Light gray
        }
    }

    /// Light theme (inspired by One Light)
    pub fn light() -> Self {
        Self {
            keyword: Color::rgb(166.0 / 255.0, 38.0 / 255.0, 164.0 / 255.0),       // Purple
            function: Color::rgb(64.0 / 255.0, 120.0 / 255.0, 242.0 / 255.0),      // Blue
            function_call: Color::rgb(64.0 / 255.0, 120.0 / 255.0, 242.0 / 255.0), // Blue
            type_name: Color::rgb(193.0 / 255.0, 132.0 / 255.0, 1.0 / 255.0),      // Yellow
            string: Color::rgb(80.0 / 255.0, 161.0 / 255.0, 79.0 / 255.0),         // Green
            number: Color::rgb(152.0 / 255.0, 104.0 / 255.0, 1.0 / 255.0),         // Orange
            comment: Color::rgb(160.0 / 255.0, 161.0 / 255.0, 167.0 / 255.0),      // Gray
            operator: Color::rgb(0.0 / 255.0, 132.0 / 255.0, 137.0 / 255.0),       // Cyan
            punctuation: Color::rgb(56.0 / 255.0, 58.0 / 255.0, 66.0 / 255.0),     // Dark gray
            variable: Color::rgb(228.0 / 255.0, 86.0 / 255.0, 73.0 / 255.0),       // Red
            constant: Color::rgb(152.0 / 255.0, 104.0 / 255.0, 1.0 / 255.0),       // Orange
            default: Color::rgb(56.0 / 255.0, 58.0 / 255.0, 66.0 / 255.0),         // Dark gray
        }
    }

    pub fn get_color(&self, capture_name: &str) -> Color {
        // Fast direct match
        match capture_name {
            "keyword" => return self.keyword,
            "function" | "function.method" => return self.function,
            "function.call" | "function.macro" => return self.function_call,
            "type" | "type.builtin" => return self.type_name,
            "string" => return self.string,
            "number" => return self.number,
            "comment" | "comment.documentation" => return self.comment,
            "operator" => return self.operator,
            "punctuation" | "punctuation.bracket" | "punctuation.delimiter" => return self.punctuation,
            "variable" | "variable.parameter" | "variable.builtin" => return self.variable,
            "constant" | "constant.builtin" => return self.constant,
            "constructor" => return self.type_name,
            "property" => return self.variable,
            "attribute" => return self.constant,
            "escape" => return self.operator,
            "label" => return self.keyword,
            _ => {}
        }

        // Helix & Tree-sitter hierarchical prefix matching
        if capture_name.starts_with("keyword") {
            self.keyword
        } else if capture_name.starts_with("function") {
            if capture_name.contains("call") || capture_name.contains("macro") {
                self.function_call
            } else {
                self.function
            }
        } else if capture_name.starts_with("type") || capture_name == "namespace" || capture_name.starts_with("constructor") {
            self.type_name
        } else if capture_name.starts_with("string") {
            self.string
        } else if capture_name.starts_with("number") || capture_name.starts_with("float") || capture_name.contains("numeric") {
            self.number
        } else if capture_name.starts_with("comment") {
            self.comment
        } else if capture_name.starts_with("operator") {
            self.operator
        } else if capture_name.starts_with("punctuation") || capture_name.starts_with("delimiter") || capture_name.starts_with("bracket") {
            self.punctuation
        } else if capture_name.starts_with("constant") || capture_name.starts_with("boolean") {
            self.constant
        } else if capture_name.starts_with("variable") || capture_name.starts_with("property") || capture_name.starts_with("field") {
            self.variable
        } else if capture_name.starts_with("attribute") {
            self.constant
        } else if capture_name.starts_with("label") {
            self.keyword
        } else {
            self.default
        }
    }
}
