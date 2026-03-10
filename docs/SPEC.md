# Extended Backus-Naur Form (EBNF) Specification

This document defines the formal grammar for the Lumina programming language.

## Tokens & Terminals

```ebnf
IDENTIFIER  ::= [a-zA-Z_] [a-zA-Z0-9_]*
NUMBER      ::= [0-9]+ ("." [0-9]+)?
TEXT_STRING ::= '"' [^"]* '"'
BOOLEAN     ::= "true" | "false"

COMMENT     ::= "--" [^\n]* "\n"
```

## Top Level Declarations

```ebnf
Program ::= Declaration*

Declaration ::= EntityDecl | RuleDecl | LetDecl | UpdateDecl | ShowDecl

EntityDecl ::= "entity" IDENTIFIER "{" EntityField* "}"

EntityField ::= (MetadataDecl)* IDENTIFIER (":" Type | ":=" Expression)

MetadataDecl ::= "@doc" TEXT_STRING | "@range" Expression "to" Expression

Type ::= "Number" | "Text" | "Boolean"
```

## Flow and Rules

```ebnf
RuleDecl ::= "rule" TEXT_STRING "{" TriggerClause ActionClause "}"

TriggerClause ::= WhenClause | EveryClause

WhenClause ::= "when" Expression "becomes" Expression (TemporalModifier)?
EveryClause ::= "every" NUMBER TimeScale

TemporalModifier ::= "for" NUMBER TimeScale
TimeScale ::= "ms" | "s" | "m" | "h" | "d"

ActionClause ::= "then" Statement
```

## Instantiations

```ebnf
LetDecl ::= "let" IDENTIFIER "=" IDENTIFIER "{" FieldInitializer ("," FieldInitializer)* "}"
FieldInitializer ::= IDENTIFIER ":" Expression

UpdateDecl ::= "update" MemberAccess "to" Expression
ShowDecl ::= "show" Expression
```

## Expressions

```ebnf
Expression ::= LogicalOrExpr

LogicalOrExpr ::= LogicalAndExpr ("or" LogicalAndExpr)*
LogicalAndExpr ::= EqualityExpr ("and" EqualityExpr)*
EqualityExpr ::= RelationalExpr (("==" | "!=") RelationalExpr)*
RelationalExpr ::= AdditiveExpr (("<" | "<=" | ">" | ">=") AdditiveExpr)*
AdditiveExpr ::= MultiplicativeExpr (("+" | "-") MultiplicativeExpr)*
MultiplicativeExpr ::= UnaryExpr (("*" | "/") UnaryExpr)*

UnaryExpr ::= ("not" | "-" | "+") UnaryExpr | PrimaryExpr

PrimaryExpr ::= NUMBER | TEXT_STRING | BOOLEAN | MemberAccess | "(" Expression ")"

MemberAccess ::= IDENTIFIER ("." IDENTIFIER)+
```
