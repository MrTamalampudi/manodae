use std::collections::HashSet;

use crate::{production::Production, symbol::unique_symbols};

//implemented based on following reference
//github.com/MRTamalampudi/references/blob/main/toc-cd/eliminate_unit_productions.pdf
//both derivable() & eliminate_unit_productions()
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
    productions: &Vec<Production<AST, Token, TranslatorStack>>,
) -> Vec<Production<AST, Token, TranslatorStack>> {
    let mut new_productions = productions.clone();
    let mut non_terminals: Vec<String> = unique_symbols(productions)
        .iter()
        .filter(|symbol| symbol.is_non_terminal())
        .map(|symbol| symbol.to_string())
        .collect();
    non_terminals.push(String::from("Start"));
    let non_unit_productions: Vec<&Production<AST, Token, TranslatorStack>> = productions
        .iter()
        .filter(|production| {
            production.body.len() > 1
                || (production.body.len() == 1 && production.body.get(0).unwrap().is_terminal())
        })
        .collect();
    for nt in non_terminals.iter() {
        let derivable = derivable(productions, nt);
        for b in derivable.iter() {
            let b_non_unit_productions: Vec<_> = non_unit_productions
                .iter()
                .filter(|production| production.head == *b)
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
                if !new_productions.contains(&production) {
                    new_productions.push(production);
                }
            }
        }
    }

    let no_unit_productions: Vec<Production<AST, Token, TranslatorStack>> = new_productions
        .iter()
        .cloned()
        .filter(|production| production.body.len() != 1 || production.body[0].is_terminal())
        .collect();

    //println!("non unit_productins {:#?}", no_unit_productions);
    no_unit_productions
}

pub fn eliminate_useless_productions<AST: Clone, Token: Clone, TranslatorStack: Clone>(
    productions: Vec<Production<AST, Token, TranslatorStack>>,
) -> Vec<Production<AST, Token, TranslatorStack>> {
    let mut nt_production_bodies: HashSet<_> = productions
        .iter()
        .flat_map(|production| production.body.clone())
        .filter(|symbol| symbol.is_non_terminal())
        .map(|symbol| symbol.to_string())
        .collect();
    nt_production_bodies.insert(String::from("Start"));

    productions
        .iter()
        .cloned()
        .filter(|production| nt_production_bodies.contains(&production.head))
        .collect()
}
