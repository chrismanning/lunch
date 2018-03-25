use std::str::ParseBoolError;
use xdg::BaseDirectoriesError;

use super::exec::FieldCode;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    errors {
        NoMatchFound(term: String) {
            description("Match not found")
            display("No match found for search term '{}'", term)
        }

        InvalidLocale(locale: String) {
            description("Error interpreting system locale")
            display("Invalid locale '{}'", locale)
        }

        UnknownEntryKey(key: String) {
            description("Error parsing Desktop Entry")
            display("Unknown entry key '{}'", key)
        }

        MissingRequiredEntryKey(required_key: String) {
            description("Error parsing Desktop Entry")
            display("Missing required entry key '{}'", required_key)
        }

        TypeNotApplication

        ApplicationNotFound

        UnexpectedArgType(field_code: FieldCode) {
            description("Error parsing Exec args")
            display("Field code '{:?}' not expected in Exec args", field_code)
        }

        InvalidArgs(args: Vec<String>, field_codes: Vec<FieldCode>) {
            description("")
            display("")
        }

        InvalidCommandLine(cmd_line: String) {
            description("Unable to parse command line")
            display("Exec string '{}' not valid", cmd_line)
        }

        NotDesktopEnvironment

        NoGroupsFound

        UnknownError
    }
    foreign_links {
        InvalidValueFormat(ParseBoolError);
        XdgError(BaseDirectoriesError);
        Io(::std::io::Error);
        Fst(::fst::Error);
        FstLevenshtein(::fst_levenshtein::Error);
    }
}
