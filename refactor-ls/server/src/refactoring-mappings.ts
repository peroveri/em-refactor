import {
    CodeAction,
    CodeActionKind,
    Command,
    Range,
    TextDocument,
    TextDocumentEdit,
    TextDocuments,
    TextEdit,
    WorkspaceFolder,
    ApplyWorkspaceEditParams,
} from 'vscode-languageserver';

import config from './config';

export interface RefactorArgs {
    file: string;
    version: number;
    refactoring: string;
    selection: string;
    unsafe: boolean;
}
class ByteRange {
    constructor(public start: Number, public end: Number) { }
    isRange = () => this.start >= 0 && this.end >= 0;
    isEmpty = () => this.start === this.end;
    toArgumentString = () => `${this.start}:${this.end}`;
    static Empty = () => new ByteRange(0, 0);
    static Null = () => new ByteRange(-1, -1);
    static fromRange(range: Range, doc: TextDocument): ByteRange {
        const hasSelection = range && range.start && range.end;
        if (!hasSelection || doc === undefined) return this.Null();

        if (range.start.character === range.end.character && range.start.line === range.end.line) return this.Empty();
        return new ByteRange(doc.offsetAt(range.start), doc.offsetAt(range.end))
    }
}
interface Change {
    file_name: string;
    start: number;
    end: number;
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
        mapToCodeAction(byteRange, 'extract-function', doc, false),
        mapToCodeAction(byteRange, 'introduce-closure', doc, false),
        mapToCodeAction(byteRange, 'box-field', doc, true),
        mapToCodeAction(byteRange, 'extract-block', doc, true),
        mapToCodeAction(byteRange, 'extract-function', doc, true),
        mapToCodeAction(byteRange, 'introduce-closure', doc, true),
    ];
}

const concatUris = (uri: string, relativePath: string) =>
    uri + "/" + relativePath; // TODO: combine properly

const mapChange = (doc: TextDocument | undefined, change: Change): TextEdit => {
    if (doc === undefined) throw "document was undefined"; // doc shouldn't be undefined here
    return {
        newText: change.replacement,
        range: {
            start: doc.positionAt(change.start),
            end: doc.positionAt(change.end)
        }
    };
};

const mapDocumentChanges = (changes: Change[], workspaceUri: string, documents: TextDocuments): TextDocumentEdit[] =>
    changes.map(change => ({
        doc: documents.get(concatUris(workspaceUri, change.file_name)),
        change: change
    }))
        .filter(e => e.doc !== undefined)
        .map(edit => ({
            edits: [
                mapChange(edit.doc, edit.change)
            ],
            textDocument: {
                uri: (edit.doc ? edit.doc.uri : ''),
                version: null
            }
        } as TextDocumentEdit));

export const mapRefactorResultToWorkspaceEdit = (arg: RefactorArgs, stdout: string, workspaceUri: string, documents: TextDocuments): ApplyWorkspaceEditParams => ({
    edit: {
        documentChanges: mapDocumentChanges(JSON.parse(stdout) as [Change], workspaceUri, documents)
    },
    label: arg.refactoring
});

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

export const convertToCmd = (relativeFilePath: string, refactoring: string, selection: string, new_fn: string | null, unsafe: boolean) => {
    const refactorToolManifestPath = config.refactorToolManifestPath; // TODO: hardcoded path to refactoring project
    const refactorArgs = `--output-changes-as-json --file=${relativeFilePath} --refactoring=${refactoring} --selection=${selection}` + (new_fn === null ? '' : ` --new_function=${new_fn}`) + (unsafe ? ' --unsafe' : '');

    // The +nightly version should match the one used in the refactoring crate
    return `cargo +nightly-2019-10-23 run --bin cargo-my-refactor --manifest-path=${refactorToolManifestPath} -- ${refactorArgs}`;
}
