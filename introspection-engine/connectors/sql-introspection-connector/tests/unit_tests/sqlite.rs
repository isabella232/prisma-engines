use crate::*;
use barrel::types;
use test_harness::*;

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_simple_table_with_gql_types_must_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("Blog", |t| {
                t.add_column("bool", types::boolean());
                t.add_column("float", types::float());
                t.add_column("date", types::date());
                t.add_column("id", types::primary());
                t.add_column("int", types::integer());
                t.add_column("string", types::text());
            });
        })
        .await;
    let dm = r#"
            model Blog {
                bool    Boolean
                date    DateTime
                float   Float
                id      Int @id
                int     Int 
                string  String
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_compound_primary_keys_must_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("Blog", |t| {
                t.add_column("id", types::integer());
                t.add_column("authorId", types::text());
                t.inject_custom("PRIMARY KEY (\"id\", \"authorId\")");
            });
        })
        .await;

    let dm = r#"
            model Blog {
                authorId String
                id Int
                @@id([id, authorId])
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_unique_index_must_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("Blog", |t| {
                t.add_column("id", types::primary());
                t.add_column("authorId", types::text());
            });
            migration.inject_custom(format!(
                "Create Unique Index \"{}\".\"test\" on \"Blog\"( \"authorId\")",
                api.schema_name()
            ));
        })
        .await;

    let dm = r#"
            model Blog {
                authorId String @unique
                id Int @id
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_multi_column_unique_index_must_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("firstname", types::text());
                t.add_column("lastname", types::text());
            });
            migration.inject_custom(format!(
                "Create Unique Index \"{}\".\"test\" on \"User\"( \"firstname\", \"lastname\")",
                api.schema_name()
            ));
        })
        .await;

    let dm = r#"
            model User {
                firstname String
                id Int @id
                lastname String
                @@unique([firstname, lastname], name: "test")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_required_and_optional_columns_must_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("requiredname", types::text().nullable(false));
                t.add_column("optionalname", types::text().nullable(true));
            });
        })
        .await;

    let dm = r#"
            model User {
                id Int @id
                optionalname String?
                requiredname String
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

//#[test_one_connector(connector = "sqlite")]
//#[ignore]
//fn introspecting_a_table_with_datetime_default_values_should_work(api: &TestApi) {
//    let barrel = api.barrel();
//    let _setup_schema = barrel.execute(|migration| {
//        migration.create_table("User", |t| {
//            t.add_column("id", types::primary());
//            t.add_column("name", types::text());
//            t.inject_custom("\"joined\" date DEFAULT CURRENT_DATE")
//        });
//    });
//
//    let dm = r#"
//            model User {
//                id Int @id
//                joined DateTime? @default(now())
//                name String
//            }
//        "#;
//    let result = dbg!(api.introspect().await);
//    custom_assert(&result, dm);
//}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_default_values_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("a", types::text());
                t.add_column("id", types::primary());
                t.inject_custom("\"bool\" Boolean NOT NULL DEFAULT false");
                t.inject_custom("\"bool2\" Boolean NOT NULL DEFAULT 0");
                t.inject_custom("\"float\" Float NOT NULL DEFAULT 5.3");
                t.inject_custom("\"int\" INTEGER NOT NULL DEFAULT 5");
                t.inject_custom("\"string\" TEXT NOT NULL DEFAULT \"Test\"");
            });
        })
        .await;

    let dm = r#"
            model User {
                a String
                bool Boolean @default(false)
                bool2 Boolean @default(false)
                float Float @default(5.3)
                id Int @id
                int Int @default(5)
                string String @default("Test")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_a_non_unique_index_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("a", types::text());
                t.add_column("id", types::primary());
            });
            migration.inject_custom(format!(
                "Create Index \"{}\".\"test\" on \"User\"(\"a\")",
                api.schema_name()
            ));
        })
        .await;

    let dm = r#"
            model User {
                a String
                id Int @id
                @@index([a], name: "test")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_table_with_a_multi_column_non_unique_index_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("a", types::text());
                t.add_column("b", types::text());
                t.add_column("id", types::primary());
            });
            migration.inject_custom(format!(
                "Create Index \"{}\".\"test\" on \"User\"(\"a\",\"b\")",
                api.schema_name()
            ));
        })
        .await;

    let dm = r#"
            model User {
                a String
                b String
                id Int @id
                @@index([a,b], name: "test")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

//relations
#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_one_to_one_req_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_id INTEGER NOT NULL UNIQUE,
                        FOREIGN KEY(user_id) REFERENCES User(id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               post Post? 
            }
            
            model Post {
               id Int @id
               user_id User
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_two_one_to_one_relations_between_the_same_models_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "post_id INTEGER NOT NULL UNIQUE,
                        FOREIGN KEY(post_id) REFERENCES Post(id)",
                )
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_id INTEGER NOT NULL UNIQUE,
                        FOREIGN KEY(user_id) REFERENCES User(id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               post_id Post  @relation("PostToUser_post_id")
               post Post?    @relation("Post_user_idToUser")
            }
            
            model Post {
               id Int @id
               user_id User  @relation("Post_user_idToUser")
               user    User? @relation("PostToUser_post_id", references: [post_id])
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_one_to_one_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_id INTEGER UNIQUE,
                        FOREIGN KEY(user_id) REFERENCES User(id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               post Post? 
            }
            
            model Post {
               id Int @id
               user_id User?
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_one_to_one_relation_referencing_non_id_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("email varchar(10) UNIQUE");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_email varchar(10) UNIQUE,
                FOREIGN KEY (user_email) REFERENCES User(email)",
                );
            });
        })
        .await;
    let dm = r#"
            model User {
               email        String? @unique 
               id           Int     @id 
               post         Post? 
            }
            
            model Post {
               id           Int     @id  
               user_email   User?   @relation(references: [email])
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_one_to_many_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_id INTEGER,
                        FOREIGN KEY(user_id) REFERENCES User(id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               posts Post[] 
            }
            
            model Post {
               id Int @id
               user_id User?
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_one_req_to_many_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_id INTEGER NOT NULL,
                        FOREIGN KEY(user_id) REFERENCES User(id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               posts Post[] 
            }
            
            model Post {
               id Int @id
               user_id User
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_prisma_many_to_many_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("_PostToUser", |t| {
                t.inject_custom(
                    "A TEXT NOT NULL,
                          B TEXT NOT NULL,
                          FOREIGN KEY (A) REFERENCES  Post(id) ON DELETE CASCADE,
                          FOREIGN KEY (B) REFERENCES  User(id) ON DELETE CASCADE",
                )
            });
            migration.inject_custom(format!(
                "CREATE UNIQUE INDEX \"{}\".test ON \"_PostToUser\" (\"A\", \"B\");",
                api.schema_name()
            ))
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               posts Post[] 
            }
            
            model Post {
               id Int @id
               users User[] 
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

// Todo
#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_many_to_many_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("PostsToUsers", |t| {
                t.inject_custom(
                    "user_id TEXT NOT NULL,
                          post_id TEXT NOT NULL,
                          FOREIGN KEY (user_id) REFERENCES  User(id) ON DELETE CASCADE,
                          FOREIGN KEY (post_id) REFERENCES  Post(id) ON DELETE CASCADE",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               postsToUserses PostsToUsers[] @relation(onDelete: CASCADE)
            }
            
            model Post {
               id Int @id
               postsToUserses PostsToUsers[] @relation(references: [post_id], onDelete: CASCADE)
            }
            
            model PostsToUsers {
              post_id Post 
              user_id User
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_many_to_many_relation_with_extra_fields_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("PostsToUsers", |t| {
                t.inject_custom(
                    "date    date,
                          user_id TEXT NOT NULL,
                          post_id TEXT NOT NULL,
                          FOREIGN KEY (user_id) REFERENCES  User(id),
                          FOREIGN KEY (post_id) REFERENCES  Post(id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
               id Int @id
               postsToUserses PostsToUsers[] 
            }
            
            model Post {
               id Int @id
               postsToUserses PostsToUsers[] @relation(references: [post_id])
            }
            
            model PostsToUsers {
              date    DateTime?
              post_id Post 
              user_id User
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_a_self_relation_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "recruited_by INTEGER,
                          direct_report INTEGER,
                          FOREIGN KEY (recruited_by) REFERENCES User (id),
                          FOREIGN KEY (direct_report) REFERENCES User (id)",
                )
            });
        })
        .await;

    let dm = r#"
            model User {
                direct_report                  User?  @relation("UserToUser_direct_report")
                id                             Int    @id
                recruited_by                   User?  @relation("UserToUser_recruited_by")
                users_UserToUser_direct_report User[] @relation("UserToUser_direct_report")
                users_UserToUser_recruited_by  User[] @relation("UserToUser_recruited_by")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

// on delete cascade

#[test_one_connector(connector = "sqlite")]
async fn introspecting_cascading_delete_behaviour_should_work(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom(
                    "user_id INTEGER,\
                     FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE",
                );
            });
        })
        .await;

    let dm = r#"
            model User {
               id      Int @id
               posts Post[] @relation(onDelete: CASCADE)
            }
            
            model Post {
               id      Int @id
               user_id User?
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_one_connector(connector = "sqlite")]
async fn introspecting_id_fields_with_foreign_key_should_ignore_the_relation(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("test", types::text());
                t.inject_custom("user_id INTEGER Not Null Primary Key");
                t.inject_custom("FOREIGN KEY (`user_id`) REFERENCES `User`(`id`)");
            });
        })
        .await;

    let dm = r#"
            model User {
               id      Int @id
            }

            model Post {
               test    String
               user_id Int @id
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}
