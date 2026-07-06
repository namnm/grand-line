/// Internal macro db utils to use in tests.
#[macro_export]
macro_rules! tmp_db {
    ($($e:ident),* $(,)?) => {{
        let tmp = tmp_db().await?;
        $(create_table(&tmp.db, $e).await?;)*
        tmp
    }};
}
