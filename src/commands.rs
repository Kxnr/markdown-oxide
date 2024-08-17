use std::fs::File;
use std::path::Path;

use crate::config::Settings;
use chrono::offset::Local;
use chrono::NaiveDateTime;
use fuzzydate::parse;
use serde_json::Value;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::{MessageType, ShowDocumentParams, Url};

fn datetime_to_file(
    datetime: NaiveDateTime,
    dailynote_format: &str,
    root_dir: &Path,
) -> Option<Url> {
    let filename = datetime.format(dailynote_format).to_string();
    let path = root_dir.join(&filename);

    Url::from_file_path(path.with_extension("md")).ok()
}

pub async fn note(
    client: &tower_lsp::Client,
    root_dir: &Path,
    settings: &Settings,
    notebook: &str,
    date_str: Option<&str>,
) -> Result<Option<Value>> {
    // if date_str is None, use the current time.

    let notebook = settings
        .notebooks
        .get(notebook)
        .ok_or(Error::invalid_params("Notebook does not exist"))?;
    let note_format = &notebook.note_format;
    let note_path = root_dir.join(&notebook.folder);
    let note_file = match date_str {
        Some(date_str) => parse(date_str)
            .ok()
            .and_then(|dt| datetime_to_file(dt, &note_format, &note_path)),
        None => datetime_to_file(Local::now().naive_local(), &note_format, &note_path),
    };

    if let Some(uri) = note_file {
        // file creation can fail and return an Err, ignore this and try
        // to open the file on the off chance the client knows what to do
        // TODO: log failure to create file
        let _ = uri.to_file_path().map(|path| {
            path.parent().map(|parent| std::fs::create_dir_all(parent));

            let _ = File::create_new(path.as_path().to_owned());
        });

        client
            .show_document(ShowDocumentParams {
                uri,
                external: Some(false),
                take_focus: Some(true),
                selection: None,
            })
            .await
            .map(|success| Some(success.into()))
    } else {
        client
            .log_message(
                MessageType::ERROR,
                format!("could not parse {date_str:?}: {:?}", date_str.map(parse)),
            )
            .await;
        Err(Error::invalid_params(format!(
            "Could not parse journal format ({note_format:?}) as a valid uri: {:?}.",
            note_format
        )))
    }
}

// tests
#[cfg(test)]
mod tests {
    use fuzzydate::parse;

    use super::datetime_to_file;

    #[test]
    fn test_string_to_file() {
        let input = "today";

        let parsed_datetime = parse(input).unwrap();

        let _ = datetime_to_file(
            parsed_datetime,
            "%Y-%m-%d",
            &std::fs::canonicalize("./").unwrap(),
        )
        .unwrap();
    }
}
