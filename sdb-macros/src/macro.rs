#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(box_patterns)]


use ::proc_macro::TokenStream as TokenStreamOld;
use ::proc_macro2::{Span, TokenStream};
use ::proc_macro_error::*;
use ::quote::{quote, ToTokens};
use ::syn::*;

mod parts;
mod tester;

use parts::*;

/// A macro for running SurrealDb queries and transactions without a bunch of boilerplate.
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
    let query_func = parse_macro_input!(input as QueryParse);

    if let Err( err ) = tester::check_syntax(&query_func) {
        err.emit();
    }

    let client = &query_func.client;
    let trans = Ident::new("db_trans", Span::call_site());
    let (push_steps, unpack, result_act) = query_func.prepare(&trans);
    let arg_steps = query_func.arg_steps();

    let out = match (
        query_func.get_tail(), 
        unpack.is_empty()
    ) {
        (Some( tail ), _) => {
            let SdbStatement::ParseTail { ref path, .. } = tail else { unreachable!() };
            let run = tail.get_runner();
            quote!(
                {
                    let result: ::sdb::prelude::SdbResult<#path> = #client
                        . transaction()
                        #arg_steps
                        #push_steps
                        #run
                        .await;

                    result
                }
            )
        },
        (None, true) => quote!{
            #client . transaction()
                #arg_steps
                #push_steps
                .run()
                .await
        },
        (None, false) => quote!{    
            let mut #trans = #client . transaction()
                #arg_steps
                #push_steps
                .run()
                .await #result_act;

            #unpack
        }
    };

    #[cfg(feature = "macro-print")]
    println!("\n{out}\n");

    out.into()
}


//

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
/// let inserted_record_ids = sdb::insert!(
///     client => authors (id, name) => [
///         ("philip_p", "Philip Pullman"),
///         ("susanna_c", "Susanna Clarke"),
///         ("george_rrm", "George R. R. Martin"),
///         ("leo_t", "Leo Tolstoy"),
///         ("charles_d", "Charles Dickens")
///     ]
/// )
/// 
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn insert(input: TokenStreamOld) -> TokenStreamOld {
    let insert = parse_macro_input!(input as InsertParse);

    let client = &insert.client;
    let table_name = insert.table.to_string();
    let fields = insert.fields.to_token_stream().to_string();
    let field_count = insert.fields.elems.len();

    let mut out = match insert.ignore.is_some() {
        true => quote!{
            use ::serde_json::to_string;
            let mut sql = format!("INSERT IGNORE INTO {} {} VALUES\n", #table_name, #fields); 
        },
        false => quote!{
            use ::serde_json::to_string;
            let mut sql = format!("INSERT INTO {} {} VALUES\n", #table_name, #fields);
        }
    };

    for (i, row) in insert.values.rows.iter().enumerate() {
        if row.elems.len() != field_count {
            emit_error!(row, "Rows must all have the same number of elements")
        }

        if i != 0 {
            out.extend(quote!{ sql.push_str(",\n"); });
        }
        out.extend(quote!{
            sql.push('(');
        });
        for (idx, field) in row.elems.iter().enumerate() {
            if idx != 0 {
                out.extend(quote!{ sql.push(','); });
            }
            out.extend(quote!{
                sql.push_str(&to_string( &#field ).unwrap());
            })
        }
        out.extend(quote!{
            sql.push(')');
        });
    }

    let out = match insert.ret {
        Some(ret) => {
            let tail = format!("RETURN {}", ret.expected);
            quote!{ {
                #out
                sql.push_str( #tail );
                #client . transaction()
                    .push( &sql )
                    .run_parse_vec()
                    .await
            } }
        },
        None => quote!{ {
            #out
            sql.push_str("RETURN NONE");
            #client . transaction()
            .push( &sql )
            .run()
            .await
        } }
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
        let Some( attr_name ) = attr.path.get_ident() else { continue };
        if attr_name.ne("table") { continue }
        let Ok( val ) = attr.parse_args::<LitStr>() else { continue };
        table_name = Some( val );
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
        let Some( last ) = path.path.segments.last() else { continue };
        let ty_name = last.ident.to_string();

        if ident.to_string().ne("id") {
            if last.ident.eq("String") {
                field_names.extend(quote!{ #ident: #ident.to_string(), });
                field_defs.extend(quote!{ #ident: impl ToString, });
            }
            else if last.ident.eq("RecordLink") {
                field_names.extend(quote!{ #ident: #ident.into(), });
                field_defs.extend(quote!{ #ident: impl Into<#path>, });
            }
            else {
                field_names.extend(quote!{ #ident, });
                field_defs.extend(quote!{ #ident: #path, });
            }
        }

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
    }

    if !has_id_field {
        emit_call_site_error!( "Missing `id` field";
            help = "SurrealRecord derive macro requires an `id` to be defined like so:\n\tpub id: RecordId,\n\n"
        );

        return quote!{}.into()
    }

    let output = quote!{
        impl ::sdb::prelude::SurrealRecord for #struct_name {
            fn id(&self) -> &sdb::prelude::RecordId {
                &self.id
            }

            fn table_name(&self) -> String {
                #table_name.to_string()
            }
        }

        impl #struct_name {
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
