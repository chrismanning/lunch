use std::str::ParseBoolError;
// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    errors {
        InvalidLocale
        UnknownEntryKey
        MissingRequiredEntryKey
    }
    foreign_links {
        InvalidValueFormat(ParseBoolError);
    }
}
