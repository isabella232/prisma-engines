use crate::common::*;
use datamodel::ast::Span;
use datamodel::dml;
use datamodel::error::DatamodelError;

#[test]
fn relation_happy_path() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        user User @relation(fields: [userId], references: [id])
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model
        .assert_has_field("posts")
        .assert_arity(&dml::FieldArity::List)
        .assert_relation_to("Post")
        .assert_relation_base_fields(&[])
        .assert_relation_to_fields(&[]);

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_has_field("user")
        .assert_arity(&dml::FieldArity::Required)
        .assert_relation_to("User")
        .assert_relation_base_fields(&["userId"])
        .assert_relation_to_fields(&["id"]);
}

#[test]
fn relation_must_error_when_base_field_does_not_exist() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String        
        user User @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument fields must refer only to existing fields. The following fields do not exist in this model: userId", Span::new(162, 217)));
}

#[test]
fn relation_must_error_when_base_field_is_not_scalar() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        otherId Int        
        
        user User @relation(fields: [other], references: [id])
        other OtherModel @relation(fields: [otherId], references: [id])
    }
    
    model OtherModel {
        id Int @id
        posts Post[]
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(0,DatamodelError::new_validation_error("The argument fields must refer only to scalar fields. But it is referencing the following relation fields: other", Span::new(210, 264)));
    errors.assert_is_at(1,DatamodelError::new_directive_validation_error("The type of the field `other` in the model `Post` is not matching the type of the referenced field `id` in model `User`.","@relation", Span::new(210, 264)));
}

#[test]
fn optional_relation_field_must_succeed_when_all_underlying_fields_are_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String?
        lastName  String?
        posts     Post[]
        
        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String?
        userLastName  String?
          
        user          User?   @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    // must not crash
    let _ = parse(dml);
}

#[test]
fn optional_relation_field_must_error_when_one_underlying_field_is_required() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String
        lastName  String?
        posts     Post[]
        
        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String
        userLastName  String?
          
        user          User?   @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The relation field `user` uses the scalar fields userFirstName, userLastName. At least one of those fields is required. Hence the relation field must be required as well.", Span::new(338, 443)));
}

#[test]
fn required_relation_field_must_succeed_when_at_least_one_underlying_fields_is_required() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String
        lastName  String?
        posts     Post[]
        
        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String
        userLastName  String?
          
        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    // must not crash
    let _ = parse(dml);
}

#[test]
fn required_relation_field_must_error_when_all_underlying_fields_are_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String?
        lastName  String?
        posts     Post[]
        
        @@unique([firstName, lastName])
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String?
        userLastName  String?
          
        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The relation field `user` uses the scalar fields userFirstName, userLastName. All those fields are optional. Hence the relation field must be optional as well.", Span::new(340, 445)));
}

#[test]
fn relation_must_error_when_referenced_field_does_not_exist() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int        
        user User @relation(fields: [userId], references: [fooBar])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer only to existing fields in the related model `User`. The following fields do not exist in the related model: fooBar", Span::new(181, 240)));
}

#[test]
fn relation_must_error_when_referenced_field_is_not_scalar() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int        
        user User @relation(fields: [userId], references: [posts])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer only to scalar fields in the related model `User`. But it is referencing the following relation fields: posts", Span::new(181, 239)));
}

#[test]
fn relation_must_error_when_referenced_fields_are_not_a_unique_criteria() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userName Int        
        user User @relation(fields: [userName], references: [firstName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer to a unique criteria in the related model `User`. But it is referencing the following fields that are not a unique criteria: firstName", Span::new(183, 247)));
}

#[test]
fn relation_must_error_when_referenced_fields_are_multiple_uniques() {
    let dml = r#"
    model User {
        id Int @id
        firstName String @unique
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userName Int        
        // the relation is referencing two uniques. That is too much.
        user User @relation(fields: [userName], references: [id, firstName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer to a unique criteria in the related model `User`. But it is referencing the following fields that are not a unique criteria: id, firstName", Span::new(191, 329)));
}

#[test]
fn relation_must_error_when_types_of_base_field_and_referenced_field_do_not_match() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId String  // this type does not match
        user   User    @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_directive_validation_error("The type of the field `userId` in the model `Post` is not matching the type of the referenced field `id` in model `User`.","@relation", Span::new(204, 264)));
}

#[test]
fn must_error_when_fields_argument_is_missing_for_one_to_many() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_directive_validation_error(
        "The relation field `user` on Model `Post` must specify the `fields` argument in the @relation directive.",
        "@relation",
        Span::new(172, 214),
    ));
}

#[test]
#[ignore]
fn must_error_when_references_argument_is_missing_for_one_to_many() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_directive_validation_error(
        "The relation field `user` on Model `Post` must specify the `references` argument in the @relation directive.",
        "@relation",
        Span::new(172, 214),
    ));
}

#[test]
#[ignore]
fn must_error_when_both_arguments_are_missing_for_one_to_many() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_directive_validation_error(
            "The relation field `user` on Model `Post` must specify the `fields` argument in the @relation directive.",
            "@relation",
            Span::new(172, 183),
        ),
    );
    errors.assert_is_at(1, DatamodelError::new_directive_validation_error(
        "The relation field `user` on Model `Post` must specify the `references` argument in the @relation directive.",
        "@relation",
        Span::new(172, 183),
    ));
}

#[test]
fn must_error_when_fields_argument_is_missing_for_one_to_one() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        post      Post
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_directive_validation_error(
            "The relation fields `post` on Model `User` and `user` on Model `Post` do not provide the `fields` argument in the @relation directive. You have to provide it on one of the two fields.", 
            "@relation", Span::new(77, 91)
        ),
    );
    errors.assert_is_at(
        1,
        DatamodelError::new_directive_validation_error(
            "The relation fields `user` on Model `Post` and `post` on Model `User` do not provide the `fields` argument in the @relation directive. You have to provide it on one of the two fields.", 
       "@relation", Span::new(170, 212)
        ),
    );
}

#[test]
#[ignore]
fn must_error_when_references_argument_is_missing_for_one_to_one() {
    let dml = r#"
    model User {
        id        Int @id
        firstName String
        post      Post
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(fields: [userId])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is_at(
        0,
        DatamodelError::new_directive_validation_error(
            "The relation fields `post` on Model `User` and `user` on Model `Post` do not provide the `references` argument in the @relation directive. You have to provide it on one of the two fields.", 
            "@relation", Span::new(77, 91)
        ),
    );
    errors.assert_is_at(
        1,
        DatamodelError::new_directive_validation_error(
            "The relation fields `user` on Model `Post` and `post` on Model `User` do not provide the `references` argument in the @relation directive. You have to provide it on one of the two fields.", 
            "@relation", Span::new(170, 212)
        ),
    );
}