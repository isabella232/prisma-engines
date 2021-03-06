use crate::common::*;
use datamodel::{ast, diagnostics::DatamodelError};

#[test]
fn shound_fail_on_attribute_duplication() {
    let dml = r#"
    type ID = String @id @default(cuid())

    model Model {
        id ID @id
    }
    "#;

    let error = parse_error(dml);

    error.assert_is_at(
        0,
        DatamodelError::new_duplicate_attribute_error("id", ast::Span::new(23, 25)),
    );
    error.assert_is_at(
        1,
        DatamodelError::new_duplicate_attribute_error("id", ast::Span::new(77, 79)),
    );
}

#[test]
fn shound_fail_on_attribute_duplication_recursive() {
    let dml = r#"
    type MyStringWithDefault = String @default(cuid())
    type ID = MyStringWithDefault @id

    model Model {
        id ID @default(cuid())
    }
    "#;

    let error = parse_error(dml);

    error.assert_is_at(
        0,
        DatamodelError::new_duplicate_attribute_error("default", ast::Span::new(40, 47)),
    );
    error.assert_is_at(
        1,
        DatamodelError::new_duplicate_attribute_error("default", ast::Span::new(128, 135)),
    );
}

#[test]
fn should_fail_on_endless_recursive_type_def() {
    let dml = r#"
    type MyString = ID
    type MyStringWithDefault = MyString
    type ID = MyStringWithDefault

    model Model {
        id ID
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "Recursive type definitions are not allowed. Recursive path was: ID -> MyStringWithDefault -> MyString -> ID.",
        ast::Span::new(21, 23),
    ));
}

#[test]
fn shound_fail_on_unresolvable_type() {
    let dml = r#"
    type MyString = Hugo
    type MyStringWithDefault = MyString
    type ID = MyStringWithDefault

    model Model {
        id ID
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_type_not_found_error("Hugo", ast::Span::new(21, 25)));
}

#[test]
fn should_fail_on_custom_related_types() {
    let dml = r#"
    type UserViaEmail = User @relation(references: email)
    type UniqueString = String @unique

    model User {
        id Int @id
        email UniqueString
        posts Post[]
    }

    model Post {
        id Int @id
        user UserViaEmail
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "Only scalar types can be used for defining custom types.",
        ast::Span::new(25, 29),
    ));
}

#[test]
fn should_fail_on_native_type_with_invalid_datasource_name() {
    let dml = r#"
        datasource db {
          provider = "postgres"
          url = "postgresql://"
        }

        generator client {
          provider = "prisma-client-js"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            bigInt Int    @pg.Integer
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "The prefix pg is invalid. It must be equal to the name of an existing datasource e.g. db. Did you mean to use db.Integer?",
        ast::Span::new(300, 310),
    ));
}

#[test]
fn should_fail_on_native_type_with_invalid_number_of_arguments() {
    let dml = r#"
        datasource pg {
          provider = "postgres"
          url = "postgresql://"
        }

        generator client {
          provider = "prisma-client-js"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            bigInt Int    @pg.Integer
            foobar String @pg.VarChar(2, 3, 4)
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "Native type VarChar takes 1 optional arguments, but received 3.",
        ast::Span::new(338, 357),
    ));
}

#[test]
fn should_fail_on_native_type_with_unknown_type() {
    let dml = r#"
        datasource pg {
          provider = "postgres"
          url = "postgresql://"
        }

        generator client {
          provider = "prisma-client-js"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            bigInt Int    @pg.Numerical(3, 4)
            foobar String @pg.VarChar(5)
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "Native type Numerical is not supported for postgresql connector.",
        ast::Span::new(300, 318),
    ));
}

#[test]
fn should_fail_on_native_type_with_incompatible_type() {
    let dml = r#"
        datasource pg {
          provider = "postgres"
          url = "postgresql://"
        }

        generator client {
          provider = "prisma-client-js"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            foobar Boolean @pg.VarChar(5)
            foo Int @pg.BigInt
        }
    "#;

    let error = parse_error(dml);

    error.assert_length(2);

    error.assert_is_at(
        0,
        DatamodelError::new_connector_error(
            "Native type VarChar is not compatible with declared field type Boolean, expected field type String.",
            ast::Span::new(301, 314),
        ),
    );

    error.assert_is_at(
        1,
        DatamodelError::new_connector_error(
            "Native type BigInt is not compatible with declared field type Int, expected field type BigInt.",
            ast::Span::new(336, 345),
        ),
    );
}

#[test]
fn should_fail_on_native_type_with_invalid_arguments() {
    let dml = r#"
        datasource pg {
          provider = "postgres"
          url = "postgresql://"
        }

        generator client {
          provider = "prisma-client-js"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            foobar String @pg.VarChar(a)
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "Expected a numeric value, but failed while parsing \"a\": invalid digit found in string.",
        ast::Span::new(300, 313),
    ));
}
