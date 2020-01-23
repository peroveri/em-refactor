import { singleton, inject } from "tsyringe";
import { Connection, ShowMessageNotification, MessageType } from "vscode-languageserver";

@singleton()
export class NotificationService {
    constructor(
        @inject("Connection") private connection: Connection) { }

    sendErrorNotification = (message: string) =>
        this.sendNotification(message, MessageType.Error);

    sendInfoNotification = (message: string) =>
        this.sendNotification(message, MessageType.Info);

    sendNotification(message: string, type: MessageType) {
        this.connection.sendNotification(ShowMessageNotification.type, {
            message, type,
        });
    }

    logError(message: string) {
        console.error(message);
    }
    logInfo(message: string) {
        console.log(message);
    }
}