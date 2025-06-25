#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use structmeta::{NameArgs, NameValue, StructMeta};
use syn::{
    Attribute, Expr, FnArg, Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, LitStr, Pat, Path,
    Result, Signature, Type, Visibility, parse::Parse, parse2, spanned::Spanned,
};
use uri_template_ex::UriTemplate;

use crate::utils::{
    description_expr, expand_option_ty, expr_to_option, get_doc, get_only_attr, is_context,
    opt_expr, ret_span, take_doc,
};
use crate::{
    syn_utils::{get_element, is_path, is_type},
    utils::arg_name_of,
};

#[derive(StructMeta, Default)]
pub struct ResourceAttr {
    #[struct_meta(unnamed)]
    uri: Option<LitStr>,
    name: Option<LitStr>,
    description: Option<Expr>,
    title: Option<Expr>,
    mime_type: Option<LitStr>,
    pub dump: bool,
}

pub struct ResourceEntry {
    vis: Visibility,
    uri: Option<UriTemplate>,
    name: String,
    mime_type: Option<String>,
    description: String,
    attr_description: Option<Expr>,
    attr_title: Option<Expr>,
    args: Vec<ResourceFnArg>,
    fn_ident: Ident,
    ret_span: Span,
}

impl ResourceEntry {
    pub fn from_impl_item_fn(f: &mut ImplItemFn, attr: ResourceAttr) -> Result<Self> {
        let f_span = f.span();
        Self::new(&f.vis, &mut f.sig, &f.attrs, f_span, attr)
    }
    pub fn from_item_fn(f: &mut ItemFn, attr: ResourceAttr) -> Result<Self> {
        let f_span = f.span();
        Self::new(&f.vis, &mut f.sig, &f.attrs, f_span, attr)
    }
    fn new(
        vis: &Visibility,
        sig: &mut Signature,
        attrs: &[Attribute],
        f_span: Span,
        attr: ResourceAttr,
    ) -> Result<Self> {
        let mut name = None;
        let mut uri = None;
        let mut mime_type = None;
        if let Some(attr_uri) = &attr.uri {
            let uri_value = attr_uri.value();
            uri = match UriTemplate::new(&uri_value) {
                Ok(uri) => Some(uri),
                Err(e) => {
                    bail!(attr_uri.span(), "Invalid URI template: `{uri_value}` ({e})",)
                }
            };
            name = attr.name.map(|n| n.value());
            mime_type = attr.mime_type.map(|m| m.value());
        }
        let name = name.unwrap_or_else(|| sig.ident.to_string());
        let description = if attr.description.is_some() {
            String::new()
        } else {
            take_doc(&mut attrs.to_vec())
        };
        let args = sig
            .inputs
            .iter_mut()
            .map(|f| ResourceFnArg::new(f, &uri))
            .collect::<Result<Vec<_>>>()?;
        let fn_ident = sig.ident.clone();
        Ok(Self {
            vis: vis.clone(),
            name,
            uri,
            mime_type,
            description,
            attr_description: attr.description,
            attr_title: attr.title,
            args,
            fn_ident,
            ret_span: ret_span(sig, f_span),
        })
    }
    pub fn build_list(items: &[Self]) -> Result<TokenStream> {
        let arms = items
            .iter()
            .filter_map(|r| r.build_list_arm().transpose())
            .collect::<Result<Vec<TokenStream>>>()?;
        Ok(quote! {
            async fn resources_list(&self,
                p: ::mcp_attr::schema::ListResourcesRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::ListResourcesResult> {
                    Ok(vec![#(#arms,)*].into())
            }
        })
    }

    fn build_list_arm(&self) -> Result<Option<TokenStream>> {
        let Some(uri) = self.uri.as_ref() else {
            return Ok(None);
        };
        if uri.var_names().count() > 0 {
            return Ok(None);
        }
        let name = &self.name;
        let uri = uri.expand(());
        let mime_type = opt_expr(&self.mime_type, |x| quote!(#x.to_string()));
        let description = if self.attr_description.is_some() {
            expr_to_option(&self.attr_description)
        } else {
            description_expr(&self.description)
        };
        let title = expr_to_option(&self.attr_title);
        Ok(Some(quote! {
            ::mcp_attr::schema::Resource {
                name: #name.to_string(),
                uri: #uri.to_string(),
                mime_type: #mime_type,
                description: #description,
                size: None,
                annotations: None,
                meta: Default::default(),
                title: #title,
            }
        }))
    }

    pub fn build_templates_list(items: &[Self]) -> Result<TokenStream> {
        let arms = items
            .iter()
            .filter_map(|r| r.build_templates_list_arm().transpose())
            .collect::<Result<Vec<TokenStream>>>()?;
        Ok(quote! {
            async fn resources_templates_list(&self,
                p: ::mcp_attr::schema::ListResourceTemplatesRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::ListResourceTemplatesResult> {
                    Ok(vec![#(#arms,)*].into())
            }
        })
    }

    fn build_templates_list_arm(&self) -> Result<Option<TokenStream>> {
        let Some(uri) = self.uri.as_ref() else {
            return Ok(None);
        };
        if uri.var_names().count() == 0 {
            return Ok(None);
        }
        let name = &self.name;
        let uri = uri.to_string();
        let mime_type = opt_expr(&self.mime_type, |x| quote!(#x.to_string()));
        let description = if self.attr_description.is_some() {
            expr_to_option(&self.attr_description)
        } else {
            description_expr(&self.description)
        };
        let title = expr_to_option(&self.attr_title);
        Ok(Some(quote! {
            ::mcp_attr::schema::ResourceTemplate {
                name: #name.to_string(),
                uri_template: #uri.to_string(),
                mime_type: #mime_type,
                description: #description,
                annotations: None,
                meta: Default::default(),
                title: #title,
            }
        }))
    }

    pub fn build_read(items: &[Self]) -> Result<TokenStream> {
        let stmts = items
            .iter()
            .map(|r| r.build_read_stmt())
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            #[allow(unreachable_code)]
            async fn resources_read(&self,
                p: ::mcp_attr::schema::ReadResourceRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::ReadResourceResult> {
                    #(#stmts)*
                    ::mcp_attr::bail_public!(::mcp_attr::ErrorCode::INVALID_PARAMS, "resource `{}` not found", p.uri);
                }
        })
    }

    pub fn build_route(&self) -> Result<TokenStream> {
        let fn_ident = &self.fn_ident;
        let route_ident = crate::route_ident(fn_ident);
        let vis = &self.vis;
        let args = self
            .args
            .iter()
            .map(|a| a.build_read())
            .collect::<Result<Vec<_>>>()?;
        let name = &self.name;
        let description = if let Some(attr_desc) = &self.attr_description {
            quote! {
                .with_description(#attr_desc)
            }
        } else if !self.description.is_empty() {
            let description = &self.description;
            quote! {
                .with_description(#description)
            }
        } else {
            quote!()
        };
        let mime_type = if let Some(mime_type) = &self.mime_type {
            quote! {
                .with_mime_type(#mime_type)
            }
        } else {
            quote!()
        };
        let title = if let Some(attr_title) = &self.attr_title {
            quote! {
                .with_title(#attr_title)
            }
        } else {
            quote!()
        };
        let uri = opt_expr(&self.uri, |uri| {
            let uri = uri.to_string();
            quote!(#uri)
        });
        
        // Generate completion information
        let mut completion_infos = Vec::new();
        let resource_uri = if let Some(uri) = &self.uri {
            uri.to_string()
        } else {
            name.clone()
        };
        
        for arg in &self.args {
            if let ResourceFnArg::Var(uri_var) = arg {
                if let Some(complete_expr) = &uri_var.complete_expr {
                    // Check if it's a method syntax in standalone usage (error case)
                    match complete_expr {
                        crate::CompleteFuncExpr::InstanceMethod(method_name) => {
                            return Err(syn::Error::new(
                                method_name.span(),
                                "Method syntax (.method_name) can only be used within impl blocks, not in standalone functions"
                            ));
                        }
                        crate::CompleteFuncExpr::Expr(expr) => {
                            completion_infos.push((
                                resource_uri.clone(),
                                uri_var.name.clone(),
                                expr.clone()
                            ));
                        }
                    }
                }
            }
        }
        
        // Generate completion function calls
        let completions = completion_infos.iter().map(|(uri_str, arg_name, complete_expr)| {
            quote! {
                ::mcp_attr::server::builder::CompletionInfo {
                    name: #uri_str.to_string(),
                    argument: #arg_name.to_string(),
                    complete_fn: ::mcp_attr::server::builder::CompleteFn {
                        f: std::sync::Arc::new(|params: &::mcp_attr::schema::CompleteRequestParams, cx: &::mcp_attr::server::RequestContext| {
                            Box::pin(async move {
                                #complete_expr(params, cx).await
                            })
                        }),
                    },
                }
            }
        }).collect::<Vec<_>>();
        
        Ok(quote! {
            #[allow(clippy::needless_question_mark)]
            #vis fn #route_ident() -> ::mcp_attr::Result<::mcp_attr::server::builder::ResourceDefinition> {
                let completions = vec![#(#completions),*];
                Ok(::mcp_attr::server::builder::ResourceDefinition::new(
                    #name,
                    #uri,
                    |
                        p: &::mcp_attr::schema::ReadResourceRequestParams,
                        _captures: &::mcp_attr::helpers::uri_template_ex::Captures,
                        cx: &::mcp_attr::server::RequestContext | {
                        Box::pin(async move {
                            Ok(::mcp_attr::schema::ReadResourceResult::from(
                                #fn_ident(#(#args,)*).await?,
                            ))
                        })
                    }
                )?
                #description
                #mime_type
                #title
                .with_completions(completions))
            }
        })
    }

    fn build_read_stmt(&self) -> Result<Option<TokenStream>> {
        let description = if self.attr_description.is_some() {
            expr_to_option(&self.attr_description)
        } else {
            description_expr(&self.description)
        };
        let name = &self.name;
        let mime_type = opt_expr(&self.mime_type, |x| quote!(#x.to_string()));
        let args = self
            .args
            .iter()
            .map(|a| a.build_read())
            .collect::<Result<Vec<TokenStream>>>()?;
        let fn_ident = &self.fn_ident;
        let ret_span = self.ret_span;
        if let Some(uri) = self.uri.as_ref() {
            let uri = uri.to_string();
            Ok(Some(quote_spanned! {ret_span=>
                {
                    static URI_TEMPLATE : ::std::sync::LazyLock<::mcp_attr::helpers::uri_template_ex::UriTemplate> =
                        ::std::sync::LazyLock::new(|| ::mcp_attr::helpers::uri_template_ex::UriTemplate::new(#uri).unwrap());
                    if let Some(_captures) = URI_TEMPLATE.captures(&p.uri) {
                        let _captures = &_captures;
                        #[allow(clippy::useless_conversion)]
                        {
                            return Ok(<::mcp_attr::schema::ReadResourceResult as ::std::convert::From<_>>::from(Self::#fn_ident(#(#args,)*).await?));
                        }
                    }
                }
            }))
        } else {
            Ok(Some(quote_spanned! {ret_span=>
                #[allow(clippy::useless_conversion)]
                {
                    return Ok(<::mcp_attr::schema::ReadResourceResult as ::std::convert::From<_>>::from(Self::#fn_ident(#(#args,)*).await?));
                }
            }))
        }
    }

    pub fn get_completion_info(&self) -> Vec<(String, String, crate::CompleteFuncExpr)> {
        let mut completions = Vec::new();
        if let Some(uri) = &self.uri {
            let uri_str = uri.to_string();
            for arg in &self.args {
                if let ResourceFnArg::Var(uri_var) = arg {
                    if let Some(complete_expr) = &uri_var.complete_expr {
                        completions.push((
                            uri_str.clone(),
                            uri_var.name.clone(),
                            complete_expr.clone(),
                        ));
                    }
                }
            }
        }
        completions
    }
}

#[allow(clippy::large_enum_variant)]
enum ResourceFnArg {
    Receiver(Span),
    Context(Span),
    Url(Type, Span),
    Var(UriVar),
}

impl ResourceFnArg {
    fn new(f: &mut FnArg, uri: &Option<UriTemplate>) -> Result<Self> {
        let span = f.span();
        let typed_arg = match f {
            FnArg::Typed(pat_type) => pat_type,
            FnArg::Receiver(_) => return Ok(Self::Receiver(span)),
        };
        if is_context(&typed_arg.ty) {
            return Ok(Self::Context(span));
        }
        if let Some(uri) = uri {
            let complete_attr = crate::utils::take_only_attr::<crate::CompleteAttr>(
                &mut typed_arg.attrs,
                "complete",
            )?;
            let name = arg_name_of(typed_arg)?;
            if let Some(index) = uri.find_var_name(&name) {
                let (ty, required) = expand_option_ty(&typed_arg.ty);
                Ok(Self::Var(UriVar {
                    name,
                    index,
                    ty,
                    required,
                    span,
                    complete_expr: complete_attr.map(|attr| attr.func),
                }))
            } else {
                bail!(span, "URL Template does not contain variable `{name}`")
            }
        } else {
            Ok(Self::Url((*typed_arg.ty).clone(), span))
        }
    }
    fn build_read(&self) -> Result<TokenStream> {
        match self {
            ResourceFnArg::Receiver(span) => Ok(quote_spanned!(*span=> self)),
            ResourceFnArg::Context(span) => Ok(quote_spanned!(*span=> cx)),
            ResourceFnArg::Url(ty, span) => {
                Ok(quote_spanned!(*span=> ::std::convert::Into::<#ty>::into(&p.uri)))
            }
            ResourceFnArg::Var(x) => x.build_read(),
        }
    }
}

struct UriVar {
    name: String,
    index: usize,
    ty: Type,
    required: bool,
    span: Span,
    complete_expr: Option<crate::CompleteFuncExpr>,
}
impl UriVar {
    fn build_read(&self) -> Result<TokenStream> {
        let name = &self.name;
        let index = self.index;
        let ty = &self.ty;
        let span = self.span;
        Ok(if self.required {
            quote_spanned!(span=> ::mcp_attr::helpers::parse_resource_arg::<#ty>(_captures, #index, #name)?)
        } else {
            quote_spanned!(span=> ::mcp_attr::helpers::parse_resource_arg_opt::<#ty>(_captures, #index, #name)?)
        })
    }
}
