import "reflect-metadata";

import { bootstrapAndReturnConnection } from './services/connection';
import { container } from "tsyringe";
import { ExecuteCommandService } from "./services/ExecuteCommandService";
import { NotificationService } from "./services/NotificationService";

// don't create a connection if we're testing
// should perhaps use mocks instead
if(typeof global.it !== 'function') { 

    const connection = bootstrapAndReturnConnection();
    
    connection.onExecuteCommand(container.resolve(ExecuteCommandService).handleExecuteCommand);

    container.resolve(NotificationService).sendInfoNotification("Language server started");
}
