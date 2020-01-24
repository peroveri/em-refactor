import { Range, ApplyWorkspaceEditParams, TextDocumentEdit, TextEdit, WorkspaceFolder } from "vscode-languageserver";
import { RefactorArgs } from "../../modules/"

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

const concatUris = (uri: string, relativePath: string) =>
    uri + "/" + relativePath; // TODO: combine properly

const mapRange = (change: Change): Range =>
    Range.create(change.line_start, change.char_start, change.line_end, change.char_end);

export const mapRefactorResultToWorkspaceEdit = (arg: RefactorArgs, stdout: string, workspaceUri: string): ApplyWorkspaceEditParams => {
    let changes = JSON.parse(stdout) as [Change];

    let documentChanges: TextDocumentEdit[] = [];

    for (const change of changes) {
        let uri = concatUris(workspaceUri, change.file_name);
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
    return {
        edit: {
            documentChanges
        },
        label: arg.refactoring
    } as ApplyWorkspaceEditParams;
}


export const getFileRelativePath = (fileUri: string, workspace: WorkspaceFolder[] | null) => {
    if (workspace === null || workspace.length === 0) return undefined;
    let workspaceUri = workspace[0].uri;
    return getRelativePath(workspaceUri, fileUri);
}

const getRelativePath = (workspaceUri: string, fileUri: string) => {
    if (fileUri.startsWith(workspaceUri)) {
        let sub = fileUri.substring(workspaceUri.length);
        if (sub.startsWith("/")) sub = sub.substring(1);
        return sub;
    }
    return undefined;
}
