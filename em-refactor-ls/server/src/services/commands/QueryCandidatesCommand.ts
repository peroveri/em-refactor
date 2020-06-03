import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams, CreateFile, TextDocumentEdit, TextEdit, Position } from 'vscode-languageserver';
import { config } from "../mappings";
import { ShellService } from "../ShellService";
import { NotificationService } from "../NotificationService";
import { WorkspaceService } from "../WorkspaceService";

@singleton()
export class QueryCandidatesCommand {
    constructor(
        @inject(NotificationService) private notificationService: NotificationService,
        @inject(ShellService) private shell: ShellService,
        @inject(WorkspaceService) private workspace: WorkspaceService,
    ) {
    }

    canHandle = (params: ExecuteCommandParams) => 
        params.command === config.candidatesCommand &&
        params.arguments && params.arguments[0];

    excuteCommand = async (params: ExecuteCommandParams) => {
        if(params.arguments === undefined || params.arguments.length < 0) {
            throw new Error("Missing param");
        }
        let res = await this.shell.queryCandidates(params.arguments[0]);
        if (res instanceof Error) {
            this.notificationService.sendErrorNotification(res.message);
        } else {
            let workspaceInfo = await this.workspace.getWorkspaceUri();
            let newFileUri = workspaceInfo?.join(`${params.arguments[0]}-candidates.json`);
            if (newFileUri !== undefined) {
                let edits = newFile(newFileUri, res);
                await this.workspace.applyEdits(edits);
            }
        }
    }
}
const newFile = (name: string, content: string) =>
    [{
        edit: {
            documentChanges: [
                CreateFile.create(name, { overwrite: true }),
            ],
            label: config.candidatesCommand
        }
    }, {
        edit: {
            documentChanges: [
                TextDocumentEdit.create({
                    uri: name,
                    version: null
                }, [
                    TextEdit.insert(Position.create(0, 0), content)
                ])
            ],
            label: config.candidatesCommand
        },
    }];
