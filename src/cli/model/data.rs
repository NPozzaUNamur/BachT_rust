/// The BachT AST used to represent agents
#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'b> {
    BachtAstEmptyAgent(),

    // bacht_ast_primitive(primitive, token),
    BachtAstPrimitive(&'b str, &'b str),

    // bacht_ast_agent(operator, agent_i, agent_ii),
    // uses box to avoid recursive type see: [RustBook](https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes)
    BachtAstAgent(&'b str, Box<Expr<'b>>, Box<Expr<'b>>)
}