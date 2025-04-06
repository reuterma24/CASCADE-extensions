use quote::ToTokens;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use syn::{
    punctuated::Punctuated,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Attribute, Block, Expr, ExprCall, ExprMethodCall, ImplItemFn, Item, ItemFn, ItemImpl, ItemMod,
    Macro, Signature, Token, Visibility,
};

const RUST_EXTENSION: &str = "rs";
const JSON_FILE_PATH: &str = "extracted.json";

pub fn is_rust_file(path: &PathBuf) -> bool {
    path.extension().map_or(false, |ext| ext == RUST_EXTENSION)
}

pub fn save_json<T: Serialize>(out_dir: &str, data: &T) {
    let json = serde_json::to_string_pretty(&data).unwrap();

    let path = Path::new(out_dir).join(JSON_FILE_PATH);
    let mut file = File::create(path).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn get_relative_path(root: &String, file_path: &String) -> String {
    let root_path = Path::new(root);
    let code_file_path = Path::new(file_path);

    // Trim the absolute path to a relative path
    let relative_code_file_path = code_file_path
        .strip_prefix(root_path)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| code_file_path.to_path_buf());

    relative_code_file_path.to_string_lossy().into_owned()
}

// custom trait to dynamically dispatch ItemFn and ImplItemFn later
pub trait FnLike {
    fn get_attrs(&self) -> &Vec<Attribute>;
    fn get_sig(&self) -> &Signature;
    fn get_vis(&self) -> &Visibility;
    fn get_block(&self) -> &Block;
}

impl FnLike for ItemFn {
    fn get_attrs(&self) -> &Vec<Attribute> {
        &self.attrs
    }

    fn get_sig(&self) -> &Signature {
        &self.sig
    }

    fn get_vis(&self) -> &Visibility {
        &self.vis
    }

    fn get_block(&self) -> &Block {
        &self.block
    }
}

impl FnLike for ImplItemFn {
    fn get_attrs(&self) -> &Vec<Attribute> {
        &self.attrs
    }

    fn get_sig(&self) -> &Signature {
        &self.sig
    }

    fn get_vis(&self) -> &Visibility {
        &self.vis
    }

    fn get_block(&self) -> &Block {
        &self.block
    }
}

pub struct Function<'ast> {
    pub function: Box<&'ast dyn FnLike>,
    pub context: Vec<Item>, // holds all the module and Impl items that are wrapping the function
}

pub struct FnVisitor<'ast> {
    pub functions: Vec<Function<'ast>>,
    context_stack: Vec<Item>,
}

impl<'ast> FnVisitor<'ast> {
    pub fn new() -> FnVisitor<'ast> {
        FnVisitor {
            functions: Vec::<Function>::new(),
            context_stack: Vec::<Item>::new(),
        }
    }

    fn add_fn(&mut self, node: &'ast dyn FnLike) {
        // construct Function struct from node and current context stack
        self.functions.push(Function {
            function: Box::new(node),
            context: self.context_stack.clone(),
        });
    }
}

impl<'ast> Visit<'ast> for FnVisitor<'ast> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.add_fn(node);

        // Delegate to the default impl to visit any nested functions
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.add_fn(node);

        visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.context_stack.push(Item::Mod(node.clone()));

        if let Some((_, items)) = &node.content {
            for item in items {
                // Delegate to default impl to visit any nested items
                self.visit_item(item);
            }
        }

        self.context_stack.pop();
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        self.context_stack.push(Item::Impl(node.clone()));

        for item in &node.items {
            // Delegate to default impl to visit any nested items
            self.visit_impl_item(item);
        }

        self.context_stack.pop();
    }
}

pub struct ExprVisitor {
    pub function_names: Vec<String>,
}

impl ExprVisitor {
    pub fn new() -> ExprVisitor {
        ExprVisitor {
            function_names: Vec::<String>::new(),
        }
    }

    fn add_expr_method_call(&mut self, node: &ExprMethodCall) {
        self.function_names
            .push(node.method.to_token_stream().to_string());
    }

    fn add_expr_call(&mut self, node: &ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            if let Some(last_segment) = expr_path.path.segments.last() {
                self.function_names.push(last_segment.ident.to_string());
            }
        } else {
            self.function_names
                .push(node.func.to_token_stream().to_string());
        }
    }

    fn parse_macro_input(&mut self, mac: &Macro) {
        let parser = Punctuated::<Expr, Token![,]>::parse_terminated;

        if let Ok(macro_arguments) = mac.parse_body_with(parser) {
            for expr in macro_arguments {
                self.visit_expr(&expr);
            }
        }
    }
}

impl<'ast> Visit<'ast> for ExprVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.add_expr_method_call(&node);

        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        self.add_expr_call(node);

        visit::visit_expr_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.parse_macro_input(node);

        visit::visit_macro(self, node);
    }
}

pub trait ToImplFullName {
    fn full_name(&self) -> String;
}

impl ToImplFullName for ItemImpl {
    fn full_name(&self) -> String {
        let impl_ = self.impl_token.to_token_stream().to_string();
        let trait_ = self.trait_.as_ref().map(|t| {
            let excl = t.0.as_ref().map(|_| "!").unwrap_or_default();
            let path = t.1.to_token_stream().to_string();
            let for_ = t.2.to_token_stream().to_string();
            format!("{}{} {}", excl, path, for_)
        });
        let generics = self.generics.to_token_stream().to_string();
        let where_clause = self
            .generics
            .where_clause
            .as_ref()
            .map(|w| w.to_token_stream().to_string())
            .unwrap_or_default();

        let self_type = self.self_ty.to_token_stream().to_string();

        format!(
            "{} {} {} {} {}",
            impl_,
            generics,
            trait_.unwrap_or_default(),
            self_type,
            where_clause
        )
    }
}

pub struct BodyReplace {
    new_body: Block,
    function_name: String,
    pub successful: bool,
}

impl BodyReplace {
    pub fn new(new_body: Block, function_name: String) -> BodyReplace {
        BodyReplace {
            new_body,
            function_name,
            successful: false,
        }
    }

    pub fn name_matches(&self, name: &String) -> bool {
        self.function_name == *name
    }
}

impl VisitMut for BodyReplace {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        if self.name_matches(&node.sig.ident.to_string()) {
            node.block = Box::new(self.new_body.clone());

            self.successful = true;
            return;
        }

        // Delegate to the default impl to visit any nested functions
        visit_mut::visit_item_fn_mut(self, node);
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        if self.name_matches(&node.sig.ident.to_string()) {
            node.block = self.new_body.clone();

            self.successful = true;
            return;
        }

        // Delegate to the default impl to visit any nested functions
        visit_mut::visit_impl_item_fn_mut(self, node);
    }
}

pub fn inject_test_module_after_function(source: &Path, function_name: &str, module_code: &str) {
    let file_content = fs::read_to_string(source).unwrap();
    let mut syntax_tree = syn::parse_file(&file_content).unwrap();

    let new_test_module: Item = syn::parse_str(module_code).unwrap();

    // Find the index of the function
    if let Some(index) = syntax_tree
        .items
        .iter()
        .position(|item| matches!(item, Item::Fn(func) if func.sig.ident == function_name))
    {
        syntax_tree.items.insert(index + 1, new_test_module); // Insert after the function
    }

    let formatted_code = prettyplease::unparse(&syntax_tree);
    fs::write(source, formatted_code).unwrap();
}

pub fn inject_test_module_after_impl(source: &Path, impl_name: &str, module_code: &str) {
    let file_content = fs::read_to_string(source).unwrap();
    let mut syntax_tree = syn::parse_file(&file_content).unwrap();

    let new_test_module: Item = syn::parse_str(module_code).unwrap();

    // Find the index of the function
    if let Some(index) = syntax_tree.items.iter().position(
        |item| matches!(item, Item::Impl(impl_block) if compare_ignore_whitespace(&impl_block.full_name(), impl_name)),
    ) {
        syntax_tree.items.insert(index + 1, new_test_module); // Insert after the impl block
    }

    let formatted_code = prettyplease::unparse(&syntax_tree);
    fs::write(source, formatted_code).unwrap();
}

fn compare_ignore_whitespace(s1: &str, s2: &str) -> bool {
    s1.chars()
        .filter(|c| !c.is_whitespace())
        .eq(s2.chars().filter(|c| !c.is_whitespace()))
}
