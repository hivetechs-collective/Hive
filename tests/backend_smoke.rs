use hive_ai::core::database::generate_id;

#[test]
fn generates_unique_ids() {
    let first = generate_id();
    let second = generate_id();

    assert_ne!(first, second);
    assert!(first.len() > 0);
    assert!(second.len() > 0);
}
