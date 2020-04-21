import "reflect-metadata";

import { bootstrapAndReturnConnection } from './services/services';
import { container } from "tsyringe";
import { ExecuteCommandService } from "./services/ExecuteCommandService";

// don't create a connection if we're testing
// should perhaps use mocks instead
if(typeof global.it !== 'function') { 

    const connection = bootstrapAndReturnConnection();
    
    connection.onExecuteCommand(container.resolve(ExecuteCommandService).handleExecuteCommand);
}
