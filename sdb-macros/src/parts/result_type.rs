use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::*, punctuated::Punctuated, token::*, Type, *};

use proc_macro_error::{__export::*, abort};

//
//
//

const ERROR_HELP: &str = r#"Only types which implement serde::Deserialise are valid here. "#;

#[derive(Debug)]
pub enum QueryResultType {
    Option(TypePath),
    Single(TypePath),
    Vec(TypePath),
}

impl QueryResultType {
    pub fn call_next(&self) -> TokenStream {
        match &self {
            Self::Option(ty) => quote!(next_opt::< #ty >()),
            Self::Single(ty) => quote!(next_one::< #ty >()),
            Self::Vec(ty) => quote!(next_vec::< #ty >()),
        }
    }
}

impl Parse for QueryResultType {
    fn parse(input: ParseStream) -> Result<Self> {
        // let colon: Colon = input.parse()?;

        // Brackets are a shortcut for Option< T >
        if let Some( ( punct, _) ) = input.cursor().punct()
            && punct.as_char() == '<'
            && let Ok( brack ) = input.parse::<AngleBracketedGenericArguments>()
            && brack.args.len() == 1
            && let GenericArgument::Type( first_arg ) = brack.args.first().unwrap()
        {
            if let Type::Infer( inf ) = first_arg {
                return Ok(Self::Option( value_ty_path( inf ) ))
            }

            if let Type::Path( ty_path ) = first_arg {
                return Ok(Self::Option( ty_path.clone() ))
            }
        }

        let in_type = match input.parse::<Type>() {
            Ok(v) => v,
            Err(_) => {
                let span = input.span();
                // let code = input.to_string();
                abort!(
                    span, "Expected a valid type next";
                    help = ERROR_HELP;
                    note = input.to_string()
                )
            }
        };

        match in_type {
            Type::Infer( inf ) => {
                Ok(Self::Single( value_ty_path(&inf) ))
            },
            Type::Slice( TypeSlice { elem: box Type::Infer( inf ), .. } ) => {
                Ok( Self::Vec( value_ty_path(&inf) ))
            },
            Type::Slice( TypeSlice { elem: box Type::Path( path ), .. } ) => {
                Ok( Self::Vec( path ))
            },

            Type::Path( ref path ) if
                let Some( outer ) = path.path.segments.first()
                && let PathArguments::AngleBracketed( brackets ) = &outer.arguments
                && brackets.args.len() == 1
                && let GenericArgument::Type( inner ) = brackets.args.first().unwrap()
                && let Type::Path( ty ) = inner.clone() =>
            {
                let out_str = outer.ident.to_string();
                match out_str.as_str() {
                    "Vec" => Ok( Self::Vec( ty )),
                    "Option" => Ok( Self::Option( ty )),
                    _ => panic!("OTUSFDJD")
                }
            },

            Type::Path( path ) => Ok(Self::Single( path )),

            _ => {
                abort!(
                    in_type, "Incompatible query parse type.";
                    help = ERROR_HELP
                )
            }
        }
    }
}

fn value_ty_path(inf: &TypeInfer) -> TypePath {
    let span = inf.underscore_token.span;
    let mut segments: Punctuated<PathSegment, Colon2> = Punctuated::new();
    segments.extend([
        PathSegment {
            ident: Ident::new("sdb", span),
            arguments: PathArguments::None,
        },
        PathSegment {
            ident: Ident::new("Value", span),
            arguments: PathArguments::None,
        },
    ]);
    TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments,
        },
    }
}

impl ToTokens for QueryResultType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match &self {
            Self::Option(ty) => quote!( Option< #ty > ),
            Self::Single(ty) => quote!( #ty ),
            Self::Vec(ty) => quote!( Vec< #ty > ),
        });
    }
}
