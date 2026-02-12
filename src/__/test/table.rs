use crate::*;

// TODO merge multi tables into one
#[derive(Clone)]
pub struct Table<T>(Arc<Mutex<HashMap<Id, T>>>);

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(default!())))
    }
}

impl<T> Table<T>
where
    T: Entity,
{
    pub fn new(entities: Vec<T>) -> Self {
        let mut inner = HashMap::new();
        for entity in entities {
            inner.insert(entity._id(), entity);
        }
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.0.try_lock()?.len())
    }
    pub fn get(&self, _: &Context, id: Id) -> Result<T> {
        self.0.try_lock()?.get(&id).ok_or_else(none!()).cloned()
    }

    pub fn filter_id<F>(&self, _: &Context, condition: F) -> Result<Vec<Id>>
    where
        F: Fn(&T) -> bool,
    {
        // TODO search must outside transaction?
        Ok(self
            .0
            .try_lock()?
            .iter()
            .filter_map(|(id, item)| if condition(item) { Some(*id) } else { None })
            .collect())
    }

    pub fn filter<F>(&self, _: &Context, condition: F) -> Result<Vec<T>>
    where
        F: Fn(&T) -> bool,
    {
        // TODO search must outside transaction?
        Ok(self
            .0
            .try_lock()?
            .iter()
            .filter_map(|(_id, item)| {
                if condition(item) {
                    Some(item.clone())
                } else {
                    None
                }
            })
            .collect())
    }
    pub fn get_by<F>(&self, _: &Context, condition: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> bool,
    {
        let items = self.0.try_lock()?;
        let items: Vec<&T> = items
            .iter()
            .filter_map(|(_, item)| if condition(item) { Some(item) } else { None })
            .collect();

        if items.len() == 0 {
            Ok(None)
        } else if items.len() == 1 {
            Ok(Some(items[0].clone()))
        } else {
            Unexpected!("multiple items found")
        }
    }
    pub fn random_by<F>(&self, _: &Context, condition: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> bool,
    {
        let items = self.0.try_lock()?;
        let items: Vec<&T> = items
            .iter()
            .filter_map(|(_, item)| if condition(item) { Some(item) } else { None })
            .collect();

        let mut rng = rand::thread_rng();
        if let Some(&item) = items.choose(&mut rng) {
            Ok(Some(item.clone()))
        } else {
            Ok(None)
        }
    }
    pub fn max_by<F, K>(&self, _: &Context, key_fn: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> Option<K>,
        K: Ord,
    {
        Ok(self
            .0
            .try_lock()?
            .values()
            .filter(|value| key_fn(value).is_some())
            .max_by_key(|value| key_fn(value).unwrap())
            .cloned())
    }
    pub fn min_by<F, K>(&self, _: &Context, key_fn: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> Option<K>,
        K: Ord,
    {
        Ok(self
            .0
            .try_lock()?
            .values()
            .filter(|value| key_fn(value).is_some())
            .min_by_key(|value| key_fn(value))
            .cloned())
    }
    pub fn save(&self, context: &Context, v: &T) -> Result<()> {
        if !context.in_mocking_transaction()? {
            return Unexpected!("save outside mocking transaction");
        }
        self.0.try_lock()?.insert(v._id(), v.clone());
        Ok(())
    }
}

impl<T> Table<T>
where
    T: Versioned,
{
    pub fn lock(&self, context: &Context, v: &T) -> Result<()> {
        context.mock_bind_transaction(v._current_version())?;
        let mut items = self.0.try_lock()?;
        match items.get(&v._id()) {
            Some(item) => {
                // TODO 隔离版本
                // 注意由于 test::Table 本质上还是内存里的共享对象，因此_current_version 总是相同的
                if item._current_version() != v._current_version() {
                    Unexpected!(
                        "version not match {}!= {}",
                        item._current_version(),
                        v._current_version()
                    )
                } else {
                    v._current_version()._increase()?;
                    items.insert(v._id(), v.clone());
                    Ok(())
                }
            }
            None => {
                if !v._current_version().is_zero() {
                    return Unexpected!("unexpected non zero version to create");
                }
                v._current_version()._increase()?;
                items.insert(v._id(), v.clone());
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! test_table {
    ($($element:expr),* $(,)?) => {
        test::Table::new(vec![$($element),*])
    };
}

#[cfg(feature = "runtime")]
tests! {
    #[derive(Clone, Versioned, Entity)]
    struct Line {
        pub id: Id,
        pub version: Version,
        number: i64,
    }

    async fn test_table() {
        let table: Table<Line> = default!();
        let id = Id::generate().await;
        let line = Line {
            id,
            version: default!(),
            number: 10,
        };
        let context = &Context::mocking();
        transaction!(context, {
            table.lock(context, &line)?;
            let line = table.get(&context, id)?;
            assert_eq!(line.number, 10);
            assert_eq!(line.version, Version::from(1));
            table.save(context, &line)?;
        });
        let line = table.get(&context, id)?;
        assert_eq!(line.number, 10);
        assert_eq!(line.version, Version::from(1));
        let updated_line = Line {
            id,
            version: Version::from(1),
            number: 11,
        };
        transaction!(context, {
            table.lock(&context, &updated_line)?;
            table.save(&context, &updated_line)?;
        });
        assert_eq!(updated_line.version, Version::from(2));

        async fn fail_func_on_table(
            context: &Context,
            table: &Table<Line>,
            line: &Line,
        ) -> Result<()> {
            transaction!(context, {
                table.lock(&context, &line)?;
                if line.id != Id::from(1111111111) {
                    return Unexpected!("error we set");
                }
                table.save(context, &line)?;
            });
            Ok(())
        }

        if let Ok(_) = fail_func_on_table(context, &table, &updated_line).await {
            panic!("should not ok")
        }
        // fail func in transaction should not increase version
        assert_eq!(updated_line.version, Version::from(2));
        transaction!(context, {
            table.lock(&context, &updated_line)?;
            table.save(&context, &updated_line)?;
        });
        assert_eq!(updated_line.version, Version::from(3));
        let search_result = table.filter(&context, |session| session.number > 12)?;
        assert_eq!(search_result.len(), 0);
        let search_result = table.filter_id(&context, |session| session.number > 2)?;
        assert_eq!(search_result.len(), 1);

        assert_eq!(table.len()?, 1);

        let new_id = Id::generate().await;
        let new_line = Line {
            id: new_id,
            version: default!(),
            number: 100,
        };

        transaction!(context, {
            table.lock(context, &new_line)?;
            table.save(context, &new_line)?;
        });

        assert_eq!(table.len()?, 2);

        let max_line = table
            .max_by(context, |value| Some(value.number))?
            .ok_or_else(none!())?;
        assert_eq!(max_line.number, 100);

        let min_line = table
            .min_by(context, |value| Some(value.number))?
            .ok_or_else(none!())?;
        assert_eq!(min_line.number, 11);

        let max_line = table.max_by(context, |_value| None::<Option<i64>>)?;
        assert!(max_line.is_none());

        let max_line = table
            .max_by(context, |value| {
                if value.number < 20 {
                    Some(value.number)
                } else {
                    None
                }
            })?
            .ok_or_else(none!())?;
        assert_eq!(max_line.number, 11);

        let min_line = table.min_by(context, |_value| None::<Option<i64>>)?;
        assert!(min_line.is_none());

        let min_line = table
            .min_by(context, |value| {
                if value.number > 20 {
                    Some(value.number)
                } else {
                    None
                }
            })?
            .ok_or_else(none!())?;
        assert_eq!(min_line.number, 100);

        let random_line = table
            .random_by(context, |value| value.number > 20)?
            .ok_or_else(none!())?;
        assert_eq!(random_line.number, 100);

        let random_line = table.random_by(context, |value| value.number > 2000)?;
        assert!(random_line.is_none());

        let line = Line {
            id: Id::generate().await,
            version: default!(),
            number: 10,
        };
        let table = Table::new(vec![line]);
        assert_eq!(table.len()?, 1);

        let line1 = Line {
            id: Id::generate().await,
            version: default!(),
            number: 10,
        };
        let line2 = Line {
            id: Id::generate().await,
            version: default!(),
            number: 10,
        };
        let table = test_table![line1, line2];
        assert_eq!(table.len()?, 2);
    }
}
