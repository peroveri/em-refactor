export interface RefactorOutputs {
    candidates: any[];
    changes: Change[][];
    errors: RefactorError[];
}
export interface Change {
    byte_end: number;
    byte_start: number;
    char_end: number;
    char_start: number;
    file_name: string;
    line_end: number;
    line_start: number;
    replacement: string;
}
interface RefactorError {
    is_error: boolean;
    kind: string;
    message: string;
    codes: string[];
}
