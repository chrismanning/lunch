use super::freedesktop::find_all_desktop_files;

use super::Launch;
use super::Search;

trait Lunchable: Launch + Search {}

impl<T> Lunchable for T
where
    T: Launch + Search,
{
}

struct LunchEnv {
    lunchables: Vec<Box<Lunchable>>,
}

impl LunchEnv {
    pub fn new() -> LunchEnv {
        let desktop_files = find_all_desktop_files();
        unimplemented!()
    }

    pub fn keyword(&self, keyword: &str) -> Option<&Box<Lunchable>> {
        self.lunchables.iter().next()
    }
}
