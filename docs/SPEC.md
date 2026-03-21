# Extended Backus-Naur Form (EBNF) Specification (v1.5)

This document contains the formal grammar of Lumina v1.5.

## Global Program Structure
```ebnf
program ::= statement* EOF

statement ::= import_stmt
            | fn_decl
            | entity_decl
            | let_stmt
            | rule_decl
            | action_stmt
            | aggregate_decl
            | external_decl
            | NEWLINE

aggregate_decl ::= 'aggregate' IDENT 'over' IDENT '{' NEWLINE (IDENT ':=' aggregate_func NEWLINE)* '}'
aggregate_func ::= ('avg' | 'min' | 'max' | 'sum' | 'count' | 'any' | 'all') '(' IDENT? ')'
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

field ::= metadata* (stored_field | derived_field)
stored_field ::= IDENT ':' type NEWLINE
derived_field ::= IDENT ':=' expr NEWLINE

metadata ::= '@doc' STRING NEWLINE
           | '@range' NUMBER 'to' NUMBER NEWLINE
           | '@affects' IDENT (',' IDENT)* NEWLINE

type ::= 'Text' | 'Number' | 'Boolean' | IDENT | type '[]'
```

## Abstract Values and Creation
```ebnf
let_stmt ::= 'let' IDENT '=' (expr | entity_init) NEWLINE

entity_init ::= IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
```

## 5. Rules and Temporal Logic
```ebnf
rule_decl ::= 'rule' STRING '{' NEWLINE
              ( 'when' condition NEWLINE
              | 'every' duration NEWLINE )
              ('then' action NEWLINE)+ 
              ('on clear' '{' action+ '}')?
              ('cooldown' duration)?
              '}'

condition ::= fleet_trigger | expr

fleet_trigger ::= ('any' | 'all') IDENT '.' IDENT 'becomes' expr ('for' duration)?

expr ::= or_expr ('becomes' expr)? ('for' duration)?

duration ::= NUMBER ('s' | 'm' | 'h' | 'd')
```

## 6. External Entities
```ebnf
external_decl ::= 'external' 'entity' IDENT '{' NEWLINE
                  field*
                  sync_config
                  '}'

sync_config ::= 'sync' ':' STRING NEWLINE
              'on' ':' ("realtime" | "poll" | "webhook") NEWLINE
              ('poll_interval' ':' duration NEWLINE)?
```

## 7. Runtime Actions
```ebnf
action ::= show_action
         | update_action
         | create_action
         | delete_action
         | alert_action

show_action ::= 'show' expr
update_action ::= 'update' IDENT '.' IDENT 'to' expr
create_action ::= 'create' IDENT '{' NEWLINE (IDENT ':' expr NEWLINE)* '}'
delete_action ::= 'delete' IDENT
alert_action ::= 'alert' 'severity' ':' STRING (',' 'message' ':' STRING)? (',' 'source' ':' expr)? (',' 'code' ':' STRING)? (',' 'payload' ':' '{' ... '}')?
```

## 8. Expressions (Pratt Operator Precedence)
```ebnf
expr ::= or_expr

or_expr ::= and_expr ('or' and_expr)*
and_expr ::= not_expr ('and' not_expr)*
not_expr ::= 'not' not_expr | cmp_expr
cmp_expr ::= add_expr (cmp_op add_expr)?
add_expr ::= mul_expr (('+' | '-') mul_expr)*
mul_expr ::= unary_expr (('*' | '/' | 'mod') unary_expr)*
unary_expr ::= '-' primary | primary

primary ::= NUMBER | STRING | BOOL | IDENT | field_access | prev_expr
          | '(' expr ')'
          | if_expr
          | call_expr
          | list_literal
          | index_expr

prev_expr ::= 'prev' '(' IDENT ')'
list_literal ::= '[' (expr (',' expr)*)? ']'
index_expr ::= primary '[' expr ']'

if_expr ::= 'if' expr 'then' expr 'else' expr
field_access ::= primary '.' IDENT
call_expr ::= IDENT '(' (expr (',' expr)*)? ')'
cmp_op ::= '==' | '!=' | '>' | '<' | '>=' | '<='
```

---

## 9. Version History
*   **v1.5**: LSP, External Entities, `prev()`, Aggregates, `on clear`, Cooldowns, Playground v2.
*   **v1.4**: Functions, Modules, String Interpolation, Lists, Go FFI, REPL v2, Diagnostics.
*   **v1.3**: Core reactive engine, Entities, Rules, Actions, CLI.

---

## 10. Token Terminals
```ebnf
STRING ::= '"' (interpolated_content)* '"'
interpolated_content ::= char | '{' expr '}'
IDENT ::= LETTER (LETTER | DIGIT | '_')*
NUMBER ::= DIGIT+ ('.' DIGIT+)?
BOOL ::= 'true' | 'false'
```
