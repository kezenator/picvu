#[test]
fn test_open_empty_database()
{
    let _connection = crate::Store::new(":memory:").expect("Could not open connection");
}
