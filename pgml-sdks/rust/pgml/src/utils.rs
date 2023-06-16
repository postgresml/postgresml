/// A more type flexible version of format!
#[macro_export]
macro_rules! query_builder {
    ($left:expr, $( $x:expr ),* ) => {{
        let mut query = $left.to_string();
        $( query = query.replacen("%s", &$x, 1); )*
        query
    }};
}

#[macro_export]
macro_rules! transaction_wrapper {
    ($e:expr, $a:expr) => {
        let mut transaction = $a.begin().await?;
        $e.execute(&mut transaction).await?;
        sqlx::query("DEALLOCATE ALL")
            .execute(&mut transaction)
            .await?;
        transaction.commit().await?;
    };
    ($n:ident, $e:expr, $a:expr, $i:ident) => {
        let mut transaction = $a.begin().await?;
        $n = $e.$i(&mut transaction).await?;
        sqlx::query("DEALLOCATE ALL")
            .execute(&mut transaction)
            .await?;
        transaction.commit().await?;
    };
}
