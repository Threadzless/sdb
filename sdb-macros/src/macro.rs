#![feature(let_chains, if_let_guard, box_patterns, async_closure)]

use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::*;

mod parts;
use parts::*;

#[cfg(feature = "query-test")]
mod query_tester;

mod syntaxer;
use syntaxer::*;

use proc_macro_error::proc_macro_error as proc_macro_error_call;




/// A macro for running SurrealDb queries and transactions without a bunch of boilerplate.
///
/// Example
/// ===============
/// This should explain the basics, but there's way more to this macro than shown here.
/// See the `sdb` crate's documentation for a full breakdown of the macro
///
/// ```rust
/// # use sdb::prelude::*;
/// # use serde::{Serialize, Deserialize};
/// # async fn test_main() -> SdbResult<()> {
/// # // Connect to a the demo SurrealDb (launched using `./launch-demo-db.sh`)
/// # let client = SurrealClient::demo().unwrap();
///
/// let long_books = sdb::query!( ( client ) =>
///         "SELECT * FROM books WHERE word_count > $0 LIMIT 5" -> Vec<BookSchema>
///     );
///
/// println!("The longest books are:");
/// for book in long_books {
///     println!("  {}", book.title)
/// }
/// # Ok( () )
/// # }
/// # tokio_test::block_on(async {
/// #     test_main().await.unwrap()
/// # });
///
/// // A schema definition, used in the macro to guarentee some fields
/// #[derive(Serialize, Deserialize, SurrealRecord)]
/// struct BookSchema {
///     pub id: RecordId,
///     pub title: String,
///     pub word_count: usize,
///     pub summary: Option<String>, // <- Still parses correctly if field not in query results
/// }
/// ```
#[proc_macro_error_call]
#[proc_macro]
pub fn query(input: TokenStreamOld) -> TokenStreamOld {
    let query_func = parse_macro_input!(input as QueryFunc);

    let vars = query_func.arg_vars();
    let queries = query_func.full_queries();
    if check_syntax(&vars, &queries).is_ok() {
        #[cfg(feature = "query-test")]
        {
            let full_sql = queries
                .iter()
                .map(|(sql, _)| sql.to_string())
                .collect::<Vec<String>>()
                .join(";\n");
            query_tester::live_query_test(full_sql)
        }
    }

    let client = &query_func.args.client;
    let trans = Ident::new("db_trans", Span::call_site());

    let mut arg_num = 0;
    let args_steps = query_func.args.fields.iter()
        .map(|expr| {
            let var_name = LitStr::new(&format!("{arg_num}"), Span::call_site());
            arg_num += 1;
            quote!( .push_var( #var_name, #expr ))
        })
        .collect::<TokenStream>();

    let ( push_steps, unpack, result_act ) = query_func.prepare( &trans );

    let out_base = quote!(
        #client . transaction()
            #args_steps
            #push_steps 
            .run()
            .await #result_act
    );

    let out = match query_func.has_trailing() {
        true => quote!( #out_base. #unpack ),
        false => quote!(
            let mut #trans = #out_base;
            #unpack
        )
    };

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}




//
//
//

#[proc_macro_error_call]
#[proc_macro_derive(SurrealRecord, attributes(table))]
pub fn derive_surreal_record(input: TokenStreamOld) -> TokenStreamOld {
    let obj = parse_macro_input!(input as DeriveInput);

    let Data::Struct( st ) = obj.data else {
        panic!("Derive only works on Structs (so far)")
    };

    let mut out = TokenStream::new();
    let mut has_id_field = false;

    for field in st.fields {
        let Some( ident ) = field.ident else { continue };
        has_id_field |= ident.to_string().eq("id");

        let Type::Path( path ) = field.ty else { continue };
        let Some( last ) = path.path.segments.last() else { continue };
        if last.ident.to_string().ne("RecordLink") {
            continue;
        };
        let PathArguments::AngleBracketed( inner_ty ) = &last.arguments else { continue };
        let inner_ty = &inner_ty.args;
        out.extend(quote!(
            pub fn #ident ( &self ) -> & #inner_ty {
                self . #ident . record() . unwrap()
            }
        ));
    }

    let struct_name = &obj.ident;
    if !out.is_empty() {
        out = quote!(
            impl #struct_name {
                #out
            }
        );
    }

    if has_id_field {
        out.extend(quote!(
            impl sdb::prelude::SurrealRecord for #struct_name {
                fn id(&self) -> sdb::prelude::RecordId {
                    self.id.clone()
                }
            }
        ))
    } else {
        panic!("Derive requires an Id field to work")
    }

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}

//
//
//

