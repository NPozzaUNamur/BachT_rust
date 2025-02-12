use std::io::Write;

pub mod bacht;

fn main() {
    println!("\nWelcome the BachT interpreter cli 25.2.0 !\nYou can try this command: (tell(bach);get(rust))||(get(bach);tell(rust))\nRun 'exit' to leave the interpreter\n");use std::io::Write;
    print!("> ");
    std::io::stdout().flush().unwrap();

    let mut store = bacht::bacht_store::BachTStore::new();
    let mut input = String::new();

    while let Ok(_) = std::io::stdin().read_line(&mut input) {
        input = String::from(input.trim());
        if input == "exit" {break;}
        let res = bacht::bacht_parser::parse(&input);
        match res {
            Ok(ag) => println!("{:?}", ag),
            Err(e) => println!("{}", e)
        }
        input.clear();
        std::io::stdout().flush().unwrap();
        print!("> ");
        std::io::stdout().flush().unwrap();
    }
}
