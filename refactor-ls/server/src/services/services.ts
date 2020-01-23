import { container } from "tsyringe";
import {
    ProposedFeatures,
    createConnection,
    Connection,
    InitializeParams,
    TextDocuments,
    DidChangeConfigurationNotification,
    CodeActionKind} from 'vscode-languageserver';
import { CodeActionService } from "./CodeActionService";
import { ExecuteCommandService } from "./ExecuteCommandService";
import { HoverService } from "./HoverService";

let hasConfigurationCapability: boolean = false;
let hasWorkspaceFolderCapability: boolean = false;

export function bootstrap() {
    const [connection, documents] = initConnection();

    container.register<Connection>("Connection", { useValue: connection });
    container.register<TextDocuments>("TextDocuments", { useValue: documents });

    connection.onCodeAction(container.resolve(CodeActionService).handleCodeAction);
    connection.onExecuteCommand(container.resolve(ExecuteCommandService).handleExecuteCommand);
    connection.onHover(container.resolve(HoverService).handleOnHover);
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
        let hasEditCapability = true;
        if (hasEditCapability) {
            // connection.client.register()
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

        return {
            capabilities: {
                textDocumentSync: documents.syncKind,
                codeActionProvider: { // TODO: code actions literal support
                    codeActionKinds: [
                        CodeActionKind.RefactorExtract + '.function',
                        `${CodeActionKind.Refactor}.generate_test_file`
                    ]
                },
                executeCommandProvider: {
                    commands: [
                        'refactor.extract.function',
                        `${CodeActionKind.Refactor}.generate_test_file`
                    ]
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

