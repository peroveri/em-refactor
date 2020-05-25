import {
    CodeAction,
    CodeActionKind,
    Command,
    Range,
    TextDocument,
} from 'vscode-languageserver';

import { ByteRange, config } from '.';

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
    title: `Refactor - ${refactoring}` + (unsafe ? ' - unsafe' : ''),
    command: {
        title: 'refactor',
        command: config.refactorCommand,
        arguments: [mapToRefactorArgs(doc, range, refactoring, unsafe)]
    },
    kind: CodeActionKind.Refactor
});

/**
 * TODO: Query the refactoring tool for possible refactorings at a given range.
 */
export function listRefactorCodeActions(doc: TextDocument, range: Range, refactorings: string[], isUnsafeRefactoringShown: boolean): (Command | CodeAction)[] {
    const byteRange = ByteRange.fromRange(range, doc);
    if (!byteRange.isRange() || byteRange.isEmpty()) {
        return [];
    }
    return refactorings.map(r => mapToCodeAction(byteRange, r, doc, false))
        .concat(isUnsafeRefactoringShown ? refactorings.map(r => mapToCodeAction(byteRange, r, doc, true)) : [])
}
