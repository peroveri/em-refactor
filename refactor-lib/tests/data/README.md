## Box Field

Grammars
* Struct definition (named fields)
* Struct definition (tuple)
* Struct expression 
* Field access expression
* Assignment expression
* Struct pattern
  * TODO: Tuple Struct, Tuple, Grouped, Slice and Path patterns? (or do we not need to test those?)
* Attribute macros
* Expression macros
* self
* Error propagation expression: `?`
* Visibility modifiers

Type system / other
* Coercion(?)
* Copy trait cannot contain Drop
* Paths may not need deref (*), but it is an optimization
* Overlapping changes
* Binding to Box, e.g. user defined Box<T> is allowed!

## Extract block

A <- Statements before B
B <- Extracted statements 
C <- Statements after B 

Grammars
* Block structure, B = Expression
* Block structure, B = Statements + Expression
* Should also support single expressions, 
  not 100% sure, but since declarations (use, struct, fn, ...), etc must appear in blocks it will always be safe to wrap an expression with a block without further actions?

Item declarations
* Function
* Use
* Mod
* Struct
* ++
x (A, B), (B, C)
x name_conflict, no_name_conflict

Ownership model (Declared, Used)
* A, A
* A, AB
* A, ABC
* A, B
* A, BC
* A, C
* B, B
* B, BC
* B, C
* C, C
x (borrow, mutable, move)

Desugaring
* While
* For
* ++

Macro invocation
* Extract macro invocation
* Extract macro invocation argument

Other
* Copy trait (move semantics "removed")
* Error propagation expression: `?`
* Futures? Closures?
* From single expression

## Introduce closure
Desugaring
* While, For, ++

Control flow statements
* Return, Break, Continue
x Conditionals

Grammar / structure of the input
* The selection is a block
Assignment to block (body of the new closure)
* The block is assigned to
* The block is an expression in another block

Return value
* The block has an expression

Mutating vars
* 
* Other closure inside mutating implicit vars isn't possible (without passing args)

## Inline macro
Call site (statement)
- Expands to 1, multiple statements

Call site (expr)

Call site (item)

Binding locals
- New bindings shouldnt affect outside

Invoke inline inside a macro is not supported

Standard macros have some special rules (cannot be inlined?)
- format! use of libraries
- vec! box syntax
- try! deprecated