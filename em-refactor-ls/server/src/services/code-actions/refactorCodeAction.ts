import {
    CodeAction,
    CodeActionKind,
    Command,
    Range,
    TextDocument,
} from 'vscode-languageserver';

import { config, listRefactorings } from '../mappings';
import { ByteRange, LSPExtensionSettings, RefactorArgs } from '../../models';

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
 * Lists available refactorings as commands
 */
export function listRefactorCodeActions(doc: TextDocument, range: Range, settings: LSPExtensionSettings): (Command | CodeAction)[] {
    const byteRange = ByteRange.fromRange(range, doc);
    if (!byteRange.isRange() || byteRange.isEmpty()) {
        return [];
    }
    const refactorings = listRefactorings(settings.isMicroRefactoringsShown);
    return refactorings.map(r => mapToCodeAction(byteRange, r, doc, false))
        .concat(settings.isUnsafeRefactoringShown ? refactorings.map(r => mapToCodeAction(byteRange, r, doc, true)) : [])
}
