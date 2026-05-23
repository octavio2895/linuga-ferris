use std::io;

fn generate_response(_input: &str) -> String {
    String::from("Gut! Lass uns anfangen.")
}

fn main() {
    println!("Was möchtest du üben? (What would you like to practice?)");
    let mut input = String::new();

    match io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            return;
        }
    }
    let input = input.trim();

    println!("You typed: {}", input);

    let response = generate_response(input);
    println!("{}", response);
    println!("You originally typed: {}", input)
}
