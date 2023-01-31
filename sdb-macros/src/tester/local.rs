use std::ops::Range;

use proc_macro::Literal;
use proc_macro2::Span;
use proc_macro_error::*;
use quote::ToTokens;
use regex::Regex;


mod bracketer;
mod pointer;

pub use bracketer::*;
pub use pointer::*;
use syn::LitStr;

use crate::parts::TransactionParse;

const VAR_FINDER_REGEX: &str = r"\$([[:alnum:]_]+)\b";

pub struct SqlStatement<'a> {
    literal: LitStr,
    sql: String,
    sections: Vec<SqlSection<'a>>,
}

pub struct SqlSection<'a> {
    pub literal: &'a LitStr,
    pub range: Range<usize>,
    pub text: String,
}

/// Perform SurrealQL syntax checking without sending it to the server. this should cover
/// basic stuff like non-marching parenthesies,
pub fn check(trans: &TransactionParse) -> Result<(), Diagnostic> {
    let vars = trans.arg_vars();
    let queries = trans.full_queries() ;
    for (lit, sql) in queries {
        check_trans_vars(&vars, &sql, lit)?;

        let regions = check_brackets_match(&sql, lit)?;
        let parts = query_parts(&sql, regions);

        check_clause_ordering(&sql, &lit, &parts)?;
    }

    Ok(())
}

/// make sure all referenced transaction vars (things that starts with `$`) are
/// defined before they are used
fn check_trans_vars(vars: &[(String, usize)], sql: &str, lit: &LitStr) -> Result<(), Diagnostic> {
    let var_finder = Regex::new(VAR_FINDER_REGEX).unwrap();
    for var_use in var_finder.captures_iter(sql) {
        let use_name = var_use.get(1).unwrap().as_str();
        if vars.iter().any(|(s, _line)| use_name.eq(s)) {
            continue;
        }
        let var_start = sql.find(use_name).unwrap();
        let sql_ptr = SqlErrorPointer::new(sql).tick(var_start - 1, use_name.len() + 1);

        panic!();
        emit_error!(
            lit, "Transaction variable used before defined";
            help = "Query variable `${}` isn't defined before it is used.{:?}", use_name, sql_ptr;
        );
    }
    Ok(())
}

/// Breaks apart nested queries into individual query strings
///
/// Input: `LET $c = (SELECT * FROM (SELECT title FROM books))`
///
/// Output:
/// ```rust,no_test
/// [ "Let $c = ()",  "SELECT * FROM ()",  "SELECT title FORM books" ];
/// ```
fn query_parts(sql: &str, mut regions: Vec<(usize, usize)>) -> Vec<String> {
    let mut base_sql = String::from(sql);

    let len = sql.chars().count() - 1;
    regions.push((0, len));
    let mut parts = vec![];

    for (left, right) in regions {
        let space = left..=right;
        parts.push(base_sql[space.clone()].replace('\x00', ""));

        base_sql = base_sql
            .chars()
            .enumerate()
            .map(|(i, c)| match space.contains(&i) {
                true => '\x00',
                false => c,
            })
            .collect::<String>();
    }

    parts
}

//

//

fn check_clause_ordering(_sql: &str, lit: &LitStr, parts: &Vec<String>) -> Result<(), Diagnostic> {
    for part in parts.iter() {
        if part.starts_with("SELECT") {
            check_clause_order(
                lit, part, &parts,
                "SELECT",
                &["FROM", "WHERE", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH", "TIMEOUT", "PARALLEL" ],
                &["RETURN"],
                SELECT_SYNTAX,
            )?;
        }
        else if part.starts_with("UPDATE") {
            check_clause_order(
                lit, part, &parts,
                "UPDATE", 
                &["CONTENT", "MERGE", "PATCH", "SET", "WHERE", "RETURN", "TIMEOUT", "PARALLEL"],
                &["FROM", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH"],
                UPDATE_SYNTAX,
            )?;
        }
        else if part.starts_with("RELATE") {
            check_clause_order(
                lit, part, &parts,
                "RELATE", 
                &["CONTENT", "SET", "WHERE", "RETURN", "TIMEOUT", "PARALLEL"],
                &["FROM", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH"],
                RELATE_SYNTAX,
            )?;
        }
        else if part.starts_with("DELETE") {
            check_clause_order(
                lit, part, &parts,
                "DELETE", 
                &["WHERE", "RETURN", "TIMEOUT", "PARALLEL"],
                &["FROM", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH"],
                DELETE_SYNTAX,
            )?;
        }
        else {
            
        }
    }
    Ok(())
}

//

//

fn check_clause_order(
    lit: &LitStr,
    part: &str,
    parts: &Vec<String>,
    kind: &str,
    clauses: &[&str],
    not_clauses: &[&str],
    kind_syntax: &str,
) -> Result<(), Diagnostic> {
    let mut last_name = kind;
    let mut last_index = 0;

    for clause in clauses.iter() {
        match part.find(clause) {
            Some( now_idx ) if now_idx < last_index => {
                // let mut l = Literal::string(&lit.value());
                // l.set_span(lit.span().unwrap());

                // let now = now_idx + 1;
                // let span = l.subspan(now..now+clause.len())
                //     .unwrap_or( lit.span().unwrap() );
                // let other = last_index + 1;
                // let other = l.subspan(other..other+last_name.len())
                //     .unwrap_or( lit.span().unwrap() );
                return Err(
                    Diagnostic::spanned(
                        span_range(lit, now_idx, clause.len()),
                        Level::Error,
                        format!("{kind} statement clauses out of order")
                    )
                    .help(format!("`{}` clause must come after `{}` clause", clause, last_name))
                    .span_note(
                        span_range(lit, last_index, last_name.len()),
                        format!("The `{}` clause", last_name)
                    )
                    .note( kind_syntax.to_string() )
                );
            },
            Some(now_idx) => {
                last_name = clause;
                last_index = now_idx;
            },
            None => { }
        }
    }

    for kw in not_clauses.iter() {
        if part.contains(kw) {
            return Err(
                Diagnostic::spanned(
                    lit.span().into(),
                    Level::Error,
                    format!("UPDATE query doesn't support `{kw}` clauses")
                )
                .help(format!("`{}` clause doesn't apply to UPDATE queries", kw))
            )
            // emit_error!(
            //     lit, "UPDATE query doesn't support that clause";
            //     help = "`{}` clause doesn't apply to UPDATE queries", kw;
            // );
        }
    }

    Ok( () )
}

fn span_range( lit: &LitStr, start: usize, width: usize ) -> proc_macro2::Span {
    let offset = 1 + lit.to_token_stream()
        .to_string()
        .find('"')
        .unwrap();
    let start = start + offset;
    let end = start + width;
    let span = match lit.token().subspan(start..end) {
        Some(span) => span,
        None => lit.span(),
    };
    span
}

// // 
// fn check_select_clause_order(part: &str, lit: &Span) -> Result<(), Diagnostic> {
//     let mut last_name = "SELECT";
//     let mut last_index = 0;

//     for clause in &SELECT_CLAUSE_ORDER[1..] {
//         let Some( now_idx ) = part.find(clause) else { continue };
//         if now_idx < last_index {
//             emit_error!(
//                 lit, "SELECT statement clauses out of order";
//                 help = "`{}` clause must come after `{}` clause", clause, last_name;
//             );

//             return Err(
//                 Diagnostic::spanned(*lit, Level::Error, "SELECT statement clauses out of order".into())
//                 .help(format!("`{}` clause must come after `{}` clause", clause, last_name))
//             )
//         }
//         last_name = clause;
//         last_index = now_idx;
//     }

//     Ok( () )
// }


// ///
// fn check_update_clause_order(part: &str, lit: &Span) -> Result<(), Diagnostic> {
//     let mut last_name = "UPDATE";
//     let mut last_index = 0;
    
//     for clause in &UPDATE_CLAUSE_ORDER[1..] {
//         let Some( now_idx ) = part.find(clause) else { continue };
//         if now_idx < last_index {
//             // emit_error!(
//             //     lit, "UPDATE Clauses out of order";
//             //     help = "`{}` clause must come after `{}` clause", clause, last_name;
//             // );

//             return Err(
//                 Diagnostic::spanned(
//                     *lit,
//                     Level::Error,
//                     "UPDATE Clauses out of order".into()
//                 )
//                 .help(format!("`{}` clause must come after `{}` clause", clause, last_name))
//             )
//         }
//         last_name = clause;
//         last_index = now_idx;
//     }

//     for kw in &["ORDER", "SPLIT", "GROUP"] {
//         if part.contains(kw) {
//             return Err(
//                 Diagnostic::spanned(
//                     *lit,
//                     Level::Error,
//                     format!("UPDATE query doesn't support `{kw}` clauses")
//                 )
//                 .help(format!("`{}` clause doesn't apply to UPDATE queries", kw))
//             )
//             // emit_error!(
//             //     lit, "UPDATE query doesn't support that clause";
//             //     help = "`{}` clause doesn't apply to UPDATE queries", kw;
//             // );
//         }
//     }

//     Ok( () )
// }

fn check_brackets_match(sql: &str, lit: &LitStr) -> Result<Vec<(usize, usize)>, Diagnostic> {
    match brackets_are_balanced(sql) {
        Ok(v) => Ok(v),
        Err((left, right)) => {
            let mut l = Literal::string(lit.value().as_str());
            l.set_span(lit.span().unwrap());
            let highlight = l.subspan((left+1)..(right+1)).unwrap().into();
            Err(
                Diagnostic::spanned(
                    highlight,
                    Level::Error,
                    "Brackets are not balanced".to_string()
                )
                .help("Make sure your brackets, braces, and parathesies are balanced".into())
            )
        }
    }
}




const SELECT_SYNTAX: &str = r#"SELECT statement syntax:
SELECT @projections
    FROM @targets
    [ WHERE @condition ]
    [ SPLIT [ AT ] @field ... ]
    [ GROUP [ BY ] @field ... ]
    [ ORDER [ BY ]
        @field [
            RAND()
            | COLLATE
            | NUMERIC
        ] [ ASC | DESC ] ...
    ] ]
    [ LIMIT [ BY ] @limit ]
    [ START [ AT ] @start ]
    [ FETCH @field ... ]
    [ TIMEOUT @duration ]
    [ PARALLEL ]
"#;

const UPDATE_SYNTAX: &str = r#"UPDATE statement syntax:
UPDATE @targets
    [ CONTENT @value
        | MERGE @value
        | PATCH @value
        | SET @field = @value ...
    ]
    [ WHERE @condition ]
    [ RETURN [ NONE | BEFORE | AFTER | DIFF | @projections ... ]
    [ TIMEOUT @duration ]
    [ PARALLEL ]
"#;

const RELATE_SYNTAX: &str = r#"RELATE statement syntax:
RELATE @from -> @table -> @with
    [ CONTENT @value
        | SET @field = @value ...
    ]
    [ RETURN [ NONE | BEFORE | AFTER | DIFF | @projections ... ]
    [ TIMEOUT @duration ]
    [ PARALLEL ]
"#;

const DELETE_SYNTAX: &str = r#"DELETE statement syntax:
DELETE @targets
    [ WHERE @condition ]
    [ RETURN [ NONE | BEFORE | AFTER | DIFF | @projections ... ]
    [ TIMEOUT @duration ]
    [ PARALLEL ]
"#;

const INSERT_SYNTAX: &str = r#"INSERT statement syntax:
INSERT [ IGNORE ] INTO @what
    [ @value
        | (@fields) VALUES (@values)
        [ ON DUPLICATE KEY UPDATE @field = @value ... ]
    ]
"#;

const CREATE_SYNTAX: &str = r#"CREATE statement syntax:
CREATE @targets
    [ CONTENT @value
        | SET @field = @value ...
    ]
    [ RETURN [ NONE | BEFORE | AFTER | DIFF | @projections ... ]
    [ TIMEOUT @duration ]
    [ PARALLEL ]
"#;

const LET_SYNTAX: &str = r#"LET statement syntax:
LET $@parameter = @value
"#;

const USE_SYNTAX: &str = r#"USE statement syntax:
USE [ NS @ns ] [ DB @db ];
"#;