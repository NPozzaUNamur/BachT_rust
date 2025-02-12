use nom::{
    IResult, Parser, Err,
    error::{Error, ErrorKind},
    sequence::delimited, bytes::tag,
    combinator::{opt, complete, all_consuming}
};
use regex::{Regex};

/// The BachT AST used to represent agents
#[derive(Debug, PartialEq)]
pub(crate) enum Expr<'b> {
    // BachtAstEmptyAgent(),

    // bacht_ast_primitive(primitive, token),
    BachtAstPrimitive(&'b str, &'b str),

    // bacht_ast_agent(operator, agent_i, agent_ii),
    // uses box to avoid recursive type see: [RustBook](https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes)
    BachtAstAgent(&'b str, Box<Expr<'b>>, Box<Expr<'b>>)
}

/// Parses a token from the input string using a regular expression.
/// Note that the token must start with a lowercase letter and can contain any number of letters, digits, and underscores.
/// It must be on the first position of the input string.
///
/// ### Arguments
///
/// * `input` - A string slice that holds the input to be parsed.
///
/// ### Returns
///
/// * `IResult<&str, &str>` - A result containing the remaining input and the parsed token,
///   or an error if the token could not be parsed.
///
/// ### Errors
///
/// * Returns `Err::Error` if the input does not match the regular expression for a valid token.
///
fn token(input: &str) -> IResult<&str, &str> {
    Regex::new(r"^[a-z][a-zA-Z0-9_]*").unwrap().find(input).map(
        |m| (&input[m.end()..], m.as_str())
    ).ok_or(Err::Error(Error::new(input, ErrorKind::RegexpFind)))
}

/// Parses a primitive expression from the input string.
///
/// This function attempts to parse one of the following primitives: `tell`, `ask`, `get`, or `nask`.
/// Each primitive is expected to be followed by a token enclosed in parentheses.
///
/// ### Arguments
///
/// * `input` - A string slice that holds the input to be parsed.
///
/// ### Returns
///
/// * `IResult<&str, Expr>` - A result containing the remaining input and the parsed expression,
///   or an error if none of the primitives could be parsed.
///
fn primitive(input: &str) -> IResult<&str, Expr> {

    delimited(tag("tell("), token, tag(")")).parse(input).map(
        |(next_input, token)| (next_input, Expr::BachtAstPrimitive("tell", token))

    ).or_else(|_| delimited(tag("ask("), token, tag(")")).parse(input).map(
        |(next_input, token)| (next_input, Expr::BachtAstPrimitive("ask", token)))

    ).or_else(|_| delimited(tag("get("), token, tag(")")).parse(input).map(
        |(next_input, token)| (next_input, Expr::BachtAstPrimitive("get", token)))

    ).or_else(|_| delimited(tag("nask("), token, tag(")")).parse(input).map(
        |(next_input, token)| (next_input, Expr::BachtAstPrimitive("nask", token)))
    )
}

/// Parses an agent expression from the input string.
/// It handles the following operators: `;`, `||`, and `+`.
///
/// ### Arguments
///
/// * `input` - A string slice that holds the agent to be parsed.
///
/// ### Returns
///
/// * `IResult<&str, Expr>` - A result containing the remaining input and the parsed agent expression,
///   or an error if the input could not be parsed as an agent expression.
///
/// ### Exemples
///
/// `agent(tell(token1);tell(token2)||tell(token3)+tell(token4))`
///
///  ̀``Expr::BachtAstAgent("+",
///       Box::new(Expr::BachtAstAgent("||",
///           Box::new(Expr::BachtAstAgent(";",
///               Box::new(Expr::BachtAstPrimitive("tell", "token1")),
///               Box::new(Expr::BachtAstPrimitive("tell", "token2"))
///           )),
///           Box::new(Expr::BachtAstPrimitive("tell", "token3"))
///       )),
///       Box::new(Expr::BachtAstPrimitive("tell", "token4"))
///  )```
fn agent(input: &str) -> IResult<&str, Expr> { composition_choice(input) }

fn composition_choice(input: &str) -> IResult<&str, Expr> {
    (composition_para, complete(opt((tag("+"), composition_choice)))).parse(input).map(
        |(next_input, (agi, next))| match next {
            None => (next_input, agi),
            Some((_, agii)) => (next_input, Expr::BachtAstAgent("+", Box::new(agi), Box::new(agii)))
        }
    )
}

fn composition_para(input: &str) -> IResult<&str, Expr> {
    (composition_seq, complete(opt((tag("||"), composition_para)))).parse(input).map(
        |(next_input, (agi, next))| match next {
            None => (next_input, agi),
            Some((_, agii)) => (next_input, Expr::BachtAstAgent("||", Box::new(agi), Box::new(agii)))
        }
    )
}

fn composition_seq(input: &str) -> IResult<&str, Expr> {
    (simple_agent, complete(opt((tag(";"), composition_seq)))).parse(input).map(
        |(next_input, (agi, next))| match next {
            None => (next_input, agi),
            Some((_, agii)) => (next_input, Expr::BachtAstAgent(";", Box::new(agi), Box::new(agii)))
        }
    )
}

fn simple_agent(input: &str) -> IResult<&str, Expr> {
    primitive(input).or_else(|_| parenthesized_agent(input))
}

fn parenthesized_agent(input: &str) -> IResult<&str, Expr> {
    delimited(tag("("), composition_choice, tag(")")).parse(input)
}


/// Parses an agent expression from the input string.
///
/// This function serves as the entry point for parsing an agent expression,
/// delegating the actual parsing to the `agent` function and ensuring that the entire input is consumed.
///
/// ### Arguments
///
/// * `input` - A string slice that holds the agent to be parsed.
///
/// ### Returns
///
/// * `Result<Expr, Err<Error<&str>>>` - A result containing the parsed agent expression,
///   or an error if the input could not be parsed as an agent expression or if the entire input was not consumed.
///
/// ### Errors
///
/// * Returns `Err::Error` if the input could not be parsed as an agent expression or if the entire input was not consumed.
pub(crate) fn parse_agent(input: &str) -> Result<Expr, Err<Error<&str>>> {
    match all_consuming(agent).parse(input) {
        Ok(("", expr)) => Ok(expr),
        Ok((_, _)) => Err(Err::Error(Error::new(input, ErrorKind::Complete))),
        Err(err) => Err(err)
    }
}

pub(crate) fn parse(input: &str) -> Result<Expr, Err<Error<&str>>> {
    parse_agent(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Primitive section

    #[test]
    fn the_parser_should_be_able_to_parse_a_tell_primitive() {
        let res = primitive("tell(token)");
        assert!(matches!(res, Ok(("", Expr::BachtAstPrimitive("tell", "token")))));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_an_ask_primitive() {
        let res = primitive("ask(token)");
        assert!(matches!(res, Ok(("", Expr::BachtAstPrimitive("ask", "token")))));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_a_get_primitive() {
        let res = primitive("get(token)");
        assert!(matches!(res, Ok(("", Expr::BachtAstPrimitive("get", "token")))));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_a_nask_primitive() {
        let res = primitive("nask(token)");
        assert!(matches!(res, Ok(("", Expr::BachtAstPrimitive("nask", "token")))));
    }

    #[test]
    fn the_parser_should_refuse_hallucinate_primitives() {
        let res = primitive("non(token)");
        assert!(matches!(res, Err(Err::Error(_))));
    }

    /// Token Section
    #[test]
    fn the_parser_should_be_able_to_parse_a_token() {
        let res = token("token");
        assert_eq!(res, Ok(("", "token")));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_a_token_with_capital_character_and_number() {
        let res = token("tOkEN12E");
        assert_eq!(res, Ok(("", "tOkEN12E")));
    }

    #[test]
    fn the_parser_should_refuse_token_with_special_character() {
        let res = primitive("tell(tOkEN12E@)");
        assert!(matches!(res, Err(Err::Error(_))));
    }

    #[test]
    fn the_parser_should_refuse_token_with_first_character_as_number() {
        let res = token("7oken");
        assert_eq!(res, Err(Err::Error(Error::new("7oken", ErrorKind::RegexpFind))));
    }

    #[test]
    fn the_parser_should_refuse_token_with_first_character_as_capitals() {
        let res = token("Token");
        assert_eq!(res, Err(Err::Error(Error::new("Token", ErrorKind::RegexpFind))));
    }

    // Agent section

    // Not supported in scala version too
    // #[test]
    // fn the_parser_should_be_able_to_parse_an_empty_agent() {
    //     let res = agent("");
    //     assert_eq!(res, Ok(("", Expr::BachtAstEmptyAgent())));
    // }

    #[test]
    fn the_parser_should_be_able_to_parse_a_simple_agent() {
        let res = parse_agent("tell(token)");
        assert_eq!(res, Ok(Expr::BachtAstPrimitive("tell", "token")));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_a_simple_agent_in_brackets() {
        let res = parse_agent("(tell(token))");
        assert_eq!(res, Ok(Expr::BachtAstPrimitive("tell", "token")));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_sequence_operator() {
        let res = parse_agent("tell(token1);tell(token2)");
        let expect_res = Ok(Expr::BachtAstAgent(";",
            Box::new(Expr::BachtAstPrimitive("tell", "token1")),
            Box::new(Expr::BachtAstPrimitive("tell", "token2"))
        ));
        assert_eq!(res, expect_res);
    }

    #[test]
    fn the_parser_should_be_able_to_parse_parallel_operator() {
        let res = parse_agent("tell(token1)||tell(token2)");
        assert_eq!(res, Ok(Expr::BachtAstAgent("||",
            Box::new(Expr::BachtAstPrimitive("tell", "token1")),
            Box::new(Expr::BachtAstPrimitive("tell", "token2"))
        )));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_choice_operator() {
        let res = parse_agent("tell(token1)+tell(token2)");
        assert_eq!(res, Ok(Expr::BachtAstAgent("+",
            Box::new(Expr::BachtAstPrimitive("tell", "token1")),
            Box::new(Expr::BachtAstPrimitive("tell", "token2"))
        )));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_multiple_operators() {
        let res = parse_agent("tell(token1)||tell(token2)||tell(token3)");
        assert_eq!(res, Ok(Expr::BachtAstAgent("||",
            Box::new(Expr::BachtAstPrimitive("tell", "token1")),
            Box::new(Expr::BachtAstAgent("||",
                Box::new(Expr::BachtAstPrimitive("tell", "token2")),
                Box::new(Expr::BachtAstPrimitive("tell", "token3"))
            ))
        )));
    }

    #[test]
    fn the_parser_should_be_able_to_parse_nested_operators() {
        let res1 = parse_agent("tell(token1);tell(token2)||tell(token3)+tell(token4)");
        assert_eq!(res1, Ok(Expr::BachtAstAgent("+",
            Box::new(Expr::BachtAstAgent("||",
                Box::new(Expr::BachtAstAgent(";",
                    Box::new(Expr::BachtAstPrimitive("tell", "token1")),
                    Box::new(Expr::BachtAstPrimitive("tell", "token2"))
                )),
                Box::new(Expr::BachtAstPrimitive("tell", "token3"))
            )),
            Box::new(Expr::BachtAstPrimitive("tell", "token4"))
        )));

        let res2 = parse_agent("tell(token1)+tell(token2)||tell(token3);tell(token4)");
        assert_eq!(res2, Ok(Expr::BachtAstAgent("+",
            Box::new(Expr::BachtAstPrimitive("tell", "token1")),
            Box::new(Expr::BachtAstAgent("||",
                Box::new(Expr::BachtAstPrimitive("tell", "token2")),
                Box::new(Expr::BachtAstAgent(";",
                    Box::new(Expr::BachtAstPrimitive("tell", "token3")),
                    Box::new(Expr::BachtAstPrimitive("tell", "token4"))
                ))
            ))
        )));
    }

    #[test]
    fn the_parser_should_refuse_hallucinate_operator() {
        let res = parse_agent("tell(token1)??tell(token2)");
        assert!(matches!(res, Err(_)));
    }

    #[test]
    fn the_parser_should_refuse_hallucinate_token() {
        let res = parse_agent("tell(token1)@");
        assert!(matches!(res, Err(_)));
    }
}