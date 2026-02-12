// TODO support sync supply

#[macro_export]
macro_rules! use_supply {
    () => {
        use std::any::{Any, TypeId, type_name};

        pub static SUPPLIER: LazyLock<Supplier> = LazyLock::new(|| Supplier {
            suppliables: runtime::RwLock::new(HashMap::new()),
        });

        #[async_trait]
        pub trait Suppliable: Sized + Clone {
            // TODO add health
            async fn supply() -> Result<Self>;
        }

        pub struct Supplier {
            suppliables: runtime::RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
        }

        impl Supplier {
            pub async fn supply<S>(&self) -> Result<S>
            where
                S: Suppliable + Any + Send + Sync + ?Sized,
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
                self.suppliables
                    .write()
                    .await
                    .insert(type_id, Box::new(supply.clone()));

                debug!("created {}", type_name::<S>());
                Ok(supply)
            }

            pub async fn replace<S>(&self, supply: S)
            where
                S: Suppliable + Any + Send + Sync + ?Sized,
            {
                let type_id = TypeId::of::<S>();
                debug!("replacing {}", type_name::<S>());
                self.suppliables
                    .write()
                    .await
                    .insert(type_id, Box::new(supply));
            }
        }
    };
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
        SUPPLIER.replace($e).await
    };
}

#[cfg(feature = "runtime")]
#[cfg(test)]
mod tests {
    use crate::*;
    use_supply!();

    #[automock]
    #[async_trait]
    pub trait Supply: Send + Sync {
        async fn ok(&self);
    }

    struct Engine {
        supply: Arc<dyn Supply>,
    }

    impl Engine {
        async fn ok(&self) {
            self.supply.ok().await;
        }
    }

    struct SupplyStruct {}

    #[async_trait]
    impl Supply for SupplyStruct {
        async fn ok(&self) {}
    }

    #[async_trait]
    impl Suppliable for Arc<dyn Supply> {
        async fn supply() -> Result<Self> {
            Ok(Arc::new(SupplyStruct {}))
        }
    }

    #[async_trait]
    impl Suppliable for Arc<Engine> {
        async fn supply() -> Result<Self> {
            Ok(Arc::new(Engine { supply: supply!() }))
        }
    }

    #[test::case]
    async fn test_supply() {
        let engine: Arc<Engine> = supply!();
        engine.ok().await;
        replace_supply!(engine);
    }
}
