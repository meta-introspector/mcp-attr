#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use structmeta::{NameArgs, NameValue, StructMeta};
use syn::{
    Attribute, Expr, FnArg, Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, LitStr, Pat, Path,
    Result, Signature, Type, Visibility, parse::Parse, parse2, spanned::Spanned,
};
use uri_template_ex::UriTemplate;

use crate::{
    route_ident,
    utils::{
        arg_name_of, description_expr, expand_option_ty, expr_to_option, get_doc, get_only_attr,
        is_context, ret_span, take_doc,
    },
};
use crate::{
    syn_utils::{get_element, is_path, is_type},
    utils::take_only_attr,
};

#[derive(StructMeta, Default)]
pub struct ToolArgAttr {
    #[struct_meta(unnamed)]
    name: Option<LitStr>,
}

#[derive(StructMeta, Default)]
pub struct ToolAttr {
    #[struct_meta(unnamed)]
    name: Option<LitStr>,
    description: Option<Expr>,
    pub dump: bool,
}

pub struct ToolEntry {
    vis: Visibility,
    name: String,
    fn_ident: Ident,
    description: String,
    attr_description: Option<Expr>,
    args: Vec<ToolFnArg>,
    ret_span: Span,
}
impl ToolEntry {
    pub fn from_impl_item_fn(f: &mut ImplItemFn, attr: ToolAttr) -> Result<Self> {
        let f_span = f.span();
        Self::new(&f.vis, &mut f.sig, &f.attrs, f_span, attr)
    }
    pub fn from_item_fn(f: &mut ItemFn, attr: ToolAttr) -> Result<Self> {
        let f_span = f.span();
        Self::new(&f.vis, &mut f.sig, &f.attrs, f_span, attr)
    }
    fn new(
        vis: &Visibility,
        sig: &mut Signature,
        attrs: &[Attribute],
        f_span: Span,
        attr: ToolAttr,
    ) -> Result<Self> {
        let name = attr
            .name
            .map(|n| n.value())
            .unwrap_or_else(|| sig.ident.to_string());
        let fn_ident = sig.ident.clone();
        let description = if attr.description.is_some() {
            // 属性にdescriptionがある場合は、それを文字列として保存（後でbuild_metadataで処理）
            String::new()
        } else {
            get_doc(attrs)
        };
        let args = sig
            .inputs
            .iter_mut()
            .map(ToolFnArg::new)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            vis: vis.clone(),
            name,
            fn_ident,
            description,
            attr_description: attr.description,
            args,
            ret_span: ret_span(sig, f_span),
        })
    }
    pub fn build_list(items: &[Self]) -> Result<TokenStream> {
        let items = items
            .iter()
            .map(|t| t.build_metadata())
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            async fn tools_list(&self,
                p: ::mcp_attr::schema::ListToolsRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::ListToolsResult> {
                    Ok(vec![#(#items,)*].into())
            }
        })
    }
    fn build_metadata(&self) -> Result<TokenStream> {
        let name = &self.name;
        let description = if let Some(attr_desc) = &self.attr_description {
            expr_to_option(&Some(attr_desc.clone()))
        } else {
            description_expr(&self.description)
        };
        let args = self
            .args
            .iter()
            .map(|a| a.build_list())
            .collect::<Result<Vec<TokenStream>>>()?;
        Ok(quote! {
            {
                let mut input_schema = ::mcp_attr::schema::ToolInputSchema::new();
                #(#args)*
                ::mcp_attr::schema::Tool {
                    name: #name.into(),
                    input_schema,
                    description: #description,
                    annotations: None,
                    meta: Default::default(),
                    output_schema: None,
                    title: None,
                }
            }
        })
    }
    pub fn build_call(items: &[Self]) -> Result<TokenStream> {
        let arms = items
            .iter()
            .map(|t| t.build_call_arms())
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            async fn tools_call(&self,
                p: ::mcp_attr::schema::CallToolRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::CallToolResult> {
                    match p.name.as_str() {
                        #(#arms)*
                        _ => return ::std::result::Result::Err(::mcp_attr::server::errors::tool_not_found(&p.name)),
                    }
            }
        })
    }
    fn build_call_arms(&self) -> Result<TokenStream> {
        let name = &self.name;
        let args = self
            .args
            .iter()
            .map(|a| a.build_call())
            .collect::<Result<Vec<_>>>()?;
        let fn_ident = &self.fn_ident;
        let ret_span = self.ret_span;
        Ok(quote_spanned! {ret_span=>
            #name => {
                #[allow(clippy::useless_conversion)]
                {
                    return Ok(<::mcp_attr::schema::CallToolResult as ::std::convert::From<_>>::from(Self::#fn_ident(#(#args,)*).await?));
                }
            }
        })
    }
    pub fn build_route(&self) -> Result<TokenStream> {
        let fn_ident = &self.fn_ident;
        let route_ident = route_ident(fn_ident);
        let vis = &self.vis;
        let args = self
            .args
            .iter()
            .map(|a| a.build_call())
            .collect::<Result<Vec<_>>>()?;
        let metadata = self.build_metadata()?;
        Ok(quote! {
            #vis fn #route_ident() -> ::mcp_attr::Result<::mcp_attr::server::builder::ToolDefinition> {
                Ok(::mcp_attr::server::builder::ToolDefinition::new(
                    #metadata,
                    |p: &::mcp_attr::schema::CallToolRequestParams, cx: &::mcp_attr::server::RequestContext| {
                        Box::pin(async move {
                            Ok(::mcp_attr::schema::CallToolResult::from(
                                #fn_ident(#(#args,)*).await?,
                            ))
                        })
                    }
                ))
            }
        })
    }
}

#[allow(clippy::large_enum_variant)]
enum ToolFnArg {
    Property(ToolArg),
    Context(Span),
    Receiver(Span),
}
impl ToolFnArg {
    fn new(f: &mut FnArg) -> Result<Self> {
        let span = f.span();
        let mut typed_arg = match f {
            FnArg::Typed(pat_type) => pat_type,
            FnArg::Receiver(receiver) => return Ok(Self::Receiver(span)),
            _ => bail!(f.span(), "Unsupported function argument pattern"),
        };
        let arg_attr = take_only_attr::<ToolArgAttr>(&mut typed_arg.attrs, "arg")?;
        let has_arg_attr = arg_attr.is_some();
        let arg_arg = arg_attr.unwrap_or_default();
        let description = take_doc(&mut typed_arg.attrs);
        if is_context(&typed_arg.ty) && !has_arg_attr {
            return Ok(Self::Context(span));
        }
        let name = if let Some(name) = &arg_arg.name {
            name.value()
        } else {
            arg_name_of(typed_arg)?
        };
        let (ty, required) = expand_option_ty(&typed_arg.ty);

        Ok(Self::Property(ToolArg {
            name,
            ty,
            description,
            required,
            span,
        }))
    }
    fn build_list(&self) -> Result<TokenStream> {
        match self {
            Self::Property(arg) => arg.build_list(),
            Self::Context(..) | Self::Receiver(..) => Ok(quote!()),
        }
    }
    fn build_call(&self) -> Result<TokenStream> {
        match self {
            Self::Property(arg) => arg.build_call(),
            Self::Context(span) => Ok(quote_spanned!(*span=> cx)),
            Self::Receiver(span) => Ok(quote_spanned!(*span=> self)),
        }
    }
}

struct ToolArg {
    name: String,
    ty: Type,
    description: String,
    required: bool,
    span: Span,
}
impl ToolArg {
    fn build_list(&self) -> Result<TokenStream> {
        let name = &self.name;
        let ty = &self.ty;
        let description = &self.description;
        let required = self.required;
        Ok(quote! {
            input_schema.insert_property::<#ty>(#name, #description, #required)?;
        })
    }
    fn build_call(&self) -> Result<TokenStream> {
        let name = &self.name;
        let ty = &self.ty;
        let required = self.required;
        let span = self.span;
        if self.required {
            Ok(
                quote_spanned! {span=> ::mcp_attr::helpers::parse_tool_arg::<#ty>(&p.arguments, #name)?},
            )
        } else {
            Ok(
                quote_spanned! {span=> ::mcp_attr::helpers::parse_tool_arg_opt::<#ty>(&p.arguments, #name)?},
            )
        }
    }
}
