use std::{collections::HashSet, hash::Hash};

use crate::{
    production::Production,
    symbol::{unique_symbols, Symbol},
};

//implemented based on following reference
//github.com/MRTamalampudi/references/blob/main/toc-cd/eliminate_unit_productions.pdf

fn derivable<AST, Token, TranslatorStack>(
    productions: &Vec<Production<AST, Token, TranslatorStack>>,
    head: &String,
) -> HashSet<String> {
    let is_non_terminal = |production: &&Production<AST, Token, TranslatorStack>| {
        production.body.get(0).unwrap().is_non_terminal()
    };
    let mut derivables: HashSet<String> = productions
        .iter()
        .filter(|production| {
            production.body.len() == 1 && production.head == *head && is_non_terminal(production)
        })
        .map(|production| production.body.get(0).unwrap().to_string())
        .collect::<HashSet<String>>();
    let mut derivables_: HashSet<String> = HashSet::new();

    while derivables.ne(&derivables_) {
        derivables_.clear();
        derivables_ = derivables.clone();

        for head_ in derivables_.iter() {
            let set: HashSet<String> = productions
                .iter()
                .filter(|production| {
                    production.body.len() == 1
                        && production.head == *head_
                        && is_non_terminal(production)
                        && production.body.get(0).unwrap().to_string().ne(head)
                })
                .map(|production| production.body.get(0).unwrap().to_string())
                .collect::<HashSet<String>>();
            derivables.extend(set);
        }
    }
    derivables
}

pub fn eliminate_unit_productions<AST: Clone, Token: Clone, TranslatorStack: Clone>(
    productions: &mut Vec<Production<AST, Token, TranslatorStack>>,
) -> Vec<Production<AST, Token, TranslatorStack>> {
    let mut non_terminals: Vec<String> = unique_symbols(productions)
        .iter()
        .filter(|symbol| symbol.is_non_terminal())
        .map(|symbol| symbol.to_string())
        .collect();
    non_terminals.push(String::from("Start"));
    for nt in non_terminals.iter() {
        let nt_non_unit_production_bodies: Vec<Vec<Symbol>> = productions
            .iter()
            .filter(|production| {
                production.head == *nt
                    && (production.body.len() > 1 || production.body.get(0).unwrap().is_terminal())
            })
            .map(|production| production.body.clone())
            .collect();
        let derivable = derivable(productions, nt);
        for b in derivable.iter() {
            let productions_clone = productions.clone();
            let b_non_unit_productions: Vec<&Production<AST, Token, TranslatorStack>> =
                productions_clone
                    .iter()
                    .filter(|production| {
                        production.head == *b
                            && (production.body.len() > 1
                                || production.body.get(0).unwrap().is_terminal())
                    })
                    .collect();

            for b_production in b_non_unit_productions.iter() {
                let production = Production {
                    head: nt.clone(),
                    body: b_production.body.clone(),
                    cursor_pos: 0,
                    index: productions.len() + 1,
                    error_message: None,
                    action: b_production.action.clone(),
                };
                let productions_clone_2 = productions.clone();
                let productions_bodies: Vec<&Production<AST, Token, TranslatorStack>> =
                    productions_clone_2
                        .iter()
                        .filter(|production_| production_.eq(&&production))
                        .collect();
                if productions_bodies.len() < 1 {
                    productions.push(production);
                }
            }
        }
    }
    let unit_productions: Vec<&Production<AST, Token, TranslatorStack>> = productions
        .iter()
        .filter(|production| {
            production.body.len() == 1 && production.body.get(0).unwrap().is_non_terminal()
        })
        .collect();

    let no_unit_productions: Vec<Production<AST, Token, TranslatorStack>> = productions
        .iter()
        .cloned()
        .filter(|production| !unit_productions.contains(&production))
        .collect();

    no_unit_productions
}
