use super::freedesktop::find_all_desktop_files;

use super::Launch;

struct LunchEnv {
    things: Vec<Box<Launch>>,
}

impl LunchEnv {
    #[cfg(feature = "freedesktop")]
    fn new() -> LunchEnv {

        unimplemented!()
    }

    fn from_cache() -> LunchEnv {
        unimplemented!()
    }
}
