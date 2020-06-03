import { CodeAction, CodeActionParams, Command, TextDocument, TextDocumentIdentifier, CodeActionKind } from 'vscode-languageserver';
import { ByteRange, GenerateTestFileArgs, config } from "../../../models";

const getDocName = (doc: TextDocumentIdentifier): string =>
    doc.uri.substring(doc.uri.lastIndexOf("/") + 1, doc.uri.lastIndexOf("."));

export const mapToCodeAction = (params: CodeActionParams, refactoring: string, should_fail: boolean, selection: string): CodeAction => ({
    title: `Generate ${getDocName(params.textDocument)}.json for ${refactoring} ${should_fail ? ' (failing)' : ''}`,
    command: {
        title: 'generate',
        command: config.generateTestJsonCommand,
        arguments: [{
            file_uri: params.textDocument.uri,
            refactoring,
            selection,
            should_fail
        } as GenerateTestFileArgs]
    },
    kind: CodeActionKind.Refactor
});

export const generateJsonCodeActions = (refactorings: string[], document: TextDocument, params: CodeActionParams): (Command | CodeAction)[] => {
    let byteRange = ByteRange.fromRange(params.range, document);
    if (!byteRange.isRange()) {
        return [];
    }
    return refactorings
        .map(refactoring => mapToCodeAction(params, refactoring, true, byteRange.toArgumentString()))
        .concat(refactorings
            .map(refactoring => mapToCodeAction(params, refactoring, false, byteRange.toArgumentString())));
}
