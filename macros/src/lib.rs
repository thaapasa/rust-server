use proc_macro::TokenStream as TS;

use crate::proc_format_uri::proc_format_uri;

mod proc_format_uri;

/// This procedural macro allows you to construct URLs or URIs with all the variables
/// automatically URL-encoded (using the urlencoding crate).
///
/// Usage:
///
/// ```ignore
/// let uri = format_uri!("http://localhost/{path}/operation?param={param}");
/// let uri = format_uri!("http://localhost/{path}/operation?param={param}",
///     path=my.path,
///     param=my.param
/// );
/// ```
///
/// To keep some variables unescaped, use the `:raw` suffix for them:
///
/// ```ignore
/// let uri = format_uri!("{host:raw}/path?param={param}");
/// let uri = format_uri!("{host:raw}/path?param={param}",
///     host=my.host,
///     param=my.param
/// );
/// ```
#[proc_macro]
pub fn format_uri(input: TS) -> TS {
    proc_format_uri(input.into()).into()
}
