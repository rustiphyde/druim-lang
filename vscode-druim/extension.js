const vscode = require("vscode");
const druimLanguage = require("./druim-language");

function getDruimLexicalContextAt(
    document,
    offset
) {
    const source = document.getText();

    let context = "code";
    let escaped = false;

    for (let i = 0; i < offset; i++) {
        if (context === "single-comment") {
            if (source.startsWith("-:", i)) {
                context = "code";
                i++;
            }

            continue;
        }

        if (context === "multiline-comment") {
            if (source.startsWith("--:", i)) {
                context = "code";
                i += 2;
            }

            continue;
        }

        if (context === "text") {
            if (escaped) {
                escaped = false;
                continue;
            }

            if (source[i] === "\\") {
                escaped = true;
                continue;
            }

            if (source.startsWith(":.", i)) {
                context = "interpolation";
                i++;
                continue;
            }

            if (source[i] === "\"") {
                context = "code";
            }

            continue;
        }

        if (context === "interpolation") {
            if (source.startsWith(".:", i)) {
                context = "text";
                i++;
                continue;
            }

            if (source[i] === "\"") {
                context = "text";
                continue;
            }

            continue;
        }

        /*
         * Code context.
         *
         * Longest-match ordering matters here because
         * :-- begins with :-.
         */
        if (source.startsWith(":--", i)) {
            context = "multiline-comment";
            i += 2;
            continue;
        }

        if (source.startsWith(":-", i)) {
            context = "single-comment";
            i++;
            continue;
        }

        if (source[i] === "\"") {
            context = "text";
        }
    }

    return context;
}

function isDruimCodeContext(
    document,
    position
) {
    const offset =
        document.offsetAt(position);

    const context =
        getDruimLexicalContextAt(
            document,
            offset
        );

    return (
        context === "code" ||
        context === "interpolation"
    );
}

function isDruimFunctionListContext(
    document,
    position
) {
    const source = document.getText();
    const offset = document.offsetAt(position);

    let context = "code";
    let escaped = false;

    let inFunctionParameters = false;
    let callParenDepth = 0;

    for (let i = 0; i < offset; i++) {
        if (context === "single-comment") {
            if (source.startsWith("-:", i)) {
                context = "code";
                i++;
            }

            continue;
        }

        if (context === "multiline-comment") {
            if (source.startsWith("--:", i)) {
                context = "code";
                i += 2;
            }

            continue;
        }

        if (context === "text") {
            if (escaped) {
                escaped = false;
                continue;
            }

            if (source[i] === "\\") {
                escaped = true;
                continue;
            }

            if (source.startsWith(":.", i)) {
                context = "interpolation";
                i++;
                continue;
            }

            if (source[i] === "\"") {
                context = "code";
            }

            continue;
        }

        if (context === "interpolation") {
            if (source.startsWith(".:", i)) {
                context = "text";
                i++;
                continue;
            }
        }

        if (source.startsWith(":--", i)) {
            context = "multiline-comment";
            i += 2;
            continue;
        }

        if (source.startsWith(":-", i)) {
            context = "single-comment";
            i++;
            continue;
        }

        if (
            context === "code" &&
            source[i] === "\""
        ) {
            context = "text";
            escaped = false;
            continue;
        }

        /*
         * Function parameter list:
         *
         * fn name :( ... )(
         */
        if (source.startsWith(":(", i)) {
            inFunctionParameters = true;
            i++;
            continue;
        }

        if (
            inFunctionParameters &&
            source.startsWith(")(", i)
        ) {
            inFunctionParameters = false;
            i++;
            continue;
        }

        /*
         * Function call argument list.
         *
         * Once a call begins, nested parentheses remain
         * part of that argument list until the matching
         * call parenthesis closes.
         */
        if (
            !inFunctionParameters &&
            source[i] === "("
        ) {
            if (callParenDepth > 0) {
                callParenDepth++;
                continue;
            }

            let previous = i - 1;

            while (
                previous >= 0 &&
                /\s/.test(source[previous])
            ) {
                previous--;
            }

            let nameStart = previous;

            while (
                nameStart >= 0 &&
                /[A-Za-z0-9_]/.test(
                    source[nameStart]
                )
            ) {
                nameStart--;
            }

            nameStart++;

            const callee =
                source.slice(
                    nameStart,
                    previous + 1
                );

            if (
                /^(?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+$/.test(
                    callee
                )
            ) {
                callParenDepth = 1;
            }

            continue;
        }

        if (
            !inFunctionParameters &&
            source[i] === ")" &&
            callParenDepth > 0
        ) {
            callParenDepth--;
        }
    }

    return (
        inFunctionParameters ||
        callParenDepth > 0
    );
}

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

    let context = "code";
    let escaped = false;

    for (let i = 0; i < offset; i++) {
        /*
         * --------------------------------------------------
         * LEXICAL STATE
         * --------------------------------------------------
         */

        if (context === "single-comment") {
            if (source.startsWith("-:", i)) {
                context = "code";
                i++;
            }

            continue;
        }

        if (context === "multiline-comment") {
            if (source.startsWith("--:", i)) {
                context = "code";
                i += 2;
            }

            continue;
        }

        if (context === "text") {
            if (escaped) {
                escaped = false;
                continue;
            }

            if (source[i] === "\\") {
                escaped = true;
                continue;
            }

            if (source.startsWith(":.", i)) {
                context = "interpolation";
                i++;
                continue;
            }

            if (source[i] === "\"") {
                context = "code";
            }

            continue;
        }

        if (context === "interpolation") {
            if (source.startsWith(".:", i)) {
                context = "text";
                i++;
                continue;
            }

            /*
             * Interpolation is executable Druim code,
             * so scope delimiters below are allowed to
             * be processed normally.
             */
        }

        /*
         * Comment and text openers are recognized in
         * executable code, including interpolation.
         */

        if (source.startsWith(":--", i)) {
            context = "multiline-comment";
            i += 2;
            continue;
        }

        if (source.startsWith(":-", i)) {
            context = "single-comment";
            i++;
            continue;
        }

        if (
            context === "code" &&
            source[i] === "\""
        ) {
            context = "text";
            escaped = false;
            continue;
        }

        /*
         * --------------------------------------------------
         * BLOCK SCOPE
         * --------------------------------------------------
         *
         * :{ creates one lexical scope.
         * }{ changes segment but keeps that same scope.
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
                .find(
                    (item) =>
                        item.type === "block"
                );

            if (scope) {
                scope.end = i + 2;

                const index =
                    stack.lastIndexOf(scope);

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
         * --------------------------------------------------
         * LOOP SCOPE
         * --------------------------------------------------
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
                .find(
                    (item) =>
                        item.type === "loop"
                );

            if (scope) {
                scope.end = i + 2;

                const index =
                    stack.lastIndexOf(scope);

                if (index !== -1) {
                    stack.splice(index, 1);
                }
            }

            i++;
            continue;
        }

        /*
         * --------------------------------------------------
         * FUNCTION SCOPE
         * --------------------------------------------------
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
                .find(
                    (item) =>
                        item.type === "function"
                );

            if (scope) {
                scope.end = i + 2;

                const index =
                    stack.lastIndexOf(scope);

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

function isDruimFunctionVisible(
    document,
    symbol,
    useOffset
) {
    const useScope = getDruimScopeAt(
        document,
        useOffset
    );

    if (symbol.scope === "glo") {
        return true;
    }

    if (
        symbol.ownerScope.type ===
        "block-segment"
    ) {
        return (
            useScope.local.type ===
                "block-segment" &&
            sameDruimScope(
                symbol.ownerScope,
                useScope.local
            )
        );
    }

    return useScope.chain.some(
        (scope) =>
            sameDruimScope(
                symbol.ownerScope,
                scope
            )
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
        /^[ \t]*(?:(loc|glo)\s+)?fn\s+((?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+)\s+:\(([\s\S]*?)\)\(/gm;

    let match;

    while ((match = functionPattern.exec(source)) !== null) {
        const matchPosition =
            document.positionAt(match.index);

        if (
            !isDruimCodeContext(
                document,
                matchPosition
            )
        ) {
            continue;
        }

        const scope = match[1] || "Normal";
            const functionName = match[2];
            const parameterText = match[3];

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
                    /^((?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+)/
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

            functions.set(
                functionName,
                {
                    name: functionName,
                    range: nameRange,
                    parameters: parameterForms,
                    scope,
                    ownerScope,
                    offset: nameOffset
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
        /^[ \t]*(?:(stone)\s+)?(?:(loc|glo)\s+)?((?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+)\s*(=;|:=|:>|\?=|=(?!=))/gm;

    let bindingMatch;

    while (
        (bindingMatch = bindingPattern.exec(source)) !== null
    ) {
        const matchPosition =
            document.positionAt(bindingMatch.index);

        if (
            !isDruimCodeContext(
                document,
                matchPosition
            )
        ) {
            continue;
        }

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

    /*
    * Collect explicit scope transitions performed through Mutate.
    *
    * loc target << expression;
    * glo target << expression;
    * stone loc target << expression;
    * stone glo target << expression;
    *
    * Mutate does not create a new identity declaration, so these
    * entries retain the original declaration range while recording
    * the scope commitment that becomes active at the mutation point.
    */
    const scopedMutatePattern =
        /^[ \t]*(?:(stone)\s+)?(loc|glo)\s+((?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+)\s*<</gm;

    let scopedMutateMatch;

    while (
        (scopedMutateMatch =
            scopedMutatePattern.exec(source)) !== null
    ) {
        const matchPosition =
            document.positionAt(
                scopedMutateMatch.index
            );

        if (
            !isDruimCodeContext(
                document,
                matchPosition
            )
        ) {
            continue;
        }

        const stone =
            scopedMutateMatch[1] === "stone";

        const scope =
            scopedMutateMatch[2];

        const name =
            scopedMutateMatch[3];

        const nameOffsetWithinMatch =
            scopedMutateMatch[0].lastIndexOf(name);

        const nameOffset =
            scopedMutateMatch.index +
            nameOffsetWithinMatch;

        /*
        * Resolve the identity as it exists immediately before
        * this scope-modified mutation.
        */
        const current = resolveDruimBinding(
            document,
            {
                functions,
                bindings,
                parameters
            },
            name,
            nameOffset
        );

        if (!current) {
            continue;
        }

        /*
        * Explicit loc -> glo and glo -> loc transitions are
        * invalid and therefore must not be indexed as successful
        * scope transitions.
        */
        if (
            scope === "glo" &&
            current.scope === "loc"
        ) {
            continue;
        }

        if (
            scope === "loc" &&
            current.scope === "glo"
        ) {
            continue;
        }

        const scopeInfo = getDruimScopeAt(
            document,
            nameOffset
        );

        const ownerScope =
            scope === "glo"
                ? scopeInfo.global
                : scopeInfo.local;

        bindings.push({
            name,
            kind: "scope-mutate",
            operator: "<<",
            stone,
            scope,
            ownerScope,

            /*
            * Go To Definition should still lead to the identity's
            * original declaration rather than to the Mutate token.
            */
            range: current.range,

            /*
            * The new scope commitment becomes visible only after
            * this Mutate is reached in source order.
            */
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

    const blockStructure = druimLanguage.structures.block;
    const loopStructure = druimLanguage.structures.loop;
    const boxStructure = druimLanguage.structures.box;
    const bagStructure = druimLanguage.structures.bag;
    const functionStructure = druimLanguage.structures.function;
    const lineCommentStructure = druimLanguage.structures.lineComment;
    const multilineCommentStructure = druimLanguage.structures.multilineComment;

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
        // COLLECTION DEMOTION
        // ==================================================
        //
        // If Backspace removes the opener from an empty
        // auto-closed Box or Bag:
        //
        // :[|]:  ->  :|
        // :||:   ->  :|
        //
        // also remove the generated closing delimiter.
        // ==================================================

        /*
        * VS Code treats [] as an auto-closing pair.
        *
        * Backspace inside:
        *
        * :[|]:
        *
        * removes both "[" and "]" itself, temporarily leaving:
        *
        * ::
        *
        * Remove the second ":" so the structure fully demotes to:
        *
        * :
        */
        if (
            change.rangeLength === 2 &&
            isDruimCodeContext(
                event.document,
                change.range.start
            )
        ) {
            const position = change.range.start;

            const line = event.document.lineAt(position.line);
            const lineText = line.text;

            if (
                position.character > 0 &&
                lineText[position.character - 1] === ":" &&
                lineText[position.character] === ":"
            ) {
                const generatedColonRange = new vscode.Range(
                    position.line,
                    position.character,
                    position.line,
                    position.character + 1
                );

                applyingDruimEdit = true;

                try {
                    const applied = await editor.edit(
                        (editBuilder) => {
                            editBuilder.delete(
                                generatedColonRange
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
                        position.line,
                        position.character
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

        if (
            change.rangeLength === 1 &&
            isDruimCodeContext(
                event.document,
                change.range.start
            )
        ) {
            const position = change.range.start;

            const line = event.document.lineAt(position.line);
            const lineText = line.text;

            if (position.character > 0) {
                const beforePosition =
                    lineText[position.character - 1];

                let close = null;

                if (
                    beforePosition === ":" &&
                    lineText.startsWith(
                        boxStructure.close,
                        position.character
                    )
                ) {
                    close = boxStructure.close;
                } else if (
                    beforePosition === ":" &&
                    lineText.startsWith(
                        bagStructure.close,
                        position.character
                    )
                ) {
                    close = bagStructure.close;
                }

                if (close) {
                    const closeRange = new vscode.Range(
                        position.line,
                        position.character,
                        position.line,
                        position.character + close.length
                    );

                    applyingDruimEdit = true;

                    try {
                        const applied = await editor.edit(
                            (editBuilder) => {
                                editBuilder.delete(closeRange);
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
                            position.line,
                            position.character
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
        // FUNCTION DEMOTION
        // ==================================================
        //
        // Backspace from an untouched empty parameter pair:
        //
        // fn example :(|)(
        //     
        // ):
        //
        // VS Code removes the paired "()", temporarily leaving:
        //
        // fn example :(
        //     
        // ):
        //
        // Remove the remaining generated function body so the
        // declaration returns to:
        //
        // fn example :
        // ==================================================

        if (
            change.text === "" &&
            change.rangeLength === 2 &&
            isDruimCodeContext(
                event.document,
                change.range.start
            )
        ) {
            const position = change.range.start;

            const lineNumber = position.line;

            if (
                position.character > 0 &&
                lineNumber + 2 < event.document.lineCount
            ) {
                const openerLine =
                    event.document.lineAt(lineNumber);

                const bodyLine =
                    event.document.lineAt(lineNumber + 1);

                const closeLine =
                    event.document.lineAt(lineNumber + 2);

                const lineText = openerLine.text;

                const isEmptyGeneratedFunction =
                    lineText[position.character - 1] === ":" &&
                    lineText.slice(position.character) === "(" &&
                    bodyLine.text.trim() === "" &&
                    closeLine.text.trim() ===
                        functionStructure.close;

                if (isEmptyGeneratedFunction) {
                    const generatedRange =
                        new vscode.Range(
                            lineNumber,
                            position.character,
                            lineNumber + 2,
                            closeLine.text.length
                        );

                    applyingDruimEdit = true;

                    try {
                        const applied = await editor.edit(
                            (editBuilder) => {
                                editBuilder.delete(
                                    generatedRange
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

                        const cursor =
                            new vscode.Position(
                                lineNumber,
                                position.character
                            );

                        editor.selection =
                            new vscode.Selection(
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
        // LOOP DEMOTION
        // ==================================================
        //
        // Backspace from the untouched first setup line:
        //
        // :<
        //     |
        // >?<
        //
        // >?<
        //
        // >:
        //
        // removes the generated loop and leaves the colon that
        // originally introduced it.
        //
        // Detection is structural, so indentation depth does not
        // matter.
        // ==================================================

        if (
            change.text === "" &&
            change.rangeLength === 1 &&
            isDruimCodeContext(
                event.document,
                change.range.start
            )
        ) {
            const setupLineNumber =
                change.range.start.line;

            if (
                setupLineNumber > 0 &&
                setupLineNumber + 5 <
                    event.document.lineCount
            ) {
                const openerLineNumber =
                    setupLineNumber - 1;

                const openerLine =
                    event.document.lineAt(
                        openerLineNumber
                    );

                const setupLine =
                    event.document.lineAt(
                        setupLineNumber
                    );

                const firstSeparatorLine =
                    event.document.lineAt(
                        setupLineNumber + 1
                    );

                const conditionLine =
                    event.document.lineAt(
                        setupLineNumber + 2
                    );

                const secondSeparatorLine =
                    event.document.lineAt(
                        setupLineNumber + 3
                    );

                const processLine =
                    event.document.lineAt(
                        setupLineNumber + 4
                    );

                const closeLine =
                    event.document.lineAt(
                        setupLineNumber + 5
                    );

                const openerText =
                    openerLine.text.trimEnd();

                const isEmptyGeneratedLoop =
                    openerText.endsWith(
                        loopStructure.open
                    ) &&
                    setupLine.text.trim() === "" &&
                    firstSeparatorLine.text.trim() ===
                        loopStructure.separators[0] &&
                    conditionLine.text.trim() === "" &&
                    secondSeparatorLine.text.trim() ===
                        loopStructure.separators[1] &&
                    processLine.text.trim() === "" &&
                    closeLine.text.trim() ===
                        loopStructure.close;

                if (isEmptyGeneratedLoop) {
                    /*
                    * Leave the ":" and remove everything
                    * beginning with "<".
                    */
                    const lessThanCharacter =
                        openerText.length - 1;

                    const generatedRange =
                        new vscode.Range(
                            openerLineNumber,
                            lessThanCharacter,
                            setupLineNumber + 5,
                            closeLine.text.length
                        );

                    applyingDruimEdit = true;

                    try {
                        const applied = await editor.edit(
                            (editBuilder) => {
                                editBuilder.delete(
                                    generatedRange
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

                        const cursor =
                            new vscode.Position(
                                openerLineNumber,
                                lessThanCharacter
                            );

                        editor.selection =
                            new vscode.Selection(
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
        // SINGLE-LINE COMMENT DEMOTION
        // ==================================================
        //
        // Backspace from:
        //
        // :-|  -:
        //
        // removes the typed "-" and temporarily leaves:
        //
        // :|  -:
        //
        // Remove the generated closer too, leaving:
        //
        // :|
        // ==================================================

        if (
            change.text === "" &&
            change.rangeLength === 1
        ) {
            const position =
                change.range.start;

            const line =
                event.document.lineAt(
                    position.line
                );

            const lineText =
                line.text;

            if (
                position.character > 0 &&
                lineText[position.character - 1] === ":" &&
                lineText.slice(
                    position.character,
                    position.character + 4
                ) === "  -:"
            ) {
                const generatedCloseRange =
                    new vscode.Range(
                        position.line,
                        position.character,
                        position.line,
                        position.character + 4
                    );

                applyingDruimEdit = true;

                try {
                    const applied =
                        await editor.edit(
                            (editBuilder) => {
                                editBuilder.delete(
                                    generatedCloseRange
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

                    const cursor =
                        new vscode.Position(
                            position.line,
                            position.character
                        );

                    editor.selection =
                        new vscode.Selection(
                            cursor,
                            cursor
                        );

                    return;
                } finally {
                    applyingDruimEdit = false;
                }
            }
        }

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
                    previousLine.text.trim() === multilineCommentStructure.open &&
                    currentLine.text.trim() === "" &&
                    nextLine.text.trim() === multilineCommentStructure.close;

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
                                    `${indent}${lineCommentStructure.open}  ${lineCommentStructure.close}`
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

            if (
                isDruimFunctionListContext(
                    event.document,
                    position
                )
            ) {
                const line =
                    event.document.lineAt(
                        position.line
                    );

                const lineText = line.text;

                /*
                * If this "-" completed a forbidden comment
                * opener inside parameters or arguments:
                *
                * :-
                *
                * remove the "-" immediately so the source
                * returns to:
                *
                * :
                */
                if (
                    position.character > 0 &&
                    lineText[
                        position.character - 1
                    ] === ":"
                ) {
                    const typedHyphenRange =
                        new vscode.Range(
                            position.line,
                            position.character,
                            position.line,
                            position.character + 1
                        );

                    applyingDruimEdit = true;

                    try {
                        const applied =
                            await editor.edit(
                                (editBuilder) => {
                                    editBuilder.delete(
                                        typedHyphenRange
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

                        const cursor =
                            new vscode.Position(
                                position.line,
                                position.character
                            );

                        editor.selection =
                            new vscode.Selection(
                                cursor,
                                cursor
                            );

                        return;
                    } finally {
                        applyingDruimEdit = false;
                    }
                }

                return;
            }

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
                beforeCaret.endsWith(multilineCommentStructure.open) &&
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
                                `\n${indent}\t\n${indent}${multilineCommentStructure.close}`
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
                beforeCaret.endsWith(lineCommentStructure.open) &&
                !beforeCaret.endsWith(multilineCommentStructure.open)
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
                                `  ${lineCommentStructure.close}`
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

        if (
            change.text.includes("{") &&
            isDruimCodeContext(
                event.document,
                new vscode.Position(
                    change.range.start.line,
                    Math.max(
                        0,
                        change.range.start.character - 1
                    )
                )
            )
        ) {
            const position = change.range.start;
            const line = event.document.lineAt(position.line);
            const lineText = line.text;

            const caretCharacter = position.character + 1;
            const beforeCaret = lineText.slice(0, caretCharacter);

            // --------------------------------------------------
            // BLOCK CONTINUATION: }{
            // --------------------------------------------------

            const delimiterStart =
                beforeCaret.lastIndexOf(blockStructure.continuation);

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
                            `${blockStructure.continuation}\n\t`;
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
                                    `${blockStructure.continuation}\n\t`
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
                            `\n\t\n${blockStructure.close}`
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

        if (
            change.text === "<" &&
            isDruimCodeContext(
                event.document,
                new vscode.Position(
                    change.range.start.line,
                    Math.max(
                        0,
                        change.range.start.character - 1
                    )
                )
            )
        ) {
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

            const firstSeparator = loopStructure.separators[0];
            const secondSeparator = loopStructure.separators[1];

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.insert(
                            insertPosition,
                            `\n${innerIndent}\n` +
                            `${indent}${firstSeparator}\n` +
                            `${innerIndent}\n` +
                            `${indent}${secondSeparator}\n` +
                            `${innerIndent}\n` +
                            `${indent}${loopStructure.close}`
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
        // COLLECTION ENTER EXPANSION
        // ==================================================
        //
        // Empty Box and Bag structures remain inline:
        //
        // :[]:
        // :||:
        //
        // Pressing Enter while the cursor is between the
        // delimiters expands them into multiline form.
        // ==================================================

        if (
            change.text.includes("\n") &&
            isDruimCodeContext(
                event.document,
                change.range.start
            )
        ) {
            const start = change.range.start;

            const newlineCount =
                (change.text.match(/\n/g) || []).length;

            if (newlineCount !== 1) {
                return;
            }

            const openerLineNumber = start.line;
            const closerLineNumber = start.line + 1;

            if (
                closerLineNumber >= event.document.lineCount
            ) {
                return;
            }

            const openerLine =
                event.document.lineAt(openerLineNumber);

            const closerLine =
                event.document.lineAt(closerLineNumber);

            const openerText = openerLine.text.trimEnd();
            const closerText = closerLine.text.trim();

            let structure = null;

            if (
                openerText.endsWith(boxStructure.open) &&
                closerText === boxStructure.close
            ) {
                structure = boxStructure;
            } else if (
                openerText.endsWith(bagStructure.open) &&
                closerText === bagStructure.close
            ) {
                structure = bagStructure;
            }

            if (!structure) {
                return;
            }

            const indent =
                openerLine.text.match(/^\s*/)?.[0] ?? "";

            const innerIndent = `${indent}\t`;

            const closerRange = new vscode.Range(
                closerLineNumber,
                0,
                closerLineNumber,
                closerLine.text.length
            );

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.replace(
                            closerRange,
                            `${innerIndent}\n` +
                            `${indent}${structure.close}`
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
                    closerLineNumber,
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

        if (
            (
                change.text === "[]" ||
                change.text === "["
            ) &&
            isDruimCodeContext(
                event.document,
                new vscode.Position(
                    change.range.start.line,
                    Math.max(
                        0,
                        change.range.start.character - 1
                    )
                )
            )
        ) {
            const start = change.range.start;

            if (start.character === 0) {
                return;
            }

            const line =
                event.document.lineAt(start.line);

            const lineText = line.text;

            /*
            * The typed "[" must actually form ":["
            * with the preceding character.
            */
            if (
                lineText.slice(
                    start.character - 1,
                    start.character + 1
                ) !== ":["
            ) {
                return;
            }

            /*
            * "::[" is Get with an indexed selector,
            * not a Box opener.
            */
            if (
                start.character >= 2 &&
                lineText.slice(
                    start.character - 2,
                    start.character + 1
                ) === "::["
            ) {
                return;
            }

            const closePosition = new vscode.Position(
                start.line,
                start.character + 1
            );

            applyingDruimEdit = true;

            try {
                let applied;

                /*
                * VS Code may already have generated "]".
                *
                * If so, replace it with Druim's "]:".
                * Otherwise insert the complete closer.
                */
                if (
                    lineText[start.character + 1] === "]"
                ) {
                    const generatedCloseRange =
                        new vscode.Range(
                            start.line,
                            start.character + 1,
                            start.line,
                            start.character + 2
                        );

                    applied = await editor.edit(
                        (editBuilder) => {
                            editBuilder.replace(
                                generatedCloseRange,
                                boxStructure.close
                            );
                        },
                        {
                            undoStopBefore: false,
                            undoStopAfter: false
                        }
                    );
                } else {
                    applied = await editor.edit(
                        (editBuilder) => {
                            editBuilder.insert(
                                closePosition,
                                boxStructure.close
                            );
                        },
                        {
                            undoStopBefore: false,
                            undoStopAfter: false
                        }
                    );
                }

                if (!applied) {
                    return;
                }

                editor.selection = new vscode.Selection(
                    closePosition,
                    closePosition
                );

                return;
            } finally {
                applyingDruimEdit = false;
            }
        }

        // ==================================================
        // BAG STRUCTURE
        // ==================================================

        if (
            change.text === "|" &&
            isDruimCodeContext(
                event.document,
                new vscode.Position(
                    change.range.start.line,
                    Math.max(
                        0,
                        change.range.start.character - 1
                    )
                )
            )
        ) {
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
                            bagStructure.close
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

        if (
            change.text === "()" &&
            isDruimCodeContext(
                event.document,
                new vscode.Position(
                    change.range.start.line,
                    Math.max(
                        0,
                        change.range.start.character - 1
                    )
                )
            )
        ) {
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

            const lineText = event.document.lineAt(start.line).text;
            const indent = lineText.match(/^\s*/)?.[0] ?? "";
            const innerIndent = `${indent}\t`;

            applyingDruimEdit = true;

            try {
                const applied = await editor.edit(
                    (editBuilder) => {
                        editBuilder.replace(
                            generatedCloseRange,
                            `${functionStructure.separator}\n` +
                            `${innerIndent}\n` +
                            `${indent}${functionStructure.close}`
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

                    function tokenInCode(token) {
                        let searchFrom = 0;

                        while (searchFrom < text.length) {
                            const index =
                                text.indexOf(
                                    token,
                                    searchFrom
                                );

                            if (index === -1) {
                                return false;
                            }

                            const position =
                                new vscode.Position(
                                    lineNumber,
                                    index
                                );

                            if (
                                isDruimCodeContext(
                                    document,
                                    position
                                )
                            ) {
                                return true;
                            }

                            searchFrom =
                                index + token.length;
                        }

                        return false;
                    }

                    // --------------------------------------------------
                    // BLOCKS
                    //
                    // :{   opens a segment
                    // }{   closes that segment and opens the next
                    // }:   closes the final segment
                    // --------------------------------------------------

                    if (tokenInCode(":{")) {
                        blockSegmentStart = lineNumber;
                    }

                    if (
                        tokenInCode("}{") &&
                        blockSegmentStart !== null
                    ) {
                        addRange(blockSegmentStart, lineNumber);

                        // This same }{ becomes the opener for the next segment.
                        blockSegmentStart = lineNumber;
                    }

                    if (
                        tokenInCode("}:") &&
                        blockSegmentStart !== null
                    ) {
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

                    if (tokenInCode(":<")) {
                        loopSegmentStart = lineNumber;
                    }

                    if (
                        tokenInCode(">?<") &&
                        loopSegmentStart !== null
                    ) {
                        addRange(loopSegmentStart, lineNumber);

                        // This same >?< becomes the opener for the next segment.
                        loopSegmentStart = lineNumber;
                    }

                    if (
                        tokenInCode(">:") &&
                        loopSegmentStart !== null
                    ) {
                        addRange(loopSegmentStart, lineNumber);

                        loopSegmentStart = null;
                    }

                    // --------------------------------------------------
                    // MULTILINE COMMENTS
                    // --------------------------------------------------

                   if (tokenInCode(":--")) {
                        stack.push({
                            type: "comment",
                            line: lineNumber,
                            kind: vscode.FoldingRangeKind.Comment
                        });
                    }

                    // --------------------------------------------------
                    // BOXES
                    // --------------------------------------------------

                    if (tokenInCode(":[")) {
                        stack.push({
                            type: "box",
                            line: lineNumber
                        });
                    }

                    // --------------------------------------------------
                    // BAGS
                    // --------------------------------------------------

                    if (tokenInCode(":|")) {
                        stack.push({
                            type: "bag",
                            line: lineNumber
                        });
                    }

                    // --------------------------------------------------
                    // FUNCTIONS
                    // --------------------------------------------------

                    const functionMatch = text.match(
                        /\bfn\s+(?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+\s+:\(.*\)\(\s*$/
                    );

                    if (
                        functionMatch &&
                        isDruimCodeContext(
                            document,
                            new vscode.Position(
                                lineNumber,
                                functionMatch.index
                            )
                        )
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
                        const hasClosingToken =
                            closing.type === "comment"
                                ? text.includes(closing.close)
                                : tokenInCode(closing.close);

                        if (!hasClosingToken) {
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

            if (
            isDruimFunctionListContext(
                document,
                selection.start
            ) ||
            isDruimFunctionListContext(
                document,
                selection.end
            )
        ) {
            return;
        }

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
            provideCompletionItems(
                document,
                position
            ) {

                if (
                    !isDruimCodeContext(
                        document,
                        position
                    )
                ) {
                    return [];
                }

                const items = [];

                for (const entry of druimLanguage.completionKeywords) {
                    const conversion =
                        druimLanguage.getConversionExpression(
                            entry.token
                        );

                    if (conversion) {
                        const item = new vscode.CompletionItem(
                            entry.token,
                            vscode.CompletionItemKind.Function
                        );

                        item.detail = conversion.signature;
                        item.documentation = new vscode.MarkdownString(
                            conversion.description
                        );

                        item.insertText = new vscode.SnippetString(
                            `${entry.token}($1)`
                        );

                        items.push(item);

                        continue;
                    }

                    const item = new vscode.CompletionItem(
                        entry.token,
                        vscode.CompletionItemKind.Keyword
                    );

                    item.detail = entry.name;
                    item.documentation = new vscode.MarkdownString(
                        entry.description
                    );

                    items.push(item);
                }

                for (const entry of druimLanguage.literalList) {
                    // void is already included through the type list.
                    if (entry.token === "void") {
                        continue;
                    }

                    const item = new vscode.CompletionItem(
                        entry.token,
                        vscode.CompletionItemKind.Value
                    );

                    item.detail = entry.name;
                    item.documentation = new vscode.MarkdownString(
                        entry.description
                    );

                    items.push(item);
                }

                for (const entry of druimLanguage.coreFunctionList) {
                    const item = new vscode.CompletionItem(
                        entry.token,
                        vscode.CompletionItemKind.Function
                    );

                    item.detail = entry.signature;
                    item.documentation = new vscode.MarkdownString(
                        entry.description
                    );

                    item.insertText = new vscode.SnippetString(
                        `${entry.token}($1)`
                    );

                    items.push(item);
                }

                for (const entry of druimLanguage.completionOperators) {
                    const item = new vscode.CompletionItem(
                        `${entry.name}  ${entry.token}`,
                        vscode.CompletionItemKind.Operator
                    );

                    item.insertText = entry.token;
                    item.detail = entry.name;
                    item.documentation = new vscode.MarkdownString(
                        entry.description
                    );

                    items.push(item);
                }

                return items;
            }
        }
    );

    const hoverProvider = vscode.languages.registerHoverProvider(
        { language: "druim" },
        {
            provideHover(document, position) {
                const line = document.lineAt(position.line).text;

                if (
                    !isDruimCodeContext(
                        document,
                        position
                    )
                ) {
                    return;
                }

                const tokens = [
                    ...druimLanguage.structureByToken.keys(),
                    ...druimLanguage.operatorByToken.keys(),
                    ...druimLanguage.keywordByToken.keys(),
                    ...druimLanguage.typeByToken.keys(),
                    ...druimLanguage.literalByToken.keys(),
                    ...druimLanguage.coreFunctionByToken.keys()
                ]
                    .filter((token, index, array) => array.indexOf(token) === index)
                    .sort((a, b) => b.length - a.length);

                let offset = 0;
                let previousToken = null;
                let indexedSelectorDepth = 0;

                function tokenMatchesAt(token, start) {
                    if (!line.startsWith(token, start)) {
                        return false;
                    }

                    if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(token)) {
                        const before =
                            start > 0
                                ? line[start - 1]
                                : "";

                        const after =
                            start + token.length < line.length
                                ? line[start + token.length]
                                : "";

                        if (
                            /[A-Za-z0-9_]/.test(before) ||
                            /[A-Za-z0-9_]/.test(after)
                        ) {
                            return false;
                        }
                    }

                    return true;
                }

                while (offset < line.length) {
                    const scanPosition =
                        new vscode.Position(
                            position.line,
                            offset
                        );

                    if (
                        !isDruimCodeContext(
                            document,
                            scanPosition
                        )
                    ) {
                        offset++;
                        previousToken = null;
                        indexedSelectorDepth = 0;
                        continue;
                    }

                    let matchedToken = null;

                    // If the previous token was Get or Has, "[" is an indexed selector,
                    // not part of the Box opener ":["
                    if (
                        line[offset] === "[" &&
                        (previousToken === "::" || previousToken === ":?")
                    ) {
                        matchedToken = "[";
                        indexedSelectorDepth++;
                    }

                    // If an indexed selector is open, "]" closes that selector.
                    // This preserves:
                    //
                    // ]::  => ] + ::
                    // ]:?  => ] + :?
                    //
                    // instead of incorrectly reading ]: as a Box closer.
                    else if (
                        line[offset] === "]" &&
                        indexedSelectorDepth > 0
                    ) {
                        matchedToken = "]";
                        indexedSelectorDepth--;
                    }

                    // Everywhere else, use normal longest-token-first matching.
                    // This preserves actual Box delimiters:
                    //
                    // :[  => Box open
                    // ]:  => Box close
                    else {
                        for (const token of tokens) {
                            if (tokenMatchesAt(token, offset)) {
                                matchedToken = token;
                                break;
                            }
                        }
                    }

                    if (!matchedToken) {
                        offset++;
                        previousToken = null;
                        continue;
                    }

                    const start = offset;
                    const end = start + matchedToken.length;

                    if (
                        position.character >= start &&
                        position.character < end
                    ) {
                        let info = null;

                        if (
                            matchedToken === "num" ||
                            matchedToken === "dec" ||
                            matchedToken === "text" ||
                            matchedToken === "flag"
                        ) {
                            const afterToken = line.slice(end);
                            const nextNonWhitespace = afterToken.match(/^\s*(.)/);

                            if (
                                nextNonWhitespace &&
                                nextNonWhitespace[1] === "("
                            ) {
                                info =
                                    druimLanguage.getConversionExpression(
                                        matchedToken
                                    );
                            } else {
                                info =
                                    druimLanguage.getType(
                                        matchedToken
                                    );
                            }
                        } else {
                            info =
                                druimLanguage.getHoverDocumentation(
                                    matchedToken
                                );
                        }

                        if (!info) {
                            const structureMatch =
                                druimLanguage.getStructureByToken(matchedToken);

                            if (structureMatch) {
                                info = structureMatch.structure;
                            }
                        }

                        if (!info) {
                            return;
                        }

                        const range = new vscode.Range(
                            position.line,
                            start,
                            position.line,
                            end
                        );

                        const markdown = new vscode.MarkdownString();

                        markdown.appendMarkdown(
                            `### ${info.name}`
                        );

                        if (info.category) {
                            markdown.appendMarkdown(
                                ` — ${info.category}`
                            );
                        }

                        markdown.appendMarkdown("\n\n");

                        if (info.signature) {
                            markdown.appendCodeblock(
                                info.signature,
                                "druim"
                            );
                        } else {
                            markdown.appendCodeblock(
                                matchedToken,
                                "druim"
                            );
                        }

                        if (info.description) {
                            markdown.appendMarkdown(
                                `${info.description}\n\n`
                            );
                        }

                        if (
                            info.parameters &&
                            info.parameters.length > 0
                        ) {
                            markdown.appendMarkdown(
                                `**Parameters**\n\n`
                            );

                            for (const parameter of info.parameters) {
                                const flags = [];

                                if (parameter.optional) {
                                    flags.push("optional");
                                }

                                if (parameter.variadic) {
                                    flags.push("variadic");
                                }

                                const suffix =
                                    flags.length > 0
                                        ? ` _(${flags.join(", ")})_`
                                        : "";

                                markdown.appendMarkdown(
                                    `- \`${parameter.name}\`${suffix} — ${parameter.description}\n`
                                );
                            }

                            markdown.appendMarkdown("\n");
                        }

                        if (info.returns) {
                            markdown.appendMarkdown(
                                `**Returns:** \`${info.returns.type}\`\n\n`
                            );

                            markdown.appendMarkdown(
                                `${info.returns.description}\n\n`
                            );
                        }

                        if (
                            info.details &&
                            info.details.length > 0
                        ) {
                            markdown.appendMarkdown(
                                `**Behavior**\n\n`
                            );

                            for (const detail of info.details) {
                                markdown.appendMarkdown(
                                    `- ${detail}\n`
                                );
                            }

                            markdown.appendMarkdown("\n");
                        }

                        if (info.example) {
                            markdown.appendMarkdown(
                                `**Example**\n\n`
                            );

                            markdown.appendCodeblock(
                                info.example,
                                "druim"
                            );

                            if (info.exampleResult) {
                                markdown.appendMarkdown(
                                    `Result: \`${info.exampleResult}\`\n\n`
                                );
                            }
                        }

                        if (
                            info.diagnostics &&
                            info.diagnostics.length > 0
                        ) {
                            markdown.appendMarkdown(
                                `**Diagnostics**\n\n`
                            );

                            for (const diagnostic of info.diagnostics) {
                                markdown.appendMarkdown(
                                    `- ${diagnostic}\n`
                                );
                            }
                        }

                        markdown.supportHtml = false;

                        return new vscode.Hover(
                            markdown,
                            range
                        );
                    }

                    previousToken = matchedToken;
                    offset = end;
                }

                const wordRange = document.getWordRangeAtPosition(
                    position,
                    /(?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+/
                );

                if (!wordRange) {
                    return;
                }

                const identifier =
                    document.getText(wordRange);

                const index =
                    buildDruimSymbolIndex(document);

                const useOffset =
                    document.offsetAt(wordRange.start);

                const afterWord =
                    line.slice(wordRange.end.character);

                if (/^\s*\(/.test(afterWord)) {
                    const functionSymbol =
                        index.functions.get(identifier);

                    if (functionSymbol) {
                        if (
                            isDruimFunctionVisible(
                                document,
                                functionSymbol,
                                useOffset
                            )
                        ) {
                            const markdown =
                                new vscode.MarkdownString();

                            markdown.appendMarkdown(
                                `### ${functionSymbol.name} — Function\n\n`
                            );

                            markdown.appendCodeblock(
                                `${functionSymbol.name}(${functionSymbol.parameters.join(", ")})`,
                                "druim"
                            );

                            const scopeLabel = {
                                normal: "Ordinary",
                                loc: "Local",
                                glo: "Global"
                            }[functionSymbol.scope] ?? functionSymbol.scope;

                            markdown.appendMarkdown(
                                `**Scope:** ${scopeLabel}\n\n`
                            );

                            if (functionSymbol.parameters.length === 0) {
                                markdown.appendMarkdown(
                                    "**Parameters:** none\n\n"
                                );
                            } else {
                                markdown.appendMarkdown(
                                    "**Parameters**\n\n"
                                );

                                for (
                                    const parameter of functionSymbol.parameters
                                ) {
                                    markdown.appendMarkdown(
                                        `- \`${parameter}\`\n`
                                    );
                                }

                                markdown.appendMarkdown("\n");
                            }

                            return new vscode.Hover(
                                markdown,
                                wordRange
                            );
                        }
                    }
                }    

                const scopeInfo =
                    getDruimScopeAt(
                        document,
                        useOffset
                    );

                const effectiveScopedMutate =
                    index.bindings
                        .filter((binding) => {
                            if (
                                binding.kind !== "scope-mutate" ||
                                binding.name !== identifier ||
                                binding.offset > useOffset
                            ) {
                                return false;
                            }

                            if (binding.scope === "glo") {
                                return true;
                            }

                            if (binding.scope === "loc") {
                                return sameDruimScope(
                                    binding.ownerScope,
                                    scopeInfo.local
                                );
                            }

                            return false;
                        })
                        .sort(
                            (a, b) =>
                                b.offset - a.offset
                        )[0];

                const binding =
                    effectiveScopedMutate ??
                    resolveDruimBinding(
                        document,
                        index,
                        identifier,
                        useOffset
                    );

                if (!binding) {
                    return;
                }

                const markdown =
                    new vscode.MarkdownString();

                markdown.appendMarkdown(
                    `### ${binding.name} — Binding\n\n`
                );

                markdown.appendCodeblock(
                    binding.name,
                    "druim"
                );

                const kindLabel = {
                    define: "Define",
                    "define-empty": "DefineEmpty",
                    copy: "Copy",
                    bind: "Bind",
                    guard: "Guard",
                    "scope-mutate": "Mutate"
                }[binding.kind] ?? binding.kind;

                markdown.appendMarkdown(
                    `**Kind:** ${kindLabel}\n\n`
                );

                const scopeLabel = {
                    normal: "Ordinary",
                    loc: "Local",
                    glo: "Global"
                }[binding.scope] ?? binding.scope;

                markdown.appendMarkdown(
                    `**Scope:** ${scopeLabel}\n\n`
                );

                if (binding.stone) {
                    markdown.appendMarkdown(
                        `**Stone:** yes\n\n`
                    );
                } else {
                    markdown.appendMarkdown(
                        `**Stone:** no\n\n`
                    );
                }

                return new vscode.Hover(
                    markdown,
                    wordRange
                );
            }
        }
    );

    const signatureProvider = vscode.languages.registerSignatureHelpProvider(
        { language: "druim" },
        {
            provideSignatureHelp(document, position) {
                if (
                    !isDruimCodeContext(
                        document,
                        position
                    )
                ) {
                    return null;
                }

                /*
                * Find the function call currently being written.
                *
                * Examples:
                *
                * calculate(
                * calculate(value,
                * calculate(value, multiplier
                */
                function findActiveCall(
                    document,
                    position
                ) {
                    const source =
                        document.getText();

                    const endOffset =
                        document.offsetAt(position);

                    let parenDepth = 0;
                    let bracketDepth = 0;

                    for (
                        let i = endOffset - 1;
                        i >= 0;
                        i--
                    ) {
                        const context =
                            getDruimLexicalContextAt(
                                document,
                                i
                            );

                        if (
                            context !== "code" &&
                            context !== "interpolation"
                        ) {
                            continue;
                        }

                        const char = source[i];

                        if (char === "]") {
                            bracketDepth++;
                            continue;
                        }

                        if (char === "[") {
                            if (bracketDepth > 0) {
                                bracketDepth--;
                            }

                            continue;
                        }

                        if (bracketDepth > 0) {
                            continue;
                        }

                        if (char === ")") {
                            parenDepth++;
                            continue;
                        }

                        if (char === "(") {
                            if (parenDepth > 0) {
                                parenDepth--;
                                continue;
                            }

                            let end = i;
                            let start = end - 1;

                            while (
                                start >= 0 &&
                                /[A-Za-z0-9_]/.test(source[start])
                            ) {
                                const nameContext =
                                    getDruimLexicalContextAt(
                                        document,
                                        start
                                    );

                                if (
                                    nameContext !== "code" &&
                                    nameContext !== "interpolation"
                                ) {
                                    break;
                                }

                                start--;
                            }

                            start++;

                            const functionName =
                                source.slice(start, end);

                            if (
                                !functionName ||
                                !/^(?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+$/.test(
                                    functionName
                                )
                            ) {
                                continue;
                            }

                            return {
                                functionName,
                                openParenOffset: i
                            };
                        }
                    }

                    return null;
                }

                function countActiveParameter(
                    document,
                    openParenOffset,
                    endOffset
                ) {
                    const source =
                        document.getText();

                    let activeParameter = 0;
                    let parenDepth = 0;
                    let bracketDepth = 0;

                    for (
                        let i = openParenOffset + 1;
                        i < endOffset;
                        i++
                    ) {
                        const context =
                            getDruimLexicalContextAt(
                                document,
                                i
                            );

                        if (
                            context !== "code" &&
                            context !== "interpolation"
                        ) {
                            continue;
                        }

                        const char = source[i];

                        if (char === "[") {
                            bracketDepth++;
                            continue;
                        }

                        if (char === "]") {
                            if (bracketDepth > 0) {
                                bracketDepth--;
                            }

                            continue;
                        }

                        if (bracketDepth > 0) {
                            continue;
                        }

                        if (char === "(") {
                            parenDepth++;
                            continue;
                        }

                        if (char === ")") {
                            if (parenDepth > 0) {
                                parenDepth--;
                            }

                            continue;
                        }

                        if (parenDepth > 0) {
                            continue;
                        }

                        if (char === ",") {
                            activeParameter++;
                        }
                    }

                    return activeParameter;
                }                

                const activeCall = findActiveCall(
                    document,
                    position
                );

                if (!activeCall) {
                    return null;
                }

                const functionName =
                    activeCall.functionName;

                const conversionExpression =
                    druimLanguage.getConversionExpression(functionName);

                if (conversionExpression) {
                    const signatureDocumentation =
                        new vscode.MarkdownString();

                    signatureDocumentation.appendMarkdown(
                        `**${conversionExpression.name} — ${conversionExpression.category}**\n\n`
                    );

                    signatureDocumentation.appendMarkdown(
                        `${conversionExpression.description}\n\n`
                    );

                    if (conversionExpression.returns) {
                        signatureDocumentation.appendMarkdown(
                            `**Returns:** \`${conversionExpression.returns.type}\`  \n` +
                            `${conversionExpression.returns.description}\n\n`
                        );
                    }

                    if (
                        conversionExpression.details &&
                        conversionExpression.details.length > 0
                    ) {
                        signatureDocumentation.appendMarkdown(
                            `**Behavior**\n\n`
                        );

                        for (const detail of conversionExpression.details) {
                            signatureDocumentation.appendMarkdown(
                                `- ${detail}\n`
                            );
                        }

                        signatureDocumentation.appendMarkdown("\n");
                    }

                    const signature = new vscode.SignatureInformation(
                        conversionExpression.signature,
                        signatureDocumentation
                    );

                    signature.parameters =
                        conversionExpression.parameters.map(
                            (parameter) =>
                                new vscode.ParameterInformation(
                                    parameter.name,
                                    parameter.description
                                )
                        );

                    const help = new vscode.SignatureHelp();

                    help.signatures = [signature];
                    help.activeSignature = 0;
                    help.activeParameter = 0;

                    return help;
                }

                const coreFunction =
                    druimLanguage.getCoreFunction(functionName);

                if (coreFunction) {
                    const signatureDocumentation =
                        new vscode.MarkdownString();

                    signatureDocumentation.appendMarkdown(
                        `**${coreFunction.name} — ${coreFunction.category}**\n\n`
                    );

                    signatureDocumentation.appendMarkdown(
                        `${coreFunction.description}\n\n`
                    );

                    if (coreFunction.returns) {
                        signatureDocumentation.appendMarkdown(
                            `**Returns:** \`${coreFunction.returns.type}\`  \n` +
                            `${coreFunction.returns.description}\n\n`
                        );
                    }

                    if (
                        coreFunction.details &&
                        coreFunction.details.length > 0
                    ) {
                        signatureDocumentation.appendMarkdown(
                            `**Behavior**\n\n`
                        );

                        for (const detail of coreFunction.details) {
                            signatureDocumentation.appendMarkdown(
                                `- ${detail}\n`
                            );
                        }

                        signatureDocumentation.appendMarkdown("\n");
                    }

                    if (coreFunction.example) {
                        signatureDocumentation.appendMarkdown(
                            `**Example**\n\n`
                        );

                        signatureDocumentation.appendCodeblock(
                            coreFunction.example,
                            "druim"
                        );

                        if (coreFunction.exampleResult) {
                            signatureDocumentation.appendMarkdown(
                                `Returns: \`${coreFunction.exampleResult}\`\n`
                            );
                        }
                    }

                    const signature = new vscode.SignatureInformation(
                        coreFunction.signature,
                        signatureDocumentation
                    );

                    const parameters = coreFunction.parameters;

                    let parameterSearchOffset =
                        coreFunction.signature.indexOf("(") + 1;

                    signature.parameters = parameters.map(
                        (parameter) => {
                            let label = parameter.name;

                            if (parameter.optional) {
                                label = `[${parameter.name}]`;
                            }

                            const start = coreFunction.signature.indexOf(
                                label,
                                parameterSearchOffset
                            );

                            const end = start + label.length;

                            parameterSearchOffset = end;

                            return new vscode.ParameterInformation(
                                [start, end],
                                parameter.description
                            );
                        }
                    );

                    let activeParameter =
                        countActiveParameter(
                            document,
                            activeCall.openParenOffset,
                            document.offsetAt(position)
                        );

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

                const callOffset =
                    document.offsetAt(position);

                if (
                    !isDruimFunctionVisible(
                        document,
                        symbol,
                        callOffset
                    )
                ) {
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

                let activeParameter =
                    countActiveParameter(
                        document,
                        activeCall.openParenOffset,
                        document.offsetAt(position)
                    );

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

                if (
                    !isDruimCodeContext(
                        document,
                        position
                    )
                ) {
                    return null;
                }
                /*
                * Get the identifier under the cursor.
                */
                const wordRange = document.getWordRangeAtPosition(
                    position,
                    /(?=[A-Za-z0-9_]*[A-Za-z_])[A-Za-z0-9_]+/
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
                    const functionSymbol =
                        index.functions.get(identifier);

                    if (!functionSymbol) {
                        return null;
                    }

                    const useOffset =
                        document.offsetAt(
                            wordRange.start
                        );

                    if (
                        !isDruimFunctionVisible(
                            document,
                            functionSymbol,
                            useOffset
                        )
                    ) {
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