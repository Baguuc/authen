use authen::model::comma_separated_vec::CommaSeparatedVec;


#[tokio::test]
async fn vec_splits_the_string() {
    let original_string = String::from("id,email,password_hash");
    let vec = CommaSeparatedVec::parse(original_string);

    assert_eq!(vec.as_ref().len(), 3)
}