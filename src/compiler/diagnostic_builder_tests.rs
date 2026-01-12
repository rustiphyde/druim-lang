#[cfg(test)]
mod tests {
    use crate::compiler::error::{Diagnostic, Severity, Span, Source, Note};
    use crate::compiler::diagnostic::render;

        #[test]
    fn diagnostic_error_builder_matches_manual_construction() {
        let source = Source::new("let x = ;".to_string());
        let span = Span { start: 8, end: 8 };

        // Manual construction (ground truth)
        let manual = Diagnostic {
            severity: Severity::Error,
            message: "unexpected token".to_string(),
            span,
            help: None,
            secondary: vec![],
            notes: vec![],
        };

        // Builder construction
        let built = Diagnostic::error("unexpected token", span);

        let manual_rendered = render(&manual, &source);
        let built_rendered = render(&built, &source);

        assert_eq!(manual_rendered, built_rendered);
    }

        #[test]
    fn diagnostic_with_secondary_builder_matches_manual_construction() {
        let source = Source::new("let total = price * qty;".to_string());

        let primary_span = Span { start: 21, end: 24 }; // qty
        let secondary_span = Span { start: 13, end: 18 }; // price

        // Manual construction (ground truth)
        let manual = Diagnostic {
            severity: Severity::Error,
            message: "unknown variable `qty`".to_string(),
            span: primary_span,
            help: None,
            secondary: vec![(secondary_span, "defined here")],
            notes: vec![],
        };

        // Builder construction
        let built = Diagnostic::error("unknown variable `qty`", primary_span)
            .with_secondary(secondary_span, "defined here");

        let manual_rendered = render(&manual, &source);
        let built_rendered = render(&built, &source);

        assert_eq!(manual_rendered, built_rendered);
    }

        #[test]
    fn diagnostic_with_note_builder_matches_manual_construction() {
        let source = Source::new("let total = price * qty;".to_string());

        let primary_span = Span { start: 21, end: 24 }; // qty
        let note_span = Span { start: 13, end: 18 }; // price

        let note = Note {
            severity: Severity::Note,
            message: "`price` is defined here".to_string(),
            span: Some(note_span),
        };

        // Manual construction (ground truth)
        let manual = Diagnostic {
            severity: Severity::Error,
            message: "unknown variable `qty`".to_string(),
            span: primary_span,
            help: None,
            secondary: vec![],
            notes: vec![note.clone()],
        };

        // Builder construction
        let built = Diagnostic::error("unknown variable `qty`", primary_span)
            .with_note(note);

        let manual_rendered = render(&manual, &source);
        let built_rendered = render(&built, &source);

        assert_eq!(manual_rendered, built_rendered);
    }

        #[test]
    fn diagnostic_with_help_builder_matches_manual_construction() {
        let source = Source::new("let x = ;".to_string());
        let span = Span { start: 8, end: 8 };

        // Manual construction (ground truth)
        let manual = Diagnostic {
            severity: Severity::Error,
            message: "expected expression".to_string(),
            span,
            help: Some("expressions cannot be empty"),
            secondary: vec![],
            notes: vec![],
        };

        // Builder construction
        let built = Diagnostic::error("expected expression", span)
            .with_help("expressions cannot be empty");

        let manual_rendered = render(&manual, &source);
        let built_rendered = render(&built, &source);

        assert_eq!(manual_rendered, built_rendered);
    }

    #[test]
    fn diagnostic_builder_order_does_not_matter() {
        let span = Span { start: 5, end: 6 };
        let secondary = Span { start: 1, end: 2 };
        
        let a = Diagnostic::error("test", span)
            .with_secondary(secondary, "secondary")
            .with_help("help_text")
            .with_note(Note::note("note text", Some(span)));

        let b = Diagnostic::error("test", span)
            .with_note(Note::note("note text", Some(span)))
            .with_help("help_text")
            .with_secondary(secondary, "secondary"); 
            
        assert_eq!(a, b);
    }
    
}