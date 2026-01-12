pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod error;
pub mod diagnostic;

#[cfg(test)]
mod lexer_tests;

#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod diagnostic_tests;

#[cfg(test)]
mod diagnostic_builder_tests;

