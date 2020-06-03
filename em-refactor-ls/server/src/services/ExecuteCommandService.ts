import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams, ApplyWorkspaceEditParams } from 'vscode-languageserver';
import { GenerateTestFileCommand, QueryCandidatesCommand, RunCargoCheckCommand, RefactorCommand } from './commands';

@singleton()
export class ExecuteCommandService {
    constructor(
        @inject(GenerateTestFileCommand) private generateTestFileCommand: GenerateTestFileCommand,
        @inject(QueryCandidatesCommand) private queryCandidatesCommand: QueryCandidatesCommand,
        @inject(RefactorCommand) private refactorCommand: RefactorCommand,
        @inject(RunCargoCheckCommand) private runCargoCheckCommand: RunCargoCheckCommand,
    ) {
    }

    /**
     * Handles the workspace/executeCommand request
     */
    handleExecuteCommand = async (params: ExecuteCommandParams): Promise<ApplyWorkspaceEditParams | void | any> => {
        try {
            if (this.refactorCommand.canHandle(params)) {
                return this.refactorCommand.excuteCommand(params);
            } else if (await this.generateTestFileCommand.canHandle(params)) {
                return this.generateTestFileCommand.excuteCommand(params);
            } else if (this.runCargoCheckCommand.canHandle(params)) {
                return this.runCargoCheckCommand.excuteCommand();
            } else if (await this.queryCandidatesCommand.canHandle(params)) {
                return this.queryCandidatesCommand.excuteCommand(params);
            }
        } catch (e) {
            return Promise.reject(`Unhandled expection in handleExecuteCommand:\n${JSON.stringify(e)}`);
        }
    };
}
