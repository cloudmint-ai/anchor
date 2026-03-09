use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, Fields, Item, ItemFn, ItemMod, ItemStruct, ItemTrait, ReturnType,
    TraitBound, TypeParamBound, Visibility, parse_macro_input, parse_quote, parse_str,
};

#[proc_macro_attribute]
pub fn cases(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_mod = parse_macro_input!(input as ItemMod);

    if let Some((_, ref mut items)) = input_mod.content {
        for item in items {
            match item {
                Item::Fn(func) => {
                    if let Err(err) = process_test_item_prepend(func) {
                        return err.to_compile_error().into();
                    }
                }
                _ => {}
            }
        }
    }

    TokenStream::from(quote! { #input_mod })
}

fn process_test_item_prepend(func: &mut ItemFn) -> Result<(), syn::Error> {
    let func_name = func.sig.ident.to_string();
    if !func_name.starts_with("test_") {
        return Ok(());
    }
    for attr in &func.attrs {
        if attr.path().to_token_stream().to_string().contains("test")
            || attr.path().to_token_stream().to_string().contains("case")
        {
            // VSCode 有未知的重复调用行为，跳过。
            return Ok(());
            // return Err(syn::Error::new_spanned(
            //     attr,
            //     format!(
            //         "{}, path {:?} unexpected",
            //         func_name,
            //         attr.path().to_token_stream().to_string()
            //     ), // "tests 宏内不再需要手动添加 #[test::case]",
            // ));
        }
    }
    let test_attr = parse_quote! {#[test::__::case]};
    if let Some(pos) = func
        .attrs
        .iter()
        .position(|attr| attr.path().is_ident("ignore"))
    {
        func.attrs.insert(pos, test_attr);
    } else {
        func.attrs.push(test_attr);
    }
    Ok(())
}

// infi loop not supported
#[proc_macro_attribute]
pub fn case(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;

    if let ReturnType::Type(_, _) = input.sig.output {
        return syn::Error::new_spanned(input.sig.output, "test::case should NOT return")
            .to_compile_error()
            .into();
    }

    let mut ignore = quote! {};
    for attr in &input.attrs {
        if attr.path().is_ident("ignore") {
            ignore = quote! { #[ignore] };
            break;
        }
    }

    let result = if is_async {
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            #[test]
            #ignore
            fn #fn_name() -> Result<()> {
                test::__::init();
                Runtime::new()?.block_on(async {
                    #fn_block
                    Ok::<(), Error>(())
                })?;
                Ok(())
            }
        }
    } else {
        let body = quote! {
            #ignore
            fn #fn_name() -> Result<()>  {
                test::__::init();
                #fn_block
                Ok(())
            }
        };
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            #[test]
            #body

            #[cfg(target_arch = "wasm32")]
            #[wasm_bindgen_test::wasm_bindgen_test]
            #body
        }
    };

    result.into()
}

#[cfg(feature = "async")]
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let is_async = input.sig.asyncness.is_some();
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;

    if !is_async {
        return syn::Error::new_spanned(input.sig.output, "main should be async")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #[__::main(crate="__::tokio")]
        async fn #fn_name() -> Result<()> {
            let result: Result<()> = {
                #fn_block
            };
            if let Err(e) = result {
                error!("main error {:?}", e)
            }
            Ok(())
        }
    };

    TokenStream::from(expanded)
}

#[cfg(feature = "api")]
#[proc_macro_derive(Protocol)]
pub fn protocol_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl api::Protocol for #name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Entity)]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Entity for #name {
            fn _id(&self) -> Id {
                self.id
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Versioned)]
pub fn versioned_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        #[async_trait]
        impl Versioned for #name {
            fn _current_version(&self) -> Version {
                self.version
            }
            fn _increase_version(&mut self) {
                self.version.increase();
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn data_for_engine(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    check_all_fields_public(&input);

    let default_derive = match &input.data {
        Data::Struct(_) => quote! {
            #[derive(Debug, Clone, Serialize, Deserialize)]
        },
        Data::Enum(_) => quote! {
            #[derive(Debug, Clone, Serialize, Deserialize, EnumDisplay, Eq, PartialEq)]
        },
        Data::Union(_) => {
            panic!("unexpected union {}", &input.ident);
        }
    };

    TokenStream::from(quote! {
       #default_derive
       #[serde(crate = "_serde")]
       #input
    })
}

fn check_all_fields_public(input: &DeriveInput) {
    let struct_name = &input.ident;

    if !matches!(input.vis, Visibility::Public(_)) {
        panic!("{} should be pub", struct_name);
    }

    let fields = match &input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        Data::Enum(_) => return,
        Data::Union(_) => {
            // TODO 编译错误统一panic 化
            panic!("unexpected union {}", struct_name);
        }
    };

    match fields {
        Fields::Named(fields_named) => {
            for field in &fields_named.named {
                if !matches!(field.vis, Visibility::Public(_)) {
                    let field_name = field.ident.as_ref().unwrap();
                    panic!("{}::{} should be pub", struct_name, field_name);
                }
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            for (index, field) in fields_unnamed.unnamed.iter().enumerate() {
                if !matches!(field.vis, Visibility::Public(_)) {
                    panic!("{}::{} should be pub", struct_name, index);
                }
            }
        }
        Fields::Unit => {
            // 单元结构体没有字段，直接通过
        }
    }
}

// entity 属性宏，用来标注 entity 对象
#[proc_macro_attribute]
pub fn entity_for_engine(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let default_derive = match &input.data {
        Data::Struct(_) => quote! {
            #[derive(Entity, Debug, Clone)]
        },
        Data::Enum(_) => quote! {
            panic!("unexpected enum {}", &input.ident);
        },
        Data::Union(_) => {
            panic!("unexpected union {}", &input.ident);
        }
    };

    TokenStream::from(quote! {
       #default_derive
       #input
    })
}

// versioned 属性宏，用来标注 最小一致性单元，versioned entity 对象
#[proc_macro_attribute]
pub fn versioned_for_engine(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let default_derive = match &input.data {
        Data::Struct(_) => quote! {
            #[derive(Versioned, Entity, Debug, Clone)]
        },
        Data::Enum(_) => quote! {
            panic!("unexpected enum {}", &input.ident);
        },
        Data::Union(_) => {
            panic!("unexpected union {}", &input.ident);
        }
    };

    TokenStream::from(quote! {
       #default_derive
       #input
    })
}

// 自动扩展 Engine 的定义与相关初始化函数和必要的框架函数
// Engine: { supply: Arc<dyn Supply> }
// Engine 组装是同步的，是否需要异步能力根据实际情况
// 为了保持 async 和 非 async 时 engine 代码一致，隐去了 async_trait
#[cfg(feature = "async")]
#[proc_macro_attribute]
pub fn engine_for_engine(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_trait = parse_macro_input!(input as ItemTrait);

    if input_trait.ident.to_string() != "Supply" {
        return syn::Error::new_spanned(&input_trait.ident, "Trait name should be Supply")
            .to_compile_error()
            .into();
    }

    let mut new_trait = input_trait.clone();
    add_send_sync_bounds_and_health(&mut new_trait);

    // TODO 检查 trait 是否是pub

    TokenStream::from(quote! {
        #[cfg_attr(test, automock)]
        #[async_trait]
        #new_trait

        #[derive(Clone)]
        pub struct Engine {
            supply: Arc<dyn Supply>,
        }

        impl Engine {
            pub fn new(supply: Arc<dyn Supply>) -> Self {
                Self { supply }
            }
            // health 为整体启动前和运行过程中自检使用，自动组装时不需要调用，否则调用次数会过高。
            // 一般是无状态自检
            pub async fn health(&self) -> Result<()> {
                self.supply.health().await
            }
        }

        #[cfg(test)]
        impl Engine {
            pub fn mock<F>(setup: F) -> Self
            where
                F: FnOnce(&mut MockSupply),
            {
                let mut supply = MockSupply::new();
                setup(&mut supply);
                supply.expect_health().returning(move || Ok(()));
                Engine::new(Arc::new(supply))
            }
        }
    })
}

#[cfg(not(feature = "async"))]
#[proc_macro_attribute]
pub fn engine_for_engine(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_trait = parse_macro_input!(input as ItemTrait);

    if input_trait.ident.to_string() != "Supply" {
        return syn::Error::new_spanned(&input_trait.ident, "Trait name should be Supply")
            .to_compile_error()
            .into();
    }

    let mut new_trait = input_trait.clone();
    add_send_sync_bounds(&mut new_trait);

    // TODO 检查 trait 是否是pub

    TokenStream::from(quote! {
        #[cfg_attr(test, automock)]
        #new_trait

        #[derive(Clone)]
        pub struct Engine {
            supply: Arc<dyn Supply>,
        }

        impl Engine {
            pub fn new(supply: Arc<dyn Supply>) -> Self {
                Self { supply }
            }
        }

        #[cfg(test)]
        impl Engine {
            pub fn mock<F>(setup: F) -> Self
            where
                F: FnOnce(&mut MockSupply),
            {
                let mut supply = MockSupply::new();
                setup(&mut supply);
                Engine::new(Arc::new(supply))
            }
        }
    })
}

fn add_send_sync_bounds(trait_item: &mut ItemTrait) {
    // 创建 Send bound
    let send_bound = parse_str::<TraitBound>("Send").unwrap();
    let sync_bound = parse_str::<TraitBound>("Sync").unwrap();

    // 添加到 supertraits
    trait_item
        .supertraits
        .push(TypeParamBound::Trait(send_bound));
    trait_item
        .supertraits
        .push(TypeParamBound::Trait(sync_bound));
}

#[cfg(feature = "async")]
fn add_send_sync_bounds_and_health(trait_item: &mut ItemTrait) {
    add_send_sync_bounds(trait_item);

    trait_item.items.push(parse_quote! {
        async fn health(&self) -> Result<()>;
    });
}

#[proc_macro_attribute]
pub fn engine_for_runtime(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    // TODO 自动实现 health

    match input {
        Item::Struct(item_struct) => _engine_for_runtime_struct(item_struct),
        _ => {
            return syn::Error::new_spanned(&input, "input type not supported")
                .to_compile_error()
                .into();
        }
    }
}

#[cfg(feature = "async")]
fn _engine_for_runtime_struct(input: ItemStruct) -> TokenStream {
    // TODO it doesnt work in VSCODE
    // let file_path = Span::call_site().file();
    // let engine_name = std::path::Path::new(&file_path)
    //     .file_stem()
    //     .and_then(|s| s.to_str())
    //     .expect("no file_path");
    // let engine_ident = Ident::new(engine_name, Span::call_site().into());
    // pub use engine::#engine_ident::*;

    // TODO 自动supply 再考虑考虑
    // TODO 能自动supply 应该也能自动health
    // let field_inits = input.fields.iter().map(|field| {
    //     let field_name = field.ident.as_ref().expect("field_name");
    //     quote! {
    //         #field_name: supply!()
    //     }
    // });
    //             EngineSupply{
    //                 #(#field_inits),*
    //             })))

    // TODE 检查 EngineSupply 的名字

    TokenStream::from(quote! {
        #[derive(Clone)]
        #input
        #[async_trait]
        impl Suppliable for Engine {
            async fn supply() -> Result<Self> {
                let engine_supply: EngineSupply = supply!();
                Ok(Self::new(Arc::new(engine_supply)))
            }
        }
    })
}

#[cfg(not(feature = "async"))]
fn _engine_for_runtime_struct(input: ItemStruct) -> TokenStream {
    // TODE 检查 EngineSupply 的名字
    TokenStream::from(quote! {
        #[derive(Clone)]
        #input
        impl Suppliable for Engine {
            fn supply() -> Result<Self> {
                let engine_supply: EngineSupply = supply!();
                Ok(Self::new(Arc::new(engine_supply)))
            }
        }
    })
}
