use crate::code_extractor::analyze_file;
use crate::contexts::*;
use crate::test_extractor::match_test;
use crate::utils::{is_rust_file, save_json};
use std::{collections::HashMap, path::PathBuf};
use walkdir::WalkDir;

pub fn extraction(args: &[String]) {
    if args.len() < 2 {
        invalid_invocation_error();
    }

    extract(args[0].clone(), args[1].clone());
}

fn extract(project_path: String, output_path: String) {
    let rust_files: Vec<PathBuf> = WalkDir::new(&project_path)
        .into_iter()
        .filter_map(|r| r.ok().map(|e| e.into_path())) //convert into path
        .filter(is_rust_file)
        .collect();

    let mut contexts: Vec<Context> = vec![];

    // extract every function in the project
    for entry in rust_files {
        analyze_file(&project_path, entry.as_path(), &mut contexts);
    }

    //TODO: this ignores name clashes and overwrites the entry
    let mut context_map: HashMap<String, Context> = contexts
        .clone()
        .into_iter()
        .map(|context| (context.signature.name.clone(), context))
        .collect();

    for context in contexts {
        match_test(&context, &mut context_map);
    }

    save_json(
        &output_path,
        &context_map
            .into_iter()
            .map(|(_, v)| v)
            .filter(|c| !c.is_test_method)
            .collect::<Vec<Context>>(),
    );
}

fn invalid_invocation_error() {
    eprintln!("Usage: rust-tool extract <pathToProject> <outPath>");
    std::process::exit(-1);
}
