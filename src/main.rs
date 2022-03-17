use spur::parser::create_session_from_args;
use std::{io, thread};
fn main() {
    let mut current_session = create_session_from_args();
    current_session.start();
    // Input
    let mut input = String::new();
    let main_handler = thread::spawn(move || loop {
        if input == String::from("end\n") {
            current_session.end();
            break;
        } else if input == String::from("cancel\n") {
            current_session.cancel();
            break;
        }
        input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // current_session.execute(&input);
    });
    main_handler.join().unwrap();
}
