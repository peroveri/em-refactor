import { singleton, inject } from "tsyringe";
import * as shell from "shelljs";
import { NotificationService, SettingsService } from "./";
import { RefactorArgs } from "./mappings";

@singleton()
export class ShellService {
    private isExecuting: boolean = false;
    constructor(
        @inject(SettingsService) private settings: SettingsService,
        @inject(NotificationService) private notifications: NotificationService) { }

    private shellExec(cmd: string) {
        if(this.isExecuting) {
            return new Error("Shell already executing");
        }
        this.isExecuting = true;
        this.notifications.logInfo(`executing cmd:\n${cmd}`);
        let result = shell.exec(cmd);
        this.isExecuting = false;
        return result;
    }

    async callRefactoring(relativeFilePath: string, arg: RefactorArgs) {
        let settings = await this.settings.getSettings();
        let cmd = convertToCmd(relativeFilePath, arg.refactoring, arg.selection, arg.unsafe, settings.refactoringCargoTomlPath, settings.cargoToolchain);
        if (cmd instanceof Error) {
            return new Error(cmd.message);
        }
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }

        const result = this.shellExec(cmd);
        if(result instanceof Error) {
            return result;
        }
        if (result.code === 0) {
            this.notifications.logInfo(`\nstdout: ${result.stdout}`);
            this.notifications.logInfo(`\nstderr: ${result.stderr}`);
        } else {
            this.notifications.logError(`Got error code: ${result.code}`);
            this.notifications.logError(result.stdout);
            this.notifications.logError(result.stderr);
        }
        return result;
    }

    async runCargoCheck() {
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let settings = await this.settings.getSettings();
        let res = this.shellExec(`cargo ${settings.cargoToolchain} check --target-dir=./target/refactorings --all-targets`);
        if(res instanceof Error) {
            this.notifications.sendErrorNotification(res.message);
        } else {
            this.notifications.sendInfoNotification(`cargo check returned with status: ${res.code}`);
        }
    }
    async queryCandidates(refactoring: string) {
        /* https://github.com/shelljs/shelljs/wiki/Electron-compatibility */
        if (shell.config.execPath === null) {
            shell.config.execPath = shell.which('node').toString();
        }
        let settings = await this.settings.getSettings();
        let cmd = convertToCandidateCmd(refactoring, settings.refactoringCargoTomlPath, settings.cargoToolchain);
        if (cmd instanceof Error) {
            return new Error(cmd.message);
        }
        let res = this.shellExec(cmd);
        if(res instanceof Error) {
            return res;
        }
        if(res.code !== 0) {
            return new Error(`candidates failed with code: ${res.code}\nstderr:${res.stderr}`);
        }
        return res.toString();
    }
}

const convertToCmd = (relativeFilePath: string, refactoring: string, selection: string, unsafe: boolean, maifestPath: string, toolchain: string): string | Error => {
    if (!isValidBinaryPath(maifestPath)) {
        return new Error(`'${maifestPath}' is not a valid binary file`);
    }
    const refactorArgs = `--target-dir=./target/refactorings refactor ${refactoring} ${relativeFilePath} ${selection}` + (unsafe ? ' --unsafe' : '');

    return `cargo ${toolchain} run --bin cargo-em-refactor --manifest-path=${maifestPath} -- ${refactorArgs}`;
}
const convertToCandidateCmd = (refactoring: string, maifestPath: string, toolchain: string): string | Error => {
    if (!isValidBinaryPath(maifestPath)) {
        return new Error(`'${maifestPath}' is not a valid binary file`);
    }
    const refactorArgs = `--target-dir=./target/refactorings candidates ${refactoring}`;

    return `cargo ${toolchain} run --bin cargo-em-refactor --manifest-path=${maifestPath} -- ${refactorArgs}`;
}

const isValidBinaryPath = (binaryPath: string): boolean =>
    !!binaryPath && binaryPath.length > 0;
