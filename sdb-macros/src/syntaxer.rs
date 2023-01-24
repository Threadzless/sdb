use proc_macro_error::*;
use regex::Regex;
use syn::LitStr;

mod bracketer;
mod pointer;

pub use bracketer::*;
pub use pointer::*;

const VAR_FINDER_REGEX: &str = r"\$([[:alnum:]_]+)\b";
const SELECT_CLAUSE_ORDER: &[&str] = &[
    "SELECT", "FROM", "WHERE", "SPLIT", "GROUP", "ORDER", "LIMIT", "START", "FETCH", "TIMEOUT",
    "PARALLEL",
];

#[allow(unused)]
const UPDATE_CLAUSE_ORDER: &[&str] = &[
    "UPDATE", "*", // CONTENT, MERGE, PATCH, SET
    "WHERE", "RETURN", "TIMEOUT", "PARALLEL",
];

/// Perform SurrealQL syntax checking without sending it to the server. this should cover
/// basic stuff like non-marching parenthesies,
pub fn check_syntax(vars: &[(String, usize)], queries: &Vec<(String, &LitStr)>) -> Result<(), ()> {
    for (sql, lit) in queries {
        if check_trans_vars(vars, sql, lit).is_err() {
            continue;
        }

        let Ok( regions ) = check_brackets_match(sql, lit) else { continue };

        let Ok( parts ) = query_parts(sql, regions) else { continue };

        #[cfg(feature = "macro-print")]
        println!("Checking Syntax for query\n > {sql}");

        if check_clause_ordering(sql, lit, &parts).is_err() {
            continue;
        }
    }

    Ok(())
}

/// make sure all referenced transaction vars (things that starts with `$`) are
/// defined before they are used
fn check_trans_vars(vars: &[(String, usize)], sql: &str, lit: &LitStr) -> Result<(), ()> {
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

/// Breaks apart nested queries into individual strings
///
/// Input: `LET $c = (SELECT * FROM (SELECT title FROM books))`
///
/// Output:
/// ```rust,no_test
/// [ "Let $c = ()",  "SELECT * FROM ()",  "SELECT title FORM books" ];
/// ```
fn query_parts(sql: &str, mut regions: Vec<(usize, usize)>) -> Result<Vec<String>, ()> {
    let mut base_sql = String::from(sql);

    let len = sql.chars().count() - 1;
    regions.push((0, len));
    let mut parts = vec![];

    for (left, right) in regions {
        let space = left..=right;
        let part = base_sql[space.clone()].replace('\x00', "");
        parts.push(part);

        base_sql = base_sql
            .chars()
            .enumerate()
            .map(|(i, c)| match space.contains(&i) {
                true => '\x00',
                false => c,
            })
            .collect::<String>();
    }

    Ok(parts)
}

fn check_clause_ordering(_sql: &str, lit: &LitStr, parts: &Vec<String>) -> Result<(), ()> {
    for part in parts {
        #[cfg(feature = "macro-print")]
        println!("   > {part}");

        if part.starts_with("SELECT") {
            check_select_clause_order(&part, lit);
        }
        else if part.starts_with("UPDATE") {
            check_update_clause_order(&part, lit);
        }
        else {
            
        }
    }
    Ok(())
}


// 
fn check_select_clause_order(part: &str, lit: &LitStr) {
    let mut last_name = "SELECT";
    let mut last_index = 0;

    for clause in &SELECT_CLAUSE_ORDER[1..] {
        let Some( now_idx ) = part.find(clause) else { continue };
        if now_idx < last_index {
            emit_error!(
                lit, "SELECT Clauses out of order";
                help = "`{}` clause must come after `{}` clause", clause, last_name;
            );

            return;
        }
        last_name = clause;
        last_index = now_idx;
    }
}


///
fn check_update_clause_order(part: &str, lit: &LitStr) {
    let mut last_name = "UPDATE";
    let mut last_index = 0;
    
    for clause in &UPDATE_CLAUSE_ORDER[1..] {
        let Some( now_idx ) = part.find(clause) else { continue };
        if now_idx < last_index {
            emit_error!(
                lit, "UPDATE Clauses out of order";
                help = "`{}` clause must come after `{}` clause", clause, last_name;
            );

            return;
        }
        last_name = clause;
        last_index = now_idx;
    }

    for kw in vec!["ORDER", "SPLIT", "GROUP"] {
        if part.find(kw).is_some() {
            emit_error!(
                lit, "UPDATE query doesn't support that clause";
                help = "`{}` clause doesn't apply to UPDATE queries", kw;
            );            
        }
    }
}

fn check_brackets_match(sql: &str, lit: &LitStr) -> Result<Vec<(usize, usize)>, ()> {
    match brackets_are_balanced(sql) {
        Err((left, right)) => {
            let sql_pt = SqlErrorPointer::new(sql).tick(left, 1).tick(right, 1);

            emit_error!( lit, "Brackets are not balanced";
                help = "Make sure your brackets, braces, and parathesies are balanced.{:?}",
                    sql_pt;
            );

            Err(())
        }
        Ok(v) => Ok(v),
    }
}
