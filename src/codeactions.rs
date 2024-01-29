use std::{path::{Path, PathBuf}, iter};

use pathdiff::diff_paths;
use rayon::iter::*;
use tower_lsp::lsp_types::{CodeActionParams, CodeActionOrCommand, CodeActionResponse, Diagnostic, DiagnosticSeverity, CodeAction, WorkspaceEdit, DocumentChanges, DocumentChangeOperation, ResourceOp, CreateFile, Url, CodeActionKind};

use crate::vault::{Vault, self};

pub fn code_actions(vault: &Vault, params: CodeActionParams, path: &Path) -> Option<Vec<CodeActionOrCommand>> {



    // Diagnostics
    // get all links for changed file
    let Some(pathreferences) = vault.select_references(Some(path)) else {
        return None;
    };
    let Some(allreferences) = vault.select_references(None) else {
        return None;
    };

    let referenceables = vault.select_referenceable_nodes(None);
    let unresolved_file_links = pathreferences.par_iter().filter(|(path, reference)| {
        !referenceables
            .iter()
            .any(|referenceable| referenceable.matches_reference(vault.root_dir(), reference, path))
        && !reference.data().reference_text.contains("#")
        && reference.data().range.start.line == params.range.start.line
        && reference.data().range.start.character <= params.range.start.character
        && reference.data().range.end.character >= params.range.end.character
        // TODO: Extract this to a match condition
    });


    Some(
        unresolved_file_links
            .filter_map(|(path, reference)| {


                let mut new_path_buf = PathBuf::new();
                new_path_buf.push(vault.root_dir());
                new_path_buf.push(&reference.data().reference_text);
                new_path_buf.set_extension("md");

                let new_path = Url::from_file_path(&new_path_buf).ok()?;

                Some(CodeActionOrCommand::CodeAction(
                    CodeAction {
                        title: format!("Create File: {:?}", diff_paths(new_path_buf, vault.root_dir())?),
                        edit: Some(WorkspaceEdit {
                            document_changes: Some(DocumentChanges::Operations(vec![
                                DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                                uri: new_path,
                                options: None,
                                annotation_id: None
                                }))
                            ])),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                ))

            })
            .collect())

}