use std::future::Future;
use crate::blackboard_interface::BlackboardInterfaceTrait;
use crate::model::error::CLIError;
use crate::model::data::Expr;
use crate::model::data::Expr::*;


pub trait SimulatorTrait {
    fn new() -> Self;
    
    fn run_one<'b>(&self, agent: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
    
    fn bacht_exec_all(&self, agent: Expr<'_>) -> impl Future<Output=Result<bool, CLIError>>;
    
    fn exec_primitive(&self, primitive: &str, coord_data: &str) -> impl Future<Output=Result<bool, CLIError>>;

    fn run_one_primitive<'b>(&self, prim: &'b str, token: &'b str) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
    
    fn run_one_sequence<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
    
    fn run_one_parallel<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
    
    fn run_one_choice<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
    
    fn parallel_branch_exec<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
    
    fn choice_branch_exec<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>>;
}

pub struct Simulator<B: BlackboardInterfaceTrait> {
    blackboard: B,
}

impl<B: BlackboardInterfaceTrait> SimulatorTrait for Simulator<B> {
    fn new() -> Self {
        Simulator { 
            blackboard: B::new() 
        }
    }

    async fn run_one<'b>(&self, agent: Expr<'b>) -> Result<(bool, Expr<'b>), CLIError> {
        // Must use Box::pin to allow recursive calls of async functions
        match agent {
            BachtAstPrimitive(prim, token) => Box::pin(self.run_one_primitive(prim, token)).await,
            BachtAstAgent(";", ag_i, ag_ii) => Box::pin(self.run_one_sequence(*ag_i, *ag_ii)).await,
            BachtAstAgent("||", ag_i, ag_ii) => Box::pin(self.run_one_parallel(*ag_i, *ag_ii)).await,
            BachtAstAgent("+", ag_i, ag_ii) => Box::pin(self.run_one_choice(*ag_i, *ag_ii)).await,
            _ => panic!("Unknown agent")
        }
    }

    async fn bacht_exec_all(&self, agent: Expr<'_>) -> Result<bool, CLIError> {
        if agent == BachtAstEmptyAgent() { return Ok(true); }
        let mut current_agent = agent;
        loop {
            match self.run_one(current_agent).await {
                Ok((false, _ag_cont)) => {
                    return Ok(false);
                },
                Ok((true, BachtAstEmptyAgent())) => {
                    return Ok(true);
                },
                Ok((true, ag_cont)) => {
                    current_agent = ag_cont;
                },
                Err(e) => return Err(e)
            };
        }
    }

    async fn exec_primitive(&self, primitive: &str, coord_data: &str) -> Result<bool, CLIError> {
        match primitive {
            "tell" => self.blackboard.tell(coord_data).await,
            "ask" => self.blackboard.ask(coord_data).await,
            "get" => self.blackboard.get(coord_data).await,
            "nask" => self.blackboard.nask(coord_data).await,
            _ => Err(CLIError::UnknownPrimitive(primitive.to_string()))
        }
    }

    async fn run_one_primitive<'b>(&self, prim: &'b str, token: &'b str) -> Result<(bool, Expr<'b>), CLIError> {
        match self.exec_primitive(prim, token).await {
            Ok(true) => Ok((true, BachtAstEmptyAgent())),
            Ok(false) => Ok((false, BachtAstPrimitive(prim, token))),
            Err(e) => Err(e)
        }
    }
    async fn run_one_sequence<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> Result<(bool, Expr<'b>), CLIError> {
        match self.run_one(ag_i).await {
            Ok((false, ag_i)) => Ok((false, BachtAstAgent(";", Box::new(ag_i), Box::new(ag_ii)))), //ag_i shadowing to get back ownership and recreate agent
            Ok((true, BachtAstEmptyAgent())) => Ok((true, ag_ii)),
            Ok((true, ag_cont)) => Ok((true, BachtAstAgent(";", Box::new(ag_cont), Box::new(ag_ii)))),
            Err(e) => Err(e)
        }
    }

    fn run_one_parallel<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>> {
        let branch_choice = rand::random::<bool>();
        if branch_choice {self.parallel_branch_exec(ag_i, ag_ii)}
        else {self.parallel_branch_exec(ag_ii, ag_i)}
    }

    fn run_one_choice<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> impl Future<Output=Result<(bool, Expr<'b>), CLIError>> {
        let branch_choice = rand::random::<bool>();
        if branch_choice {self.choice_branch_exec(ag_i, ag_ii)}
        else {self.choice_branch_exec(ag_ii, ag_i)}
    }

    async fn parallel_branch_exec<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> Result<(bool, Expr<'b>), CLIError> {
        match self.run_one(ag_i).await {
            Ok((false, ag_i)) => {
                match self.run_one(ag_ii).await {
                    Ok((false, ag_ii)) => Ok((false, BachtAstAgent("||", Box::new(ag_i), Box::new(ag_ii)))),
                    Ok((true, BachtAstEmptyAgent())) => Ok((true, ag_i)),
                    Ok((true, ag_cont)) => Ok((true, BachtAstAgent("||", Box::new(ag_i), Box::new(ag_cont)))),
                    Err(e) => Err(e)
                }
            },
            Ok((true, BachtAstEmptyAgent())) => Ok((true, ag_ii)),
            Ok((true, ag_cont)) => Ok((true, BachtAstAgent("||", Box::new(ag_cont), Box::new(ag_ii)))),
            Err(e) => Err(e)
        }
    }

    async fn choice_branch_exec<'b>(&self, ag_i: Expr<'b>, ag_ii: Expr<'b>) -> Result<(bool, Expr<'b>), CLIError> {
        match self.run_one(ag_i).await {
            Ok((false, ag_i)) => {
                match self.run_one(ag_ii).await {
                    Ok((false, ag_ii)) => Ok((false, BachtAstAgent("+", Box::new(ag_i), Box::new(ag_ii)))),
                    Ok((true, BachtAstEmptyAgent())) => Ok((true, BachtAstEmptyAgent())),
                    Ok((true, ag_cont)) => Ok((true, ag_cont)),
                    Err(e) => Err(e)
                }
            },
            Ok((true, BachtAstEmptyAgent())) => Ok((true, BachtAstEmptyAgent())),
            Ok((true, ag_cont)) => Ok((true, ag_cont)),
            Err(e) => Err(e)
        }
    }
    
}


/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod tests {
    use mockall::Sequence;
    use super::*;
    use crate::blackboard_interface::MockBlackboardInterfaceTrait;
    // Primitive tests
    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_tell_primitive() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_tell().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.exec_primitive("tell", "token").await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_ask_primitive() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_ask().times(1).returning(|_| Box::pin(async move {Ok(true)}));

        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.exec_primitive("ask", "token").await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_get_primitive() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_get().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.exec_primitive("get", "token").await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_nask_primitive() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_nask().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.exec_primitive("nask", "token").await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_refuse_hallucinate_primitive() {
        let mock_bb = MockBlackboardInterfaceTrait::default();

        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.exec_primitive("wrong", "token").await.is_err());
    }

    // Run-one tests

    #[tokio::test]
    async fn the_simulator_should_be_able_to_run_a_tell_primitive() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_tell().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        let agent = BachtAstPrimitive("tell", "token");
        match interpreter.run_one(agent).await {
            Ok((res, ag)) => {
                assert!(res);
                assert_eq!(ag, BachtAstEmptyAgent());
            },
            Err(_) => panic!("Error while running the agent")
        }
    }

    // agent

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_an_empty_agent() {
        let mock_bb = MockBlackboardInterfaceTrait::default();

        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.bacht_exec_all(BachtAstEmptyAgent()).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_sequence_of_agent() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        let mut seq = Sequence::new();
        mock_bb.expect_tell().times(1).in_sequence(&mut seq).returning(|_| Box::pin(async move {Ok(true)}));
        mock_bb.expect_ask().times(1).in_sequence(&mut seq).returning(|_| Box::pin(async move {Ok(true)}));
        
        let agent = BachtAstAgent(";",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_parallelism_of_agent() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_tell().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        mock_bb.expect_ask().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        
        let agent = BachtAstAgent("||",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_be_able_to_execute_a_choice_of_agent() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_tell().times(0..=1).returning(|_| Box::pin(async move {Ok(true)}));
        mock_bb.expect_ask().times(0..=1).returning(|_| Box::pin(async move {Ok(true)}));
        
        let agent = BachtAstAgent("+",
          Box::new(BachtAstPrimitive("tell", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_refuse_when_impossible_execution() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_nask().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        mock_bb.expect_ask().times(1).returning(|_| Box::pin(async move {Ok(false)}));
        
        let agent = BachtAstAgent(";",
          Box::new(BachtAstPrimitive("nask", "token")),
          Box::new(BachtAstAgent(";",
             Box::new(BachtAstPrimitive("ask", "token")),
             Box::new(BachtAstPrimitive("tell", "token"))
          ))
        );
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(!interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_accept_a_choice_combination_even_if_one_operator_cant_be_executed() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_nask().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        mock_bb.expect_ask().times(0..=1).returning(|_| Box::pin(async move {Ok(false)}));
        
        let agent = BachtAstAgent("+",
          Box::new(BachtAstPrimitive("nask", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_refuse_a_choice_parallel_if_one_operator_cant_be_executed() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_nask().times(1).returning(|_| Box::pin(async move {Ok(true)}));
        mock_bb.expect_ask().times(0..=2).returning(|_| Box::pin(async move {Ok(false)}));
        
        let agent = BachtAstAgent("||",
          Box::new(BachtAstPrimitive("nask", "token")),
          Box::new(BachtAstPrimitive("ask", "token"))
        );
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(!interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }

    #[tokio::test]
    async fn the_simulator_should_handle_complex_correct_expression() {
        let mut mock_bb = MockBlackboardInterfaceTrait::default();
        mock_bb.expect_tell().times(1..=4).returning(|_| Box::pin(async move {Ok(true)}));
        
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
        
        let interpreter: Simulator<MockBlackboardInterfaceTrait> = Simulator{blackboard: mock_bb};
        assert!(interpreter.bacht_exec_all(agent).await.is_ok_and(|v| v));
    }
}