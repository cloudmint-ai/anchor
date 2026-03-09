#[derive(Debug)]
pub enum Executor<'c> {
    Transaction(&'c mut sqlx::Transaction<'static, sqlx::Postgres>),
    Pool(sqlx::pool::Pool<sqlx::Postgres>),
}

impl<'c> sqlx::Executor<'c> for Executor<'c> {
    type Database = sqlx::Postgres;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::stream::BoxStream<
        'e,
        std::result::Result<
            sqlx::Either<
                <Self::Database as sqlx::Database>::QueryResult,
                <Self::Database as sqlx::Database>::Row,
            >,
            sqlx::Error,
        >,
    >
    where
        'c: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self {
            Self::Transaction(transaction) => transaction.fetch_many(query),
            Self::Pool(pool) => pool.fetch_many(query),
        }
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> futures::future::BoxFuture<
        'e,
        std::result::Result<Option<<Self::Database as sqlx::Database>::Row>, sqlx::Error>,
    >
    where
        'c: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        match self {
            Self::Transaction(transaction) => transaction.fetch_optional(query),
            Self::Pool(pool) => pool.fetch_optional(query),
        }
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> futures::future::BoxFuture<
        'e,
        std::result::Result<<Self::Database as sqlx::Database>::Statement<'q>, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        match self {
            Self::Transaction(transaction) => transaction.prepare_with(sql, parameters),
            Self::Pool(pool) => pool.prepare_with(sql, parameters),
        }
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<
        'e,
        std::result::Result<sqlx::Describe<Self::Database>, sqlx::Error>,
    >
    where
        'c: 'e,
    {
        match self {
            Self::Transaction(transaction) => transaction.describe(sql),
            Self::Pool(pool) => pool.describe(sql),
        }
    }
}
