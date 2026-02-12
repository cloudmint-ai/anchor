use crate::*;
// 之所以用 supply 不用 interface，主要原因包括与engine搭配
// 也包括 supply 明确了是外部依赖而非对外提供的能力接口
#[macro_export]
macro_rules! __use_supply_for_runtime {
    () => {
        use std::any::{Any, TypeId, type_name};

        pub static SUPPLIER: LazyLock<Supplier> = LazyLock::new(|| Supplier {
            suppliables: RwLock::new(HashMap::new()),
        });

        #[async_trait]
        pub trait Suppliable: Sized + Clone {
            // TODO add health
            async fn supply() -> Result<Self>;
        }

        pub struct Supplier {
            suppliables: RwLock<HashMap<TypeId, Box<dyn Any + Sync + Send>>>,
        }

        impl Supplier {
            pub async fn supply<S>(&self) -> Result<S>
            where
                S: Suppliable + Any + Send + Sync,
            {
                let type_id = TypeId::of::<S>();
                if let Some(supply) = self.suppliables.read().await.get(&type_id) {
                    debug!("using created {}", type_name::<S>());
                    return supply
                        .as_ref()
                        .downcast_ref::<S>()
                        .ok_or_else(none!())
                        .cloned();
                }
                debug!("creating {}", type_name::<S>());
                let supply = S::supply().await?;
                // TODO 这里可以根据anchor 是否有runtime 通过block_on 自动调用health
                self.suppliables
                    .write().await
                    .insert(type_id, Box::new(supply.clone()));

                debug!("created {}", type_name::<S>());
                Ok(supply)
            }

            pub async fn replace<S>(&self, supply: S) -> Result<()>
            where
                S: Suppliable + Any + Send + Sync + ?Sized,
            {
                let type_id = TypeId::of::<S>();
                debug!("replacing {}", type_name::<S>());
                self.suppliables
                    .write().await
                    .insert(type_id, Box::new(supply));
                Ok(())
            }
        }

        #[macro_export]
        macro_rules! supply {
            () => {
                SUPPLIER.supply().await?
            };
        }

        #[macro_export]
        macro_rules! replace_supply {
            ($e:expr) => {
                SUPPLIER.replace($e).await?
            };
        }
    };
}

tests! {
    __use_supply_for_runtime!();

    #[__::engine_for_engine]
    pub trait Supply {
        async fn ok(&self);
    }

    impl Engine {
        async fn ok(&self) {
            self.supply.ok().await;
        }
    }
    struct SupplyStruct {}
    #[async_trait]
    impl Supply for SupplyStruct {
        async fn health(&self) -> Result<()> {
            Ok(())
        }
        async fn ok(&self) {}
    }

    #[async_trait]
    impl Suppliable for Arc<dyn Supply> {
       async fn supply() -> Result<Self> {
            Ok(Arc::new(SupplyStruct {}))
        }
    }

    #[async_trait]
    impl Suppliable for Engine {
       async fn supply() -> Result<Self> {
            Ok(Engine { supply: supply!() })
        }
    }

    async fn test_supply() {
        let engine: Engine = supply!();
        engine.ok().await;
        replace_supply!(engine);
    }
}
