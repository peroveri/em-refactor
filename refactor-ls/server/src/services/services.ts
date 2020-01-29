import { container } from "tsyringe";
import {
    ProposedFeatures,
    createConnection,
    Connection,
    InitializeParams,
    TextDocuments,
    DidChangeConfigurationNotification,
} from 'vscode-languageserver';
import { CodeActionService } from "./CodeActionService";
import { listAllCommands, listAllCodeActionKinds } from "./mappings/code-action-mappings";

let hasConfigurationCapability: boolean = false;
let hasWorkspaceFolderCapability: boolean = false;
let hasCodeActionLiteralSupport: boolean = false;

export function bootstrapAndReturnConnection() {
    const [connection, documents] = initConnection();

    container.register<Connection>("Connection", { useValue: connection });
    container.register<TextDocuments>("TextDocuments", { useValue: documents });

    return connection;
}

function initConnection(): [Connection, TextDocuments] {
    let connection = createConnection(ProposedFeatures.all);

    // Create a simple text document manager. The text document manager
    // supports full document sync only
    let documents: TextDocuments = new TextDocuments();


    connection.onInitialized(() => {
        if (hasConfigurationCapability) {
            // Register for all configuration changes.
            connection.client.register(DidChangeConfigurationNotification.type, undefined);
        }
        if (hasWorkspaceFolderCapability) {
            connection.workspace.onDidChangeWorkspaceFolders(_event => {
                connection.console.log('Workspace folder change event received.');
            });
        }
        if (hasCodeActionLiteralSupport) {
            connection.onCodeAction(e => container.resolve(CodeActionService).handleCodeAction(e));
        }
    });

    connection.onInitialize((params: InitializeParams) => {
        let capabilities = params.capabilities;

        // Does the client support the `workspace/configuration` request?
        // If not, we will fall back using global settings
        hasConfigurationCapability = !!(
            capabilities.workspace && !!capabilities.workspace.configuration
        );
        hasWorkspaceFolderCapability = !!(
            capabilities.workspace && !!capabilities.workspace.workspaceFolders
        );

        hasCodeActionLiteralSupport = !!capabilities?.textDocument?.codeAction?.codeActionLiteralSupport?.codeActionKind?.valueSet;

        return {
            capabilities: {
                textDocumentSync: documents.syncKind,
                codeActionProvider: !!hasCodeActionLiteralSupport ? {
                    codeActionKinds: listAllCodeActionKinds()
                } : undefined,
                executeCommandProvider: {
                    commands: listAllCommands()
                },
                hoverProvider: true
            }
        };
    });

    connection.onDidChangeWatchedFiles(_change => {
        // Monitored files have change in VSCode
        connection.console.log('We received an file change event');
    });

    // Make the text document manager listen on the connection
    // for open, change and close text document events
    documents.listen(connection);

    // Listen on the connection
    connection.listen();

    return [connection, documents];
}
