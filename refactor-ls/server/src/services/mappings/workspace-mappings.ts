import { Range, ApplyWorkspaceEditParams, TextDocumentEdit, TextEdit, WorkspaceFolder } from "vscode-languageserver";
import { RefactorArgs } from ".";
import * as path from "path";

export interface RefactorOutputs {
    candidates: any[];
    changes: Change[][];
    errors: RefactorError[];
}
interface Change {
    byte_end: number;
    byte_start: number;
    char_end: number;
    char_start: number;
    file_name: string;
    line_end: number;
    line_start: number;
    replacement: string;
}
interface RefactorError {
    is_error: boolean;
    kind: string;
    message: string;
    codes: string[];
}

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

export class WorkspaceFolderInfo {
    constructor(public uri: string) { }

    getFileRelativePath(fileUri: string) {
        return path.relative(this.uri, fileUri);
    }
    join(otherPath: string) {
        return path.join(this.uri, otherPath);
    }

    static map(folders: WorkspaceFolder[] | null) {
        if (folders === null || folders.length <= 0 || !folders[0].uri) {
            return undefined;
        }
        return new WorkspaceFolderInfo(folders[0].uri);
    }
}
