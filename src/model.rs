#[macro_export]
macro_rules! declare_model_table {
    ($model:ident, $schema:literal, $table:literal) => {
        #[allow(dead_code)]
        impl $model {
            pub const SCHEMA: &'static str = $schema;
            pub const TABLE: &'static str = $table;
            pub const QUALIFIED_TABLE: &'static str =
                concat!("\"", $schema, "\".", "\"", $table, "\"");
        }
    };
}
