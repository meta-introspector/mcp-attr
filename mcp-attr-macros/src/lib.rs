#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use structmeta::{NameArgs, NameValue, StructMeta};
use syn::{
    Attribute, Error, FnArg, Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, LitStr, Pat, Path,
    Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned, parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};
use uri_template_ex::UriTemplate;

use syn_utils::{get_element, into_macro_output, is_path, is_type};
use utils::{get_trait_path, is_defined};

use crate::prompts::{PromptAttr, PromptEntry};
use crate::resources::{ResourceAttr, ResourceEntry};
use crate::tools::{ToolAttr, ToolEntry};
use crate::utils::{build_if, drain_attr, get_doc};

#[macro_use]
mod syn_utils;
mod utils;

mod prompts;
mod resources;
mod tools;

#[proc_macro_attribute]
pub fn mcp_server(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item: TokenStream = item.into();
    let mut es = Vec::new();
    match build_mcp_server(attr.into(), item.clone(), &mut es) {
        Ok(mut s) => {
            for e in es {
                s.extend(e.to_compile_error());
            }
            s
        }
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro]
pub fn route(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_macro_output(build_route(item.into()))
}

#[proc_macro_attribute]
pub fn tool(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    into_macro_output(build_tool(attr.into(), item.into()))
}

#[proc_macro_attribute]
pub fn resource(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    into_macro_output(build_resource(attr.into(), item.into()))
}

#[proc_macro_attribute]
pub fn prompt(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    into_macro_output(build_prompt(attr.into(), item.into()))
}

#[proc_macro_attribute]
pub fn complete_fn(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    into_macro_output(build_complete_fn(attr.into(), item.into()))
}

fn build_mcp_server(
    attr: TokenStream,
    item: TokenStream,
    es: &mut Vec<Error>,
) -> Result<TokenStream> {
    let mut item_impl: ItemImpl = parse2(item)?;
    let mut attr: McpAttr = parse2(attr)?;
    let trait_path = get_trait_path(&item_impl)?.clone();
    if item_impl.unsafety.is_some() {
        bail!(item_impl.span(), "Unsafe is not allowed");
    }
    if item_impl.defaultness.is_some() {
        bail!(item_impl.span(), "Default is not allowed");
    }
    let is_defined_resources_list = is_defined(&item_impl.items, "resources_list");
    let impl_doc = get_doc(&item_impl.attrs);
    let mut b = McpBuilder::new();
    let mut items_trait = Vec::new();
    let mut items_type = Vec::new();
    for mut item in item_impl.items {
        match b.push(&mut item) {
            Ok(ItemPlacement::Type) => items_type.push(item),
            Ok(ItemPlacement::Trait) => items_trait.push(item),
            Ok(ItemPlacement::Exclude) => {}
            Err(e) => {
                items_type.push(item);
                es.push(e);
            }
        }
    }
    let (b, complete_fns) = b.build(&items_trait, &impl_doc)?;
    let (impl_generics, ty_generics, where_clause) = item_impl.generics.split_for_impl();

    let self_ty = &item_impl.self_ty;
    let attrs = &item_impl.attrs;
    let ts = quote! {
        #[automatically_derived]
        #(#attrs)*
        impl<#impl_generics> #trait_path for #self_ty #ty_generics #where_clause {
            #(#items_trait)*
            #b
        }

        #[automatically_derived]
        #(#attrs)*
        impl<#impl_generics> #self_ty #ty_generics #where_clause {
            #(#items_type)*
            #(#complete_fns)*
        }
    };
    if attr.dump {
        dump_code(ts);
    }
    Ok(ts)
}

#[derive(Debug)]
enum ItemPlacement {
    /// Place item in the type impl block (items_type)
    Type,
    /// Place item in the trait impl block (items_trait)
    Trait,
    /// Exclude item completely
    Exclude,
}

struct McpBuilder {
    prompts: Vec<PromptEntry>,
    resources: Vec<ResourceEntry>,
    tools: Vec<ToolEntry>,
    complete_fns: Vec<ImplItemFn>,
}

impl McpBuilder {
    fn new() -> Self {
        Self {
            prompts: Vec::new(),
            resources: Vec::new(),
            tools: Vec::new(),
            complete_fns: Vec::new(),
        }
    }
    fn push(&mut self, item: &mut ImplItem) -> Result<ItemPlacement> {
        if let ImplItem::Fn(f) = item {
            let Some(attr) = drain_attr(&mut f.attrs)? else {
                return Ok(ItemPlacement::Trait);
            };
            match attr {
                ItemAttr::Prompt(attr) => {
                    self.prompts.push(PromptEntry::from_impl_item_fn(f, attr)?)
                }
                ItemAttr::Resource(attr) => self
                    .resources
                    .push(ResourceEntry::from_impl_item_fn(f, attr)?),
                ItemAttr::Tool(attr) => self.tools.push(ToolEntry::from_impl_item_fn(f, attr)?),
                ItemAttr::CompleteFn => {
                    // Transform the complete_fn in-place and get inner function
                    let inner_fn = apply_complete_fn_transformation(f)?;
                    self.complete_fns.push(inner_fn);
                    // The original function (f) is now transformed to wrapper
                    return Ok(ItemPlacement::Type);
                }
            }
            return Ok(ItemPlacement::Type);
        }
        Ok(ItemPlacement::Trait)
    }

    fn build(
        &mut self,
        items: &[ImplItem],
        impl_doc: &str,
    ) -> Result<(TokenStream, &Vec<ImplItemFn>)> {
        let capabilities = build_if(!is_defined(items, "capabilities"), || {
            self.build_capabilities(items)
        })?;
        let instructions = build_if(!is_defined(items, "instructions"), || {
            self.build_instructions(impl_doc)
        })?;
        let prompts = build_if(!self.prompts.is_empty(), || self.build_prompts())?;
        let resources = build_if(!self.resources.is_empty(), || self.build_resources(items))?;
        let tools = build_if(!self.tools.is_empty(), || self.build_tools())?;

        let completion_complete = build_if(!is_defined(items, "completion_complete"), || {
            self.build_completion_complete()
        })?;
        Ok((
            quote! {
                #capabilities
                #instructions
                #prompts
                #resources
                #tools
                #completion_complete
            },
            &self.complete_fns,
        ))
    }
    fn build_capabilities(&self, items: &[ImplItem]) -> Result<TokenStream> {
        let prompts = if !self.prompts.is_empty() || is_defined(items, "prompts_list") {
            quote!(Some(::mcp_attr::schema::ServerCapabilitiesPrompts {
                ..::std::default::Default::default()
            }))
        } else {
            quote!(None)
        };
        let resources = if !self.resources.is_empty() || is_defined(items, "resources_read") {
            quote!(Some(::mcp_attr::schema::ServerCapabilitiesResources {
                ..::std::default::Default::default()
            }))
        } else {
            quote!(None)
        };
        let tools = if !self.tools.is_empty() || is_defined(items, "tools_list") {
            quote!(Some(::mcp_attr::schema::ServerCapabilitiesTools {
                ..::std::default::Default::default()
            }))
        } else {
            quote!(None)
        };
        Ok(quote! {
            fn capabilities(&self) -> ::mcp_attr::schema::ServerCapabilities {
                ::mcp_attr::schema::ServerCapabilities {
                    prompts: #prompts,
                    resources: #resources,
                    tools: #tools,
                    ..::std::default::Default::default()
                }
            }
        })
    }
    fn build_prompts(&self) -> Result<TokenStream> {
        let list = self.build_prompts_list()?;
        let get = self.build_prompts_get()?;
        Ok(quote! {
            #list
            #get
        })
    }
    fn build_resources(&self, items: &[ImplItem]) -> Result<TokenStream> {
        let list = build_if(!is_defined(items, "resources_list"), || {
            self.build_resources_list()
        })?;
        let templates_list = self.build_resources_templates_list()?;
        let read = self.build_resources_read()?;
        Ok(quote! {
            #list
            #templates_list
            #read
        })
    }
    fn build_tools(&self) -> Result<TokenStream> {
        let list = self.build_tools_list()?;
        let call = self.build_tools_call()?;
        Ok(quote! {
            #list
            #call
        })
    }
    fn build_prompts_list(&self) -> Result<TokenStream> {
        PromptEntry::build_list(&self.prompts)
    }
    fn build_prompts_get(&self) -> Result<TokenStream> {
        PromptEntry::build_get(&self.prompts)
    }
    fn build_resources_list(&self) -> Result<TokenStream> {
        ResourceEntry::build_list(&self.resources)
    }
    fn build_resources_templates_list(&self) -> Result<TokenStream> {
        ResourceEntry::build_templates_list(&self.resources)
    }
    fn build_resources_read(&self) -> Result<TokenStream> {
        ResourceEntry::build_read(&self.resources)
    }

    fn build_tools_list(&self) -> Result<TokenStream> {
        ToolEntry::build_list(&self.tools)
    }
    fn build_tools_call(&self) -> Result<TokenStream> {
        ToolEntry::build_call(&self.tools)
    }

    fn build_instructions(&self, impl_doc: &str) -> Result<TokenStream> {
        if impl_doc.is_empty() {
            Ok(quote! {
                fn instructions(&self) -> Option<String> {
                    None
                }
            })
        } else {
            Ok(quote! {
                fn instructions(&self) -> Option<String> {
                    Some(#impl_doc.into())
                }
            })
        }
    }

    fn build_completion_complete(&self) -> Result<TokenStream> {
        let mut prompt_completions = Vec::new();
        let mut resource_completions = Vec::new();

        // Collect completion info from prompts
        for prompt in &self.prompts {
            for (prompt_name, arg_name, complete_expr) in prompt.get_completion_info() {
                prompt_completions.push((prompt_name, arg_name, complete_expr));
            }
        }

        // Collect completion info from resources
        for resource in &self.resources {
            for (uri, arg_name, complete_expr) in resource.get_completion_info() {
                resource_completions.push((uri, arg_name, complete_expr));
            }
        }

        // If no completions are defined, return empty method
        if prompt_completions.is_empty() && resource_completions.is_empty() {
            return Ok(quote! {});
        }

        // Generate prompt completion arms
        let prompt_arms: Vec<TokenStream> = prompt_completions
            .iter()
            .map(|(prompt_name, arg_name, complete_expr)| {
                let call_expr = self.generate_completion_call(complete_expr);
                let span = match complete_expr {
                    CompleteFuncExpr::Expr(expr) => expr.span(),
                    CompleteFuncExpr::InstanceMethod(ident) => ident.span(),
                };
                quote_spanned! {span=>
                    (#prompt_name, #arg_name) => {
                        #call_expr
                    },
                }
            })
            .collect();

        // Generate resource completion arms
        let resource_arms: Vec<TokenStream> = resource_completions
            .iter()
            .map(|(uri, arg_name, complete_expr)| {
                let call_expr = self.generate_completion_call(complete_expr);
                let span = match complete_expr {
                    CompleteFuncExpr::Expr(expr) => expr.span(),
                    CompleteFuncExpr::InstanceMethod(ident) => ident.span(),
                };
                quote_spanned! {span=>
                    (#uri, #arg_name) => {
                        #call_expr
                    },
                }
            })
            .collect();

        Ok(quote! {
            async fn completion_complete(
                &self,
                p: ::mcp_attr::schema::CompleteRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext,
            ) -> ::mcp_attr::Result<::mcp_attr::schema::CompleteResult> {
                match &p.ref_ {
                    ::mcp_attr::schema::CompleteRequestParamsRef::PromptReference(prompt_ref) => {
                        match (prompt_ref.name.as_str(), p.argument.name.as_str()) {
                            #(#prompt_arms)*
                            _ => Ok(::mcp_attr::schema::CompleteResult::default()),
                        }
                    },
                    ::mcp_attr::schema::CompleteRequestParamsRef::ResourceTemplateReference(resource_ref) => {
                        match (resource_ref.uri.as_str(), p.argument.name.as_str()) {
                            #(#resource_arms)*
                            _ => Ok(::mcp_attr::schema::CompleteResult::default()),
                        }
                    }
                }
            }
        })
    }

    fn generate_completion_call(&self, complete_func_expr: &CompleteFuncExpr) -> TokenStream {
        match complete_func_expr {
            CompleteFuncExpr::Expr(expr) => {
                // Call as a standalone function (no special handling for Self::)
                let span = expr.span();
                quote_spanned! {span=>
                    #expr(&p, cx).await
                }
            }
            CompleteFuncExpr::InstanceMethod(method_name) => {
                // Call as instance method with correct parameters for wrapper function
                let span = method_name.span();
                quote_spanned! {span=>
                    self.#method_name(&p, cx).await
                }
            }
        }
    }
}

#[derive(StructMeta, Default)]
struct McpAttr {
    dump: bool,
}

#[derive(Debug, Clone)]
enum CompleteFuncExpr {
    Expr(syn::Expr),
    InstanceMethod(syn::Ident),
}

impl Parse for CompleteFuncExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let method_name = input.parse::<syn::Ident>()?;
            Ok(CompleteFuncExpr::InstanceMethod(method_name))
        } else {
            let expr = input.parse::<syn::Expr>()?;
            Ok(CompleteFuncExpr::Expr(expr))
        }
    }
}

#[derive(StructMeta)]
struct CompleteAttr {
    #[struct_meta(unnamed)]
    func: CompleteFuncExpr,
}

enum ItemAttr {
    Prompt(PromptAttr),
    Resource(ResourceAttr),
    Tool(ToolAttr),
    CompleteFn,
}

fn apply_complete_fn_transformation(impl_fn: &mut ImplItemFn) -> Result<ImplItemFn> {
    let original_ident = &impl_fn.sig.ident;
    let inner_ident = format_ident!("{}_inner", original_ident);

    // Create the inner function with original implementation
    let inner_fn = ImplItemFn {
        sig: syn::Signature {
            ident: inner_ident.clone(),
            ..impl_fn.sig.clone()
        },
        block: impl_fn.block.clone(),
        attrs: impl_fn.attrs.clone(),
        vis: impl_fn.vis.clone(),
        defaultness: impl_fn.defaultness,
    };

    // Transform the original function to wrapper with new signature (self allowed in mcp_server context)
    let new_sig = build_complete_fn_signature(&impl_fn.sig, true)?;
    let call_expr = build_complete_fn_body(&impl_fn.sig, &inner_ident)?;

    impl_fn.sig = new_sig;
    impl_fn.block = parse_quote! {
        {
            #call_expr
        }
    };

    // Return inner function to be added to complete_fns
    Ok(inner_fn)
}

fn build_route(item: TokenStream) -> Result<TokenStream> {
    struct PathList(Punctuated<Path, Token![,]>);
    impl Parse for PathList {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self(Punctuated::parse_terminated(input)?))
        }
    }
    let mut path_list: PathList = parse2(item)?;
    let mut exprs = Vec::new();
    for mut path in path_list.0 {
        let last = path.segments.last_mut().unwrap();
        let fn_ident = last.ident.clone();
        last.ident = route_ident(&fn_ident);
        last.ident.set_span(Span::call_site());
        exprs.push(quote! {
            {
                let _dummy = #fn_ident; // Ensure rust-analyzer can rename the function.
                ::std::convert::Into::<::mcp_attr::server::builder::Route>::into(#path()?)
            }
        });
    }
    Ok(quote! {
        [#(#exprs),*]
    })
}
fn route_ident(ident: &Ident) -> Ident {
    format_ident!("__route_of_{}", ident)
}
fn build_tool(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut f: ItemFn = parse2(item)?;
    let attr: ToolAttr = parse2(attr)?;
    let dump = attr.dump;
    let e = ToolEntry::from_item_fn(&mut f, attr)?;
    let ret = e.build_route();
    Ok(make_extend(f, ret, dump))
}

fn build_resource(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut f: ItemFn = parse2(item)?;
    let attr: ResourceAttr = parse2(attr)?;
    let dump = attr.dump;
    let e = ResourceEntry::from_item_fn(&mut f, attr)?;
    let ret = e.build_route();
    Ok(make_extend(f, ret, dump))
}

fn build_prompt(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut f: ItemFn = parse2(item)?;
    let attr: PromptAttr = parse2(attr)?;
    let dump = attr.dump;
    let e = PromptEntry::from_item_fn(&mut f, attr)?;
    let ret = e.build_route();
    Ok(make_extend(f, ret, dump))
}

fn build_complete_fn(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        bail!(
            attr.span(),
            "complete_fn attribute does not accept parameters"
        );
    }

    let original_fn: ItemFn = parse2(item)?;
    let original_ident = &original_fn.sig.ident;
    let inner_ident = format_ident!("{}_inner", original_ident);

    // Validate the original function signature (self not allowed for global complete_fn)
    validate_complete_fn_signature(&original_fn.sig, false)?;

    let mut inner_fn = original_fn.clone();
    inner_fn.sig.ident = inner_ident.clone();

    let vis = &original_fn.vis;
    let new_sig = build_complete_fn_signature(&original_fn.sig, false)?;
    let call_expr = build_complete_fn_body(&original_fn.sig, &inner_ident)?;

    Ok(quote! {
        #inner_fn

        #vis #new_sig {
            #call_expr
        }
    })
}

fn build_complete_fn_signature(
    original_sig: &syn::Signature,
    allow_self: bool,
) -> Result<syn::Signature> {
    // Validate the original signature first
    validate_complete_fn_signature(original_sig, allow_self)?;

    let has_self = original_sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, FnArg::Receiver(_)));

    let mut new_sig = original_sig.clone();
    let sig_span = original_sig.ident.span();
    new_sig.inputs.clear();

    if has_self {
        new_sig
            .inputs
            .push(parse_quote_spanned! { sig_span=> &self });
    }

    new_sig
        .inputs
        .push(parse_quote_spanned! { sig_span=> p: &::mcp_attr::schema::CompleteRequestParams });

    // Always use _cx to avoid unused variable warnings
    new_sig
        .inputs
        .push(parse_quote_spanned! { sig_span=> _cx: &::mcp_attr::server::RequestContext });

    new_sig.output = parse_quote_spanned! { sig_span=> -> ::mcp_attr::Result<::mcp_attr::schema::CompleteResult> };

    Ok(new_sig)
}

fn validate_complete_fn_signature(sig: &syn::Signature, allow_self: bool) -> Result<()> {
    // Check if function is async
    if sig.asyncness.is_none() {
        bail!(sig.ident.span(), "completion function must be async");
    }

    // Check if function has required value parameter
    let mut has_value_param = false;
    let mut has_self = false;

    for input in &sig.inputs {
        match input {
            FnArg::Receiver(receiver) => {
                has_self = true;
                if !allow_self {
                    bail!(
                        receiver.self_token.span(),
                        "completion functions using #[complete_fn] attribute cannot have `self` parameter. Use the function inside #[mcp_server] impl block instead."
                    );
                }
            }
            FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let arg_name = pat_ident.ident.to_string();
                    if arg_name == "value" || arg_name == "_value" {
                        // Validate value parameter type
                        if !is_str_reference(&pat_type.ty) {
                            let type_name =
                                quote::quote!(#pat_type.ty).to_string().replace(" ", "");
                            bail!(
                                pat_type.ty.span(),
                                "completion function value parameter must be of type `&str`, found `{}`",
                                type_name
                            );
                        }
                        has_value_param = true;
                    }
                }
            }
        }
    }

    if !has_value_param {
        let function_name = &sig.ident;
        bail!(
            sig.ident.span(),
            "completion function `{}` must have a `value: &str` parameter",
            function_name
        );
    }

    Ok(())
}

fn has_context_parameter(sig: &syn::Signature) -> bool {
    sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Type::Reference(type_ref) = &*pat_type.ty {
                if let syn::Type::Path(type_path) = &*type_ref.elem {
                    return type_path
                        .path
                        .segments
                        .last()
                        .map(|seg| seg.ident == "RequestContext")
                        .unwrap_or(false);
                }
            }
        }
        false
    })
}

#[derive(Debug)]
struct CompleteFnArg {
    name: String,
    ty: syn::Type,
    is_option: bool,
    is_str_ref: bool,
}

fn analyze_complete_fn_args(sig: &syn::Signature) -> Result<Vec<CompleteFnArg>> {
    let mut args = Vec::new();
    let mut found_value = false;
    let has_context = has_context_parameter(sig);

    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = pat_ident.ident.to_string();

                // Skip &self
                if arg_name == "self" {
                    continue;
                }

                // Check for value parameter (including _value)
                if arg_name == "value" || arg_name == "_value" {
                    found_value = true;
                    continue;
                }

                // Skip RequestContext parameter
                if is_request_context_type(&pat_type.ty) {
                    continue;
                }

                // Only collect args after value parameter and before RequestContext
                if found_value {
                    let (base_ty, is_option) = extract_option_inner_type(&pat_type.ty);
                    let is_str_ref = is_str_reference(&base_ty);

                    // Validate that the argument type is supported for completion functions
                    validate_completion_arg_type(&base_ty, is_str_ref, &pat_type.ty)?;

                    args.push(CompleteFnArg {
                        name: arg_name,
                        ty: base_ty.clone(),
                        is_option,
                        is_str_ref,
                    });
                }
            }
        }
    }

    Ok(args)
}

fn is_request_context_type(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(type_ref) = ty {
        if let syn::Type::Path(type_path) = &*type_ref.elem {
            return type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "RequestContext")
                .unwrap_or(false);
        }
    }
    false
}

fn extract_option_inner_type(ty: &syn::Type) -> (syn::Type, bool) {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if last_segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return (inner_ty.clone(), true);
                    }
                }
            }
        }
    }
    (ty.clone(), false)
}

fn is_str_reference(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(type_ref) = ty {
        if let syn::Type::Path(type_path) = &*type_ref.elem {
            return type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "str")
                .unwrap_or(false);
        }
    }
    false
}

fn validate_completion_arg_type(
    base_ty: &syn::Type,
    is_str_ref: bool,
    original_ty: &syn::Type,
) -> Result<()> {
    // &str is always allowed
    if is_str_ref {
        return Ok(());
    }

    // Check if it's a commonly supported FromStr type
    if is_supported_fromstr_type(base_ty) {
        return Ok(());
    }

    // For other types, we'll generate a more specific error message
    let type_name = quote::quote!(#base_ty).to_string().replace(" ", "");
    bail!(
        original_ty.span(),
        "unsupported argument type for completion function: `{}`. Supported types are `&str`, `Option<&str>`, and types that implement `FromStr` like `i32`, `u32`, `f64`, `bool`, `String`, etc.",
        type_name
    );
}

fn is_supported_fromstr_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            let type_name = last_segment.ident.to_string();
            match type_name.as_str() {
                // Common primitive types that implement FromStr
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
                | "u128" | "usize" | "f32" | "f64" | "bool" | "String" | "char" => true,
                _ => {
                    // For other types, we'll still allow them but they must implement FromStr
                    // This is a heuristic check - we can't fully validate FromStr at macro time
                    // but we can catch obviously wrong types like Vec, HashMap, etc.
                    !is_obviously_non_fromstr_type(&type_name)
                }
            }
        } else {
            false
        }
    } else {
        false
    }
}

fn is_obviously_non_fromstr_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "Vec"
            | "HashMap"
            | "BTreeMap"
            | "HashSet"
            | "BTreeSet"
            | "VecDeque"
            | "LinkedList"
            | "Option"
            | "Result"
            | "Box"
            | "Rc"
            | "Arc"
            | "Cell"
            | "RefCell"
            | "Mutex"
            | "RwLock"
    )
}

fn build_complete_fn_body(
    original_sig: &syn::Signature,
    inner_ident: &syn::Ident,
) -> Result<TokenStream> {
    let has_self = original_sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, FnArg::Receiver(_)));
    let has_context = has_context_parameter(original_sig);
    let additional_args = analyze_complete_fn_args(original_sig)?;

    let f = if has_self {
        quote!(self.#inner_ident)
    } else {
        quote!(#inner_ident)
    };

    // Generate argument extraction code
    let arg_extractions = generate_arg_extractions(&additional_args)?;

    // Build the function call arguments
    let mut call_args = vec![quote!(&p.argument.value)];

    // Add additional arguments
    for arg in &additional_args {
        let arg_name = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());
        call_args.push(quote!(#arg_name));
    }

    // Add context if needed
    if has_context {
        call_args.push(quote!(_cx));
    }

    let sig_span = original_sig.ident.span();
    let call_expr = quote_spanned! { sig_span=> #f(#(#call_args),*).await? };

    Ok(quote_spanned! { sig_span=>
        #arg_extractions
        Ok(#call_expr.into())
    })
}

fn generate_arg_extractions(args: &[CompleteFnArg]) -> Result<TokenStream> {
    if args.is_empty() {
        return Ok(quote!());
    }

    let mut extractions = Vec::new();

    // Get context once if needed
    extractions.push(quote! {
        let default_context = ::mcp_attr::schema::CompleteRequestParamsContext::default();
        let context = p.context.as_ref().unwrap_or(&default_context);
    });

    for arg in args {
        let arg_name = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());
        let arg_key = &arg.name;

        let extraction = if arg.is_option {
            if arg.is_str_ref {
                // Option<&str>
                quote! {
                    let #arg_name = context.arguments.get(#arg_key).map(|s| s.as_str());
                }
            } else {
                // Option<T> where T: FromStr
                let ty = &arg.ty;
                quote! {
                    let #arg_name = context.arguments.get(#arg_key)
                        .map(|s| s.parse::<#ty>())
                        .transpose()
                        .map_err(|_| ::mcp_attr::Error::from(::mcp_attr::ErrorCode::INVALID_PARAMS))?;
                }
            }
        } else if arg.is_str_ref {
            // &str
            quote! {
                let #arg_name = match context.arguments.get(#arg_key) {
                    Some(s) => s.as_str(),
                    None => return Ok(::mcp_attr::schema::CompleteResult::default()),
                };
            }
        } else {
            // T where T: FromStr
            let ty = &arg.ty;
            quote! {
                let #arg_name = match context.arguments.get(#arg_key) {
                    Some(s) => s.parse::<#ty>()
                        .map_err(|_| ::mcp_attr::Error::from(::mcp_attr::ErrorCode::INVALID_PARAMS))?,
                    None => return Ok(::mcp_attr::schema::CompleteResult::default()),
                };
            }
        };

        extractions.push(extraction);
    }

    Ok(quote! {
        #(#extractions)*
    })
}

fn make_extend(source: impl ToTokens, code: Result<TokenStream>, dump: bool) -> TokenStream {
    let code = match code {
        Ok(code) => {
            if dump {
                dump_code(code);
            }
            code
        }
        Err(e) => e.to_compile_error(),
    };
    quote! {
        #source
        #code
    }
}
fn dump_code(code: TokenStream) -> ! {
    panic!("// ===== start generated code =====\n{code}\n// ===== end generated code =====\n");
}
