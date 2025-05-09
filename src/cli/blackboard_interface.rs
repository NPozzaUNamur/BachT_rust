use std::future::Future;
use mockall::automock;
use crate::model::error::CLIError;

#[automock]
pub trait BlackboardInterfaceTrait {
    
    fn new() -> Self;
    
    fn tell(&self, coord_data: &str) -> impl Future<Output=Result<bool, CLIError>>;
    
    fn ask(&self, coord_data: &str) -> impl Future<Output=Result<bool, CLIError>>;
    
    fn get(&self, coord_data: &str) -> impl Future<Output=Result<bool, CLIError>>;
    
    fn nask(&self, coord_data: &str) -> impl Future<Output=Result<bool, CLIError>>;
}