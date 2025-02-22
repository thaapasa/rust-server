use tracing::log::warn;

pub fn encode_sql(input: &str) -> String {
    warn!("Should encode {input}, implementation missing...");
    input.to_owned()
}
