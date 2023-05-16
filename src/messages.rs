use itertools::Itertools;

pub fn creat_review_group_init_message(testers: &[String], publisher: &String) -> String {
    format!(
        r#"I created a new group to help you publish your app.
The testers [{}] will test your app and the publisher [{publisher}] will eventually publish it to the appstore
    "#,
        testers.iter().join(",")
    )
}

pub fn appstore_message() -> &'static str {
    r#"Welcome to the appstore bot! 
I will shortly send you the appstore itself wher you can explore new apps."#
}
