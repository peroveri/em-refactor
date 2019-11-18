/*
 * Test that a refactoring is found.
 * Currently `cargo clean` must be run in the testFixture folder before running this test because of how cargo check works.
*/

import * as vscode from 'vscode';
import * as assert from 'assert';
import { getDocUri, activate } from './helper';
import { ApplyWorkspaceEditParams, TextDocumentEdit } from 'vscode-languageclient';

describe('Should extract a block', () => {
	const docUri = getDocUri('./src/main.rs');

	it('extracts', async () => {
		await testCompletion(docUri, new vscode.Position(16, 40), {
			edit: {
				documentChanges: [
					{
						edits: [
							{
								newText: 'let s = \n{\nlet s = "Hello, world!";\ns};',
								range: {
									end: {
										character: 28,
										line: 1
									},
									start: {
										character: 4,
										line: 1
									}
								}
							}
						],
						textDocument: {
							uri: 'file:///home/perove/dev/github.uio.no/refactor-rust/refactor-ls/client/testFixture/src/main.rs',
							version: null
						}
					}
				]
			}, label: 'extract-block'
		});
	});
});

function assertTextDocumentEditEquals(actual: TextDocumentEdit, expected: TextDocumentEdit) {

}

function assertWorkspaceEditsEquals(actual: ApplyWorkspaceEditParams, expected: ApplyWorkspaceEditParams) {

	assert.equal(actual.edit.changes, expected.edit.changes);
	assert.equal(actual.edit.documentChanges.length, expected.edit.documentChanges.length);
	let actualChange = actual
	assert.equal(actual.edit.documentChanges.length, expected.edit.documentChanges.length);
	assert.equal(actual.label, expected.label);
}

async function testCompletion(
	docUri: vscode.Uri,
	position: vscode.Position,
	expectedWorkspaceEdit: ApplyWorkspaceEditParams
) {
	await activate(docUri);

	// Executing the command `vscode.executeCompletionItemProvider` to simulate triggering completion
	const actualWorkspaceEdit = (await vscode.commands.executeCommand(
		'refactor.extract.function',
		{
			file: docUri.toString(),
			version: null,
			selection: `${position.line}:${position.character}`,
			refactoring: "extract-block",
			unsafe: false
		}
	)) as ApplyWorkspaceEditParams;

	// assert.fail(JSON.stringify(actualWorkspaceEdit));
	// assert.fail(JSON.stringify(expectedWorkspaceEdit));

	// assert.eq(JSON.stringify(actualWorkspaceEdit), JSON.stringify(expectedWorkspaceEdit));
	assert.deepStrictEqual(actualWorkspaceEdit, expectedWorkspaceEdit);
}
