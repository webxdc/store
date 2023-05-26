use itertools::Itertools;

pub fn creat_review_group_init_message(testers: &[String], publisher: &String) -> String {
    format!(
        "Publisher: [{publisher}] \n Testers: [{}]",
        testers.iter().join(",")
    )
}

pub fn appstore_message() -> &'static str {
    r#"Welcome to the appstore bot! 
I will shortly send you the appstore itself, where you can explore new apps."#
}
