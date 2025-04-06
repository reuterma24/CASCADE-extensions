use crate::contexts::{Context, ParentType};
use crate::utils::{inject_test_module_after_function, inject_test_module_after_impl, BodyReplace};

use std::fs;
use std::path::Path;
use syn::{visit_mut::VisitMut, Block};

const NEW_CODE: &str = "new_code";
const NEW_TESTS: &str = "new_tests";
const NEW_TESTS_INJECT: &str = "new_tests_inject";

pub fn modification(args: &[String]) {
    if args.len() < 4 {
        invalid_invocation_error();
    }

    let project_dir = args[0].clone();
    let context_file_path = args[1].clone();
    let code_keyword = args[2].clone();
    let test_keyword = args[3].clone();

    println!("Starting to modify ... {} - {}", test_keyword, code_keyword);

    modify(project_dir, context_file_path, &code_keyword, &test_keyword);

    println!(
        "Modification successful! {} - {}",
        test_keyword, code_keyword
    );
}

fn modify(
    dir_path: String,
    context_file_path: String,
    code_keyword: &String,
    test_keyword: &String,
) {
    // Deserialize JSON into Context struct
    let file_content = fs::read_to_string(&context_file_path).unwrap();
    let context: Context = serde_json::from_str(&file_content).unwrap();

    if *test_keyword == NEW_TESTS {
        add_generated_test_file(&context, &dir_path);
    } else if *test_keyword == NEW_TESTS_INJECT {
        inject_test_module(&context, &dir_path);
    }

    if *code_keyword == NEW_CODE {
        replace_method(&context, &dir_path);
    }
}

fn inject_test_module(context: &Context, project_dir: &str) {
    println!("Starting test module injection ...");

    let file_path = Path::new(project_dir).join(&context.code_file_path);

    let mod_to_inject = context.new_tests.clone().unwrap();

    if !context.parent.is_empty()
        && context.parent.last().unwrap().parent_type == ParentType::Implementation
    {
        inject_test_module_after_impl(
            &file_path.as_path(),
            &context.parent.last().unwrap().name,
            &mod_to_inject,
        );
    } else {
        inject_test_module_after_function(
            &file_path.as_path(),
            &context.signature.name,
            &mod_to_inject,
        );
    }

    println!("Test module complete!");
}

fn add_generated_test_file(context: &Context, project_dir: &str) {
    if context.new_tests.is_none() {
        panic!("The new test class (new_tests property) is none! Verify the passed JSON file.");
    }

    let test = context.tests.first().unwrap();
    let file_path = Path::new(project_dir).join(&test.test_file_path);
    println!("{:?}", file_path.display());

    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(file_path, context.new_tests.as_ref().unwrap()).unwrap();
}

fn replace_method(context: &Context, project_dir: &str) {
    if context.new_code.is_none() {
        panic!("The new method body (new_code property) is none! Verify the passed JSON file.");
    }

    replace_method_body(context, context.new_code.as_ref().unwrap(), project_dir);
}

fn replace_method_body(context: &Context, method_body: &str, project_dir: &str) {
    let file_path = Path::new(project_dir).join(&context.code_file_path);

    println!("{:?}", file_path);

    let file_content = fs::read_to_string(&file_path).unwrap();
    let mut syntax_tree = syn::parse_file(&file_content).unwrap();

    let new_body: Block = syn::parse_str(method_body).unwrap();
    let function_name: String = context.signature.name.clone();

    let mut visitor = BodyReplace::new(new_body, function_name);
    visitor.visit_file_mut(&mut syntax_tree);

    let formatted_code = prettyplease::unparse(&syntax_tree);
    fs::write(file_path, formatted_code).unwrap();

    println!("Method replacement successful: {}", visitor.successful);
}

fn invalid_invocation_error() {
    eprintln!("Usage: rust-tool modify <pathToProject> <pathToEntry> <test_keyword> <code_keyword>,      (pathToEntry: file with single context in json format, test_keyword: will replace test with new tests if equal to 'new_tests', code_keyword: will replace method body with new code if equal to 'new_code')");
    std::process::exit(-1);
}
