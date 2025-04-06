use std::env;

mod code_extractor;
mod contexts;
mod extractor;
mod modifier;
mod test_extractor;
mod utils;
mod verifier;

const EXTRACTION: &str = "extract";
const MODIFICATION: &str = "modify";
const VERIFICATION: &str = "verify";

fn main() {
    // only for debugging purposes
    if true {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            invalid_invocation_error();
        }

        let invocation_args = &args[2..];

        match args[1].as_str() {
            EXTRACTION => extractor::extraction(invocation_args),
            MODIFICATION => modifier::modification(invocation_args),
            VERIFICATION => verifier::verification(invocation_args),
            _ => invalid_invocation_error(),
        }
    } else {
        let _project_path =
            "C:\\Users\\Martin\\Desktop\\Masterarbeit\\dummy_project\\testing-rust".to_string();
        let _output_path = "C:\\Users\\Martin\\Desktop\\Masterarbeit\\".to_string();

        /*
        verifier::verify("C:\\Users\\Martin\\Desktop\\Masterarbeit\\dummy_project\\simple-rust\\src\\main.rs".to_string());
        let input = vec![_project_path, _output_path];
        extractor::extraction(&input);
        let input = vec![
            "/Users/mar/Desktop/Masterarbeit/dummy_project/testing-rust2".to_string(),
            "/Users/mar/Desktop/Masterarbeit/entry.json".to_string(),
            "new_code".to_string(),
            "tests".to_string(),
        ];
        modifier::modification(&input);
        */
    }
}

fn invalid_invocation_error() {
    eprintln!("Usage: rust-tool <command>,        (possible commands: extract, modify, verify)");
    std::process::exit(-1);
}
