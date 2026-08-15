const vscode = require("vscode");

function getDruimScopeAt(document, offset) {
    const source = document.getText();

    const globalScope = {
        type: "global",
        start: 0,
        end: source.length
    };

    const stack = [];
    let blockScope = null;
    let blockSegmentStart = null;

    for (let i = 0; i < offset; i++) {
        /*
         * Block chain.
         *
         * :{ creates one lexical scope.
         * }{ changes segment but keeps that same lexical scope.
         * }: closes the block scope.
         */
        if (
            source.startsWith(":{", i) &&
            (i === 0 || source[i - 1] === "\n")
        ) {
            const scope = {
                type: "block",
                start: i,
                end: null
            };

            stack.push(scope);

            blockScope = scope;
            blockSegmentStart = i;

            i++;
            continue;
        }

        if (
            source.startsWith("}{", i) &&
            (i === 0 || source[i - 1] === "\n")
        ) {
            blockSegmentStart = i;

            i++;
            continue;
        }

        if (
            source.startsWith("}:", i) &&
            (i === 0 || source[i - 1] === "\n")
        ) {
            const scope = stack
                .slice()
                .reverse()
                .find((item) => item.type === "block");

            if (scope) {
                scope.end = i + 2;

                const index = stack.lastIndexOf(scope);

                if (index !== -1) {
                    stack.splice(index, 1);
                }
            }

            blockScope = null;
            blockSegmentStart = null;

            i++;
            continue;
        }

        /*
         * Loop scope.
         *
         * One scope persists through setup, condition,
         * and process.
         */
        if (source.startsWith(":<", i)) {
            stack.push({
                type: "loop",
                start: i,
                end: null
            });

            i++;
            continue;
        }

        if (source.startsWith(">:", i)) {
            const scope = stack
                .slice()
                .reverse()
                .find((item) => item.type === "loop");

            if (scope) {
                scope.end = i + 2;

                const index = stack.lastIndexOf(scope);

                if (index !== -1) {
                    stack.splice(index, 1);
                }
            }

            i++;
            continue;
        }

        /*
         * Function body.
         *
         * )( begins the function body.
         * ): closes it.
         */
        if (source.startsWith(")(", i)) {
            stack.push({
                type: "function",
                start: i + 2,
                end: null
            });

            i++;
            continue;
        }

        if (source.startsWith("):", i)) {
            const scope = stack
                .slice()
                .reverse()
                .find((item) => item.type === "function");

            if (scope) {
                scope.end = i + 2;

                const index = stack.lastIndexOf(scope);

                if (index !== -1) {
                    stack.splice(index, 1);
                }
            }

            i++;
        }
    }

    const activeScope =
        stack.length > 0
            ? stack[stack.length - 1]
            : globalScope;

    let localScope = activeScope;

    /*
     * loc inside a block chain belongs only to the
     * current segment.
     */
    if (
        blockScope &&
        activeScope.type === "block" &&
        blockSegmentStart !== null
    ) {
        localScope = {
            type: "block-segment",
            start: blockSegmentStart,
            end: null
        };
    }

    return {
        ordinary: activeScope,
        local: localScope,
        global: globalScope,

        /*
        * Outermost → innermost.
        *
        * Global is always the root scope.
        */
        chain: [
            globalScope,
            ...stack
        ]
    };
}

function sameDruimScope(left, right) {
    return (
        left.type === right.type &&
        left.start === right.start
    );
}

function resolveDruimBinding(
    document,
    index,
    name,
    useOffset
) {
    const useScope = getDruimScopeAt(
        document,
        useOffset
    );

    /*
    * Parameters are established in their function scope
    * at call entry, so they are visible throughout the
    * function body regardless of their source position in
    * the parameter list.
    */
    const parameterCandidate = index.parameters.find(
        (parameter) =>
            parameter.name === name &&
            useScope.chain.some(
                (scope) =>
                    sameDruimScope(
                        parameter.ownerScope,
                        scope
                    )
            )
    );

    if (parameterCandidate) {
        return parameterCandidate;
    }

    /*
     * A binding cannot resolve before its declaration.
     *
     * Druim bindings become visible after they are
     * introduced.
     */
    const candidates = index.bindings.filter(
        (binding) =>
            binding.name === name &&
            binding.offset <= useOffset
    );

    if (candidates.length === 0) {
        return null;
    }

    /*
     * Search from the innermost active scope outward.
     *
     * This gives the nearest visible binding precedence.
     */
    const visibleScopes = [
        ...useScope.chain
    ].reverse();

    /*
     * A block-segment-local binding is even narrower
     * than the block scope itself, so check it first
     * when the use-site is in that segment.
     */
    if (useScope.local.type === "block-segment") {
        const localCandidate = candidates
            .filter(
                (binding) =>
                    binding.ownerScope.type ===
                        "block-segment" &&
                    sameDruimScope(
                        binding.ownerScope,
                        useScope.local
                    )
            )
            .sort(
                (a, b) =>
                    b.offset - a.offset
            )[0];

        if (localCandidate) {
            return localCandidate;
        }
    }

    for (const scope of visibleScopes) {
        const candidate = candidates
            .filter(
                (binding) =>
                    sameDruimScope(
                        binding.ownerScope,
                        scope
                    )
            )
            .sort(
                (a, b) =>
                    b.offset - a.offset
            )[0];

        if (candidate) {
            return candidate;
        }
    }

    return null;
}

function buildDruimSymbolIndex(document) {
    const source = document.getText();

    const functions = new Map();
    const bindings = [];
    const parameters = [];

    /*
     * Finds canonical Druim function declarations:
     *
     * fn calculate :(value, multiplier)(
     *
     * Parameters are preserved as source text because
     * Druim parameters may contain defaults.
     */
    const functionPattern =
        /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s+:\(([^)]*)\)\(/g;

    let match;

    while ((match = functionPattern.exec(source)) !== null) {
        const functionName = match[1];
        const parameterText = match[2];

        const nameOffset =
            match.index + match[0].indexOf(functionName);

        const namePosition =
            document.positionAt(nameOffset);

        const nameRange = new vscode.Range(
            namePosition,
            namePosition.translate(
                0,
                functionName.length
            )
        );

        const parameterForms = [];

        let current = "";
        let parenDepth = 0;
        let bracketDepth = 0;
        let inString = false;
        let escaped = false;

        for (const char of parameterText) {
            if (inString) {
                current += char;

                if (escaped) {
                    escaped = false;
                    continue;
                }

                if (char === "\\") {
                    escaped = true;
                    continue;
                }

                if (char === "\"") {
                    inString = false;
                }

                continue;
            }

            if (char === "\"") {
                inString = true;
                current += char;
                continue;
            }

            if (char === "(") {
                parenDepth++;
                current += char;
                continue;
            }

            if (char === ")") {
                parenDepth--;
                current += char;
                continue;
            }

            if (char === "[") {
                bracketDepth++;
                current += char;
                continue;
            }

            if (char === "]") {
                bracketDepth--;
                current += char;
                continue;
            }

            if (
                char === "," &&
                parenDepth === 0 &&
                bracketDepth === 0
            ) {
                if (current.trim() !== "") {
                    parameterForms.push(
                        current.trim()
                    );
                }

                current = "";
                continue;
            }

            current += char;
        }

        if (current.trim() !== "") {
            parameterForms.push(
                current.trim()
            );
        }

        /*
        * Index parameter declarations themselves.
        *
        * Valid canonical forms:
        *
        * value
        * multiplier = 2
        */
        const parameterBlockOffset =
            match.index + match[0].indexOf(":(") + 2;

        let parameterSearchOffset = 0;

        for (const parameterForm of parameterForms) {
            const parameterMatch = parameterForm.match(
                /^([A-Za-z_][A-Za-z0-9_]*)/
            );

            if (!parameterMatch) {
                continue;
            }

            const parameterName = parameterMatch[1];

            const relativeOffset = parameterText.indexOf(
                parameterForm,
                parameterSearchOffset
            );

            if (relativeOffset === -1) {
                continue;
            }

            const nameOffset =
                parameterBlockOffset + relativeOffset;

            const namePosition =
                document.positionAt(nameOffset);

            const parameterRange = new vscode.Range(
                namePosition,
                namePosition.translate(
                    0,
                    parameterName.length
                )
            );

            /*
            * The function scope begins at the body delimiter `)(`.
            *
            * getDruimScopeAt() sees the function scope once we move
            * just beyond that delimiter.
            */
            const bodyDelimiterOffset =
                match.index +
                match[0].lastIndexOf(")(");

            const functionScopeInfo = getDruimScopeAt(
                document,
                bodyDelimiterOffset + 2
            );

            parameters.push({
                name: parameterName,
                kind: "parameter",
                range: parameterRange,
                offset: nameOffset,
                ownerScope: functionScopeInfo.ordinary,
                functionName
            });

            parameterSearchOffset =
                relativeOffset + parameterForm.length;
        }        

        functions.set(
            functionName,
            {
                name: functionName,
                range: nameRange,
                parameters: parameterForms
            }
        );
    }

    /*
    * Collect target-defining binding statements.
    *
    * Supported forms:
    *
    * value = 10;
    * value =;
    * copy := source;
    * alias :> source;
    * guarded ?= first : second;
    *
    * Including canonical modifiers:
    *
    * stone value = 10;
    * loc value = 10;
    * glo value = 10;
    * stone loc value = 10;
    * stone glo value = 10;
    *
    * Mutate (<<) is intentionally excluded because it does
    * not establish a new binding.
    */
    const bindingPattern =
        /^[ \t]*(?:(stone)\s+)?(?:(loc|glo)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(=;|:=|:>|\?=|=(?!=))/gm;

    let bindingMatch;

    while (
        (bindingMatch = bindingPattern.exec(source)) !== null
    ) {
        const stone = bindingMatch[1] === "stone";
        const scope = bindingMatch[2] || "normal";
        const name = bindingMatch[3];
        const operator = bindingMatch[4];

        /*
        * lastIndexOf is intentional here.
        *
        * Using indexOf could fail for a binding whose name
        * also appeared as part of a modifier.
        */
        const nameOffsetWithinMatch =
            bindingMatch[0].lastIndexOf(name);

        const nameOffset =
            bindingMatch.index + nameOffsetWithinMatch;

        const namePosition =
            document.positionAt(nameOffset);

        const range = new vscode.Range(
            namePosition,
            namePosition.translate(
                0,
                name.length
            )
        );

        let kind;

        switch (operator) {
            case "=":
                kind = "define";
                break;

            case "=;":
                kind = "define-empty";
                break;

            case ":=":
                kind = "copy";
                break;

            case ":>":
                kind = "bind";
                break;

            case "?=":
                kind = "guard";
                break;

            default:
                continue;
        }

        const scopeInfo = getDruimScopeAt(
            document,
            nameOffset
        );

        let ownerScope;

        if (scope === "glo") {
            ownerScope = scopeInfo.global;
        } else if (scope === "loc") {
            ownerScope = scopeInfo.local;
        } else {
            ownerScope = scopeInfo.ordinary;
        }

        bindings.push({
            name,
            kind,
            operator,
            stone,
            scope,
            ownerScope,
            range,
            offset: nameOffset
        });
    }

    return {
        functions,
        bindings,
        parameters
    };
}

function activate(context) {
    let applyingDruimEdit = false;

    const listener = vscode.workspace.onDidChangeTextDocument(async (event) => {
        if (applyingDruimEdit) {
            return;
        }

        if (event.document.languageId !== "druim") {
            return;
        }

        const editor = vscode.window.activeTextEditor;

        if (!editor || editor.document !== event.document) {
            return;
        }

        if (event.contentChanges.length !== 1) {
            return;
        }

        const change = event.contentChanges[0];

        // ==================================================
        // COMMENT DEMOTION
        // ==================================================
        //
        // If an empty multiline comment has just been created:
        //
        // :--
        //     |
        // --:
        //
        // pressing Backspace collapses it back to:
        //
        // :-|  -:
        // ==================================================

        if (change.text === "") {
            const lineNumber = change.range.start.line;

            if (
                lineNumber > 0 &&
                lineNumber < event.document.lineCount - 1
            ) {
                const previousLine = event.document.lineAt(lineNumber - 1);
                const currentLine = event.document.lineAt(lineNumber);
                const nextLine = event.document.lineAt(lineNumber + 1);

                const isEmptyMultilineComment =
                    previousLine.text.trim() === ":--" &&
                    currentLine.text.trim() === "" &&
                    nextLine.text.trim() === "--:";

                if (isEmptyMultilineComment) {
                    const indentMatch = previousLine.text.match(/^\s*/);
                    const indent = indentMatch ? indentMatch[0] : "";

                    const replaceRange = new vscode.Range(
                        lineNumber - 1,
                        0,
                        lineNumber + 1,
                        nextLine.text.length
                    );

                    applyingDruimEdit = true;

                    try {
                        const applied = await editor.edit(
                            (editBuilder) => {
                                editBuilder.replace(
                                    replaceRange,
                                    `${indent}:-  -:`
                                );
                            },
                            {
                                undoStopBefore: false,
                                undoStopAfter: false
                            }
                        );

                        if (!applied) {
                            return;
                        }

                        const cursor = new vscode.Position(
                            lineNumber - 1,
                            indent.length + 2
                        );

                        editor.selection = new vscode.Selection(
                            cursor,
                            cursor
                        );

                        return;
                    } finally {
                        applyingDruimEdit = false;
                    }
                }
            }
        }

        // ==================================================
        // COMMENT TYPING
        // ==================================================

        if (change.text === "-") {
            const position = change.range.start;

            const line = event.document.lineAt(position.line);
            const lineText = line.text;

            /*
            * After the newly typed "-" the caret is here:
            *
            * :-|
            *
            * or, after promotion:
            *
            * :--|
            */
            const caretCharacter = position.character + 1;

            const beforeCaret = lineText.slice(0, caretCharacter);
            const afterCaret = lineText.slice(caretCharacter);

            // ----------------------------------------------
            // Promote an auto-created single-line comment:
            //
            // :-|  -:
            //
            // typing another "-" gives:
            //
            // :--|  -:
            //
            // which becomes:
            //
            // :--
            //     |
            // --:
            // ----------------------------------------------

            if (
                beforeCaret.endsWith(":--") &&
                afterCaret.startsWith("  -:")
            ) {
                const indentMatch = lineText.match(/^\s*/);
                const indent = indentMatch ? indentMatch[0] : "";

                const suffixStart = new vscode.Position(
                    position.line,
                    caretCharacter
                );

                const suffixEnd = new vscode.Position(
                    position.line,
                    caretCharacter + 4
                );

                applyingDruimEdit = true;

                try {
                    const applied = await editor.edit(
                        (editBuilder) => {
                            editBuilder.replace(
                                new vscode.Range(
                                    suffixStart,
                                    suffixEnd
                                ),
                                `\n${indent}\t\n${indent}--:`
                            );
                        },
                        {
                            undoStopBefore: false,
                            undoStopAfter: false
                        }
                    );

                    if (!applied) {
                        return;
                    }

                    const cursor = new vscode.Position(
                        position.line + 1,
                        indent.length + 1
                    );

                    editor.selection = new vscode.Selection(
                        cursor,
                        cursor
                    );

                    return;
                } finally {
                    applyingDruimEdit = false;
                }
            }

            // ----------------------------------------------
            // Auto-close a newly typed single-line comment.
            //
            // Typing:
            //
            // :-
            //
            // produces:
            //
            // :-|  -:
            // ----------------------------------------------

            if (
                beforeCaret.endsWith(":-") &&
                !beforeCaret.endsWith(":--")
            ) {
                const insertPosition = new vscode.Position(
                    position.line,
                    caretCharacter
                );

                applyingDruimEdit = true;

                try {
                    const applied = await editor.edit(
                        (editBuilder) => {
                            editBuilder.insert(
                                insertPosition,
                                "  -:"
                            );
                        },
                        {
                            undoStopBefore: false,
                            undoStopAfter: false
                        }
                    );

                    if (!applied) {
                        return;
                    }

                    editor.selection = new vscode.Selection(
                        insertPosition,
                        insertPosition
                    );

                    return;
                } finally {
                    applyingDruimEdit = false;
                }
            }
        }

        // ==================================================
        // BLOCK STRUCTURE
        // ==================================================
        //
        // Druim block delimiters:
        //
        // :{
        // }{
        // }:
        //
        // must always begin at column 0.
        //
        // Typing :{ at column 0 creates:
        //
        // :{
        //     |
        // }:
        //
        // Typing }{ while indented moves it to column 0
        // and begins the next segment at indentation level 1.
        // ==================================================

        if (change.text === "{") {
            const position = change.range.start;
            const line = event.document.lineAt(position.line);
            const lineText = line.text;

            const caretCharacter = position.character + 1;
            const beforeCaret = lineText.slice(0, caretCharacter);

            // --------------------------------------------------
            // BLOCK CONTINUATION: }{
            // --------------------------------------------------

            const delimiterStart = beforeCaret.lastIndexOf("}{");

            if (delimiterStart !== -1) {
                const beforeDelimiter = lineText.slice(
                    0,
                    delimiterStart
                );

                const afterDelimiter = lineText.slice(
                    caretCharacter
                );

                applyingDruimEdit = true;

                try {
                    const replaceRange = new vscode.Range(
                        position.line,
                        delimiterStart,
                        position.line,
                        caretCharacter
                    );

                    let replacement;

                    /*
                    * If there is already code before }{
                    * on this line, preserve it and move the
                    * continuation delimiter onto the next line.
                    *
                    * Example:
                    *
                    *     value = 10;}{
                    *
                    * becomes:
                    *
                    *     value = 10;
                    * }{
                    *     |
                    */
                    if (beforeDelimiter.trim() !== "") {
                        replacement =
                            `\n}{\n\t`;
                    } else {
                        /*
                        * If the line contains only indentation
                        * before }{, remove that indentation too
                        * so }{ lands at column 0.
                        */
                        const wholePrefixRange = new vscode.Range(
                            position.line,
                            0,
                            position.line,
                            caretCharacter
                        );

                        const applied = await editor.edit(
                            (editBuilder) => {
                                editBuilder.replace(
                                    wholePrefixRange,
                                    `}{\n\t`
                                );
                            },
                            {
                                undoStopBefore: false,
                                undoStopAfter: false
                            }
                        );

                        if (!applied) {
                            return;
                        }

                        const cursor = new vscode.Position(
                            position.line + 1,
                            1
                        );

                        editor.selection = new vscode.Selection(
                            cursor,
                            cursor
                        );

                        return;
                    }

                    const applied = await editor.edit(
                        (editBuilder) => {
                            editBuilder.replace(
                                replaceRange,
                                replacement
                            );
                        },
                        {
                            undoStopBefore: false,
                            undoStopAfter: false
                        }
                    );

                    if (!applied) {
                        return;
                    }

                    const cursor = new vscode.Position(
                        position.line + 2,
                        1
                    );

                    editor.selection = new vscode.Selection(
                        cursor,
                        cursor
                    );

                    return;
                } finally {
                    applyingDruimEdit = false;
                }
            }

            // --------------------------------------------------
            // BLOCK OPENER: :{
            // --------------------------------------------------

            /*
            * :{ is only valid when ":" is character 0
            * and the newly typed "{" is character 1.
            */
            if (position.character !== 1) {
                return;
            }

            const colonRange = new vscode.Range(
                position.line,
                0,
                position.line,
                1
            );

            if (event.document.getText(colonRange) !== ":") {
                return;
            }

            const insertPosition = new vscode.Position(
                position.line,
                2
            );

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.insert(
                            insertPosition,
                            "\n\t\n}:"
                        );
                    },
                    {
                        undoStopBefore: false,
                        undoStopAfter: false
                    }
                );

                if (!applied) {
                    return;
                }

                const cursor = new vscode.Position(
                    position.line + 1,
                    1
                );

                editor.selection = new vscode.Selection(
                    cursor,
                    cursor
                );

                return;
            } finally {
                applyingDruimEdit = false;
            }
        }

        // ==================================================
        // LOOP STRUCTURE
        // ==================================================
        //
        // Typing:
        //
        // :<
        //
        // creates:
        //
        // :<
        //     |
        // >?<
        //     
        // >?<
        //     
        // >:
        //
        // The three sections are:
        // setup
        // condition
        // process
        // ==================================================

        if (change.text === "<") {
            const position = change.range.start;

            if (position.character === 0) {
                return;
            }

            const colonPosition = new vscode.Position(
                position.line,
                position.character - 1
            );

            const colonRange = new vscode.Range(
                colonPosition,
                position
            );

            if (event.document.getText(colonRange) !== ":") {
                return;
            }

            const line = event.document.lineAt(position.line);

            const indentMatch = line.text.match(/^\s*/);
            const indent = indentMatch ? indentMatch[0] : "";

            const insertPosition = new vscode.Position(
                position.line,
                position.character + 1
            );

            const innerIndent = `${indent}\t`;

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.insert(
                            insertPosition,
                            `\n${innerIndent}\n${indent}>?<\n${innerIndent}\n${indent}>?<\n${innerIndent}\n${indent}>:`
                        );
                    },
                    {
                        undoStopBefore: false,
                        undoStopAfter: false
                    }
                );

                if (!applied) {
                    return;
                }

                const setupPosition = new vscode.Position(
                    position.line + 1,
                    innerIndent.length
                );

                editor.selection = new vscode.Selection(
                    setupPosition,
                    setupPosition
                );

                return;
            } finally {
                applyingDruimEdit = false;
            }
        }

        // ==================================================
        // BOX STRUCTURE
        // ==================================================
        //
        // Normal "[" remains available for indexes:
        //
        // value[index]
        //
        // But typing:
        //
        // :[
        //
        // causes VS Code to briefly create:
        //
        // :[]
        //
        // We detect that context and promote it into:
        //
        // :[
        //     |
        // ]:
        // ==================================================

        if (change.text === "[]") {
            const start = change.range.start;

            if (start.character === 0) {
                return;
            }

            const colonPosition = new vscode.Position(
                start.line,
                start.character - 1
            );

            const colonRange = new vscode.Range(
                colonPosition,
                start
            );

            // Only treat [] as a Box when the "[" is preceded by ":".
            if (event.document.getText(colonRange) !== ":") {
                return;
            }

            const line = event.document.lineAt(start.line);

            const indentMatch = line.text.match(/^\s*/);
            const indent = indentMatch ? indentMatch[0] : "";
            const innerIndent = `${indent}\t`;

            // Current temporary state:
            //
            // :[|]
            //
            // Replace only the auto-generated "]".
            const generatedCloseRange = new vscode.Range(
                start.line,
                start.character + 1,
                start.line,
                start.character + 2
            );

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.replace(
                            generatedCloseRange,
                            `\n${innerIndent}\n${indent}]:`
                        );
                    },
                    {
                        undoStopBefore: false,
                        undoStopAfter: false
                    }
                );

                if (!applied) {
                    return;
                }

                const cursor = new vscode.Position(
                    start.line + 1,
                    innerIndent.length
                );

                editor.selection = new vscode.Selection(
                    cursor,
                    cursor
                );

                return;
            } finally {
                applyingDruimEdit = false;
            }
        }

        // ==================================================
        // BAG STRUCTURE
        // ==================================================

        if (change.text === "|") {
            const position = change.range.start;

            if (position.character === 0) {
                return;
            }

            const colonPosition = new vscode.Position(
                position.line,
                position.character - 1
            );

            const colonRange = new vscode.Range(
                colonPosition,
                position
            );

            if (event.document.getText(colonRange) !== ":") {
                return;
            }

            const line = event.document.lineAt(position.line);

            const indentMatch = line.text.match(/^\s*/);
            const indent = indentMatch ? indentMatch[0] : "";
            const innerIndent = `${indent}\t`;

            const insertPosition = new vscode.Position(
                position.line,
                position.character + 1
            );

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.insert(
                            insertPosition,
                            `\n${innerIndent}\n${indent}|:`
                        );
                    },
                    {
                        undoStopBefore: false,
                        undoStopAfter: false
                    }
                );

                if (!applied) {
                    return;
                }

                const cursor = new vscode.Position(
                    position.line + 1,
                    innerIndent.length
                );

                editor.selection = new vscode.Selection(
                    cursor,
                    cursor
                );

                return;
            } finally {
                applyingDruimEdit = false;
            }
        }        

        // ==================================================
        // FUNCTION STRUCTURE
        // ==================================================
        //
        // Normal VS Code parenthesis auto-close turns:
        //
        // :(
        //
        // into:
        //
        // :()
        //
        // We then expand it to Druim's complete function
        // structural form.
        // ==================================================

        if (change.text === "()") {
            const start = change.range.start;

            if (start.character === 0) {
                return;
            }

            const colonPosition = new vscode.Position(
                start.line,
                start.character - 1
            );

            const colonRange = new vscode.Range(
                colonPosition,
                start
            );

            if (event.document.getText(colonRange) !== ":") {
                return;
            }

            const parameterPosition = new vscode.Position(
                start.line,
                start.character + 1
            );

            const generatedCloseStart = parameterPosition;

            const generatedCloseEnd = new vscode.Position(
                start.line,
                start.character + 2
            );

            const generatedCloseRange = new vscode.Range(
                generatedCloseStart,
                generatedCloseEnd
            );

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.replace(
                            generatedCloseRange,
                            ")(\n\t\n):"
                        );
                    },
                    {
                        undoStopBefore: false,
                        undoStopAfter: false
                    }
                );

                if (!applied) {
                    return;
                }

                editor.selection = new vscode.Selection(
                    parameterPosition,
                    parameterPosition
                );
            } finally {
                applyingDruimEdit = false;
            }
        }
    });

    const foldingProvider = vscode.languages.registerFoldingRangeProvider(
        { language: "druim" },
        {
            provideFoldingRanges(document) {
                const ranges = [];
                const stack = [];

                let blockSegmentStart = null;
                let loopSegmentStart = null;

                function addRange(start, boundary, kind) {
                    // Keep the closing/continuation delimiter visible.
                    const end = boundary - 1;

                    if (start !== null && end > start) {
                        ranges.push(
                            new vscode.FoldingRange(
                                start,
                                end,
                                kind
                            )
                        );
                    }
                }

                for (let lineNumber = 0; lineNumber < document.lineCount; lineNumber++) {
                    const text = document.lineAt(lineNumber).text;

                    // --------------------------------------------------
                    // BLOCKS
                    //
                    // :{   opens a segment
                    // }{   closes that segment and opens the next
                    // }:   closes the final segment
                    // --------------------------------------------------

                    if (text.includes(":{")) {
                        blockSegmentStart = lineNumber;
                    }

                    if (text.includes("}{") && blockSegmentStart !== null) {
                        addRange(blockSegmentStart, lineNumber);

                        // This same }{ becomes the opener for the next segment.
                        blockSegmentStart = lineNumber;
                    }

                    if (text.includes("}:") && blockSegmentStart !== null) {
                        addRange(blockSegmentStart, lineNumber);

                        blockSegmentStart = null;
                    }

                    // --------------------------------------------------
                    // LOOPS
                    //
                    // :<    opens setup
                    // >?<   closes current segment and opens next
                    // >:    closes final segment
                    // --------------------------------------------------

                    if (text.includes(":<")) {
                        loopSegmentStart = lineNumber;
                    }

                    if (text.includes(">?<") && loopSegmentStart !== null) {
                        addRange(loopSegmentStart, lineNumber);

                        // This same >?< becomes the opener for the next segment.
                        loopSegmentStart = lineNumber;
                    }

                    if (text.includes(">:") && loopSegmentStart !== null) {
                        addRange(loopSegmentStart, lineNumber);

                        loopSegmentStart = null;
                    }

                    // --------------------------------------------------
                    // MULTILINE COMMENTS
                    // --------------------------------------------------

                    if (text.includes(":--")) {
                        stack.push({
                            type: "comment",
                            line: lineNumber,
                            kind: vscode.FoldingRangeKind.Comment
                        });
                    }

                    // --------------------------------------------------
                    // BOXES
                    // --------------------------------------------------

                    if (text.includes(":[")) {
                        stack.push({
                            type: "box",
                            line: lineNumber
                        });
                    }

                    // --------------------------------------------------
                    // BAGS
                    // --------------------------------------------------

                    if (text.includes(":|")) {
                        stack.push({
                            type: "bag",
                            line: lineNumber
                        });
                    }

                    // --------------------------------------------------
                    // FUNCTIONS
                    // --------------------------------------------------

                    if (
                        /\bfn\s+[A-Za-z0-9_]*[A-Za-z_][A-Za-z0-9_]*\s+:\(.*\)\(\s*$/.test(text)
                    ) {
                        stack.push({
                            type: "function",
                            line: lineNumber
                        });
                    }

                    // --------------------------------------------------
                    // SIMPLE STRUCTURAL CLOSERS
                    // --------------------------------------------------

                    const closingStructures = [
                        {
                            type: "comment",
                            close: "--:"
                        },
                        {
                            type: "box",
                            close: "]:"
                        },
                        {
                            type: "bag",
                            close: "|:"
                        },
                        {
                            type: "function",
                            close: "):"
                        }
                    ];

                    for (const closing of closingStructures) {
                        if (!text.includes(closing.close)) {
                            continue;
                        }

                        for (let i = stack.length - 1; i >= 0; i--) {
                            if (stack[i].type !== closing.type) {
                                continue;
                            }

                            const opening = stack.splice(i, 1)[0];

                            addRange(
                                opening.line,
                                lineNumber,
                                opening.kind
                            );

                            break;
                        }
                    }
                }

                return ranges;
            }
        }
    );

    const toggleLineComment = vscode.commands.registerCommand(
        "druim.toggleLineComment",
        async () => {
            const editor = vscode.window.activeTextEditor;

            if (!editor || editor.document.languageId !== "druim") {
                return;
            }

            const document = editor.document;
            const selection = editor.selection;

            let startLine = selection.start.line;
            let endLine = selection.end.line;

            // A selection ending at column 0 should not include
            // the following line.
            if (
                endLine > startLine &&
                selection.end.character === 0
            ) {
                endLine--;
            }

            const isMultiLine = startLine !== endLine;

            // --------------------------------------------------
            // SINGLE LINE
            // --------------------------------------------------

            if (!isMultiLine) {
                const line = document.lineAt(startLine);
                const text = line.text;

                const indentMatch = text.match(/^\s*/);
                const indent = indentMatch ? indentMatch[0] : "";
                const content = text.slice(indent.length);

                const range = new vscode.Range(
                    startLine,
                    0,
                    startLine,
                    text.length
                );

                const isCommented =
                    content.startsWith(":-") &&
                    content.endsWith("-:");

                await editor.edit((editBuilder) => {
                    if (isCommented) {
                        let uncommented = content.slice(2, -2);

                        if (uncommented.startsWith(" ")) {
                            uncommented = uncommented.slice(1);
                        }

                        if (uncommented.endsWith(" ")) {
                            uncommented = uncommented.slice(0, -1);
                        }

                        editBuilder.replace(
                            range,
                            `${indent}${uncommented}`
                        );
                    } else {
                        editBuilder.replace(
                            range,
                            `${indent}:- ${content} -:`
                        );
                    }
                });

                return;
            }

            // --------------------------------------------------
            // MULTILINE SELECTION
            // --------------------------------------------------

            const firstLine = document.lineAt(startLine);
            const lastLine = document.lineAt(endLine);

            const fullRange = new vscode.Range(
                startLine,
                0,
                endLine,
                lastLine.text.length
            );

            const selectedText = document.getText(fullRange);

            const indentMatch = firstLine.text.match(/^\s*/);
            const indent = indentMatch ? indentMatch[0] : "";
            const innerIndent = `${indent}\t`;

            const trimmed = selectedText.trim();

            const isBlockComment =
                trimmed.startsWith(":--") &&
                trimmed.endsWith("--:");

            await editor.edit((editBuilder) => {
                if (isBlockComment) {
                    const lines = selectedText.split("\n");

                    // Remove opening and closing delimiters.
                    lines.shift();
                    lines.pop();

                    // Remove one indentation level that Ctrl+/ added.
                    const uncommented = lines
                        .map((line) => {
                            if (line.startsWith(`${indent}\t`)) {
                                return `${indent}${line.slice((indent + "\t").length)}`;
                            }

                            if (line.startsWith(`${indent}    `)) {
                                return `${indent}${line.slice((indent + "    ").length)}`;
                            }

                            return line;
                        })
                        .join("\n");

                    editBuilder.replace(
                        fullRange,
                        uncommented
                    );
                } else {
                    const indentedContent = selectedText
                        .split("\n")
                        .map((line) => {
                            // Preserve blank lines.
                            if (line.trim() === "") {
                                return "";
                            }

                            // Remove the existing outer indentation first
                            // so we don't double-indent nested code.
                            const content = line.startsWith(indent)
                                ? line.slice(indent.length)
                                : line;

                            return `${innerIndent}${content}`;
                        })
                        .join("\n");

                    editBuilder.replace(
                        fullRange,
                        `${indent}:--\n${indentedContent}\n${indent}--:`
                    );
                }
            });
        }
    );

    const completionProvider = vscode.languages.registerCompletionItemProvider(
        { language: "druim" },
        {
            provideCompletionItems() {
                const items = [];

                function keyword(label, detail, documentation) {
                    const item = new vscode.CompletionItem(
                        label,
                        vscode.CompletionItemKind.Keyword
                    );

                    item.detail = detail;
                    item.documentation = new vscode.MarkdownString(documentation);

                    items.push(item);
                }

                function operator(label, insertText, detail, documentation) {
                    const item = new vscode.CompletionItem(
                        label,
                        vscode.CompletionItemKind.Operator
                    );

                    item.insertText = insertText;
                    item.detail = detail;
                    item.documentation = new vscode.MarkdownString(documentation);

                    items.push(item);
                }

                // --------------------------------------------------
                // Keywords
                // --------------------------------------------------

                keyword(
                    "fn",
                    "Druim function declaration",
                    "Declares a Druim function."
                );

                keyword(
                    "ret",
                    "Druim return statement",
                    "Returns a value from the current function."
                );

                // --------------------------------------------------
                // Scope / identity modifiers
                // --------------------------------------------------

                keyword(
                    "loc",
                    "Local scope modifier",
                    "Applies local lifetime/scope behavior."
                );

                keyword(
                    "glo",
                    "Global scope modifier",
                    "Targets global scope."
                );

                keyword(
                    "stone",
                    "Immutable identity modifier",
                    "Makes the resulting binding identity immutable."
                );

                // --------------------------------------------------
                // Types / literals
                // --------------------------------------------------

                keyword(
                    "num",
                    "Druim number type",
                    "Integer numeric type."
                );

                keyword(
                    "dec",
                    "Druim decimal type",
                    "Decimal numeric type."
                );

                keyword(
                    "flag",
                    "Druim flag type",
                    "Boolean flag type."
                );

                keyword(
                    "text",
                    "Druim text type",
                    "Text value type."
                );

                keyword(
                    "void",
                    "Druim void value",
                    "Represents the absence of a value."
                );

                keyword(
                    "true",
                    "Druim flag literal",
                    "`true` flag value."
                );

                keyword(
                    "false",
                    "Druim flag literal",
                    "`false` flag value."
                );

                // --------------------------------------------------
                // Statement operators
                // --------------------------------------------------

                operator(
                    "Define  =",
                    "=",
                    "Define",
                    "Defines a new binding. `=` is definition-only; it does not mutate an existing binding."
                );

                operator(
                    "Mutate  <<",
                    "<<",
                    "Mutate",
                    "Mutates an existing binding."
                );

                operator(
                    "Bind  :>",
                    ":>",
                    "Bind",
                    "Creates a binding that shares identity with its source."
                );

                operator(
                    "Copy  :=",
                    ":=",
                    "Copy",
                    "Creates a fresh identity containing a copied value."
                );

                operator(
                    "Guard  ?=",
                    "?=",
                    "Guard",
                    "Druim guard operator."
                );

                operator(
                    "Print  |>",
                    "|>",
                    "Print",
                    "Prints one expression using `|> (expression);`."
                );

                return items;
            }
        }
    );

    const hoverProvider = vscode.languages.registerHoverProvider(
        { language: "druim" },
        {
            provideHover(document, position) {
                const range = document.getWordRangeAtPosition(
                    position,
                    /(?:stone|loc|glo|fn|ret|num|dec|flag|text|void|true|false|<<|:>|:=|\?=|\|>|=)/
                );

                if (!range) {
                    return;
                }

                const token = document.getText(range);

                const docs = {
                    stone: [
                        "**stone**",
                        "",
                        "Marks the resulting binding identity as immutable."
                    ].join("\n"),

                    loc: [
                        "**loc**",
                        "",
                        "Applies local lifetime/scope behavior."
                    ].join("\n"),

                    glo: [
                        "**glo**",
                        "",
                        "Targets global scope."
                    ].join("\n"),

                    fn: [
                        "**fn**",
                        "",
                        "Declares a Druim function."
                    ].join("\n"),

                    ret: [
                        "**ret**",
                        "",
                        "Returns a value from the current function."
                    ].join("\n"),

                    num: [
                        "**num**",
                        "",
                        "Integer numeric type."
                    ].join("\n"),

                    dec: [
                        "**dec**",
                        "",
                        "Decimal numeric type."
                    ].join("\n"),

                    flag: [
                        "**flag**",
                        "",
                        "Boolean flag type."
                    ].join("\n"),

                    text: [
                        "**text**",
                        "",
                        "Text value type."
                    ].join("\n"),

                    void: [
                        "**void**",
                        "",
                        "Represents the absence of a value."
                    ].join("\n"),

                    true: [
                        "**true**",
                        "",
                        "Boolean flag literal."
                    ].join("\n"),

                    false: [
                        "**false**",
                        "",
                        "Boolean flag literal."
                    ].join("\n"),

                    "=": [
                        "**Define `=`**",
                        "",
                        "Defines a new binding.",
                        "",
                        "`=` is definition-only and does not mutate an existing binding."
                    ].join("\n"),

                    "<<": [
                        "**Mutate `<<`**",
                        "",
                        "Mutates an existing binding."
                    ].join("\n"),

                    ":>": [
                        "**Bind `:>`**",
                        "",
                        "Creates a binding that shares identity with its source."
                    ].join("\n"),

                    ":=": [
                        "**Copy `:=`**",
                        "",
                        "Creates a fresh identity containing a copied value."
                    ].join("\n"),

                    "?=": [
                        "**Guard `?=`**",
                        "",
                        "Druim guard operator."
                    ].join("\n"),

                    "|>": [
                        "**Print `|>`**",
                        "",
                        "Prints one expression.",
                        "",
                        "```druim",
                        '|> ("Hello World!");',
                        "```"
                    ].join("\n")
                };

                const documentation = docs[token];

                if (!documentation) {
                    return;
                }

                const markdown = new vscode.MarkdownString(documentation);
                markdown.supportHtml = false;

                return new vscode.Hover(
                    markdown,
                    range
                );
            }
        }
    );

    const signatureProvider = vscode.languages.registerSignatureHelpProvider(
        { language: "druim" },
        {
            provideSignatureHelp(document, position) {
                const textBeforeCursor = document.getText(
                    new vscode.Range(
                        new vscode.Position(0, 0),
                        position
                    )
                );

                /*
                * Find the function call currently being written.
                *
                * Examples:
                *
                * calculate(
                * calculate(value,
                * calculate(value, multiplier
                */
                const callMatch = textBeforeCursor.match(
                    /([A-Za-z_][A-Za-z0-9_]*)\(([^()]*)$/
                );

                if (!callMatch) {
                    return null;
                }

                const functionName = callMatch[1];
                const argumentText = callMatch[2];

                /*
                * Find the matching Druim function declaration.
                *
                * Canonical form:
                *
                * fn calculate :(value, multiplier)(
                *
                * Parameters are deliberately captured as text here.
                * We are NOT assuming that parameters are only bare
                * identifiers; Druim parameter forms may include defaults.
                */
                const index = buildDruimSymbolIndex(document);

                const symbol = index.functions.get(
                    functionName
                );

                if (!symbol) {
                    return null;
                }

                const parameters = symbol.parameters;

                /*
                * Empty parameter block:
                *
                * fn example :()(
                */
                if (parameters.length === 0) {
                    const signature = new vscode.SignatureInformation(
                        `${functionName}()`
                    );

                    const help = new vscode.SignatureHelp();

                    help.signatures = [signature];
                    help.activeSignature = 0;
                    help.activeParameter = 0;

                    return help;
                }

                const signature = new vscode.SignatureInformation(
                    `${functionName}(${parameters.join(", ")})`
                );

                signature.parameters = parameters.map(
                    (parameter) =>
                        new vscode.ParameterInformation(
                            parameter
                        )
                );

                /*
                * Determine which argument is currently active.
                */
                let activeParameter = 0;
                let callParenDepth = 0;
                let callBracketDepth = 0;
                let callInString = false;
                let callEscaped = false;

                for (const char of argumentText) {
                    if (callInString) {
                        if (callEscaped) {
                            callEscaped = false;
                            continue;
                        }

                        if (char === "\\") {
                            callEscaped = true;
                            continue;
                        }

                        if (char === "\"") {
                            callInString = false;
                        }

                        continue;
                    }

                    if (char === "\"") {
                        callInString = true;
                        continue;
                    }

                    if (char === "(") {
                        callParenDepth++;
                        continue;
                    }

                    if (char === ")") {
                        callParenDepth--;
                        continue;
                    }

                    if (char === "[") {
                        callBracketDepth++;
                        continue;
                    }

                    if (char === "]") {
                        callBracketDepth--;
                        continue;
                    }

                    if (
                        char === "," &&
                        callParenDepth === 0 &&
                        callBracketDepth === 0
                    ) {
                        activeParameter++;
                    }
                }

                if (parameters.length > 0) {
                    activeParameter = Math.min(
                        activeParameter,
                        parameters.length - 1
                    );
                }

                const help = new vscode.SignatureHelp();

                help.signatures = [signature];
                help.activeSignature = 0;
                help.activeParameter = activeParameter;

                return help;
            }
        },
        "(",
        ","
    );

    const definitionProvider = vscode.languages.registerDefinitionProvider(
        { language: "druim" },
        {
            provideDefinition(document, position) {
                /*
                * Get the identifier under the cursor.
                */
                const wordRange = document.getWordRangeAtPosition(
                    position,
                    /[A-Za-z_][A-Za-z0-9_]*/
                );

                if (!wordRange) {
                    return null;
                }

                const identifier = document.getText(wordRange);

                const line = document.lineAt(position.line);
                const afterWord = line.text.slice(wordRange.end.character);

                const index = buildDruimSymbolIndex(document);

                /*
                * Function call:
                *
                * calculate(...)
                */
                if (/^\s*\(/.test(afterWord)) {
                    const functionSymbol = index.functions.get(
                        identifier
                    );

                    if (!functionSymbol) {
                        return null;
                    }

                    return new vscode.Location(
                        document.uri,
                        functionSymbol.range
                    );
                }

                /*
                * Ordinary binding reference.
                */
                const useOffset = document.offsetAt(
                    wordRange.start
                );

                const binding = resolveDruimBinding(
                    document,
                    index,
                    identifier,
                    useOffset
                );

                if (!binding) {
                    return null;
                }

                return new vscode.Location(
                    document.uri,
                    binding.range
                );
            }
        }
    );

    context.subscriptions.push(
        listener,
        foldingProvider,
        toggleLineComment,
        completionProvider,
        hoverProvider, 
        signatureProvider,
        definitionProvider
    );
}

function deactivate() {}

module.exports = {
    activate,
    deactivate
};