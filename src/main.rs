pub mod interpreter;
mod blackboard;
mod test;

use std::io::Write;

fn main() {
    println!("\nWelcome the BachT interpreter cli 25.2.1 !\nYou can try this command: (tell(bach);get(rust))||(get(bach);tell(rust))\nRun 'exit' to leave the interpreter\n");
    print!("> ");
    std::io::stdout().flush().unwrap();

    let mut store = blackboard::store::Store::new();
    let mut input = String::new();

    while let Ok(_) = std::io::stdin().read_line(&mut input) {
        input = String::from(input.trim());
        if input == "exit" {break;}
        let res = interpreter::bacht_parser::parse(&input);
        match res {
            Ok(ag) => {
                match interpreter::bacht_simulator::bacht_exec_all(&mut store, ag) {
                    true => println!("Success!"),
                    false => println!("Simulator cannot execute the given agent")
                }},
            Err(e) => println!("{}", e)
        }
        input.clear();
        std::io::stdout().flush().unwrap();
        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}
