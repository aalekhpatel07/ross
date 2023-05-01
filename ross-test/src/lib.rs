use ross_derive::Table;
use ross_db::table::fields::Field;

#[derive(Debug, Table)]
pub struct Foo {
    #[field(name="", kind=Field::Char)]
    pub a: String
}

#[test]
fn test_foo_generate_table() {
    let f = Foo { a: "aa".to_string() };
}