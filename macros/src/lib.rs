use quote::quote;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::Result;
use syn::{Error, Token, parse::Parse};

#[proc_macro_attribute]
pub fn bench_test(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(args as DirBenchArgs);

    match BenchBuilder::new(args, input).build() {
        Ok((benchs, func)) => quote! {
            #func
            #benchs
        }
        .into(),
        Err(e) => e.to_compile_error().into(),
    }
}

struct BenchBuilder {
    args: DirBenchArgs,
    func: syn::ItemFn,
    bench_attrs: Vec<syn::Attribute>,
}

impl BenchBuilder {
    fn new(args: DirBenchArgs, func: syn::ItemFn) -> Self {
        Self {
            args,
            func,
            bench_attrs: vec![],
        }
    }

    fn build(mut self) -> Result<(TokenStream2, syn::ItemFn)> {
        self.extract_bench_args()?;

        let mut pattern = self.args.resolve_dir()?;

        pattern.push(
            self.args
                .glob
                .clone()
                .map_or_else(|| "*".to_owned(), |v| v.value()),
        );

        let paths = glob::glob(&pattern.to_string_lossy()).map_err(|e| {
            Error::new_spanned(
                self.args.glob.clone().unwrap(),
                format!("failed to resolve glob pattern {e}"),
            )
        })?;

        let bound = paths.size_hint();
        let mut tests = Vec::with_capacity(bound.1.unwrap_or(bound.0));

        for entry in paths.filter_map(|p| p.ok()) {
            if !entry.is_file() {
                continue;
            }

            tests.push(self.build_bench(&entry)?)
        }

        Ok((
            quote! {
                #(#tests)*
            },
            self.func,
        ))
    }

    fn build_bench(&self, path: &Path) -> Result<TokenStream2> {
        let bench_ident = &self.func.sig.ident;
        let bench_name = self.bench_name(bench_ident.to_string(), path)?;
        let bench_attrs = &self.bench_attrs;
        let path = path.to_string_lossy();

        let loader = match self.args.loader {
            Some(ref loader) => quote! {#loader},
            None => quote! { ::core::include_str! },
        };

        Ok(quote! {
            #(#bench_attrs)*
            #[bench]
            fn #bench_name(b: &mut test::Bencher) {
                #bench_ident(b,::dir_bench::Fixture::new(#loader(#path), #path));
            }
        })
    }

    fn bench_name(&self, test_func_name: String, fixture_path: &Path) -> Result<syn::Ident> {
        assert!(fixture_path.is_file());

        let dir_path = self.args.resolve_dir()?;
        let rel_path = fixture_path.strip_prefix(dir_path).unwrap();

        assert!(rel_path.is_relative());

        let mut bench_name = test_func_name;
        bench_name.push_str("__");

        let components: Vec<_> = rel_path.iter().collect();

        for component in &components[0..components.len() - 1] {
            let component = component
                .to_string_lossy()
                .replace(|c: char| c.is_ascii_punctuation(), "_");
            bench_name.push_str(&component);
            bench_name.push('_');
        }

        bench_name.push_str(
            &rel_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .replace(|c: char| c.is_ascii_punctuation(), "_"),
        );

        if let Some(postfix) = &self.args.postfix {
            bench_name.push('_');
            bench_name.push_str(&postfix.value());
        }

        Ok(make_ident(&bench_name))
    }

    fn extract_bench_args(&mut self) -> Result<()> {
        let mut err = Ok(());

        self.func.attrs.retain(|attr| {
            if attr.path().is_ident("dir_bench_attr") {
                err = err
                    .clone()
                    .and(attr.parse_args_with(|input: syn::parse::ParseStream| {
                        self.bench_attrs
                            .extend(input.call(syn::Attribute::parse_outer)?);

                        if !input.is_empty() {
                            Err(Error::new(
                                input.span(),
                                "unexpected token after `dir_bench_attr`",
                            ))
                        } else {
                            Ok(())
                        }
                    }));

                false
            } else {
                true
            }
        });

        err
    }
}

#[derive(Default)]
struct DirBenchArgs {
    pub dir: Option<syn::LitStr>,
    pub glob: Option<syn::LitStr>,
    pub postfix: Option<syn::LitStr>,
    pub loader: Option<syn::Path>,
}

impl DirBenchArgs {
    fn resolve_dir(&self) -> Result<PathBuf> {
        let Some(dir) = &self.dir else {
            return Err(Error::new(Span::call_site(), "`dir` is required"));
        };

        let resolved = self.resolve_path(Path::new(&dir.value()))?;

        if !resolved.is_absolute() {
            return Err(Error::new_spanned(
                dir.clone(),
                format!("`{}` is not an absolute path", resolved.display()),
            ));
        } else if !resolved.exists() {
            return Err(Error::new_spanned(
                dir.clone(),
                format!("`{}` does not exist", resolved.display()),
            ));
        } else if !resolved.is_dir() {
            return Err(Error::new_spanned(
                dir.clone(),
                format!("`{}` is not a directory", resolved.display()),
            ));
        }

        Ok(resolved)
    }

    fn resolve_path(&self, path: &Path) -> Result<PathBuf> {
        let mut resolved = PathBuf::new();
        for component in path {
            resolved.push(self.resolve_component(component)?);
        }
        Ok(resolved)
    }

    fn resolve_component(&self, component: &OsStr) -> Result<PathBuf> {
        if component.to_string_lossy().starts_with('$') {
            let env_var = &component.to_string_lossy()[1..];
            let env_var_value = std::env::var(env_var).map_err(|e| {
                Error::new_spanned(
                    self.dir.clone().unwrap(),
                    format!("failed to resolve env var `{env_var}`: {e}"),
                )
            })?;
            let resolved = self.resolve_path(Path::new(&env_var_value))?;
            Ok(resolved)
        } else {
            Ok(Path::new(&component).into())
        }
    }
}

impl Parse for DirBenchArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = DirBenchArgs::default();
        let mut visited_args = HashSet::<String>::new();

        while !input.is_empty() {
            let arg = input.parse::<syn::Ident>()?;
            if visited_args.contains(&arg.to_string()) {
                return Err(Error::new_spanned(
                    arg.clone(),
                    format!("duplicated arg `{arg}`"),
                ));
            }

            input.parse::<Token![:]>()?;

            match arg.to_string().as_str() {
                "dir" => {
                    args.dir = Some(input.parse()?);
                }
                "glob" => {
                    args.glob = Some(input.parse()?);
                }
                "postfix" => {
                    args.postfix = Some(input.parse()?);
                }
                "loader" => {
                    args.loader = Some(input.parse()?);
                }
                _ => {
                    return Err(Error::new_spanned(
                        arg.clone(),
                        format!("unknown arg `{arg}`"),
                    ));
                }
            }

            visited_args.insert(arg.to_string());
            input.parse::<syn::Token![,]>().ok();
        }

        Ok(args)
    }
}

fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum "
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

fn make_ident(name: &str) -> syn::Ident {
    if is_keyword(name) {
        syn::Ident::new_raw(name, Span::call_site())
    } else {
        syn::Ident::new(name, Span::call_site())
    }
}
