#![feature(let_chains, if_let_guard, box_patterns, async_closure)]

use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{emit_call_site_error, proc_macro_error, emit_error};
use quote::quote;
use syn::*;

mod parts;
#[cfg(feature = "query-test")]
mod query_tester;
mod syntaxer;

use parts::*;
use syntaxer::*;

/// A macro for running SurrealDb queries and transactions without a bunch of boilerplate.
///
/// Example
/// ===============
/// This should explain the basics, but there's way more to this macro than shown here.
/// See the crate documentation or README for a full breakdown of the macro
/// 
/// ```rust
/// # mod sdb {
/// #   pub use sdb_base::*;
/// #   pub use sdb_macros::query;
/// # };
/// # use sdb_base::prelude::*;
/// # use sdb_macros::*;
/// #
/// # tokio_test::block_on( async {
/// #     test_main().await.unwrap();
/// # });
/// # async fn test_main() -> SdbResult<()> {
/// # let client = SurrealClient::demo();
///     let min_words = 250_000;
/// 
///     sdb::query!( client =[ min_words ]=> {
///         "SELECT * FROM books WHERE word_count > $0 LIMIT 5"
///             => long_books: Vec<BookSchema>
///     });
///
///     println!("The longest books are:");
///     for book in long_books {
///         println!("  {}", book.title)
///     }
/// # Ok( () )
/// # }
/// #
/// #
/// # #[derive(serde::Deserialize, SurrealRecord)]
/// # #[table("books")]
/// # pub struct BookSchema {
/// #     pub id: RecordId,
/// #     pub title: String,
/// #     pub word_count: Option<usize>,
/// #     pub author: RecordLink<AuthorSchema>,
/// # }
/// # 
/// # #[derive(serde::Deserialize, SurrealRecord)]
/// # #[table("authors")]
/// # pub struct AuthorSchema {
/// #     pub id: RecordId,
/// #     pub name: String,
/// # }
/// ```
/// 
/// # *Sugar Queries™*
/// Long SQL statements can get hard to read, so *Sugar Queries™* 
/// modify your base statements to do common query-related tasks 
/// after the important part is done 
/// 
/// ### `count( )`
/// Returns the number of results. Can be parsed as any number primitive
/// ### `count( <field> )`
/// Returns the number of times *`field`* was truthy in the results.
/// 
#[proc_macro_error]
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

    let client = &query_func.client;
    let trans = Ident::new("db_trans", Span::call_site());
    let (push_steps, unpack, result_act) = query_func.prepare(&trans);
    let arg_steps = query_func.arg_steps();

    let out_base = quote!(
        #client . transaction()
            #arg_steps
            #push_steps
            .run()
            .await #result_act
    );

    let out = match query_func.has_trailing() {
        true => quote!( #out_base. #unpack ),
        false => quote!(
            let mut #trans = #out_base;
            #unpack
        ),
    };

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}

//
//
//

#[proc_macro_error]
#[proc_macro_derive(SurrealRecord, attributes(table))]
pub fn derive_surreal_record(input: TokenStreamOld) -> TokenStreamOld {
    let obj = parse_macro_input!(input as DeriveInput);

    let Data::Struct( st ) = obj.data else {
        panic!("Derive only works on Structs (so far)")
    };

    let mut out = TokenStream::new();
    let mut has_id_field = false;
    let struct_name = &obj.ident;


    for field in st.fields {
        let Some( ident ) = field.ident else {
            emit_error!( field, "Anonomous fields not supported" );

            continue;
        };
        let Type::Path( path ) = field.ty else { continue };
        let Some( last ) = path.path.segments.last() else { continue };
        let ty_name = last.ident.to_string();

        match ty_name.as_str() {
            "RecordLink" => {
                let PathArguments::AngleBracketed( inner_ty ) = &last.arguments else {
                    unreachable!("There should be arguments");
                };
                let inner_ty = &inner_ty.args;
                out.extend(quote!{
                    pub fn #ident ( &self ) -> & #inner_ty {
                        self . #ident . record() . unwrap()
                    }
                });
            },
            "RecordId" if ident.to_string().eq("id") => {
                has_id_field = true;
            },
            _ => continue
        }
        // if let Some( ident ) = field.ident
        // && ident.to_string().eq("id")
        // && let Visibility::Public(_) = field.vis {
        //     has_id_field = true;
        //     match field.ty {
        //         Type::Path( typath )
        //         if typath.to_token_stream().to_string().contains("RecordId") => {
        //         }
        //     }            if Type::Path( path ) = field.ty {
        //         if .to_token_stream().to_string().contains("RecordId")
        //         has_id_field = true;
        //         continue =  {
        //     }
        // }

        // let Type::Path( path ) = field.ty else { continue };
        // let Some( last ) = path.path.segments.last() else { continue };
        // if last.ident.to_string().ne("RecordLink") {
        //     continue;
        // };
        // let PathArguments::AngleBracketed( inner_ty ) = &last.arguments else { continue };
        // let inner_ty = &inner_ty.args;
        // out.extend(quote!{
        //     pub fn #ident ( &self ) -> & #inner_ty {
        //         self . #ident . record() . unwrap()
        //     }
        // });
    }

    if !has_id_field {
        emit_call_site_error!( "Missing `id` field";
            help = "SurrealRecord derive macro requires an `id` to be defined like so:\n\tpub id: RecordId,\n\n"
        );
    }

    quote!{
        impl sdb::prelude::SurrealRecord for #struct_name {
            fn id(&self) -> sdb::prelude::RecordId {
                self.id.clone()
            }
        }

        impl #struct_name {
            #out
        }
    }
    .into()
}
