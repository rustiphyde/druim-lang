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
        let ks = kinds("Num Numx Text Emp");
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

}
