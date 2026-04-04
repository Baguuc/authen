use authen::utils::vec::detect_differences_in_vecs;

#[test]
fn function_detects_differences_in_vecs() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![3, 4, 5];
    let differences = detect_differences_in_vecs(v1, v2).unwrap();

    assert_eq!(differences.create.len(), 2);
    assert!(differences.create.contains(&4));
    assert!(differences.create.contains(&5));
    
    assert_eq!(differences.delete.len(), 2);
    assert!(differences.delete.contains(&1));
    assert!(differences.delete.contains(&2));
}

#[test]
fn function_returns_none_if_there_is_no_changes() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![1, 2, 3];
    let differences = detect_differences_in_vecs(v1, v2);
    
    assert!(differences.is_none());
}

#[test]
fn function_returns_none_if_the_vecs_are_empty() {
    let v1: Vec<()> = vec![];
    let v2: Vec<()> = vec![];
    let differences = detect_differences_in_vecs(v1, v2);
    
    assert!(differences.is_none());
}

#[test]
fn function_returns_only_create_when_v1_is_empty() {
    let v1 = vec![];
    let v2 = vec![1, 2, 3];
    let differences = detect_differences_in_vecs(v1, v2).unwrap();
    
    assert_eq!(differences.create.len(), 3);
    assert!(differences.create.contains(&1));
    assert!(differences.create.contains(&2));
    assert!(differences.create.contains(&3));

    assert_eq!(differences.delete.len(), 0);
}

#[test]
fn function_returns_only_delete_if_v2_is_empty() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![];
    let differences = detect_differences_in_vecs(v1, v2).unwrap();
    
    assert_eq!(differences.delete.len(), 3);
    assert!(differences.delete.contains(&1));
    assert!(differences.delete.contains(&2));
    assert!(differences.delete.contains(&3));

    assert_eq!(differences.create.len(), 0);
}

