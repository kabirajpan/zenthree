pub mod provider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_completion() {
        let res = provider::complete("Say 'agent online' and nothing else.");
        assert!(res.is_ok(), "Failed completion: {:?}", res.err());
        let reply = res.unwrap();
        println!("Provider response: {}", reply);
        assert!(!reply.is_empty());
    }
}
