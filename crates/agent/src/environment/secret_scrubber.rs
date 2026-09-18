use regex::Regex;
use std::sync::LazyLock;

/// Replacement mask for scrubbed secrets.
pub const REDACTED_SECRET: &str = "[REDACTED_SECRET]";

static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // OpenAI / generic API keys
        Regex::new(r"sk-[a-zA-Z0-9_-]{20,}").unwrap(),
        // Anthropic API keys
        Regex::new(r"sk-ant-[a-zA-Z0-9_-]{20,}").unwrap(),
        // Google Gemini API keys
        Regex::new(r"AIzaSy[a-zA-Z0-9_-]{25,}").unwrap(),
        // GitHub personal access tokens
        Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(),
        Regex::new(r"github_pat_[a-zA-Z0-9_]{50,}").unwrap(),
        // Slack tokens
        Regex::new(r"xox[baprs]-[a-zA-Z0-9_-]{10,}").unwrap(),
        // Private keys (PEM blocks)
        Regex::new(r"-----BEGIN (?:[A-Z ]+ )?PRIVATE KEY-----[\s\S]*?-----END (?:[A-Z ]+ )?PRIVATE KEY-----").unwrap(),
        // Basic auth inside URLs (http://user:password@host)
        Regex::new(r"(https?://[a-zA-Z0-9_.-]+:)[^@\s]+(@)").unwrap(),
        // Authorization Bearer tokens
        Regex::new(r"(?i)(Authorization:\s*Bearer\s+)[a-zA-Z0-9._\-]{20,}").unwrap(),
    ]
});

/// Secret scrubber that sanitizes tool outputs, logs, and messages before model/UI ingestion.
#[derive(Debug, Clone, Default)]
pub struct SecretScrubber;

impl SecretScrubber {
    /// Instantiates a new secret scrubber.
    pub fn new() -> Self {
        Self
    }

    /// Scrubs any recognized secret tokens or private keys from text.
    pub fn scrub(&self, input: &str) -> String {
        let mut scrubbed = input.to_string();

        // 1. Scrub PEM private key blocks
        let pem_regex = &SECRET_PATTERNS[6];
        scrubbed = pem_regex.replace_all(&scrubbed, REDACTED_SECRET).to_string();

        // 2. Scrub URL basic auth
        let url_auth_regex = &SECRET_PATTERNS[7];
        scrubbed = url_auth_regex.replace_all(&scrubbed, format!("${{1}}{REDACTED_SECRET}${{2}}")).to_string();

        // 3. Scrub Bearer header tokens
        let bearer_regex = &SECRET_PATTERNS[8];
        scrubbed = bearer_regex.replace_all(&scrubbed, format!("${{1}}{REDACTED_SECRET}")).to_string();

        // 4. Scrub standard token patterns
        for pattern in &SECRET_PATTERNS[0..6] {
            scrubbed = pattern.replace_all(&scrubbed, REDACTED_SECRET).to_string();
        }

        scrubbed
    }

    /// Returns true if the text contains any recognizable secrets.
    pub fn contains_secrets(&self, input: &str) -> bool {
        for pattern in SECRET_PATTERNS.iter() {
            if pattern.is_match(input) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrubs_api_keys() {
        let scrubber = SecretScrubber::new();

        let text1 = "Error connecting with apiKey sk-1234567890abcdef1234567890abcdef";
        assert_eq!(
            scrubber.scrub(text1),
            format!("Error connecting with apiKey {REDACTED_SECRET}")
        );

        let text2 = "Using Anthropic key sk-ant-api03-abcdef1234567890abcdef12345";
        assert_eq!(
            scrubber.scrub(text2),
            format!("Using Anthropic key {REDACTED_SECRET}")
        );

        let text3 = "Google key: AIzaSyD1234567890abcdef1234567890abc";
        assert_eq!(
            scrubber.scrub(text3),
            format!("Google key: {REDACTED_SECRET}")
        );

        let text4 = "GitHub token: ghp_1234567890abcdef1234567890abcdef1234";
        assert_eq!(
            scrubber.scrub(text4),
            format!("GitHub token: {REDACTED_SECRET}")
        );
    }

    #[test]
    fn test_scrubs_url_basic_auth() {
        let scrubber = SecretScrubber::new();
        let url = "git clone https://username:super_secret_password@github.com/repo.git";
        let scrubbed = scrubber.scrub(url);
        assert_eq!(
            scrubbed,
            format!("git clone https://username:{REDACTED_SECRET}@github.com/repo.git")
        );
    }

    #[test]
    fn test_scrubs_bearer_tokens() {
        let scrubber = SecretScrubber::new();
        let header = "curl -H 'Authorization: Bearer my_super_secret_jwt_token_1234567890'";
        let scrubbed = scrubber.scrub(header);
        assert_eq!(
            scrubbed,
            format!("curl -H 'Authorization: Bearer {REDACTED_SECRET}'")
        );
    }

    #[test]
    fn test_scrubs_private_key_blocks() {
        let scrubber = SecretScrubber::new();
        let key_block = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0\n1234567890\n-----END RSA PRIVATE KEY-----";
        let scrubbed = scrubber.scrub(key_block);
        assert_eq!(scrubbed, REDACTED_SECRET);
    }

    #[test]
    fn test_clean_text_unchanged() {
        let scrubber = SecretScrubber::new();
        let clean = "fn calculate_sum(a: i32, b: i32) -> i32 { a + b }";
        assert_eq!(scrubber.scrub(clean), clean);
        assert!(!scrubber.contains_secrets(clean));
    }
}
