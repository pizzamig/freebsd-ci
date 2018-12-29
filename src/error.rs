use derive_is_enum_variant::is_enum_variant;
use failure::Fail;

#[derive(Debug, Fail, is_enum_variant)]
pub(crate) enum ParseError {
    #[fail(display = "Invalid type for the key {}", name)]
    #[is_enum_variant(name = "is_invalidtype")]
    InvalidType { name: String },
    #[fail(display = "Missing key from the YAML: {}", key)]
    #[is_enum_variant(name = "is_missingkey")]
    MissingKey { key: String },
    #[fail(display = "Generic Error: {}", msg)]
    #[is_enum_variant(name = "is_genericerror")]
    GenericError { msg: String },
}
