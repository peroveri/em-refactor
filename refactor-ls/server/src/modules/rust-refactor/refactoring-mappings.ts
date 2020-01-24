import {
    CodeAction,
    CodeActionKind,
    Command,
    Range,
    TextDocument,
} from 'vscode-languageserver';

import { ByteRange } from '../';

export interface RefactorArgs {
    file: string;
    version: number;
    refactoring: string;
    selection: string;
    unsafe: boolean;
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
export function listActionsForRange(doc: TextDocument, range: Range, refactorings: string[]): (Command | CodeAction)[] {

    const byteRange = ByteRange.fromRange(range, doc);
    if (!byteRange.isRange() || byteRange.isEmpty()) {
        return [];
    }

    return refactorings.map(r => mapToCodeAction(byteRange, r, doc, false))
        .concat(refactorings.map(r => mapToCodeAction(byteRange, r, doc, true)));
}
