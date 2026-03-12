# Extended Backus-Naur Form (EBNF) Specification (v1.4)

This document contains the complete formal language grammar of the Lumina programming language, conforming to version 1.4 specifications.

## Global Program Structure
```ebnf
program ::= statement* EOF

statement ::= import_stmt
            | fn_decl
            | entity_decl
            | let_stmt
            | rule_decl
            | action_stmt
            | external_decl
            | NEWLINE
```

## Modules and Functions
```ebnf
import_stmt ::= 'import' STRING NEWLINE

fn_decl ::= 'fn' IDENT '(' (fn_param (',' fn_param)*)? ')' '->' type '{' expr '}'

fn_param ::= IDENT ':' type
```

## Entity and Composition Forms
```ebnf
entity_decl ::= 'entity' IDENT '{' NEWLINE field* '}'

field ::= metadata* stored_field | metadata* derived_field
stored_field ::= IDENT ':' type NEWLINE
derived_field ::= IDENT ':=' expr NEWLINE

metadata ::= '@doc' STRING NEWLINE
           | '@range' NUMBER 'to' NUMBER NEWLINE
           | '@affects' IDENT (',' IDENT)* NEWLINE

type ::= 'Text' | 'Number' | 'Boolean' | IDENT
```

## Abstract Values and Creation
```ebnf
let_stmt ::= 'let' IDENT '=' (expr | entity_init) NEWLINE

entity_init ::= IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
```

## Rules and Temporal Logic
```ebnf
rule_decl ::= 'rule' STRING '{' NEWLINE
              ( 'when' condition NEWLINE
              | 'every' duration NEWLINE )
              ('then' action NEWLINE)+ 
              '}'

condition ::= entity_condition | expr
entity_condition::= IDENT '.' IDENT ('becomes' expr)? ('for' duration)?

duration ::= NUMBER ('s' | 'm' | 'h' | 'd')
```

## External Entities
```ebnf
external_decl ::= 'external' 'entity' IDENT '{' NEWLINE
                  field*
                  sync_config
                  '}'

sync_config ::= 'sync' ':' STRING NEWLINE
              'on' ':' STRING NEWLINE
              ('poll_interval' ':' duration NEWLINE)?
```

## Runtime Actions
```ebnf
action ::= show_action
         | update_action
         | create_action
         | delete_action

show_action ::= 'show' expr
update_action ::= 'update' field_access 'to' expr
create_action ::= 'create' IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
delete_action ::= 'delete' IDENT
```

## Expressions (Pratt Operator Precedence)
```ebnf
expr ::= or_expr

or_expr ::= and_expr ('or' and_expr)*
and_expr ::= not_expr ('and' not_expr)*
not_expr ::= 'not' not_expr | cmp_expr
cmp_expr ::= add_expr (cmp_op add_expr)?
add_expr ::= mul_expr (('+' | '-') mul_expr)*
mul_expr ::= unary_expr (('*' | '/' | 'mod') unary_expr)*
unary_expr ::= '-' primary | primary

primary ::= NUMBER | STRING | BOOL | IDENT | field_access
          | '(' expr ')'
          | if_expr
          | call_expr

if_expr ::= 'if' expr 'then' expr ('else' 'if' expr 'then' expr)* 'else' expr
field_access ::= IDENT ('.' IDENT)+
call_expr ::= IDENT '(' (expr (',' expr)*)? ')'
cmp_op ::= '==' | '!=' | '>' | '<' | '>=' | '<='
```

## Token Terminals
```ebnf
STRING ::= '"' (interpolated_content)* '"'
interpolated_content ::= char | '{' expr '}'
IDENT ::= LETTER (LETTER | DIGIT | '_')*
NUMBER ::= DIGIT+ ('.' DIGIT+)?
BOOL ::= 'true' | 'false'
```
