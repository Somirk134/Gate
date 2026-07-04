pub fn sanitize_identifier(input: &str) -> String {
    input.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect()
}
