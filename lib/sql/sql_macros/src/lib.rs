use proc_macro::TokenStream as TS;

mod proc_sql;

use crate::proc_sql::proc_sql;

#[proc_macro]
pub fn sql(input: TS) -> TS {
    proc_sql(input.into()).into()
}
