#![feature(let_chains, if_let_guard, box_patterns)]

use proc_macro::TokenStream as TokenStreamOld;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::*, punctuated::Punctuated, *};

mod parts;
use parts::*;

use proc_macro_error::proc_macro_error as proc_macro_error_call;

/// A macro for running SurrealDb queries and transactions without a bunch of boilerplate.
///
/// Example
/// ===============
/// This should explain the basics, but there's way more to this macro than shown here.
/// See the `sdb` crate's documentation for a full breakdown of the macro
///
/// ```rust
/// # use sdb::{ SdbClient,  }
/// // Connect to a SurrealDb on the local machine, default port (8000)
/// // and use the database "test" inside the namespace "demo"
/// let client = SdbClient::connect( "127.0.0.1/demo/test" )
///     .await.unwrap();
///
/// // Run a query on the db
/// //                         |--- value stored as `$0` to be used in queries
/// //          Required ---|  |        |--- Unwrap errors, instead of bubbling
/// //                 vvvvvv  vvvvvv   v
/// sdb::trans_act!( ( client, 50_000 ) ! => {
/// //  |--- results of query will be parsed as `Vec<StorySchema>`
/// //  |    and stored in `longest_books`.
/// //  vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
///     longest_books: Vec<StorySchema> =
///         "SELECT * FROM books WHERE word_count > $0 LIMIT 5";
/// //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// //       |--------------- SQL to execute. ---------------|
/// });
///
/// // Write titles of books to stdout
/// println!("The longest books are:")
/// for book in longest_books {
///     println!("  {} - {} words", book.title, book.word_count)
/// }
///
/// // A schema definition, used in the macro to guarentee some fields
/// #[derive(Clone, Serialize, Deserialize, SurrealRecord)]
/// struct BookSchema {
///     pub id: RecordId,
///     pub title: String,
///     pub summary: Option<String>, // <- Still parses correctly if field not in query results
///     pub tags: Vec<String>,
/// }
/// ```
#[proc_macro_error_call]
#[proc_macro]
pub fn trans_act(input: TokenStreamOld) -> TokenStreamOld {
    let trans_func = parse_macro_input!(input as TransFunc);

    let client = &trans_func.args.client;
    let trans = Ident::new("db_trans", Span::call_site());

    let result_act = trans_func.result_act();

    // let mut out_vars = TokenStream::new();
    let mut out_types = Vec::new();
    let mut out_calls = Vec::new();
    let mut unpack = TokenStream::new();
    let mut push_steps = trans_func.args.field_assigns();

    for line in trans_func.iter_lines() {
        if let QueryLine::Select(select) = &line {
            let var_name = &select.into;
            let var_type = &select.cast;
            let call = select.method_call();
            let mut_token = select.mut_token();

            out_calls.push(quote! { #trans . #call ? });
            out_types.push(quote! { #var_type });
            // out_vars.extend_list(quote! { #mut_token #var_name }, quote! { , });
            unpack.extend(
                quote! { let #mut_token #var_name: #var_type = #trans . #call #result_act; },
            );
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
//
//

#[proc_macro_error_call]
#[proc_macro_derive(SurrealRecord)]
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
    #[allow(unused)]
    pub fn method_call(&self) -> Option<TokenStream> {
        match self {
            QueryLine::Select(sel) => Some(sel.method_call()),
            _ => None,
        }
    }

    pub fn out_call(&self, transact: &Ident, err: &TokenStream) -> Option<TokenStream> {
        let Self::Select( select ) = self else { return None };

        let into = &select.into;
        let cast = &select.cast;
        let cast_type = cast.cast_type();
        let method_call = select.method_call();

        Some(quote! {
            let #into: #cast_type = #transact . #method_call #err ;
        })
    }
}
