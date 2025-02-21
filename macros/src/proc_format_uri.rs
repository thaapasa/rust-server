use std::collections::BTreeMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse2, Error, Expr, LitStr, Token};

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

struct Assignment {
    name: Ident,
    value: Expr,
}

struct UriTemplate {
    span: Span,
    uri: String,
    assignments: Vec<Assignment>,
}

const BINDING_RE: &str = "_?\\{([a-zA-Z_][a-zA-Z0-9_]*(:[a-z_]+)*)\\}";

impl Parse for UriTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let uri = input.parse::<LitStr>()?;
        if input.lookahead1().peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let assignments = Punctuated::<Assignment, Token![,]>::parse_terminated(input)?;
            Ok(Self {
                span: uri.span(),
                uri: uri.value(),
                assignments: assignments.into_iter().collect(),
            })
        } else {
            Ok(Self {
                span: uri.span(),
                uri: uri.value(),
                assignments: vec![],
            })
        }
    }
}

impl Parse for Assignment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<Expr>()?;
        Ok(Self { name, value })
    }
}

pub fn proc_format_uri(input: TokenStream) -> TokenStream {
    try_proc_format_uri(input).unwrap_or_else(Error::into_compile_error)
}

fn build_lookup_map(assignments: Vec<Assignment>) -> Result<BTreeMap<String, Assignment>, Error> {
    let mut lookup = BTreeMap::<String, Assignment>::new();
    for assignment in assignments {
        let name = assignment.name.to_string();
        if lookup.contains_key(&name) {
            return Err(Error::new(
                name.span(),
                format!("duplicate bindings for \"{name}\""),
            ));
        }
        lookup.insert(name, assignment);
    }
    Ok(lookup)
}

fn try_proc_format_uri(input: TokenStream) -> Result<TokenStream, Error> {
    // Parse the input tokens into a syntax tree
    let UriTemplate {
        span,
        uri,
        assignments,
    } = match parse2::<UriTemplate>(input) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };

    // Index binding values by their name
    let bindings = build_lookup_map(assignments)?;

    // Generate AST of statements that build the Sql instance based on the
    // parsed template string and binding assignments
    let capacity = uri.len() * 2;
    let mut statements: Vec<TokenStream> = vec![
        // Define builder that can be used to build the SQL instance
        quote! { let mut __uri = String::with_capacity(#capacity); },
    ];
    let re = Regex::new(BINDING_RE).unwrap();
    // Start offset for binding search
    let mut offset = 0;
    while let Some(cap) = re.captures_iter(&uri[offset..]).next() {
        let outer = cap.get(0).unwrap(); // Outer capture: the whole binding including "${" and "}"
        let inner = cap.get(1).unwrap(); // Inner capture: only the binding name
        let binding_part = inner.as_str();
        let binding_vec = binding_part.split(":").collect::<Vec<_>>();
        let (binding, binding_type) = match binding_vec.len() {
            1 => (binding_vec[0], None),
            2 => (binding_vec[0], Some(binding_vec[1])),
            _ => {
                return Err(Error::new(
                    span,
                    format!(
                        "Only 1 or 2 parts are expected for variable identifiers, found '{binding_part}'"
                    ),
                ));
            }
        };

        let uri_part = &uri[offset..offset + outer.start()];
        offset += outer.end();
        if !uri_part.is_empty() {
            statements.push(quote! {
                __uri.push_str(#uri_part);
            });
        }
        let value = bindings.get(binding).map_or_else(
            || {
                let ident = Ident::new(binding, Span::call_site());
                quote! { #ident }
            },
            |a| {
                let val = &a.value;
                quote! { #val }
            },
        );
        match binding_type {
            Some("raw") => statements.push(quote! {
                __uri.push_str(#value);
            }),
            None => statements.push(quote! {
                __uri.push_str(&urlencoding::encode(#value));
            }),
            Some(x) => {
                return Err(Error::new(
                    span,
                    format!(
                        "Unrecognized variable format type {x} for {binding_part}. Did you mean 'raw'?"
                    ),
                ));
            }
        };
    }
    // If template does not end with binding, we need to also push
    // the last part of the template
    if offset < uri.len() {
        let remaining = &uri[offset..];
        statements.push(quote! {
            __uri.push_str(#remaining);
        });
    }

    // All done, create final AST that create a block that builds the URI
    // instance using the statements and finally returns the encoded URI to the caller
    Ok(quote! {
        {
            #(#statements)*
            __uri
        }
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::proc_format_uri;

    #[test]
    fn test_no_bindings() {
        assert_eq!(
            stringify(proc_format_uri(quote! {"http://localhost"})),
            stringify(quote! {
                {
                    let mut __uri = String::with_capacity(32usize);
                    __uri.push_str("http://localhost");
                    __uri
                }
            })
        );
    }

    #[test]
    fn test_single_escape() {
        assert_eq!(
            stringify(proc_format_uri(quote! {"http://localhost/{path}"})),
            stringify(quote! {
                {
                    let mut __uri = String::with_capacity(46usize);
                    __uri.push_str("http://localhost/");
                    __uri.push_str(&urlencoding::encode(path));
                    __uri
                }
            })
        );
    }

    #[test]
    fn test_single_escape_bound() {
        assert_eq!(
            stringify(proc_format_uri(
                quote! {"http://localhost/{path}", path="foo"}
            )),
            stringify(quote! {
                {
                    let mut __uri = String::with_capacity(46usize);
                    __uri.push_str("http://localhost/");
                    __uri.push_str(&urlencoding::encode("foo"));
                    __uri
                }
            })
        );
    }

    #[test]
    fn test_double_escape_bound_and_direct() {
        assert_eq!(
            stringify(proc_format_uri(
                quote! {"http://localhost/{path}/?param={value}&a=b", path="foo"}
            )),
            stringify(quote! {
                {
                    let mut __uri = String::with_capacity(84usize);
                    __uri.push_str("http://localhost/");
                    __uri.push_str(&urlencoding::encode("foo"));
                    __uri.push_str("/?param=");
                    __uri.push_str(&urlencoding::encode(value));
                    __uri.push_str("&a=b");
                    __uri
                }
            })
        );
    }

    #[test]
    fn test_including_raw_variables() {
        assert_eq!(
            stringify(proc_format_uri(
                quote! {"{host:raw}/{path}/?a=b", host=config.host, path="foo"}
            )),
            stringify(quote! {
                {
                    let mut __uri = String::with_capacity(44usize);
                    __uri.push_str(config.host);
                    __uri.push_str("/");
                    __uri.push_str(&urlencoding::encode("foo"));
                    __uri.push_str("/?a=b");
                    __uri
                }
            })
        );
    }

    #[test]
    fn test_fails_with_multiple_colons() {
        assert_eq!(
            stringify(proc_format_uri(
                quote! {"{host:raw:raw}/{path}/?a=b", host=config.host, path="foo"}
            )),
            stringify(quote! {
                ::core::compile_error! {"Only 1 or 2 parts are expected for variable identifiers, found 'host:raw:raw'"}
            })
        );
    }

    #[test]
    fn test_does_not_encode_empty_variables() {
        assert_eq!(
            stringify(proc_format_uri(quote! {"http://localhost/{}", path="foo"})),
            stringify(quote! {
                {
                    let mut __uri = String::with_capacity(38usize);
                    __uri.push_str("http://localhost/{}");
                    __uri
                }
            })
        );
    }

    #[allow(clippy::needless_pass_by_value)]
    fn stringify(s: TokenStream) -> String {
        format!("{s}")
    }
}
