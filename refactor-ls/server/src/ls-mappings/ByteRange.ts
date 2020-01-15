import { Range, TextDocument } from 'vscode-languageserver';

export class ByteRange {
    constructor(public start: Number, public end: Number) { }
    /**
     * Returns true if this is a valid range
     */
    isRange = () => this.start >= 0 && this.end >= 0;

    isEmpty = () => this.start === this.end;

    /**
     * Reurns <start>:<end>
     */
    toArgumentString = () => `${this.start}:${this.end}`;

    static Empty = () => new ByteRange(0, 0);
    
    static Null = () => new ByteRange(-1, -1);
    
    static fromRange(range: Range, doc: TextDocument): ByteRange {
        const hasSelection = range && range.start && range.end;
        if (!hasSelection || doc === undefined)
            return this.Null();
        if (range.start.character === range.end.character && range.start.line === range.end.line)
            return this.Empty();
        return new ByteRange(doc.offsetAt(range.start), doc.offsetAt(range.end));
    }
}
