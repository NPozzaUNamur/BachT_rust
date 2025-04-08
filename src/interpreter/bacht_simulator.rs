use crate::blackboard::store::StoreTrait;
use super::bacht_data::Expr;
use super::bacht_data::Expr::*;

pub(crate) fn run_one<'b>(blackboard: &mut dyn StoreTrait, agent: Expr<'b>) -> (bool, Expr<'b>) {
    match agent {
        BachtAstPrimitive(prim, token) => run_one_primitive(blackboard, prim, token),
        BachtAstAgent(";", ag_i, ag_ii) => run_one_sequence(blackboard, *ag_i, *ag_ii),
        BachtAstAgent("||", ag_i, ag_ii) => run_one_parallel(blackboard, *ag_i, *ag_ii),
        BachtAstAgent("+", ag_i, ag_ii) => run_one_choice(blackboard, *ag_i, *ag_ii),
        _ => panic!("Unknown agent")
    }
}

fn run_one_primitive<'b>(blackboard: &mut dyn StoreTrait, prim: &'b str, token: &'b str) -> (bool, Expr<'b>) {
    if exec_primitive(blackboard, prim, token) {
        (true, BachtAstEmptyAgent())
    } else {
        (false, BachtAstPrimitive(prim, token))
    }
}

fn run_one_sequence<'b>(blackboard: &mut dyn StoreTrait, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> (bool, Expr<'b>) {
    match run_one(blackboard, ag_i) {
        (false, ag_i) => (false, BachtAstAgent(";", Box::new(ag_i), Box::new(ag_ii))), //ag_i shadowing to get back ownership and recreate agent
        (true, BachtAstEmptyAgent()) => (true, ag_ii),
        (true, ag_cont) => (true, BachtAstAgent(";", Box::new(ag_cont), Box::new(ag_ii)))
    }
}

fn run_one_parallel<'b>(blackboard: &mut dyn StoreTrait, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> (bool, Expr<'b>) {
    let branch_choice = rand::random::<bool>();
    if branch_choice {parallel_branch_exec(blackboard, ag_i, ag_ii)}
    else {parallel_branch_exec(blackboard, ag_ii, ag_i)}
}

fn parallel_branch_exec<'b>(blackboard: &mut dyn StoreTrait, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> (bool, Expr<'b>) {
    match run_one(blackboard, ag_i) {
        (false, ag_i) => {
            match run_one(blackboard, ag_ii) {
                (false, ag_ii) => (false, BachtAstAgent("||", Box::new(ag_i), Box::new(ag_ii))),
                (true, BachtAstEmptyAgent()) => (true, ag_i),
                (true, ag_cont) => (true, BachtAstAgent("||", Box::new(ag_i), Box::new(ag_cont)))
            }
        },
        (true, BachtAstEmptyAgent()) => (true, ag_ii),
        (true, ag_cont) => (true, BachtAstAgent("||", Box::new(ag_cont), Box::new(ag_ii)))
    }
}

fn run_one_choice<'b>(blackboard: &mut dyn StoreTrait, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> (bool, Expr<'b>) {
    let branch_choice = rand::random::<bool>();
    if branch_choice {choice_branch_exec(blackboard, ag_i, ag_ii)}
    else {choice_branch_exec(blackboard, ag_ii, ag_i)}
}

fn choice_branch_exec<'b>(blackboard: &mut dyn StoreTrait, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> (bool, Expr<'b>) {
    match run_one(blackboard, ag_i) {
        (false, ag_i) => {
            match run_one(blackboard, ag_ii) {
                (false, ag_ii) => (false, BachtAstAgent("+", Box::new(ag_i), Box::new(ag_ii))),
                (true, BachtAstEmptyAgent()) => (true, BachtAstEmptyAgent()),
                (true, ag_cont) => (true, ag_cont)
            }
        },
        (true, BachtAstEmptyAgent()) => (true, BachtAstEmptyAgent()),
        (true, ag_cont) => (true, ag_cont)
    }
}

pub(crate) fn bacht_exec_all(blackboard: &mut dyn StoreTrait, agent: Expr) -> bool {
    let is_executed;
    let mut current_agent = agent;
    loop {
        if current_agent == BachtAstEmptyAgent() {
            is_executed = true;
            break;
        }

        let (res, new_agent) = run_one(blackboard, current_agent);
        blackboard.print_store();

        if !res {
            is_executed = false;
            break;
        }

        current_agent = new_agent;
    }
    is_executed
}

fn exec_primitive(blackboard: &mut dyn StoreTrait, primitive: &str, token: &str) -> bool {
    match primitive {
        "tell" => blackboard.tell(token.into()),
        "ask" => blackboard.ask(token),
        "get" => blackboard.get(token.into()),
        "nask" => blackboard.nask(token),
        _ => panic!("Unknown primitive")
    }
}


#[cfg(test)]
mod tests {
    use mockall::Sequence;
    use super::*;
    use crate::blackboard::store::MockStoreTrait;
    // Primitive tests
    #[test]
    fn the_simulator_should_be_able_to_execute_a_tell_primitive() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_tell().times(1).returning(|_| true);
        assert!(exec_primitive(&mut mock_bb, "tell", "token"));
    }

    #[test]
    fn the_simulator_should_be_able_to_execute_a_ask_primitive() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_ask().times(1).returning(|_| true);
        assert!(exec_primitive(&mut mock_bb, "ask", "token"));
    }

    #[test]
    fn the_simulator_should_be_able_to_execute_a_get_primitive() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_get().times(1).returning(|_| true);
        assert!(exec_primitive(&mut mock_bb, "get", "token"));
    }

    #[test]
    fn the_simulator_should_be_able_to_execute_a_nask_primitive() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_nask().times(1).returning(|_| true);
        assert!(exec_primitive(&mut mock_bb, "nask", "token"));
    }

    #[test]
    #[should_panic]
    fn the_simulator_should_refuse_hallucinate_primitive() {
        let mut mock_bb = MockStoreTrait::new();
        exec_primitive(&mut mock_bb, "wrong", "token");
    }

    // Run-one tests

    #[test]
    fn the_simulator_should_be_able_to_run_a_tell_primitive() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_tell().times(1).returning(|_| true);
        let agent = BachtAstPrimitive("tell", "token");
        let (res, ag) = run_one(&mut mock_bb, agent);
        assert!(res);
        assert_eq!(ag, BachtAstEmptyAgent());
    }

    // agent

    #[test]
    fn the_simulator_should_be_able_to_execute_an_empty_agent() {
        let mut mock_bb = MockStoreTrait::new();
        assert!(bacht_exec_all(&mut mock_bb, BachtAstEmptyAgent()));
    }

    #[test]
    fn the_simulator_should_be_able_to_execute_a_sequence_of_agent() {
        let mut mock_bb = MockStoreTrait::new();
        let mut seq = Sequence::new();
        mock_bb.expect_tell().times(1).in_sequence(&mut seq).returning(|_| true);
        mock_bb.expect_ask().times(1).in_sequence(&mut seq).returning(|_| true);
        mock_bb.expect_print_store().times(2).returning(|| ());
        let agent = BachtAstAgent(";",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        assert!(bacht_exec_all(&mut mock_bb, agent));
    }

    #[test]
    fn the_simulator_should_be_able_to_execute_a_parallelism_of_agent() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_tell().times(1).returning(|_| true);
        mock_bb.expect_ask().times(1).returning(|_| true);
        mock_bb.expect_print_store().times(2).returning(|| ());
        let agent = BachtAstAgent("||",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        assert!(bacht_exec_all(&mut mock_bb, agent));
    }

    #[test]
    fn the_simulator_should_be_able_to_execute_a_choice_of_agent() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_tell().times(0..=1).returning(|_| true);
        mock_bb.expect_ask().times(0..=1).returning(|_| true);
        mock_bb.expect_print_store().times(1).returning(|| ());
        let agent = BachtAstAgent("+",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        assert!(bacht_exec_all(&mut mock_bb, agent));
    }

    #[test]
    fn the_simulator_should_refuse_when_impossible_execution() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_nask().times(1).returning(|_| true);
        mock_bb.expect_ask().times(1).returning(|_| false);
        mock_bb.expect_print_store().times(2).returning(|| ());
        let agent = BachtAstAgent(";",
          Box::new(BachtAstPrimitive("nask", "token")),
          Box::new(BachtAstAgent(";",
             Box::new(BachtAstPrimitive("ask", "token")),
             Box::new(BachtAstPrimitive("tell", "token"))
          ))
        );
        assert!(!bacht_exec_all(&mut mock_bb, agent));
    }

    #[test]
    fn the_simulator_should_accept_a_choice_combination_even_if_one_operator_cant_be_executed() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_nask().times(1).returning(|_| true);
        mock_bb.expect_ask().times(0..=1).returning(|_| false);
        mock_bb.expect_print_store().times(1).returning(|| ());
        let agent = BachtAstAgent("+",
          Box::new(BachtAstPrimitive("nask", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        assert!(bacht_exec_all(&mut mock_bb, agent));
    }

    #[test]
    fn the_simulator_should_refuse_a_choice_parallel_if_one_operator_cant_be_executed() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_nask().times(1).returning(|_| true);
        mock_bb.expect_ask().times(0..=2).returning(|_| false);
        mock_bb.expect_print_store().times(0..=3).returning(|| ());
        let agent = BachtAstAgent("||",
          Box::new(BachtAstPrimitive("nask", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        assert!(!bacht_exec_all(&mut mock_bb, agent));
    }

    #[test]
    fn the_simulator_should_handle_complex_correct_expression() {
        let mut mock_bb = MockStoreTrait::new();
        mock_bb.expect_tell().times(1..=4).returning(|_| true);
        mock_bb.expect_print_store().times(1..=4).returning(|| ());
        let agent = BachtAstAgent("+",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstAgent("||",
            Box::new(BachtAstPrimitive("tell", "token")),
            Box::new(BachtAstAgent(";",
               Box::new(BachtAstPrimitive("tell", "token")),
               Box::new(BachtAstPrimitive("tell", "token"))
            ))
          ))
        );
        assert!(bacht_exec_all(&mut mock_bb, agent));
    }
}