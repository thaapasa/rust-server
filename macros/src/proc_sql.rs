use std::collections::BTreeMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse2, Error, Expr, LitStr, Token};

struct Assignment {
    name: Ident,
    value: Expr,
}

struct SqlQuery {
    query: String,
    assignments: Vec<Assignment>,
    span: Span,
}

const BINDING_RE: &str = "_?\\{([a-zA-Z_][a-zA-Z0-9_]*(:[a-zA-Z_][a-zA-Z0-9_]*)*)\\}";

impl Parse for SqlQuery {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let query = input.parse::<LitStr>()?;
        let assignments = if input.lookahead1().peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Punctuated::<Assignment, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect()
        } else {
            vec![]
        };
        Ok(Self {
            query: query.value(),
            assignments,
            span: query.span(),
        })
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

pub fn proc_sql(input: TokenStream) -> TokenStream {
    try_proc_sql(input).unwrap_or_else(Error::into_compile_error)
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

fn try_proc_sql(input: TokenStream) -> Result<TokenStream, Error> {
    // Parse the input tokens into a syntax tree
    let SqlQuery {
        query,
        assignments,
        span,
    } = parse2::<SqlQuery>(input)?;

    // Index binding values by their name
    let bindings = build_lookup_map(assignments)?;

    // Generate AST of statements that build the Sql instance based on the
    // parsed template string and binding assignments
    let mut statements: Vec<TokenStream> = vec![
        // Define builder that can be used to build the SQL instance
        quote! { sqlx::QueryBuilder::new("") },
    ];
    let re = Regex::new(BINDING_RE).unwrap();
    // Start offset for binding search
    let mut offset = 0;
    while let Some(cap) = re.captures_iter(&query[offset..]).next() {
        let outer = cap.get(0).unwrap(); // Outer capture: the whole binding including "{" and "}"
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
        let query_part = &query[offset..offset + outer.start()];
        offset += outer.end();
        if !query_part.is_empty() {
            statements.push(quote! {
                .push(#query_part)
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
                .push(#value)
            }),
            Some("id") => statements.push(quote! {
                .push(crate::db::encode_sql_identifier(&(#value)))
            }),
            None => statements.push(quote! {
                .push_bind(#value)
            }),
            Some(x) => {
                return Err(Error::new(
                    span,
                    format!(
                        "Unrecognized variable format type {x} for {binding_part}. Did you mean 'raw' or 'id'?"
                    ),
                ));
            }
        };
    }
    // If template does not end with binding, we need to also push
    // the last part of the template
    if offset < query.len() {
        let remaining = &query[offset..];
        statements.push(quote! {
            .push(#remaining)
        });
    }

    // All done, create final AST that create a block that builds the URI
    // instance using the statements and finally returns the encoded URI to the caller
    Ok(quote! {
        #(#statements)*
            .build()
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use quote::quote;

    use super::proc_sql;

    #[test]
    fn test_no_bindings() {
        assert_eq!(
            stringify(proc_sql(quote! {"SELECT * FROM things"})),
            stringify(quote! {
                sqlx::QueryBuilder::new("")
                    .push("SELECT * FROM things")
                    .build()
            })
        );
    }

    #[test]
    fn test_bind_values() {
        assert_eq!(
            stringify(proc_sql(
                quote! {"INSERT INTO things (name, description) VALUES ({name}, {description})"}
            )),
            stringify(quote! {
                sqlx::QueryBuilder::new("")
                    .push("INSERT INTO things (name, description) VALUES (")
                    .push_bind(name)
                    .push(", ")
                    .push_bind(description)
                    .push(")")
                    .build()
            })
        );
    }

    #[allow(clippy::needless_pass_by_value)]
    fn stringify(s: TokenStream) -> String {
        format!("{s}")
    }
}
