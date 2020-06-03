const microRefactorings = [
    "close-over-variables",
    "convert-closure-to-function",
    "extract-block",
    "inline-macro",
    "introduce-closure",
    "lift-function-declaration",
    "pull-up-item-declaration"
];
const compositeRefactorings = [
    "box-field",
    "extract-method",
];

export const listRefactorings = (isMicroRefactoringsShown: boolean) => {
    let refactorings = isMicroRefactoringsShown ? compositeRefactorings.concat(microRefactorings) : compositeRefactorings;
    return refactorings.sort();
};
