use crate::utils::{get_relative_path, FnLike, FnVisitor, Function, ToImplFullName};
use quote::ToTokens;
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};
use syn::{token::Brace, visit::Visit, AttrStyle, Attribute, Block, ImplItem, Item};
use syn::{Type, TypePath};

use crate::contexts::*;

static COUNTER: AtomicUsize = AtomicUsize::new(1);
const LANGUAGE: &str = "Rust";

fn get_and_increment_counter() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn analyze_file(project_path: &String, file_path: &Path, contexts: &mut Vec<Context>) {
    let content = fs::read_to_string(file_path).unwrap();
    let syntax_tree = syn::parse_file(&content).unwrap();
    let root_module = extract_root_module(&syntax_tree);

    // find all functions that are inside the file
    let mut visitor = FnVisitor::new();
    visitor.visit_file(&syntax_tree);

    // iterate through each function
    for function in visitor.functions {
        let doc = extract_documentation(&function.function.get_attrs());
        let signature = extract_signature(*function.function);
        let code = extract_code(&function.function.get_block());
        let parents = extract_parents(&function, &root_module);
        let relative_file_path = get_relative_path(
            project_path,
            &file_path.as_os_str().to_str().unwrap().to_string(),
        );

        let use_statement = construct_use_statement(project_path, &parents, &signature.name);

        contexts.push(Context {
            root_path: project_path.to_string(),
            id: get_and_increment_counter() as u32,
            doc,
            signature,
            language: LANGUAGE.to_string(),
            parent: parents,
            code,
            code_file_path: relative_file_path,
            code_file_content: content.clone(),
            use_statement_path: use_statement,
            ..Default::default()
        });

        //print(documentation, signature, code);
        //print_parents(&parents);
    }
}

fn extract_root_module(syntax_tree: &syn::File) -> Parent {
    let doc = extract_root_documentation(&syntax_tree.attrs);
    let use_statements = extract_use_statements(&syntax_tree.items);
    let variables = extract_variables(&syntax_tree.items);
    let attributes = extract_root_attributes(&syntax_tree.attrs);
    let other_methods = extract_other_root_methods(&syntax_tree.items);

    Parent {
        name: "root module".to_string(),
        doc,
        variables,
        imports: use_statements,
        attributes: attributes,
        other_methods: other_methods, // contains all methods, if added as parent remove the fn itself
        parent_type: ParentType::Module,
        ..Default::default()
    }
}

fn extract_parents(func: &Function, root_module: &Parent) -> Vec<Parent> {
    let mut parents: Vec<Parent> = vec![];

    // add root module first
    let mut root_module = root_module.clone();
    root_module
        .other_methods
        .retain(|f| f.signature.name != func.function.get_sig().ident.to_string());
    parents.push(root_module);

    // add remaning parents
    for context in &func.context {
        let parent = match context {
            Item::Mod(val) => Parent {
                name: val.ident.to_string(),
                doc: extract_documentation(&val.attrs),
                attributes: extract_attributes(&val.attrs),
                other_methods: extract_other_methods(&val.content, &func.function.get_sig()),
                variables: extract_variables(
                    val.content
                        .as_ref()
                        .map(|(_, items)| items)
                        .unwrap_or(&vec![]),
                ),
                imports: extract_use_statements(
                    val.content
                        .as_ref()
                        .map(|(_, items)| items)
                        .unwrap_or(&vec![]),
                ),
                parent_type: ParentType::Module,
                ..Default::default()
            },
            Item::Impl(val) => Parent {
                name: val.full_name(),
                doc: extract_documentation(&val.attrs),
                attributes: extract_attributes(&val.attrs),
                other_methods: extract_other_methods_impl(&val.items, &func.function.get_sig()),
                parent_type: ParentType::Implementation,
                impl_type: get_type_name_without_generics(&val.self_ty),
                ..Default::default()
            },
            _ => panic!("Unexpected item type for parent. This should not happen."),
        };

        parents.push(parent);
    }

    parents
}

fn print(documentation: String, signature: Signature, code: String) {
    let signature_formatted = format!(
        "\n{}\n{}\n{} {}({}) {} {}\n {}",
        documentation,
        signature.annotations.join("\n"),
        signature.modifier,
        signature.name,
        signature.params.join(", "),
        signature.returns,
        signature.traits,
        code
    );

    println!("{}\n-----", signature_formatted);
}

fn print_parents(parents: &Vec<Parent>) {
    let parents_formatted = parents
        .iter()
        .map(|parent| {
            format!(
                "{:?}:\n{}\n{}\n{}\n{}\n{} \n\n",
                parent.parent_type,
                parent.imports.join("\n"),
                parent.variables.join("\n"),
                parent.doc,
                parent.attributes.join("\n"),
                parent.name
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    println!(
        "{}---------------------------------------\n",
        parents_formatted
    )
}

fn extract_code(block: &Block) -> String {
    block.to_token_stream().to_string()
}

fn extract_documentation(attributes: &Vec<Attribute>) -> String {
    let documentation: String = attributes
        .iter()
        .filter_map(|attr| {
            if let AttrStyle::Outer = attr.style {
                if attr.meta.path().is_ident("doc") {
                    return Some(attr.to_token_stream().to_string());
                }
            }
            None
        })
        .collect::<Vec<String>>()
        .join("\n");
    documentation
}

fn extract_root_documentation(attributes: &Vec<Attribute>) -> String {
    let documentation: String = attributes
        .iter()
        .filter_map(|attr| {
            if let AttrStyle::Inner(_) = attr.style {
                if attr.meta.path().is_ident("doc") {
                    return Some(attr.to_token_stream().to_string());
                }
            }
            None
        })
        .collect::<Vec<String>>()
        .join("\n");
    documentation
}

fn extract_attributes(attributes: &Vec<Attribute>) -> Vec<String> {
    let attributes: Vec<String> = attributes
        .iter()
        .filter_map(|attr| {
            if let AttrStyle::Outer = attr.style {
                if !attr.meta.path().is_ident("doc") {
                    return Some(attr.to_token_stream().to_string());
                }
            }
            None
        })
        .collect();
    attributes
}

fn extract_root_attributes(attributes: &Vec<Attribute>) -> Vec<String> {
    let attributes: Vec<String> = attributes
        .iter()
        .filter_map(|attr| {
            if let AttrStyle::Inner(_) = attr.style {
                if !attr.meta.path().is_ident("doc") {
                    return Some(attr.to_token_stream().to_string());
                }
            }
            None
        })
        .collect();
    attributes
}

fn extract_signature(function: &dyn FnLike) -> Signature {
    let sig = function.get_sig();
    let vis = function.get_vis();
    let attrs = function.get_attrs();

    let attributes = extract_attributes(attrs);

    let visibility = vis.to_token_stream().to_string();

    let name = sig.ident.to_string();

    let params: Vec<String> = sig
        .inputs
        .iter()
        .map(|arg| arg.into_token_stream().to_string())
        .collect();

    let traits = match &sig.generics.where_clause {
        Some(where_clause) => where_clause.into_token_stream().to_string(),
        None => "".to_string(),
    };

    let returns = sig.output.to_token_stream().to_string();

    Signature {
        name,
        returns,
        params,
        traits,
        modifier: visibility,
        annotations: attributes,
    }
}

fn extract_other_methods(
    content: &Option<(Brace, Vec<Item>)>,
    sig: &syn::Signature,
) -> Vec<OtherMethod> {
    let mut other_methods: Vec<OtherMethod> = Vec::new();

    if let Some((_, items)) = content {
        for item in items {
            if let Item::Fn(item_fn) = item {
                if item_fn.sig.ident == sig.ident {
                    continue; // Skip the iteration step if its the current function
                }

                let other_method = OtherMethod {
                    doc: extract_documentation(&item_fn.attrs),
                    attributes: extract_attributes(&item_fn.attrs),
                    signature: extract_signature(item_fn),
                    code: extract_code(&item_fn.get_block()),
                };

                other_methods.push(other_method);
            }
        }
    }

    other_methods
}

fn extract_other_root_methods(items: &Vec<Item>) -> Vec<OtherMethod> {
    let mut other_methods: Vec<OtherMethod> = Vec::new();

    for item in items {
        if let Item::Fn(item_fn) = item {
            let other_method = OtherMethod {
                doc: extract_documentation(&item_fn.attrs),
                attributes: extract_attributes(&item_fn.attrs),
                signature: extract_signature(item_fn),
                code: extract_code(&item_fn.get_block()),
            };

            other_methods.push(other_method);
        }
    }

    other_methods
}

fn extract_other_methods_impl(items: &Vec<ImplItem>, sig: &syn::Signature) -> Vec<OtherMethod> {
    let mut other_methods: Vec<OtherMethod> = Vec::new();

    for item in items {
        if let ImplItem::Fn(item_fn) = item {
            if item_fn.sig.ident == sig.ident {
                continue; // Skip the iteration step if its the current function
            }

            let other_method = OtherMethod {
                doc: extract_documentation(&item_fn.attrs),
                attributes: extract_attributes(&item_fn.attrs),
                signature: extract_signature(item_fn),
                code: extract_code(&item_fn.get_block()),
            };

            other_methods.push(other_method);
        }
    }

    other_methods
}

fn extract_use_statements(content: &Vec<Item>) -> Vec<String> {
    let mut use_statements: Vec<String> = Vec::new();

    for item in content {
        if let Item::ExternCrate(item_extern_crate) = item {
            let extern_crate = item_extern_crate.into_token_stream().to_string();

            use_statements.insert(0, extern_crate);
        }

        if let Item::Use(item_use) = item {
            let use_statement = item_use.into_token_stream().to_string();

            use_statements.push(use_statement);
        }
    }

    use_statements
}

fn extract_variables(content: &Vec<Item>) -> Vec<String> {
    let mut variables: Vec<String> = Vec::new();

    for item in content {
        match item {
            Item::Const(item_const) => {
                variables.push(item_const.into_token_stream().to_string());
            }
            Item::Static(item_static) => {
                variables.push(item_static.into_token_stream().to_string());
            }
            Item::Struct(item_struct) => {
                variables.push(item_struct.to_token_stream().to_string());
            }
            _ => {}
        }
    }

    variables
}

fn construct_use_statement(
    project_path: &str,
    parents: &Vec<Parent>,
    function_name: &str,
) -> String {
    let mut path_parts = Vec::new();
    let crate_name = Path::new(project_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("crate");
    path_parts.push(crate_name);

    // Skip "root module" and collect the module path
    for parent in parents.iter().skip(1) {
        match parent.parent_type {
            ParentType::Module => path_parts.push(&parent.name),
            ParentType::Implementation => {
                path_parts.push(&parent.impl_type);
            }
        }
    }

    // Do not include function name, for statements that refer to a struct of an impl block
    if !parents.is_empty() && parents.last().unwrap().parent_type != ParentType::Implementation {
        path_parts.push(function_name);
    }

    //println!("use {};", path_parts.join("::"));
    let path = format!("use {};", path_parts.join("::"));

    path.replace('-', "_")
}

fn get_type_name_without_generics(self_ty: &Type) -> String {
    match self_ty {
        Type::Path(TypePath { path, .. }) => {
            let segments: Vec<String> = path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string()) // Get the type name without generics
                .collect();
            segments.join("::")
        }
        _ => self_ty.to_token_stream().to_string(), // as default case
    }
}
