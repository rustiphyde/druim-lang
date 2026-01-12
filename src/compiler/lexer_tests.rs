#[cfg(test)]
mod tests {
    use crate::compiler::lexer::Lexer;
    use crate::compiler::token::TokenKind;
    use crate::compiler::token::TokenKind::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        let mut lx = Lexer::new(src);
        lx.tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn colon_family_tokens() {
        let ks = kinds("a:b a::b a:=b a:?b a:>b");
        assert!(ks.contains(&Colon));
        assert!(ks.contains(&Scope));
        assert!(ks.contains(&Bind));
        assert!(ks.contains(&Present));
        assert!(ks.contains(&Cast));
    }

    #[test]
    fn keyword_vs_identifier() {
        let ks = kinds("num numx text emp");
        assert_eq!(ks[0], KwNum);
        assert_eq!(ks[1], Ident);
        assert_eq!(ks[2], KwText);
        assert_eq!(ks[3], KwEmp);
    }

    #[test]
    fn number_literals() {
        let ks = kinds("42 3.14");
        assert_eq!(ks[0], NumLit);
        assert_eq!(ks[1], DecLit);
    }

    #[test]
    fn text_literal() {
        let ks = kinds("\"hello\"");
        assert_eq!(ks[0], TextLit);
    }

    #[test]
    fn block_tokens() {
        let src = ":[ x + 1 ]: :{ a <- b; }:";
        let tokens = Lexer::new(src).tokenize().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

        assert!(kinds.contains(&TokenKind::BlockExprStart));
        assert!(kinds.contains(&TokenKind::BlockExprEnd));
        assert!(kinds.contains(&TokenKind::BlockStmtStart));
        assert!(kinds.contains(&TokenKind::BlockStmtEnd));
    }

    #[test]
    fn digit_leading_identifiers() {
        let ks = kinds("1a 9lives 123abc 123_456 1_foo");
        assert_eq!(ks[0], Ident);
        assert_eq!(ks[1], Ident);
        assert_eq!(ks[2], Ident);
        assert_eq!(ks[3], Ident);
        assert_eq!(ks[4], Ident);
    }

    #[test]
    fn pure_digit_sequences_are_numbers() {
        let ks = kinds("1 123 000");
        assert_eq!(ks[0], NumLit);
        assert_eq!(ks[1], NumLit);
        assert_eq!(ks[2], NumLit);
    }

    #[test]
    fn invalid_decimal_forms_error() {
        let mut lx = Lexer::new(".5");
        assert!(lx.tokenize().is_err());

        let mut lx = Lexer::new("1.");
        assert!(lx.tokenize().is_err());

        let mut lx = Lexer::new("1..2");
        assert!(lx.tokenize().is_err());
    }

    #[test]
    fn guard_token() {
        let ks = kinds("x ?= y;");
        assert!(ks.contains(&Guard));
    }

}
