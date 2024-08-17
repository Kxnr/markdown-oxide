use chrono::format::{parse_and_remainder, Parsed, StrftimeItems};

use crate::config::{Notebook, Settings};

impl Notebook {
    fn match_filename(&self, filename: &str) -> bool {
        // TODO: support non-strftime notebooks
        // Use Parsed directly to support formats that don't uniquely identify a date, like weekly
        // or monthly notes
        let items = StrftimeItems::new(&self.note_format)
            .parse()
            .expect("note format must be a valid strftime string");
        let mut parsed = Parsed::new();
        let parse_result = parse_and_remainder(&mut parsed, filename, items.iter());
        parse_result.is_ok()
    }
}

pub fn match_notebook<'a>(context: &'a Settings, filename: &str) -> Option<&'a Notebook> {
    for notebook in context.notebooks.values() {
        if notebook.match_filename(filename) {
            return Some(notebook);
        }
    }
    None
}
