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
        match capture_name {
            "keyword" => self.keyword,
            "function" | "function.method" => self.function,
            "function.call" | "function.macro" => self.function_call,
            "type" | "type.builtin" => self.type_name,
            "string" => self.string,
            "number" => self.number,
            "comment" | "comment.documentation" => self.comment,
            "operator" => self.operator,
            "punctuation" | "punctuation.bracket" | "punctuation.delimiter" => self.punctuation,
            "variable" | "variable.parameter" | "variable.builtin" => self.variable,
            "constant" | "constant.builtin" => self.constant,
            "constructor" => self.type_name,
            "property" => self.variable,
            "attribute" => self.constant,
            "escape" => self.operator,
            "label" => self.keyword,
            _ => self.default,
        }
    }
}
