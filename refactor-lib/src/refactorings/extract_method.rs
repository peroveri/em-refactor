
/**
 * A naive and incorrect algorithm for extract method
 * 
 * Extract method
 * 
 * Input:
 * - f - A function 
 * - m - The module containing f
 * - s - A selection in f (of consecutive statements?)
 * 
 * Assumptions:
 * - f is not a method
 * 
 * Steps:
 * g <- new function with fresh name
 * add g to m
 * vs <- all variables in s not declared in s
 * add vs as parameters of g
 * replace s with a call to g with arguments vs
 */