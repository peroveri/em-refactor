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

Type system / other
* Coercion(?)
* Copy trait cannot contain Drop
* Paths may not need deref (*), but it is an optimization
* Overlapping changes

## Extract block

A <- Statements before B
B <- Extracted statements 
C <- Statements after B 

Grammars
* Block structure, B = Expression
* Block structure, B = Statements + Expression

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
