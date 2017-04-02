use std::str::ParseBoolError;
use xdg::BaseDirectoriesError;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    errors {
        NoMatchFound
        InvalidLocale
        UnknownEntryKey
        MissingRequiredEntryKey
        TypeNotApplication
    }
    foreign_links {
        InvalidValueFormat(ParseBoolError);
        XdgError(BaseDirectoriesError);
    }
}
