use crate::proc_format_uri::proc_format_uri;
use proc_macro::TokenStream as TS;

mod proc_format_uri;

#[proc_macro]
pub fn format_uri(input: TS) -> TS {
    proc_format_uri(input.into()).into()
}
