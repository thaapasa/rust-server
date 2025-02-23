pub fn encode_sql_identifier(input: &str) -> String {
    format!("\"{}\"", input.replace("\"", "\"\""))
}
