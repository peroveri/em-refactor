import {
    CodeAction,
    CodeActionKind,
    Command,
    Range,
    TextDocument,
    TextDocumentEdit,
    TextEdit,
    WorkspaceFolder,
    ApplyWorkspaceEditParams,
} from 'vscode-languageserver';

import { ByteRange } from '../';

export interface RefactorArgs {
    file: string;
    version: number;
    refactoring: string;
    selection: string;
    unsafe: boolean;
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


const mapToRefactorArgs = (doc: TextDocument, range: ByteRange, refactoring: string, unsafe: boolean): RefactorArgs => ({
    file: doc.uri,
    version: doc.version,
    selection: range.toArgumentString(),
    refactoring,
    unsafe
});

const mapToCodeAction = (range: ByteRange, refactoring: string, doc: TextDocument, unsafe: boolean): CodeAction => ({
    title: `Refactor - ${refactoring}: ${range.toArgumentString()}` + (unsafe ? ' - unsafe' : ''),
    command: {
        title: 'refactor',
        command: CodeActionKind.RefactorExtract + '.function', // TODO: this should be something else
        arguments: [mapToRefactorArgs(doc, range, refactoring, unsafe)]
    },
    kind: CodeActionKind.RefactorExtract + '.function'
});

/**
 * TODO: Query the refactoring tool for possible refactorings at a given range.
 */
export function listActionsForRange(doc: TextDocument, range: Range): (Command | CodeAction)[] {

    const byteRange = ByteRange.fromRange(range, doc);
    if (!byteRange.isRange() || byteRange.isEmpty()) {
        return [];
    }

    return [
        mapToCodeAction(byteRange, 'box-field', doc, false),
        mapToCodeAction(byteRange, 'extract-block', doc, false),
        mapToCodeAction(byteRange, 'extract-method', doc, false),
        mapToCodeAction(byteRange, 'inline-macro', doc, false),
        mapToCodeAction(byteRange, 'introduce-closure', doc, false),
        mapToCodeAction(byteRange, 'box-field', doc, true),
        mapToCodeAction(byteRange, 'extract-block', doc, true),
        mapToCodeAction(byteRange, 'extract-method', doc, true),
        mapToCodeAction(byteRange, 'inline-macro', doc, true),
        mapToCodeAction(byteRange, 'introduce-closure', doc, true),
    ];
}

const concatUris = (uri: string, relativePath: string) =>
    uri + "/" + relativePath; // TODO: combine properly

const mapRange = (change: Change): Range =>
    Range.create(change.line_start, change.char_start, change.line_end, change.char_end);

export const mapRefactorResultToWorkspaceEdit = (arg: RefactorArgs, stdout: string, workspaceUri: string): ApplyWorkspaceEditParams => {
    let changes = JSON.parse(stdout) as [Change];

    let documentChanges: TextDocumentEdit[] = [];

    for(const change of changes) {
        let uri = concatUris(workspaceUri, change.file_name);
        let documentChange = documentChanges.find(e => e.textDocument.uri === uri);
        if(documentChange === undefined) {
            documentChange = TextDocumentEdit.create( {
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
