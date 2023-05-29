#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(box_patterns)]


use ::proc_macro::TokenStream as TokenStreamOld;
use ::proc_macro2::{Span, TokenStream};
use ::proc_macro_error::*;
use ::quote::quote;
use ::syn::*;

mod parts;
mod tester;

use parts::*;

/// A macro for running a single SurrealDb query or statement without a bunch of boilerplate.
///
/// Example
/// ===============
/// This should explain the basics, but there's way more to this macro than shown here.
/// See the crate documentation or README for a full breakdown of the macro
/// 
/// ```rust
/// # use sdb::prelude::*;
/// #
/// # tokio_test::block_on( async {
/// #     test_main().await.unwrap();
/// # });
/// # async fn test_main() -> SdbResult<()> {
/// # let client = SurrealClient::demo();
///     let min_words = 250_000;
/// 
///     let long_books = sdb::query!( client =[ min_words ]=> 
///         "SELECT * FROM books WHERE word_count > $0 LIMIT 5" as Vec<BookSchema>
///     )?;
///
///     println!("The longest books are:");
///     for book in long_books {
///         println!("  {}", book.title)
///     }
/// # Ok( () )
/// # }
/// #
/// #
/// # #[derive(serde::Serialize, serde::Deserialize, SurrealRecord)]
/// # #[table("books")]
/// # pub struct BookSchema {
/// #     pub id: RecordId,
/// #     pub title: String,
/// #     pub word_count: Option<usize>,
/// #     pub author: RecordLink<AuthorSchema>,
/// # }
/// # 
/// # #[derive(serde::Serialize, serde::Deserialize, SurrealRecord)]
/// # #[table("authors")]
/// # pub struct AuthorSchema {
/// #     pub id: RecordId,
/// #     pub name: String,
/// # }
/// ```
/// 
#[proc_macro_error]
#[proc_macro]
pub fn query(input: TokenStreamOld) -> TokenStreamOld {
    let query_func = parse_macro_input!(input as SingleQueryParse);

    let vars = query_func.arg_vars();
    let queries = query_func.full_queries();
    tester::check_syntax(vars, queries, &query_func.args);

    let client = &query_func.client;
    let trans = Ident::new("db_trans", Span::call_site());
    let (push_steps, _, _) = query_func.prepare(&trans);
    let arg_steps = query_func.arg_steps();

    let TailStatement { ref path, .. } = query_func.stmt;
    let run = query_func.stmt.get_runner();
    let out = quote!(
        {
            let result: ::sdb::prelude::SdbResult<#path> = #client
                . transaction()
                #arg_steps
                #push_steps
                #run
                .await;

            result
        }
    );

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}

//

//

//

/// A macro for running multiple SurrealDb queries without a bunch of boilerplate.
///
/// Example
/// ===============
/// This should explain the basics, but there's way more to this macro than shown here.
/// See the crate documentation or README for a full breakdown of the macro
/// 
/// ```rust
/// # use sdb::prelude::*;
/// #
/// # tokio_test::block_on( async {
/// #     test_main().await.unwrap();
/// # });
/// # async fn test_main() -> SdbResult<()> {
/// # let client = SurrealClient::demo();
///     let min_words = 250_000;
/// 
///     sdb::queries!( client =[ min_words ]=> {
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
/// # #[derive(serde::Serialize, serde::Deserialize, SurrealRecord)]
/// # #[table("books")]
/// # pub struct BookSchema {
/// #     pub id: RecordId,
/// #     pub title: String,
/// #     pub word_count: Option<usize>,
/// #     pub author: RecordLink<AuthorSchema>,
/// # }
/// # 
/// # #[derive(serde::Serialize, serde::Deserialize, SurrealRecord)]
/// # #[table("authors")]
/// # pub struct AuthorSchema {
/// #     pub id: RecordId,
/// #     pub name: String,
/// # }
/// ```
/// 
#[proc_macro_error]
#[proc_macro]
pub fn queries(input: TokenStreamOld) -> TokenStreamOld {
    let query_func = parse_macro_input!(input as MultiQueryParse);

    let vars = query_func.arg_vars();
    let queries = query_func.full_queries();
    tester::check_syntax(vars, queries, &query_func.args);

    let client = &query_func.client;
    let trans = Ident::new("db_trans", Span::call_site());
    let (push_steps, unpack, result_act) = query_func.prepare(&trans);
    let arg_steps = query_func.arg_steps();

    let out = match unpack.is_empty() {
        true => quote!{
            #client . transaction()
                #arg_steps
                #push_steps
                .run()
                .await
        },
        false => quote!{    
            let mut #trans = #client . transaction()
                #arg_steps
                #push_steps
                .run()
                .await #result_act;

            #unpack
        },
    };

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}



/// Insert multiple records into the database
/// 
/// ### Example
/// ```rust
/// # use sdb::prelude::*;
/// #
/// # tokio_test::block_on( async {
/// #     test_main().await.unwrap();
/// # });
/// # async fn test_main() -> SdbResult<()> {
/// let client = SurrealClient::demo();
/// 
/// let inserted_record_ids: Vec<RecordId> = sdb::insert!(
///     client => authors (id, name) => [
///         ("philip_p", "Philip Pullman"),
///         ("susanna_c", "Susanna Clarke"),
///         ("george_rrm", "George R. R. Martin"),
///         ("leo_t", "Leo Tolstoy"),
///         ("charles_d", "Charles Dickens")
///     ]
///     return id
/// )?;
/// # Ok( () )
/// # }
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn insert(input: TokenStreamOld) -> TokenStreamOld {
    let insert = parse_macro_input!(input as InsertParse);

    let client = &insert.client;
    let sql_build = insert.build_insert_sql();

    let runner = match insert.ret {
        Some(_) => quote!{ .run_parse_vec() },
        None => quote!{ .run() },
    };

    let out = quote!{
        {
            #sql_build
            #client . transaction()
                .push( &sql )
                #runner
                .await
        } 
    };

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}




//
//
//
/// Implements `SurrealRecord` for a struct. Very useful for defining your 
/// databases schema
#[proc_macro_error]
#[proc_macro_derive(SurrealRecord, attributes(table))]
pub fn derive_surreal_record(input: TokenStreamOld) -> TokenStreamOld {
    let obj = parse_macro_input!(input as DeriveInput);

    let Data::Struct( st ) = obj.data else {
        emit_error!( obj, "Derive only works on Structs (so far)");
        panic!("Derive only works on Structs (so far)")
    };

    let mut out = TokenStream::new();
    let mut has_id_field = false;
    let mut field_defs = TokenStream::new();
    let mut field_names = TokenStream::new();
    let struct_name = &obj.ident;

    let mut table_name = None;
    for attr in obj.attrs {
        match attr.path.get_ident() {
            Some( v ) if v.eq("table") => {
                let Ok( val ) = attr.parse_args::<LitStr>() else { continue };
                table_name = Some( val );
            },
            _ => todo!()
        }
        // let Some( attr_name ) = attr.path.get_ident() else { continue };
        // if attr_name.ne("table") { continue }
    }

    if table_name.is_none() {
        emit_error!( st.struct_token, "Expected a #[table()] attribute");
    }

    for field in st.fields {
        let Some( ident ) = field.ident else {
            emit_error!( field, "Anonomous fields not supported" );

            continue;
        };
        let Type::Path( path ) = field.ty else { continue };
        let field_ty = path.path;
        let field_end = field_ty.segments.last().unwrap();

        if ident.eq("id") {
            if field_ty.is_ident("RecordId") {
                has_id_field = true;
                continue;
            }
            emit_error!( ident, "`id` field must be a RecordId";
                help = "SurrealRecord derive macro requires an `id` to be defined like so:\n\tpub id: RecordId,\n\n"
            );
            continue;
        }

        match field_ty.get_ident() {
            None if field_end.ident.eq("RecordLink") => {
                field_names.extend(quote!{ #ident: #ident.into(), });
                field_defs.extend(quote!{ #ident: impl Into<#field_ty>, });

                if let PathArguments::AngleBracketed( inner_ty ) = &field_end.arguments {
                    let inner_ty = &inner_ty.args;
                    out.extend(quote!{
                        pub fn #ident ( &self ) -> & #inner_ty {
                            self . #ident . record() . unwrap()
                        }
                    });
                }
            },
            Some( ty ) if ty.eq("String") => {
                field_names.extend(quote!{ #ident: #ident.to_string(), });
                field_defs.extend(quote!{ #ident: impl ToString, });
            },
            _ => {
                field_names.extend(quote!{ #ident: #ident.into(), });
                field_defs.extend(quote!{ #ident: impl Into<#field_ty>, });
            }
        }
    }

    if !has_id_field {
        emit_call_site_error!( "Missing `id` field";
            help = "SurrealRecord derive macro requires an `id` to be defined like so:\n\tpub id: RecordId,\n\n"
        );

        return quote!{}.into()
    }

    let (impl_generics, ty_generics, where_clause) = obj.generics.split_for_impl();

    let output = quote!{
        impl #impl_generics ::sdb::prelude::SurrealRecord for #struct_name #ty_generics #where_clause {
            fn id(&self) -> &sdb::prelude::RecordId {
                &self.id
            }

            fn table_name(&self) -> String {
                #table_name.to_string()
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #out
            pub fn new( #field_defs ) -> Self {
                Self {
                    id: ::sdb::prelude::RecordId::placeholder( #table_name ),
                    #field_names
                }
            }
        }
    };

    // #[cfg(feature = "macro-print")]
    // println!("\n{output}\n");

    output.into()
}
