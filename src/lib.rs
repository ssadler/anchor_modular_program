#![warn(missing_docs)]

//! A replacement #\[program\] macro for anchor-lang that allows splitting
//! instructions into modules.
//!
//!
//! ```
//! mod extra;                 
//! use extra::types::*;       
//!                            
//! #[modular_program(         
//!     modules=[              
//!         extra::instructions
//!     ]                      
//! )]                         
//! mod my_program {           
//!     use super::*;          
//! }
//! ```



use std::{collections::HashMap, io::Read};
use anchor_syn::*;
use syn::*;
use quote::*;


/// Modules can either be a rust path to an instructions module,
/// or it can be an object:
///
/// ```
/// #[modular_program(modules=[
///     mymod::instructions,
///     {
///         // Required, module path to instructions
///         module: path::to::instructions,
///
///         // Optional path, override where to look for the instructions
///         file_path: "./src/mod/etc.rs",
///
///         // Optional prefix, empty for no prefix
///         prefix: "prefix",
///
///         // Optional, A macro that wraps the call to the instruction, eg:
///         // ```
///         // macro_rules ix_wrapper {
///         //     ($ix:path, $ctx:ident: $ctx_type:ty $(, $arg:ident: $arg_type:ty )*) => {
///         //         $ix($ctx, $(, $arg)*)
///         //     };
///         // }
///         // ```
///         macro: path::to::macro
///     }
/// ])]
/// mod my_program {           
///     use super::*;          
/// }
/// ```
///
#[proc_macro_attribute]
pub fn modular_program(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ProgramMacro { modules } = syn::parse_macro_input!(args as ProgramMacro);

    let fns = modules.into_iter()
        .map(|m| (m.clone(), get_program(m.clone())))
        .flat_map(|(spec, p)| {
            p.ixs.into_iter().map(|ix| build_relay(&spec, ix)).collect::<Vec<_>>()
        })
        .collect();

    let input = insert_fns_into_first_module(input, fns);
    let program = syn::parse_macro_input!(input as anchor_syn::Program);
    program.to_token_stream().into()
}


/*
 * This builds relay instructions, i.e, for `foo::instructions::do_thing`:
 *
 * pub fn foo_do_thing(ctx: Context<YourInstructionContext>, ...) -> Result<()> {
 *     foo::instructions::do_thing(ctx, ...)
 * }
 */

fn build_relay(spec: &ModuleSpec, ix: Ix) -> ItemFn {

    let item_fn = &ix.raw_method;
    let ItemFn { attrs, .. } = &item_fn;
    let Signature { ident: fn_name, inputs, generics, output, .. } = &item_fn.sig;

    let path = spec.module.clone();
    let first = path.segments[0].clone().ident;

    let new_name = match &spec.prefix {
        Some(s) if s.is_empty() => fn_name.clone(),
        o => {
            let prefix = o.clone().unwrap_or(first.to_string());
            Ident::new(format!("{}_{}", prefix, fn_name).as_str(), first.span())
        }
    };

    // Extract argument names for the function call
    let arg_names: Vec<Ident> = inputs
        .iter()
        .filter_map(|arg| match arg { FnArg::Typed(pt) => Some(&*pt.pat), _ => None })
        .filter_map(|pt| match pt { syn::Pat::Ident(id) => Some(id.ident.clone()), _ => None })
        .collect();

    if let Some(w) = &spec.wrapper {
        parse_quote! {
            #(#attrs)*
            pub fn #new_name #generics(#inputs) #output {
                { #w!(#path::#fn_name, #inputs) }
            }
        }
    } else {
        parse_quote! {
            #(#attrs)*
            pub fn #new_name #generics(#inputs) #output {
                #path::#fn_name(#(#arg_names),*)
            }
        }
    }
}


/*
 * Get an anchor Program from the given path, by parsing the file, i.e.
 * foo::instructions is converted to "$PROGRAM_DIR/src/foo/instructions.rs"
 */

fn get_program(spec: ModuleSpec) -> Program {

    let mod_path = format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        spec.get_file_path()
    );

    let mut code_str = String::new();
    std::fs::File::open(mod_path).unwrap().read_to_string(&mut code_str).unwrap();
    let parsed = syn::parse_file(&code_str).unwrap();

    let program_mod = ItemMod {
        vis: Visibility::Public(VisPublic { pub_token: Default::default() }),
        attrs: vec![],
        mod_token: syn::token::Mod::default(),
        ident: Ident::new("abc", proc_macro2::Span::call_site()),
        content: Some((
            syn::token::Brace::default(),
            parsed.items,
        )),
        semi: None,
    };

    let program = anchor_syn::parser::program::parse(program_mod).unwrap();
    assert!(program.fallback_fn.is_none(), "additional program module cant have fallback");
    program
}



/*
 * Parse the macro arguments
 */

#[derive(Debug)]
struct ProgramMacro { modules: Vec<ModuleSpec>, }

impl parse::Parse for ProgramMacro {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {

        // Parse `modules`
        let modules_ident: Ident = input.parse()?;
        if modules_ident != "modules" {
            return Err(syn::Error::new(modules_ident.span(), "expected `modules`"));
        }

        input.parse::<Token![=]>()?;

        // Parse the bracketed list `[cell, placement]`
        let content;
        syn::bracketed!(content in input);
        let specs = content.parse_terminated::<ModuleSpec, Token![,]>(|p| p.parse())?;

        // Convert Punctuated<Ident, _> to Vec<Ident>
        let modules = specs.into_iter().collect();

        Ok(ProgramMacro { modules })
    }
}

#[derive(Clone, Debug)]
struct ModuleSpec {
    module: Path,
    prefix: Option<String>,
    file_path: Option<String>,
    wrapper: Option<Path>
}

impl ModuleSpec {
    fn get_file_path(&self) -> String {
        self.file_path.clone().unwrap_or_else(|| {
            let p = self.module.segments.iter().fold(String::new(), |s, p| format!("{}/{}", s, p.ident));
            format!("./src{}.rs", p)
        })
    }
}

impl parse::Parse for ModuleSpec {
    fn parse(input: parse::ParseStream) -> Result<Self> {

        type T = (String, (Option<String>, Option<Path>));
        fn parse_field(p: parse::ParseStream) -> syn::Result<T> {
            let name = p.parse::<Ident>()?.to_string();
            p.parse::<Token![:]>()?;
            Ok(
                if name == "file_path" || name == "prefix" {
                    (name, (Some(p.parse::<LitStr>()?.value()), None))
                } else if name == "module" || name == "wrapper" {
                    (name, (None, Some(p.parse::<Path>()?)))
                } else {
                    panic!("Invalid module spec param: {}", name);
                }
            )
        }


        if input.peek(Ident) {
            let module = input.parse::<Path>()?;
            Ok(ModuleSpec { module, prefix: None, file_path: None, wrapper: None })
        } else {
            let content;
            syn::braced!(content in input);
            let fields = content.parse_terminated::<T, Token![,]>(parse_field)?;
            let mut hm = fields.clone().into_iter().collect::<HashMap<String, _>>();
            assert!(hm.len() == fields.len(), "duplicate field");
            Ok(ModuleSpec {
                module: hm.remove("module").expect("module is required").1.unwrap(),
                prefix: hm.remove("prefix").map(|t| t.0).flatten(),
                file_path: hm.remove("file_path").map(|t| t.0).flatten(),
                wrapper: hm.remove("wrapper").map(|t| t.1).flatten(),
            })
        }
    }
}


/*
 * Append instruction functions to main program module
 */

fn insert_fns_into_first_module(input: proc_macro::TokenStream, fns: Vec<ItemFn>) -> proc_macro::TokenStream {

    let mut item_mod: ItemMod = parse2(input.into()).expect("Failed to parse main program module");

    item_mod.content
        .as_mut()
        .expect("Program module has no body?")
        .1.extend(fns.into_iter().map(Into::into));

    quote! { #item_mod }.into()
}

