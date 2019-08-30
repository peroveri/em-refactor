## Overview
This project should contain a list of valid refactorings that can be used for unit tests.

Each refactoring example should contain the code before, after and the arguments passed to the refactoring tool. The arguments must contain refactoring definition name (e.g. 'extract-method'). Other arguments might be required such as position (line:column) depending on the actual refactoring.

## Example
So if for the testcase 'extract-method-01', the following files are required
* `extract-method-01.rs`
    * The code before refactoring
* `extract-method-01.after.rs`
    * The expected code after refactoring
* `extract-method-01.json`
    * The arguments to be passed

## Candidate search
When searching automatically for candidates that can be refactored, the arguments should not be required. How can that be tested automatically?