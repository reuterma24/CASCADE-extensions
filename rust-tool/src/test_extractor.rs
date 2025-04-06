/*
TODO:
 - traverse through each file, parse .rs files
 - check code for the following things:
        - check for annotations like #[test]
        - check for conditional compiler flags #[cfg(test)]

 - extract doc tests (locate "/// ```")  --> idea: just keep the doc test as it is, add it to the generated function later and invoke it


---------------------------
check if the matching test function is inside a module that is annotated with #[cfg(test)]
if yes: mod test
if no: check if inside /tests/ directory
    if yes: its a test target addressable via --test flag

-------------------------------------

runs everything: cargo test --no-fail-fast


modules:
cargo test -- mod_name

tests files:
cargo test --test test_file_name

doc test:
cargo test --doc func_name
*/

use std::collections::HashMap;
use std::path::Path;
use syn::{visit::Visit, Block};

use crate::{
    contexts::{Context, Test},
    utils::{get_relative_path, ExprVisitor},
};

pub fn match_test(context: &Context, map: &mut HashMap<String, Context>) {
    if is_test(context) {
        mark_as_test(context, map);

        let code = &context.code;

        let code_block = syn::parse_str::<Block>(&code).unwrap();
        let mut visitor = ExprVisitor::new();
        visitor.visit_block(&code_block);

        for name in visitor.function_names {
            if let Some(entry) = map.get_mut(&name) {
                let mut context_clone_no_tests = context.clone();
                context_clone_no_tests.tests.clear(); // just to make sure we have no unnecessary nesting in the resulting json

                let test = Test {
                    tests: context.code_file_content.clone(),
                    test_class_name: context.parent.last().unwrap().name.clone(),
                    test_file_path: get_relative_path(&context.root_path, &context.code_file_path),
                    test_as_context: Some(context_clone_no_tests),
                    test_type: test_type(&context).to_string(),
                    ..Default::default()
                };

                if !entry
                    .tests
                    .iter()
                    .any(|t| t.test_class_name == test.test_class_name)
                {
                    entry.tests.push(test);
                }

                /*
                println!(
                    "{} tested by {} - parent: {}",
                    entry.signature.name,
                    &context.signature.name,
                    &context.parent.last().unwrap().name
                );
                */
            }
        }
    }
}

fn mark_as_test(context: &Context, map: &mut HashMap<String, Context>) {
    if let Some(entry) = map.get_mut(&context.signature.name) {
        entry.is_test_method = true;
    }
}

fn is_test(context: &Context) -> bool {
    let fn_test_annotations = vec!["#[test]", "#[quickcheck]"];
    let is_fn_annotated = context
        .signature
        .annotations
        .iter()
        .any(|a| fn_test_annotations.contains(&a.replace(" ", "").as_str()));

    if is_fn_annotated {
        return true;
    }

    let mod_test_annotation = "#[cfg(test)]";
    let is_mod_annotated = context.parent.iter().any(|p| {
        p.attributes
            .iter()
            .any(|a| a.replace(" ", "") == mod_test_annotation)
    });

    if is_mod_annotated {
        return true;
    }

    false
}

fn test_type(context: &Context) -> &'static str {
    let file_path = Path::new(&context.code_file_path);

    if file_path.starts_with("tests/") {
        "integration_test"
    } else {
        "unit_test"
    }
}
