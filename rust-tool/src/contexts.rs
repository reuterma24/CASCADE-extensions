use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Context {
    #[serde(skip)]
    pub root_path: String,
    pub doc: String,
    pub signature: Signature,
    pub language: String,
    pub parent: Vec<Parent>, //sorted from outer- to inner most module/impl block
    pub code: String,
    pub code_file_path: String, // has to be relative to the root
    pub code_file_content: String,
    pub id: u32,
    pub tests: Vec<Test>,
    pub called_functions: Option<Vec<String>>, // This is not used, therefore we leave it empty
    pub use_statement_path: String, // Statement that is used to import this function(context)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_tests: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_code: Option<String>,
    #[serde(skip)]
    pub is_test_method: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct Signature {
    pub name: String,
    pub returns: String,
    pub params: Vec<String>, // also includes generic parameters
    pub modifier: String,
    pub annotations: Vec<String>,
    pub traits: String, // this will contain the traits
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Parent {
    pub name: String,
    pub doc: String,
    pub attributes: Vec<String>,
    pub other_methods: Vec<OtherMethod>,
    pub variables: Vec<String>, //This also contains structs
    pub imports: Vec<String>,
    pub parent_type: ParentType,
    pub impl_type: String,

    // The following properties are not relevant in Rust
    pub constructors: Option<Vec<String>>,
    pub extends: Option<String>,
    pub implements: Option<Vec<String>>,
    pub namespace: Option<String>,
    pub generics: Vec<String>, // altough they exist for impl blocks, they are made part of the name for simplicity (where-constraints as well)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OtherMethod {
    pub doc: String,
    pub attributes: Vec<String>,
    pub signature: Signature,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Test {
    pub tests: String, // complete file
    pub test_imports: Vec<String>,
    pub test_class_name: String, //mod name
    pub test_file_path: String,
    pub test_runner: String, // either immediate mod test or test
    pub test_as_context: Option<Context>,
    pub test_type: String,

    // The following properties are not relevant in Rust
    pub project_path: String,
    pub test_namespace: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum ParentType {
    #[default]
    Module,
    Implementation,
}
