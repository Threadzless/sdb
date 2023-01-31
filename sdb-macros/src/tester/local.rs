use ::proc_macro_error::*;
use ::quote::ToTokens;
use ::regex::Regex;
use ::syn::LitStr;

mod bracketer;
mod pointer;

pub(crate) use bracketer::*;
pub(crate) use pointer::*;

use crate::parts::QueryParse;

const VAR_FINDER_REGEX: &str = r"\$([[:alnum:]_]+)\b";

/// Perform SurrealQL syntax checking without sending it to the server. this should cover
/// basic stuff like non-marching parenthesies,
pub(crate) fn check(trans: &QueryParse) -> Result<(), Diagnostic> {
    let vars = trans.arg_vars();
    let queries = trans.full_queries();
    for (lit, sql) in queries {
        check_trans_vars(&vars, &sql, lit)?;

        let regions = check_brackets_match(&lit.value(), lit)?;
        let parts = query_parts(&lit.value(), regions);

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
fn query_parts(sql: &str, mut regions: Vec<(usize, usize)>) -> Vec<(usize, String)> {
    let mut base_sql = String::from(sql);

    let len = sql.chars().count() - 1;
    regions.push((0, len));
    let mut parts = vec![];

    for (left, right) in regions {
        let space = match left == 0 && right == sql.len() - 1{
            true => left..right,
            false => if left <= right {
                continue;
            }
            else { 
                (left+1)..(right-1)
            },
        };
        parts.push((
            left,
            base_sql[space.clone()].to_owned()
        ));

        base_sql = base_sql
            .chars()
            .enumerate()
            .map(|(i, c)| match space.contains(&i) {
                true => '\x00',
                false => c,
            })
            .collect::<String>();
    }

    println!("MACRO PARTS: {sql}\n{parts:#?}");
    parts
}

//

//

fn check_clause_ordering(_sql: &str, lit: &LitStr, parts: &Vec<(usize, String)>) -> Result<(), Diagnostic> {
    println!("Checking clause order for:\n  {_sql}\n  {}\n", lit.value());

    for (offset, part) in parts.iter() {
        if part.starts_with("SELECT") {
            check_clause_order(
                lit, part, *offset,
                "SELECT",
                &["FROM", "WHERE", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH", "TIMEOUT", "PARALLEL" ],
                &["RETURN"],
                SELECT_SYNTAX,
            )?;
        }
        else if part.starts_with("UPDATE") {
            check_clause_order(
                lit, part, *offset,
                "UPDATE", 
                &["CONTENT", "MERGE", "PATCH", "SET", "WHERE", "RETURN", "TIMEOUT", "PARALLEL"],
                &["FROM", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH"],
                UPDATE_SYNTAX,
            )?;
        }
        else if part.starts_with("RELATE") {
            check_clause_order(
                lit, part, *offset,
                "RELATE", 
                &["CONTENT", "SET", "WHERE", "RETURN", "TIMEOUT", "PARALLEL"],
                &["FROM", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH"],
                RELATE_SYNTAX,
            )?;
        }
        else if part.starts_with("DELETE") {
            check_clause_order(
                lit, part, *offset,
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
    part_offset: usize,
    kind: &str,
    clauses: &[&str],
    not_clauses: &[&str],
    kind_syntax: &str,
) -> Result<(), Diagnostic> {
    let mut last_name = kind;
    let mut last_index = 0;

    println!("CCO:  {part_offset:4} {part}");

    for clause in clauses.iter() {
        match part.find(clause) {
            Some( now_idx ) if now_idx < last_index => {
                println!("ERR:OFFSET {}", part_offset);
                return Err(
                    Diagnostic::spanned(
                        span_range(lit, now_idx + part_offset, clause.len()),
                        Level::Error,
                        format!("{kind} statement clauses out of order")
                    )
                    .help(format!("`{}` clause must come after `{}` clause", clause, last_name))
                    .span_note(
                        span_range(lit, last_index + part_offset, last_name.len()),
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
        let start = part.find(kw);
        if start.is_some() {
            let start = start.unwrap();

            return Err(
                Diagnostic::spanned(
                    span_range(lit, start, kw.len()), Level::Error,
                    format!("UPDATE statement doesn't support `{kw}` clauses")
                )
                .help(format!("`{kw}` clause doesn't apply to {kind} statements"))
            )
        }
    }

    Ok( () )
}

pub(crate) fn span_range( lit: &LitStr, start: usize, width: usize ) -> proc_macro2::Span {
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



fn check_brackets_match(sql: &str, lit: &LitStr) -> Result<Vec<(usize, usize)>, Diagnostic> {
    match brackets_are_balanced(sql) {
        Ok(v) => Ok(v),
        Err((left, right)) => {
            let highlight = span_range(lit, left, right - left);
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

// const INSERT_SYNTAX: &str = r#"INSERT statement syntax:
// INSERT [ IGNORE ] INTO @what
//     [ @value
//         | (@fields) VALUES (@values)
//         [ ON DUPLICATE KEY UPDATE @field = @value ... ]
//     ]
// "#;

// const CREATE_SYNTAX: &str = r#"CREATE statement syntax:
// CREATE @targets
//     [ CONTENT @value
//         | SET @field = @value ...
//     ]
//     [ RETURN [ NONE | BEFORE | AFTER | DIFF | @projections ... ]
//     [ TIMEOUT @duration ]
//     [ PARALLEL ]
// "#;

// const LET_SYNTAX: &str = r#"LET statement syntax:
// LET $@parameter = @value
// "#;

// const USE_SYNTAX: &str = r#"USE statement syntax:
// USE [ NS @ns ] [ DB @db ];
// "#;