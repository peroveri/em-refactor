import { Range, ApplyWorkspaceEditParams, TextDocumentEdit, TextEdit, WorkspaceFolder } from "vscode-languageserver";
import { Change, RefactorArgs, RefactorOutputs } from "../../../models";
import * as path from "path";

const mapRange = (change: Change): Range =>
    Range.create(change.line_start, change.char_start, change.line_end, change.char_end);

export const mapRefactorResultToWorkspaceEdits = (arg: RefactorArgs, outputs: RefactorOutputs, workspaceUri: string): ApplyWorkspaceEditParams[] => {
    let edits = [];
    for (const changes of outputs.changes) {
        let documentChanges: TextDocumentEdit[] = [];

        for (const change of changes) {
            let uri = path.join(workspaceUri, change.file_name);
            let documentChange = documentChanges.find(e => e.textDocument.uri === uri);
            if (documentChange === undefined) {
                documentChange = TextDocumentEdit.create({
                    uri,
                    version: null
                }, []);
                documentChanges.push(documentChange);
            }
            documentChange.edits.push(TextEdit.replace(mapRange(change), change.replacement));
        }

        edits.push({
            edit: {
                documentChanges
            },
            label: arg.refactoring
        } as ApplyWorkspaceEditParams)
    }
    return edits;
}
