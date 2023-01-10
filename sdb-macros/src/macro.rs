#![feature(let_chains, if_let_guard, box_patterns, async_closure)]

use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::*, punctuated::Punctuated, *};

mod parts;
use parts::*;

mod query_test;

mod inc_stream;
use inc_stream::*;

mod syntaxer;
pub(crate) use syntaxer::*;

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
/// // Connect to a the demo SurrealDb (launched using `./launch-demo-db.sh`)
/// let client = SurrealClient::demo().unwrap();
///
/// sdb::trans_act!( ( client, 50_000 ) => {
///     longest_books: Vec<BookSchema> =
///         "SELECT * FROM books WHERE word_count > $0 LIMIT 5";
/// });
///
/// println!("The longest books are:");
/// for book in longest_books {
///     println!("  {}", book.title)
/// }
/// # Ok( () )
/// # }
/// # tokio_test::block_on(async {
/// #     test_main().await.unwrap()
/// # });
///
/// // A schema definition, used in the macro to guarentee some fields
/// #[derive(Clone, Serialize, Deserialize, SurrealRecord)]
/// struct BookSchema {
///     pub id: RecordId,
///     pub title: String,
///     pub word_count: usize,
///     pub summary: Option<String>, // <- Still parses correctly if field not in query results
/// }
/// ```
#[proc_macro_error_call]
#[proc_macro]
pub fn trans_act(input: TokenStreamOld) -> TokenStreamOld {
    let trans_func = parse_macro_input!(input as TransFunc);

    let vars = trans_func.arg_vars();
    let queries = trans_func.full_queries();
    match check_syntax(&vars, &queries) {
        Ok(_) =>
        {
            #[cfg(feature = "query-test")]
            query_test::live_query_test(trans_func)
        }
        Err(_) => {}
    }

    let client = &trans_func.args.client;
    let trans = Ident::new("db_trans", Span::call_site());

    let result_act = trans_func.result_act();

    let mut out_types = Vec::new();
    let mut out_calls = Vec::new();
    let mut unpack = TokenStream::new();
    let mut push_steps = trans_func.args.field_assigns();

    for line in trans_func.iter_lines() {
        if let QueryLine::Select(select) = &line {
            let var_name = &select.into;
            let var_type = &select.cast;
            let call = quote!( next::< #var_type >() );
            let mut_token = select.mut_token();

            out_calls.push(quote! { #trans . #call ? });
            out_types.push(quote! { #var_type });
            unpack.extend(quote! { let #mut_token #var_name = #trans . #call  #result_act; });
        };

        push_steps.extend(quote! { #line });
    }

    let out = quote!(
        let #trans = #client . transaction()
            #push_steps ;

        let mut #trans = #trans.run()
            .await #result_act;

        #unpack
    );

    #[cfg(feature = "macro-print")]
    println!("\n\n{out}\n\n");

    out.into()
}

//

// --

//
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
/// // Connect to a the demo SurrealDb (launched using `./launch-demo-db.sh`)
/// let client = SurrealClient::demo().unwrap();
///
/// let long_books = sdb::query!( ( client, 50_000 ) =>
///         Vec<BookSchema> = "SELECT * FROM books WHERE word_count > $0 LIMIT 5"
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
/// #[derive(Clone, Serialize, Deserialize, SurrealRecord)]
/// struct BookSchema {
///     pub id: RecordId,
///     pub title: String,
///     pub word_count: usize,
///     pub summary: Option<String>, // <- Still parses correctly if field not in query results
/// }
/// ```
// #[proc_macro_error_call]
#[proc_macro]
pub fn query(input: TokenStreamOld) -> TokenStreamOld {
    let query_func = parse_macro_input!(input as QueryFunc);

    // query_check( &query_func );

    let client = &query_func.args.client;
    let trans = Ident::new("db_trans", Span::call_site());

    let mut out_types = TokenStream::new();
    let mut out_calls = Vec::new();
    let mut unpack = TokenStream::new();
    let mut push_steps = query_func.args.field_assigns();

    let select = &query_func.line;
    let var_type = &select.cast;
    let call = quote!( next::< #var_type >() );

    out_calls.push(quote! { #trans . #call ? });
    out_types.extend_list(quote! { #var_type }, quote! { , });
    unpack.extend_list(quote! { #trans . #call ? }, quote! { , });

    push_steps.extend(quote! { #select });

    let last = match query_func.async_tok {
        None => quote!( . await ? ),
        Some(_) => quote!(),
    };

    let parse_type = match &select.cast.scale {
        QueryResultScale::Single(ty) => quote! { #ty },
        QueryResultScale::Option(ty) => quote! { Option< #ty > },
        QueryResultScale::Vec(ty) => quote! { Vec< #ty > },
    };

    let out = quote! {
        #client . transaction( )
        #push_steps
        .run_parse::< #parse_type >( )
        #last
    };

    #[cfg(feature = "macro-print")]
    println!("\n\n{out}\n\n");

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

    let has_id_field = st.fields.iter().any(|field| match &field.ident {
        Some(ident) => ident.to_string().eq("id"),
        None => false,
    });

    let struct_name = obj.ident;

    if has_id_field {
        quote!(
            impl sdb::prelude::SurrealRecord for #struct_name {
                fn id(&self) -> sdb::prelude::RecordId {
                    self.id.clone()
                }
            }
        )
        .into()
    } else {
        panic!("Derive requires an Id field to work")
    }
}

//
//
//

pub(crate) struct QueryLineList {
    pub _paren: token::Brace,
    pub lines: Punctuated<QueryLine, Token![;]>,
}

impl Parse for QueryLineList {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            _paren: braced!(content in input),
            lines: content.parse_terminated(QueryLine::parse)?,
        })
    }
}

//
//
//

pub(crate) enum QueryLine {
    /// run a query, ignore the results
    Raw(RawQueryLine),
    /// Run a query and store the results in a transaction variable
    Let(LetQueryLine),
    /// Run a query then parse the results into a rust variable
    Select(SelectQueryLine),
}

impl Parse for QueryLine {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Raw(input.parse()?))
        } else if input.peek(Token![$]) {
            Ok(Self::Let(input.parse()?))
        } else {
            Ok(Self::Select(input.parse()?))
        }
    }
}

impl ToTokens for QueryLine {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use QueryLine::*;
        match self {
            Raw(r) => r.to_tokens(tokens),
            Let(l) => l.to_tokens(tokens),
            Select(s) => s.to_tokens(tokens),
        }
    }
}

impl QueryLine {
    pub fn out_call(&self, transact: &Ident, err: &TokenStream) -> Option<TokenStream> {
        let Self::Select( select ) = self else { return None };

        let into = &select.into;
        let cast = &select.cast;
        let cast_type = cast.cast_type();
        // let method_call = select.method_call();

        Some(quote! {
            let #into = #transact . next::< #cast_type >() #err ;
        })
        // Some(quote! {
        //     #transact . #method_call #err ;
        // })
    }
}
