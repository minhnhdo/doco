use nom::{alphanumeric, digit, IResult};
use regex::Regex;
use std::collections::HashMap;
use std::{str, i16, i32, i64, i8};

use super::super::range::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    SInt8,
    SInt16,
    SInt32,
    SInt64,
}

impl Type {
    pub fn range(&self) -> Range {
        match self {
            &Type::SInt8 => Range::from(i8::MIN as i64, i8::MAX as i64),
            &Type::SInt16 => Range::from(i16::MIN as i64, i16::MAX as i64),
            &Type::SInt32 => Range::from(i32::MIN as i64, i32::MAX as i64),
            &Type::SInt64 => Range::from(i64::MIN, i64::MAX),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Variable {
    pub name: String,
    pub typ: Type,
    pub range: Range,
}

#[derive(Debug)]
pub enum Condition {
    True,
    Conditions(HashMap<String, Variable>),
}

#[derive(Debug)]
pub enum Expression {
    Parsed(Condition),
    Unparsable(String),
}

impl Expression {
    pub fn from_str(s: &str) -> Expression {
        match parse_declaration(s.as_bytes()) {
            IResult::Done(_, Some(vars)) => Expression::Parsed(vars),
            _ => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"\([a-zA-Z0-9]*\)").unwrap();
                }
                let stripped = if let Some(idx) = s.find('(') {
                    &s[idx..]
                } else {
                    s
                };
                Expression::Unparsable(String::from(RE.replace_all(s, "")))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Ast {
    And(Vec<Ast>),
    Lt(String, i64),
    Lte(String, i64),
    Gt(String, i64),
    Gte(String, i64),
    Eq(String, i64),
    Neq(String, i64),
}

fn variable_map(var_decls: Vec<(&[u8], Type)>) -> HashMap<String, Variable> {
    let mut vars = HashMap::new();
    for &(ref name, ref typ) in var_decls.iter() {
        let name_string = String::from_utf8(name.to_vec()).unwrap();
        vars.insert(
            name_string.clone(),
            Variable {
                name: name_string,
                typ: typ.clone(),
                range: typ.range(),
            },
        );
    }
    vars
}

fn bytes_to_type(bytes: &[u8]) -> Type {
    match bytes {
        b"sint8" => Type::SInt8,
        b"sint16" => Type::SInt16,
        b"sint32" => Type::SInt32,
        b"sint64" => Type::SInt64,
        _ => unreachable!(),
    }
}

fn make_comparison(name: &[u8], op: &[u8], val: i64) -> Ast {
    let name_string = String::from_utf8(name.to_vec()).unwrap();
    match op {
        b"==" => Ast::Eq(name_string, val),
        b"!=" => Ast::Neq(name_string, val),
        b"<" => Ast::Lt(name_string, val),
        b"<=" => Ast::Lte(name_string, val),
        b">" => Ast::Gt(name_string, val),
        b">=" => Ast::Gte(name_string, val),
        _ => unreachable!(),
    }
}

fn interprete(vars: &mut HashMap<String, Variable>, ast: &Ast) -> Option<()> {
    match ast {
        &Ast::And(ref v) => for e in v.iter() {
            interprete(vars, e)?;
        },
        &Ast::Lt(ref name, val) => {
            let mut v = vars.get_mut(name)?;
            v.range = v.range.intersect(&Range::from(i64::MIN, val - 1));
        }
        &Ast::Lte(ref name, val) => {
            let mut v = vars.get_mut(name)?;
            v.range = v.range.intersect(&Range::from(i64::MIN, val));
        }
        &Ast::Gt(ref name, val) => {
            let mut v = vars.get_mut(name)?;
            v.range = v.range.intersect(&Range::from(val + 1, i64::MAX));
        }
        &Ast::Gte(ref name, val) => {
            let mut v = vars.get_mut(name)?;
            v.range = v.range.intersect(&Range::from(val, i64::MAX));
        }
        &Ast::Eq(ref name, val) => {
            let mut v = vars.get_mut(name)?;
            v.range = v.range.intersect(&Range::from(val, val));
        }
        &Ast::Neq(ref name, val) => {
            let mut v = vars.get_mut(name)?;
            v.range = v.range
                .intersect(&Range::from(i64::MIN, val - 1).union(&Range::from(val + 1, i64::MAX)));
        }
    }
    Some(())
}

named! {
    parse_variable,
    delimited!(tag!("'"), alphanumeric, tag!("'"))
}

named! {
    parse_ident,
    do_parse!(
        opt!(delimited!(tag!("("), parse_type, tag!(")"))) >>
        bytes: parse_variable >>
        (bytes)
    )
}

named! {
    parse_parentheses<Ast>,
    delimited!(tag!("("), alt_complete!(parse_comparision | parse_and), tag!(")"))
}

named! {
    parse_and<Ast>,
    map!(separated_list_complete!(ws!(tag!("&&")), parse_parentheses), Ast::And)
}

named!{
    parse_negative_number<i64>,
    map!(
        do_parse!(tag!("-") >> ds: digit >> (ds)),
        |num| -str::from_utf8(num).unwrap().parse::<i64>().unwrap()
    )
}

named! {
    parse_comparision<Ast>,
    do_parse!(
        ident: parse_ident >>
        op: ws!(alt_complete!(tag!("==") | tag!("!=") | tag!("<=") | tag!(">=") | tag!(">") | tag!("<"))) >>
        val: alt_complete!(
            map!(digit, |ds| str::from_utf8(ds).unwrap().parse::<i64>().unwrap()) |
            parse_negative_number
        ) >>
        (make_comparison(ident, op, val))
    )
}

named! {
    parse_type<Type>,
    map!(alt_complete!(tag!("sint8") | tag!("sint16") | tag!("sint32") | tag!("sint64")), bytes_to_type)
}

named! {
    parse_variable_declaration<(&[u8], Type)>,
    do_parse!(
        name: parse_variable >>
        tag!(":") >>
        typ: parse_type >>
        (name, typ)
    )
}

named! {
    parse_declaration< Option<Condition> >,
    alt_complete!(
        map!(tag!("[L]true"), |_| Some(Condition::True))
        | map!(
            do_parse!(
                tag!("[L]declare ") >>
                vars: map!(separated_nonempty_list!(tag!(", "), parse_variable_declaration), variable_map) >>
                tag!(" in ") >>
                ast: parse_parentheses >>
                (vars, ast)
            ),
            |(mut vars, ast)| { interprete(&mut vars, &ast).map(|()| Condition::Conditions(vars)) }
        )
    )
}

#[cfg(test)]
mod test {
    use nom;
    use std::collections::HashMap;
    use std::{i32, i64};

    use super::{parse_comparision, parse_declaration, parse_variable_declaration, Ast, Range,
                Type, Variable};

    #[test]
    fn test_parse_variable_declaration() {
        assert_eq!(
            nom::IResult::Done(&b""[..], (&b"a"[..], Type::SInt32)),
            parse_variable_declaration(&b"'a':sint32"[..]),
        );
    }

    #[test]
    fn test_parse_comparison() {
        assert_eq!(
            nom::IResult::Done(&b""[..], Ast::Lt(String::from("a"), 2)),
            parse_comparision(&b"'a' < 2"[..]),
        );
    }

    #[test]
    fn test_parse_simple_declaration() {
        let mut m = HashMap::new();
        m.insert(
            String::from("a"),
            Variable {
                name: String::from("a"),
                typ: Type::SInt64,
                range: Range::from(1, i64::MAX),
            },
        );
        let (_, output) = parse_declaration(&b"[L]declare 'a':sint64 in (('a' > 0))"[..]).unwrap();
        assert_eq!(Some(Condition::Conditions(m)), output);
    }

    #[test]
    fn test_parse_complex_declaration() {
        let mut m = HashMap::new();
        m.insert(
            String::from("a"),
            Variable {
                name: String::from("a"),
                typ: Type::SInt32,
                range: Range::from(i32::MIN as i64, -1),
            },
        );
        m.insert(
            String::from("b"),
            Variable {
                name: String::from("b"),
                typ: Type::SInt64,
                range: Range::from(i64::MIN, 1).union(&Range::from(3, i64::MAX)),
            },
        );
        let (_, output) = parse_declaration(
            &b"[L]declare 'a':sint32, 'b':sint64 in (((sint64)'a' < 0) && ((sint8)'b' != 2))"[..],
        ).unwrap();
        assert_eq!(Some(Condition::Conditions(m)), output);
    }

    #[test]
    fn test_parse_complex_nested_declaration() {
        let mut m = HashMap::new();
        m.insert(
            String::from("a"),
            Variable {
                name: String::from("a"),
                typ: Type::SInt32,
                range: Range::from(i32::MIN as i64, -1),
            },
        );
        m.insert(
            String::from("b"),
            Variable {
                name: String::from("b"),
                typ: Type::SInt64,
                range: Range::from(i64::MIN, 1).union(&Range::from(3, 12)),
            },
        );
        let (_, output) = parse_declaration(
            &b"[L]declare 'a':sint32, 'b':sint64 in (((sint64)'a' < 0) && (((sint8)'b' != 2) && ((sint8)'b' <= 12)))"[..],
        ).unwrap();
        assert_eq!(Some(Condition::Conditions(m)), output);
    }

    #[test]
    fn test_parse_simple_real_declaration() {
        let mut m = HashMap::new();
        m.insert(
            String::from("n"),
            Variable {
                name: String::from("n"),
                typ: Type::SInt32,
                range: Range::from(0, 1),
            },
        );
        let (_, output) = parse_declaration(
            &b"[L]declare 'n':sint32 in (((sint64)'n' >= 0) && ((sint64)'n' < 2))"[..],
        ).unwrap();
        assert_eq!(Some(Condition::Conditions(m)), output);
    }
}
