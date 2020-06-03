import { singleton, inject } from "tsyringe";
import { ExecuteCommandParams } from 'vscode-languageserver';
import { config } from "../mappings";
import { ShellService } from "../ShellService";

@singleton()
export class RunCargoCheckCommand {
    constructor(
        @inject(ShellService) private shell: ShellService,
    ) {
    }

    canHandle = (params: ExecuteCommandParams) => 
        params.command === config.cargoCheckCommand;

    excuteCommand = () =>
        this.shell.runCargoCheck();
}