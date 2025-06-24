#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use structmeta::{NameArgs, NameValue, StructMeta};
use syn::{
    Attribute, Expr, FnArg, Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, LitStr, Pat, PatType,
    Path, Result, Signature, Type, Visibility, parse::Parse, parse2, spanned::Spanned,
};
use uri_template_ex::UriTemplate;

use crate::utils::{
    arg_name_of, descriotion_expr, expand_option_ty, expr_to_option, get_doc, get_only_attr,
    is_context, ret_span, take_doc,
};
use crate::{
    syn_utils::{get_element, is_path, is_type},
    utils::take_only_attr,
};

#[derive(StructMeta, Default)]
pub struct PromptArgAttr {
    #[struct_meta(unnamed)]
    name: Option<LitStr>,
}

#[derive(StructMeta, Default)]
pub struct PromptAttr {
    #[struct_meta(unnamed)]
    name: Option<LitStr>,
    description: Option<Expr>,
    pub dump: bool,
}

pub struct PromptEntry {
    vis: Visibility,
    name: String,
    fn_ident: Ident,
    description: String,
    attr_description: Option<Expr>,
    args: Vec<PromptFnArg>,
    ret_span: Span,
}
impl PromptEntry {
    pub fn from_impl_item_fn(f: &mut ImplItemFn, attr: PromptAttr) -> Result<Self> {
        let f_span = f.span();
        Self::new(&f.vis, &mut f.sig, &f.attrs, f_span, attr)
    }
    pub fn from_item_fn(f: &mut ItemFn, attr: PromptAttr) -> Result<Self> {
        let f_span = f.span();
        Self::new(&f.vis, &mut f.sig, &f.attrs, f_span, attr)
    }
    fn new(
        vis: &Visibility,
        sig: &mut Signature,
        attrs: &[Attribute],
        f_span: Span,
        attr: PromptAttr,
    ) -> Result<Self> {
        let name = attr
            .name
            .map(|n| n.value())
            .unwrap_or_else(|| sig.ident.to_string());
        let description = if attr.description.is_some() {
            String::new()
        } else {
            get_doc(attrs)
        };
        let args = sig
            .inputs
            .iter_mut()
            .map(PromptFnArg::new)
            .collect::<Result<Vec<_>>>()?;
        let fn_ident = sig.ident.clone();

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
        let prompts = items
            .iter()
            .map(|p| p.build_metadata())
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            async fn prompts_list(&self,
                p: ::mcp_attr::schema::ListPromptsRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::ListPromptsResult> {
                    Ok(vec![#(#prompts,)*].into())
            }
        })
    }
    fn build_metadata(&self) -> Result<TokenStream> {
        let name = &self.name;
        let description = if let Some(attr_desc) = &self.attr_description {
            expr_to_option(&Some(attr_desc.clone()))
        } else {
            descriotion_expr(&self.description)
        };
        let args = self
            .args
            .iter()
            .filter_map(|a| a.build_list_expr().transpose())
            .collect::<Result<Vec<TokenStream>>>()?;
        Ok(quote! {
            ::mcp_attr::schema::Prompt {
                arguments: vec![#(#args,)*],
                name: #name.into(),
                description: #description,
                meta: Default::default(),
                title: None,
            }
        })
    }
    pub fn build_get(items: &[Self]) -> Result<TokenStream> {
        let arms = items
            .iter()
            .map(|p| p.build_get_arms())
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            async fn prompts_get(&self,
                p: ::mcp_attr::schema::GetPromptRequestParams,
                cx: &mut ::mcp_attr::server::RequestContext)
                -> ::mcp_attr::Result<::mcp_attr::schema::GetPromptResult> {
                    match p.name.as_str() {
                        #(#arms)*
                        _ => return ::std::result::Result::Err(::mcp_attr::server::errors::prompt_not_found(&p.name)),
                    }
                }
        })
    }
    fn build_get_arms(&self) -> Result<TokenStream> {
        let name = &self.name;
        let args = self
            .args
            .iter()
            .map(|a| a.build_get_expr())
            .collect::<Result<Vec<_>>>()?;
        let fn_ident = &self.fn_ident;
        let ret_span = self.ret_span;
        Ok(quote_spanned! {ret_span=>
            #name => {
                #[allow(clippy::useless_conversion)]
                {
                    return Ok(<::mcp_attr::schema::GetPromptResult as ::std::convert::From<_>>::from(Self::#fn_ident(#(#args,)*).await?));
                }
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
            .map(|a| a.build_get_expr())
            .collect::<Result<Vec<_>>>()?;
        let metadata = self.build_metadata()?;
        Ok(quote! {
            #vis fn #route_ident() -> ::mcp_attr::Result<::mcp_attr::server::builder::PromptDefinition> {
                Ok(::mcp_attr::server::builder::PromptDefinition::new(
                    #metadata,
                    |p: &::mcp_attr::schema::GetPromptRequestParams, cx: &::mcp_attr::server::RequestContext| {
                        Box::pin(async move {
                            Ok(::mcp_attr::schema::GetPromptResult::from(
                                #fn_ident(#(#args,)*).await?,
                            ))
                        })
                    }
                ))
            }
        })
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum PromptFnArg {
    Property(PromptArg),
    Context(Span),
    Receiver(Span),
}
impl PromptFnArg {
    fn new(f: &mut FnArg) -> Result<Self> {
        let span = f.span();
        let typed_arg = match f {
            FnArg::Typed(pat_type) => pat_type,
            FnArg::Receiver(receiver) => return Ok(Self::Receiver(span)),
            _ => bail!(f.span(), "Unsupported function argument pattern"),
        };
        let arg_attr = take_only_attr::<PromptArgAttr>(&mut typed_arg.attrs, "arg")?;
        let has_arg_attr = arg_attr.is_some();
        let arg_attr = arg_attr.unwrap_or_default();
        let description = take_doc(&mut typed_arg.attrs);
        if is_context(&typed_arg.ty) && !has_arg_attr {
            return Ok(Self::Context(span));
        }
        let name = if let Some(name) = &arg_attr.name {
            name.value()
        } else {
            arg_name_of(typed_arg)?
        };
        let (ty, required) = expand_option_ty(&typed_arg.ty);
        Ok(Self::Property(PromptArg {
            name,
            ty,
            description,
            required,
            span,
        }))
    }
    fn build_list_expr(&self) -> Result<Option<TokenStream>> {
        match self {
            Self::Property(arg) => Ok(Some(arg.build_list()?)),
            Self::Context(..) | Self::Receiver(..) => Ok(None),
        }
    }
    fn build_get_expr(&self) -> Result<TokenStream> {
        match self {
            Self::Property(arg) => arg.build_get(),
            Self::Context(span) => Ok(quote_spanned!(*span=> cx)),
            Self::Receiver(span) => Ok(quote_spanned!(*span=> self)),
        }
    }
}

#[derive(Debug)]
struct PromptArg {
    name: String,
    ty: Type,
    description: String,
    required: bool,
    span: Span,
}
impl PromptArg {
    fn build_list(&self) -> Result<TokenStream> {
        let name = &self.name;
        let description = descriotion_expr(&self.description);
        let required = self.required;
        Ok(quote! {
            ::mcp_attr::schema::PromptArgument {
                name: #name.into(),
                description: #description.into(),
                required: Some(#required),
                title: None,
            }
        })
    }
    fn build_get(&self) -> Result<TokenStream> {
        let name = &self.name;
        let ty = &self.ty;
        let span = self.span;
        if self.required {
            Ok(
                quote_spanned!(span=> ::mcp_attr::helpers::parse_prompt_arg::<#ty>(&p.arguments, #name)?),
            )
        } else {
            Ok(
                quote_spanned!(span=> ::mcp_attr::helpers::parse_prompt_arg_opt::<#ty>(&p.arguments, #name)?),
            )
        }
    }
}
