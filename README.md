<p align="center"><img width="400" src="./assets/manodae.svg" /></p>

<p align="center"><b>LALR(1) parser generator library for Rust</b></p>

---

## Overview

**manodae** is a Rust library for building **LALR(1) parsers**. It gives you:

- **A grammar API and macros** to describe your language grammar.
- **An LR(1)/LALR(1) table constructor** that computes FIRST/FOLLOW sets and builds the LR automaton.
- **A runtime LR(1) parser** that:
  - Works with a `logos`-based lexer.
  - Drives user‑defined semantic actions to build an AST and collect errors.
- **Optional code generation** that materializes the constructed parser tables as Rust code for **fast startup** and **zero runtime table construction cost**.

The crate is published as `manodae` and the source lives in [`MRTamalampudi/manodae`](https://github.com/MRTamalampudi/manodae).

## Features

- **LALR(1) parser construction**: builds LR(1) automata, merges compatible states into LALR(1), and produces `ACTION` / `GOTO` tables.
- **Grammar abstraction**:
  - `Grammar<AST, Token, TranslatorStack>` to hold symbols and productions.
  - Macros like `grammar!`, `start_production!`, `non_terminal_production!`, `terminal_production!` to define grammars ergonomically.
- **Token abstraction**:
  - `TokenKind` trait to define **error** and **EOF** tokens for your language.
  - Designed to pair with a `logos` lexer.
- **Runtime parser**:
  - `LR1_Parser<AST, Token, TranslatorStack>` that can parse a token stream and drive semantic actions.
  - Basic error recovery support with structured `ParseError` values.
- **Code generation**:
  - `Codegen::gen(path, grammar, generics)` emits Rust code for:
    - Grammar reconstruction.
    - LR automaton.
    - FIRST / FOLLOW sets.
    - `ACTION` and `GOTO` tables.
    - A `get_parser()` function returning a fully constructed `LR1_Parser`.

## Crate layout

- **`src/lib.rs`**: crate root; exports all main modules and a `prelude` for convenient imports.
- **`src/grammar.rs`**:
  - Defines `Grammar<AST, Token, TranslatorStack>`.
  - Provides the `grammar!` macro and helper macros (`start_production!`, `non_terminal_production!`, `terminal_production!`) for building grammars.
- **`src/parser.rs`**:
  - Defines `LR1_Parser<AST, Token, TranslatorStack>`.
  - Implements LR(1)/LALR(1) construction (`construct_LALR_Table`) and the `parse` method.
- **`src/token.rs`**:
  - Defines the `TokenKind` trait:

    ```rust
    pub trait TokenKind: ToString + Debug + Clone {
        type TokenKind: ToString;
        fn error() -> Self::TokenKind;
        fn eof() -> Self::TokenKind;
    }
    ```

- **`src/error.rs`**:
  - Defines `ParseError { span, message, production_end }` as the error type produced by the parser.
- **`src/codegen`**:
  - Contains `Codegen` and helper implementations of the `ToTokens` trait used to emit Rust code for grammar, tables, and parser.

For quick experimentation you can use only the runtime types; for production you can additionally enable code generation for faster startup.

## Getting started

Add `manodae` to your `Cargo.toml`:

```toml
[dependencies]
manodae = { git = "https://github.com/MRTamalampudi/manodae" }
logos = "0.16"
indexmap = "2"
```

You will also typically have `tabled`, `quote`, and `proc-macro2` through `manodae` itself; they rarely need to be used directly from user code.

Then, in your crate:

```rust
use logos::Logos;
use manodae::prelude::*;
```

The `prelude` exposes the most commonly used types:

- **Grammar-related**: `Grammar`, `Production`, `Productions`, `Symbol`, `Symbols`, and their IDs.
- **Parser-related**: `LR1_Parser`, `Action`, `State`, `States`, state and production IDs.
- **Codegen**: `Codegen`.
- **Helpers**: `IndexMap`, `IndexSet`, `quote`, and a few short aliases for convenience.

## Defining tokens

You define your token type using `logos` and implement `TokenKind` for it.

```rust
use logos::Logos;
use manodae::token::TokenKind;

#[derive(Logos, Debug, Clone, PartialEq)]
enum Token {
    #[regex(r"[ \t\n\r]+", logos::skip)] // whitespace
    #[error]
    Error,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,

    #[regex(r"[0-9]+")]
    Number,

    // User-defined EOF marker used by `TokenKind::eof`
    Eof,
}

impl TokenKind for Token {
    type TokenKind = Token;

    fn error() -> Self::TokenKind {
        Token::Error
    }

    fn eof() -> Self::TokenKind {
        Token::Eof
    }
}
```

The parser will:

- Use `Token::error()` when it needs an error sentinel.
- Expect `Token::eof()` as the logical end‑of‑input token (internally it also uses a dedicated `EOF` grammar symbol).

## Defining a grammar

You can construct a grammar either manually or via the `grammar!` macro. The macro is convenient for BNF‑like grammars with attached semantic actions.

Conceptually, a grammar is:

- **Nonterminals** and **terminals** identified by `SymbolId`s.
- A set of **productions**, each with:
  - A head nonterminal.
  - A sequence of body symbols.
  - An optional semantic action closure:
    - Signature: `|ast, token_stack, translator_stack, errors| { ... }`.

The `grammar!` macro has the skeleton:

```rust
use manodae::grammar;

let grammar = grammar! {
    Start -> Expr { |ast, token_stack, tl_stack, errors| {
        // semantic action for the top-level production
    }};

    [non_terminal_productions]

    Expr -> Expr Plus Term
        { |ast, token_stack, tl_stack, errors| {
            // Expr + Term action
        }}
    | Term
        { |ast, token_stack, tl_stack, errors| {
            // Expr -> Term action
        }};

    Term -> Term Star Factor
        { |ast, token_stack, tl_stack, errors| {
            // Term * Factor action
        }}
    | Factor
        { |ast, token_stack, tl_stack, errors| {
            // Term -> Factor action
        }};

    Factor -> [Number]
        { |ast, token_stack, tl_stack, errors| {
            // handle Number token
        }};
};
```

Under the hood, this expands into calls to:

- `start_production!` for the augmented start production.
- `non_terminal_production!` for rules like `Expr -> Expr Plus Term`.
- `terminal_production!` for rules that consume a single terminal, such as `Factor -> [Number]`.

You can also use these macros directly if you prefer imperative construction:

```rust
use manodae::{Grammar, start_production, non_terminal_production, terminal_production};

let mut grammar: Grammar<MyAst, Token, MyStack> = Grammar::new();

start_production!(grammar, Expr { |ast, ts, tl_stack, errors| {
    // ...
}});

non_terminal_production!(grammar, Expr, Expr Plus Term { |ast, ts, tl_stack, errors| {
    // ...
}});

terminal_production!(grammar, Factor, ["Number"] { |ast, ts, tl_stack, errors| {
    // ...
}});
```

## Building and using a parser (runtime)

The simplest way to use manodae is to construct the parser at runtime from a `Grammar`.

```rust
use logos::Logos;
use manodae::parser::LR1_Parser;
use manodae::prelude::*;

fn parse_input(source: &str) -> Result<MyAst, Vec<manodae::error::ParseError>> {
    let mut ast = MyAst::default();
    let mut errors = Vec::new();

    // 1. Define or reuse your grammar
    let grammar: Grammar<MyAst, Token, MyStack> = /* grammar! { ... } or manual construction */;

    // 2. Build parser (computes FIRST/FOLLOW, LR automata, ACTION/GOTO tables)
    let mut parser: LR1_Parser<MyAst, Token, MyStack> = LR1_Parser::new(grammar);

    // 3. Create a lexer over the input
    let lexer = Token::lexer(source);

    // 4. Parse
    parser.parse(lexer, &mut errors, &mut ast);

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}
```

Internally, `LR1_Parser::new`:

- Computes the **FIRST** and **FOLLOW** sets.
- Builds the LR(1) automaton and merges states into a compact **LALR(1)** automaton.
- Constructs `action` and `goto` tables according to the standard LALR(1) construction algorithm.

The `parse` method then:

- Maintains a **state stack** and **input token stack**.
- Looks up `ACTION[state, lookahead]` to decide whether to:
  - **SHIFT**: push a new state and consume the next token.
  - **REDUCE**: apply a production, run its semantic action, and push a `GOTO` state.
  - **ACCEPT**: successfully finish parsing.
- On error, constructs a `ParseError` using the expected token set for the current state and appends it to the `errors` vector.

## Code generation

For larger grammars, constructing the parser tables at runtime can be relatively expensive. The `Codegen` type lets you **generate Rust code** for the parser once, and then just include that code.

```rust
use std::path::PathBuf;
use manodae::codegen::Codegen;
use manodae::prelude::*;

fn main() {
    let grammar: Grammar<MyAst, Token, MyStack> = /* ... */;

    // Choose an output directory; typically OUT_DIR or a generated folder in your crate.
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    Codegen::<MyAst, Token, MyStack>::gen(
        out_dir.clone(),
        grammar,
        ["MyAst", "Token", "MyStack"],
    );
}
```

`Codegen::gen` will:

- Create a `parser_generated/` subdirectory under the given path (if needed).
- Write:
  - `grammar.rs`
  - `lr.rs`
  - `first.rs`
  - `follow.rs`
  - `action.rs`
  - `goto.rs`
  - `parser.rs` (containing `fn get_parser() -> LR1_Parser<AST, Token, TranslatorStack>`).
- Write a small `hash.txt` so that regeneration is skipped if the grammar has not changed.
- Run `rustfmt` over the generated files.

You can then include the generated parser in your project:

```rust
// In some module of your crate
include!("parser_generated/parser.rs");

fn parse_with_generated(source: &str) -> Result<MyAst, Vec<ParseError>> {
    let mut parser = get_parser(); // provided by generated code
    let mut ast = MyAst::default();
    let mut errors = Vec::new();

    let lexer = Token::lexer(source);
    parser.parse(lexer, &mut errors, &mut ast);

    if errors.is_empty() {
        Ok(ast)
    } else {
        Err(errors)
    }
}
```

> **Note**: You can also call `Codegen::gen` from a `build.rs` script, writing into `OUT_DIR`, and then `include!` from there. The basic pattern is the same; only the target path changes.

## Error handling

- **Type**: Errors are represented by `ParseError`:

  ```rust
  pub struct ParseError {
      pub span: std::ops::Range<usize>,
      pub message: String,
      pub production_end: bool,
  }
  ```

- **When created**:
  - Whenever the parser cannot find a valid `ACTION[state, lookahead]`.
  - The message is derived from the set of expected symbols in that state (e.g. `"Expected '+' or '-' or Number"`), unless a production‑specific `error_message` is supplied.
- **Where used**:
  - Your semantic actions receive `&mut Vec<ParseError>` and can:
    - Push additional errors.
    - Enrich the error list with domain‑specific diagnostics.

## When to use manodae

- **You are implementing a custom language, DSL, or configuration format** and want:
  - A deterministic, table‑driven LALR(1) parser.
  - Control over semantic actions and AST building.
  - Reasonable error reporting with spans.
- **You already use `logos`** for lexing and want a compatible LR parser.
- **You prefer explicit grammar construction** over procedural macros that hide the LR machinery.

If you only need a tiny hand‑written recursive‑descent parser, manodae may be more machinery than you need. But as your language grows more complex, the LR approach tends to scale better and stay easier to maintain.

## Status and limitations

- The library is **experimental/early‑stage** and the API may evolve.
- The error recovery algorithm is intentionally conservative and may be improved over time.
- Grammar macros are focused on LALR(1) grammars; more helpers and ergonomic sugar may be added.

## Contributing

- **Issues & feature requests**: please open an issue on the GitHub repository.
- **Pull requests** are welcome; try to include:
  - Tests illustrating the behavior.
  - Documentation updates if the public API changes.

## License

This project is licensed under the same terms as the Rust language itself (MIT/Apache‑2.0). See the repository for details.
