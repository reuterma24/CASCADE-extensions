use std::fs;

pub fn verification(args: &[String]) {
    if args.len() < 1 {
        invalid_invocation_error();
    }

    let file_path = args[0].clone();

    verify(file_path);
}

fn verify(file_path: String) {
    let content = fs::read_to_string(file_path).unwrap();

    match syn::parse_file(&content) {
        Ok(_) => {
            println!("Syntactically correct!");
            std::process::exit(0);
        }
        Err(err) => {
            println!("Not syntactically correct!\n{}", err);
            std::process::exit(-1);
        }
    }
}

fn invalid_invocation_error() {
    eprintln!("Usage: rust-tool verify <pathToFile>");
    std::process::exit(-1);
}
