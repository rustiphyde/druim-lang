/**
 * Druim Language Metadata
 *
 * Shared syntax metadata for the VS Code extension.
 *
 * This module is intentionally editor-facing only:
 * - syntax convenience
 * - delimiter completion
 * - hover documentation
 * - completion items
 * - lightweight syntax-aware navigation
 *
 * It is NOT intended to duplicate compiler semantics or diagnostics.
 */

const structures = {

    sourceBoundary: {
        id: "sourceBoundary",
        name: "Source Boundary",
        open: ":-:-:",
        close: ":-:-:",
        description: "Marks a Druim source boundary."
    },

    block: {
        id: "block",
        name: "Block",
        open: ":{",
        continuation: "}{",
        close: "}:",
        multiline: true,
        description:
            "Establishes one lexical scope. `:{` begins the block chain, `}{` continues the same block chain without nesting, and `}:` ends it."
    },

    loop: {
        id: "loop",
        name: "Loop",
        open: ":<",
        separators: [">?<", ">?<"],
        close: ">:",
        multiline: true,
        sections: [
            "setup",
            "condition",
            "process"
        ],
        description:
            "Creates a three-section Druim loop. `:<` begins setup, the first `>?<` begins the condition section, the second `>?<` begins the process section, and `>:` ends the loop."
    },

    box: {
        id: "box",
        name: "Box",
        open: ":[",
        close: "]:",
        multiline: true,
        description: "Creates an ordered Druim Box collection."
    },

    indexSelector: {
        id: "indexSelector",
        name: "Indexed Selector",
        category: "Traversal Structure",
        open: "[",
        close: "]",
        multiline: false,
        signature: "[expression]",
        description:
            "Provides an indexed selector to Get `::` or Has `:?` for values that support positional traversal.",
        details: [
            "The expression inside the selector is evaluated to determine the index.",
            "Boxes require indexed selectors.",
            "Text supports indexed selectors.",
            "Bags do not support indexed selectors.",
            "The index must evaluate to a non-negative num.",
            "A negative index produces a diagnostic.",
            "A non-num index produces a diagnostic.",
            "Indexes are zero-based.",
            "Text indexes are based on Unicode characters rather than UTF-8 bytes."
        ],
        example:
            'items::[0]\nitems:?[2]\n"Druim"::[1]',
        exampleResult:
            'Get retrieves the indexed value; Has returns true or false.',
        diagnostics: [
            "Using an indexed selector against a Bag produces a diagnostic.",
            "A negative index produces a diagnostic.",
            "A selector expression that does not evaluate to num produces a diagnostic."
        ]
    },

    bag: {
        id: "bag",
        name: "Bag",
        open: ":|",
        close: "|:",
        multiline: true,
        description: "Creates a named Druim Bag collection."
    },

    function: {
            id: "function",
            name: "Function",
            open: ":(",
            separator: ")(",
            close: "):",
            multiline: true,
            sections: [
                "parameters",
                "body"
            ],
            description:
                "Creates a Druim function structure. `:(` begins the parameter block, `)(` separates parameters from the function body, and `):` ends the function definition."
    },

    parentheses: {
        id: "parentheses",
        name: "Parentheses",
        open: "(",
        close: ")",
        multiline: false,
        description:
            "Delimits a Druim function-call argument list, contains the expression supplied to a type-conversion expression, or groups the expression supplied to Print `|>`."
    },

    lineComment: {
        id: "lineComment",
        name: "Line Comment",
        open: ":-",
        close: "-:",
        multiline: false,
        description: "Creates a single line comment."
    },

    multilineComment: {
        id: "multilineComment",
        name: "Multiline Comment",
        open: ":--",
        close: "--:",
        multiline: true,
        description: "Creates a multiline comment."
    },

    interpolation: {
        id: "interpolation",
        name: "Text Interpolation",
        open: ":.",
        close: ".:",
        multiline: false,
        description: "Embeds a Druim expression inside interpolated text."
    }
};

const operators = {
    "=": {
        token: "=",
        name: "Define",
        category: "Binding Statement",
        signature: "target = expression;",
        description:
            "Evaluates one complete expression and defines the target identifier as the resulting value.",
        details: [
            "The target must be exactly one identifier.",
            "The right-hand side must contain exactly one complete expression.",
            "The right-hand side may not be empty.",
            "Define evaluates the expression before establishing the binding.",
            "A bare identifier may not be used as the entire right-hand side when Copy `:=` or Bind `:>` expresses the intended operation.",
            "Define may not be chained with another statement operator."
        ],
        example:
            "total = price * quantity;",
        exampleResult:
            "total is defined as the evaluated result of price * quantity.",
        diagnostics: [
            "A non-identifier target produces a diagnostic.",
            "An empty right-hand side produces a diagnostic.",
            "Unexpected tokens after the complete expression produce a diagnostic.",
            "Using a bare identifier as the entire right-hand side where Copy or Bind is required produces a diagnostic.",
            "Chaining Define with another statement operator produces a diagnostic."
        ]
    },

    "=;": {
        token: "=;",
        name: "DefineEmpty",
        category: "Binding Statement",
        signature: "target =;",
        description:
            "Defines the target identifier explicitly with the value void.",
        details: [
            "`=;` is one lexically atomic operator token.",
            "The target must be exactly one identifier.",
            "DefineEmpty does not take a right-hand expression.",
            "The operator completes the statement itself; no additional semicolon follows it.",
            "Its meaning is equivalent to defining the target as void.",
            "DefineEmpty may not be chained with another statement operator."
        ],
        example:
            "result =;",
        exampleResult:
            "result is defined as void.",
        diagnostics: [
            "A non-identifier target produces a diagnostic.",
            "Tokens following `=;` as part of the same statement produce a diagnostic.",
            "Chaining DefineEmpty with another statement operator produces a diagnostic."
        ]
    },

    "<<": {
        token: "<<",
        name: "Mutate",
        category: "State Statement",
        signature: "target << expression;",
        description:
            "Changes the value stored in an existing binding identity.",
        details: [
            "Mutate operates on an existing binding rather than establishing a new one.",
            "The target resolves through Druim's normal scope rules.",
            "The right-hand expression is evaluated and its resulting value replaces the value held by the target identity.",
            "If multiple names are connected through Bind `:>`, mutation through one name is visible through the others because they share the same identity.",
            "A stone binding identity cannot be mutated."
        ],
        example:
            "count = 10;\ncount << 11;",
        exampleResult:
            "count now holds 11.",
        diagnostics: [
            "Mutating a target that does not resolve to an existing binding produces a diagnostic.",
            "Mutating a stone binding identity produces a diagnostic."
        ]
    },

    ":=": {
        token: ":=",
        name: "Copy",
        category: "Binding Statement",
        signature: "target := source;",
        description:
            "Copies the current resolved value of an existing identifier into a new independent binding.",
        details: [
            "The target must be exactly one identifier.",
            "The source must be exactly one identifier.",
            "The source identifier must already exist.",
            "Copy evaluates no expression.",
            "The target receives a snapshot of the source's current value.",
            "The target and source are independent after the copy.",
            "Future mutations of the source do not affect the copied binding.",
            "Copy does not create shared identity.",
            "Copy may not be chained with another statement operator."
        ],
        example:
            "original = 10;\ncopy := original;\noriginal << 20;",
        exampleResult:
            "original holds 20 while copy still holds 10.",
        diagnostics: [
            "A non-identifier target produces a diagnostic.",
            "A non-identifier source produces a diagnostic.",
            "Copying from an identifier that does not exist produces a diagnostic.",
            "Using a literal or expression as the source produces a diagnostic.",
            "Unexpected tokens before the statement terminator produce a diagnostic.",
            "Chaining Copy with another statement operator produces a diagnostic."
        ]
    },

    ":>": {
        token: ":>",
        name: "Bind",
        category: "Binding Statement",
        signature: "target :> source;",
        description:
            "Creates another identifier for the same underlying binding identity.",
        details: [
            "The target must be exactly one identifier.",
            "The source must be exactly one identifier.",
            "The source identifier must already exist.",
            "Bind evaluates no expression.",
            "Bind does not copy the source value.",
            "The target and source refer to the same underlying identity.",
            "Future mutations through either bound name are visible through the other.",
            "Bind may not be chained with another statement operator."
        ],
        example:
            "original = 10;\nalias :> original;\noriginal << 20;",
        exampleResult:
            "Both original and alias resolve to 20.",
        diagnostics: [
            "A non-identifier target produces a diagnostic.",
            "A non-identifier source produces a diagnostic.",
            "Binding to an identifier that does not exist produces a diagnostic.",
            "Using a literal or expression as the source produces a diagnostic.",
            "Unexpected tokens before the statement terminator produce a diagnostic.",
            "Chaining Bind with another statement operator produces a diagnostic."
        ]
    },

    "?=": {
        token: "?=",
        name: "Guard",
        category: "Binding Statement",
        signature: "target ?= branch [: branch ...];",
        description:
            "Defines a target by evaluating guarded branch expressions from left to right and selecting the first value that evaluates to true.",
        details: [
            "The target must be exactly one identifier.",
            "At least one branch expression is required.",
            "Standalone `:` separates additional Guard branches.",
            "Every branch contains exactly one complete expression.",
            "Branches are evaluated from left to right.",
            "Each evaluated branch value is converted using Druim's explicit truth-evaluation rules.",
            "Evaluation stops after the first branch whose value converts to true under Druim's canonical truth conversion.",
            "The selected branch's original value becomes the target value.",
            "The final written branch is still guarded; it is not an unconditional fallback.",
            "If every branch is false, the target is defined as void.",
            "Guard always defines its target.",
            "Guard does not create a block or additional scope.",
            "Guard may not be chained with another statement operator."
        ],
        example:
            "result ?= primary : secondary : fallback;",
        exampleResult:
            "result receives the first branch value that canonically converts to true, or void if every branch converts to false.",
        diagnostics: [
            "A non-identifier target produces a diagnostic.",
            "A Guard with no branch expression produces a diagnostic.",
            "An empty branch produces a diagnostic.",
            "Unexpected tokens within or after a branch produce a diagnostic.",
            "Statement operators inside Guard branches produce a diagnostic.",
            "Chaining Guard with another statement operator produces a diagnostic."
        ]
    },

    "|>": {
        token: "|>",
        name: "Print",
        category: "Output Statement",
        signature: "|> (expression);",
        description:
            "Evaluates one Druim expression and emits its canonical textual representation as line-oriented output.",
        parameters: [
            {
                name: "expression",
                description:
                    "The single expression whose resulting value will be written to output."
            }
        ],
        returns: {
            type: "void",
            description:
                "Print is a statement and does not produce a reusable value."
        },
        details: [
            "Print evaluates exactly one complete expression.",
            "The expression is enclosed in ordinary parentheses.",
            "The expression is evaluated before its value is converted to output text.",
            "Print uses Druim's canonical textual representation.",
            "num values are printed using their decimal integer representation.",
            "dec values are printed using their stored decimal representation.",
            "flag values are printed as \"true\" or \"false\".",
            "text values are printed as their contained text.",
            "void is printed as \"void\".",
            "Box values do not currently have a canonical textual representation and cannot be printed directly.",
            "Bag values do not currently have a canonical textual representation and cannot be printed directly.",
            "Function values do not currently have a canonical textual representation and cannot be printed directly.",
            "Core function values do not currently have a canonical textual representation and cannot be printed directly.",
            "Each Print statement emits a terminating newline.",
            "Print does not implicitly serialize collections or functions.",
            "Print does not introduce implicit type coercion beyond Druim's defined canonical textual conversion."
        ],
        example:
            '|> (fuse("Count: ", text(12)));',
        exampleResult:
            "Count: 12",
        diagnostics: [
            "A value without a canonical textual representation produces a diagnostic.",
            "Printing a Box directly produces a diagnostic.",
            "Printing a Bag directly produces a diagnostic.",
            "Printing a function directly produces a diagnostic.",
            "Printing a Core function value directly produces a diagnostic.",
            "A malformed or incomplete Print expression produces a diagnostic."
        ]
    },

    "::": {
        token: "::",
        name: "Get",
        category: "Traversal Operator",
        signature: "value::selector",
        description:
            "Retrieves a value from a traversable value using a selector supported by that value.",
        returns: {
            type: "value | void",
            description:
                "The selected value when it exists, otherwise void for a valid selector that does not resolve to an existing member or index."
        },
        details: [
            "Traversal evaluates from left to right.",
            "Get may use a named selector or indexed selector depending on the value being traversed.",
            "Bags require named selectors such as `player::name`.",
            "Boxes require indexed selectors such as `items::[0]`.",
            "Text supports indexed selectors such as `\"Druim\"::[1]`.",
            "A valid missing Bag member evaluates to void.",
            "A valid out-of-range Box or text index evaluates to void.",
            "Because Get returns the retrieved value, traversal may continue when that value is itself traversable."
        ],
        example:
            'player::name\nitems::[0]\n"Druim"::[1]\nplayer::inventory::[2]',
        exampleResult:
            "The selected value, or void when a valid selector does not resolve.",
        diagnostics: [
            "A named selector used against a Box produces a diagnostic.",
            "An indexed selector used against a Bag produces a diagnostic.",
            "Text requires an indexed selector.",
            "A negative index produces a diagnostic.",
            "An index that does not evaluate to num produces a diagnostic."
        ]
    },

    ":?": {
        token: ":?",
        name: "Has",
        category: "Traversal Operator",
        signature: "value:?selector",
        description:
            "Tests whether a value contains the member or index selected by the following selector.",
        returns: {
            type: "flag",
            description:
                "true when the selected member or index exists; otherwise false."
        },
        details: [
            "Has checks existence without retrieving the value.",
            "Bags require named selectors such as `player:?name`.",
            "Boxes require indexed selectors such as `items:?[0]`.",
            "Text supports indexed selectors such as `\"Druim\":?[1]`.",
            "A valid missing Bag member evaluates to false.",
            "A valid out-of-range Box or text index evaluates to false.",
            "Has is terminal because it evaluates to a flag.",
            "Traversal cannot continue through the result of Has.",
            "Get and Has may be composed, such as `player::inventory:?[2]`."
        ],
        example:
            'player:?name\nitems:?[0]\n"Druim":?[4]\nplayer::inventory:?[2]',
        exampleResult:
            "true if the selected member or index exists; otherwise false.",
        diagnostics: [
            "A named selector used against a Box produces a diagnostic.",
            "An indexed selector used against a Bag produces a diagnostic.",
            "Text requires an indexed selector.",
            "A negative index produces a diagnostic.",
            "An index that does not evaluate to num produces a diagnostic."
        ]
    },

    "+": {
        token: "+",
        name: "Add",
        category: "Arithmetic Operator",
        signature: "left + right",
        description:
            "Adds two num values and returns their numeric sum.",
        parameters: [
            {
                name: "left",
                description:
                    "The num value on the left side of the operator."
            },
            {
                name: "right",
                description:
                    "The num value on the right side of the operator."
            }
        ],
        returns: {
            type: "num",
            description:
                "The sum of the two num operands."
        },
        details: [
            "Add is a binary arithmetic operator.",
            "Both operands are evaluated before addition occurs.",
            "Both operands must evaluate to num values.",
            "The result is a num.",
            "The + operator is arithmetic-only.",
            "Text concatenation is not performed by +.",
            "Use the Core function fuse() to combine text values.",
            "Use an explicit conversion such as num(textValue) when numeric conversion is intended.",
            "Druim does not implicitly coerce text, dec, flag, void, Box, Bag, or function values for addition."
        ],
        example:
            "total = 12 + 5;",
        exampleResult:
            "total contains the num value 17.",
        diagnostics: [
            "Using a non-num left operand produces a diagnostic.",
            "Using a non-num right operand produces a diagnostic.",
            "Druim does not implicitly convert operands for addition."
        ]
    },

    "-": {
        token: "-",
        name: "Subtract / Negate",
        category: "Arithmetic Operator",
        signature: "left - right | -value",
        description:
            "Subtracts one num from another when used between two expressions, or reverses the numeric sign of a num or dec when used as a prefix.",
        returns: {
            type: "num | dec",
            description:
                "Binary subtraction returns num. Prefix negation preserves the numeric kind of its operand."
        },
        details: [
            "The - token has both binary and prefix forms.",
            "Binary subtraction evaluates both operands and subtracts the right num from the left num.",
            "Both operands of binary subtraction must be num values.",
            "Binary subtraction returns num.",
            "Prefix negation reverses the sign of one numeric value.",
            "Prefix negation supports num and dec values.",
            "Negating a positive num or dec produces the corresponding negative value.",
            "Negating a negative num or dec produces the corresponding positive value.",
            "Prefix negation does not convert non-numeric values.",
            "Druim does not implicitly coerce text, flag, void, Box, Bag, or function values into numeric values."
        ],
        example:
            "difference = 12 - 5;\nnegative = -12.5;",
        exampleResult:
            "difference contains 7 and negative contains -12.5.",
        diagnostics: [
            "Binary subtraction with a non-num operand produces a diagnostic.",
            "Prefix negation of a value other than num or dec produces a diagnostic.",
            "A numeric negation that cannot be represented produces a diagnostic."
        ]
    },

    "*": {
        token: "*",
        name: "Multiply",
        category: "Arithmetic Operator",
        signature: "left * right",
        description:
            "Multiplies two num values and returns their product.",
        parameters: [
            {
                name: "left",
                description:
                    "The num value on the left side of the operator."
            },
            {
                name: "right",
                description:
                    "The num value on the right side of the operator."
            }
        ],
        returns: {
            type: "num",
            description:
                "The product of the two num operands."
        },
        details: [
            "Multiply is a binary arithmetic operator.",
            "Both operands are evaluated before multiplication occurs.",
            "Both operands must evaluate to num values.",
            "The result is a num.",
            "Druim does not implicitly convert other value types for multiplication."
        ],
        example:
            "area = 6 * 7;",
        exampleResult:
            "area contains the num value 42.",
        diagnostics: [
            "Using a non-num left operand produces a diagnostic.",
            "Using a non-num right operand produces a diagnostic.",
            "Druim does not implicitly convert operands for multiplication."
        ]
    },

    "/": {
        token: "/",
        name: "Divide",
        category: "Arithmetic Operator",
        signature: "left / right",
        description:
            "Divides one num value by another using integer division.",
        parameters: [
            {
                name: "left",
                description:
                    "The num dividend."
            },
            {
                name: "right",
                description:
                    "The nonzero num divisor."
            }
        ],
        returns: {
            type: "num",
            description:
                "The integer quotient of the two num operands."
        },
        details: [
            "Divide is a binary arithmetic operator.",
            "Both operands are evaluated before division occurs.",
            "Both operands must evaluate to num values.",
            "Division uses num integer division.",
            "The result is a num.",
            "A divisor of 0 is invalid.",
            "Druim does not implicitly convert dec, text, flag, or other value types for division."
        ],
        example:
            "result = 21 / 3;",
        exampleResult:
            "result contains the num value 7.",
        diagnostics: [
            "Using a non-num dividend produces a diagnostic.",
            "Using a non-num divisor produces a diagnostic.",
            "Division by zero produces a diagnostic.",
            "Druim does not implicitly convert operands for division."
        ]
    },

    "%": {
        token: "%",
        name: "Modulo",
        category: "Arithmetic Operator",
        signature: "left % right",
        description:
            "Returns the remainder produced by dividing one num value by another.",
        parameters: [
            {
                name: "left",
                description:
                    "The num dividend."
            },
            {
                name: "right",
                description:
                    "The nonzero num divisor."
            }
        ],
        returns: {
            type: "num",
            description:
                "The remainder of the num division."
        },
        details: [
            "Modulo is a binary arithmetic operator.",
            "Both operands are evaluated before the remainder is calculated.",
            "Both operands must evaluate to num values.",
            "The result is a num.",
            "A divisor of 0 is invalid.",
            "Modulo does not perform decimal remainder arithmetic.",
            "Druim does not implicitly convert other value types for modulo."
        ],
        example:
            "remainder = 22 % 5;",
        exampleResult:
            "remainder contains the num value 2.",
        diagnostics: [
            "Using a non-num left operand produces a diagnostic.",
            "Using a non-num right operand produces a diagnostic.",
            "Modulo by zero produces a diagnostic.",
            "Druim does not implicitly convert operands for modulo."
        ]
    },

    "==": {
        token: "==",
        name: "Equal",
        category: "Comparison Operator",
        signature: "left == right",
        description:
            "Tests whether two evaluated Druim values are equal.",
        parameters: [
            {
                name: "left",
                description:
                    "The value on the left side of the comparison."
            },
            {
                name: "right",
                description:
                    "The value on the right side of the comparison."
            }
        ],
        returns: {
            type: "flag",
            description:
                "true when the two evaluated values are equal; otherwise false."
        },
        details: [
            "Both operands are evaluated before comparison.",
            "Equal returns a flag.",
            "Equality compares the resulting Druim values rather than applying canonical truth conversion.",
            "Equality does not implicitly convert one value to the other's type.",
            "Values of different types are not made equal through implicit coercion.",
            "Use an explicit conversion when values must first be compared in a common representation."
        ],
        example:
            "same = 5 == 5;",
        exampleResult:
            "same contains true.",
        diagnostics: [
            "Any diagnostic produced while evaluating either operand propagates from the comparison."
        ]
    },

    "!=": {
        token: "!=",
        name: "Not Equal",
        category: "Comparison Operator",
        signature: "left != right",
        description:
            "Tests whether two evaluated Druim values are different.",
        parameters: [
            {
                name: "left",
                description:
                    "The value on the left side of the comparison."
            },
            {
                name: "right",
                description:
                    "The value on the right side of the comparison."
            }
        ],
        returns: {
            type: "flag",
            description:
                "true when the two evaluated values are not equal; otherwise false."
        },
        details: [
            "Both operands are evaluated before comparison.",
            "Not Equal returns a flag.",
            "The comparison uses the evaluated Druim values directly.",
            "No implicit type conversion occurs before comparison.",
            "Not Equal is the logical inverse of Equal `==` for the same evaluated operands."
        ],
        example:
            "different = 5 != 6;",
        exampleResult:
            "different contains true.",
        diagnostics: [
            "Any diagnostic produced while evaluating either operand propagates from the comparison."
        ]
    },

    "<": {
        token: "<",
        name: "Less Than",
        category: "Comparison Operator",
        signature: "left < right",
        description:
            "Tests whether one num value is numerically less than another.",
        parameters: [
            {
                name: "left",
                description:
                    "The num value being compared."
            },
            {
                name: "right",
                description:
                    "The num value used as the comparison boundary."
            }
        ],
        returns: {
            type: "flag",
            description:
                "true when the left num is less than the right num; otherwise false."
        },
        details: [
            "Both operands are evaluated before comparison.",
            "Both operands must be num values.",
            "The result is always a flag.",
            "The comparison is strictly less-than; equal values produce false.",
            "Druim does not implicitly convert operands before ordered comparison."
        ],
        example:
            "result = 4 < 7;",
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A non-num operand produces a diagnostic.",
            "Druim does not implicitly convert operands for ordered comparison."
        ]
    },

    "<=": {
        token: "<=",
        name: "Less Than or Equal",
        category: "Comparison Operator",
        signature: "left <= right",
        description:
            "Tests whether one num value is numerically less than or equal to another.",
        returns: {
            type: "flag",
            description:
                "true when the left num is less than or equal to the right num; otherwise false."
        },
        details: [
            "Both operands are evaluated before comparison.",
            "Both operands must be num values.",
            "The result is always a flag.",
            "Equal num values satisfy the comparison.",
            "Druim does not implicitly convert operands before ordered comparison."
        ],
        example:
            "result = 7 <= 7;",
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A non-num operand produces a diagnostic.",
            "Druim does not implicitly convert operands for ordered comparison."
        ]
    },

    ">": {
        token: ">",
        name: "Greater Than",
        category: "Comparison Operator",
        signature: "left > right",
        description:
            "Tests whether one num value is numerically greater than another.",
        returns: {
            type: "flag",
            description:
                "true when the left num is greater than the right num; otherwise false."
        },
        details: [
            "Both operands are evaluated before comparison.",
            "Both operands must be num values.",
            "The result is always a flag.",
            "The comparison is strictly greater-than; equal values produce false.",
            "Druim does not implicitly convert operands before ordered comparison."
        ],
        example:
            "result = 9 > 4;",
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A non-num operand produces a diagnostic.",
            "Druim does not implicitly convert operands for ordered comparison."
        ]
    },

    ">=": {
        token: ">=",
        name: "Greater Than or Equal",
        category: "Comparison Operator",
        signature: "left >= right",
        description:
            "Tests whether one num value is numerically greater than or equal to another.",
        returns: {
            type: "flag",
            description:
                "true when the left num is greater than or equal to the right num; otherwise false."
        },
        details: [
            "Both operands are evaluated before comparison.",
            "Both operands must be num values.",
            "The result is always a flag.",
            "Equal num values satisfy the comparison.",
            "Druim does not implicitly convert operands before ordered comparison."
        ],
        example:
            "result = 9 >= 9;",
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A non-num operand produces a diagnostic.",
            "Druim does not implicitly convert operands for ordered comparison."
        ]
    },

    "&&": {
        token: "&&",
        name: "And",
        category: "Logical Operator",
        signature: "left && right",
        description:
            "Evaluates two expressions using Druim's canonical truth conversion and returns whether both resolve to true.",
        returns: {
            type: "flag",
            description:
                "true when both required operands canonically convert to true; otherwise false."
        },
        details: [
            "And uses Druim's canonical truth conversion rather than requiring flag operands.",
            "The left operand is evaluated first.",
            "If the left operand canonically converts to false, the result is false immediately.",
            "When the left operand converts to false, the right operand is not evaluated.",
            "If the left operand converts to true, the right operand is evaluated.",
            "The right operand is then converted using the same canonical truth conversion.",
            "The result of && is always a flag.",
            "This short-circuit behavior can prevent diagnostics or other evaluation on the right side when the left side is already false.",
            "Function and Core function values have no canonical truth conversion."
        ],
        example:
            'result = "Druim" && 1;',
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A value with no canonical truth conversion produces a diagnostic when its truth value must be evaluated.",
            "A diagnostic produced while evaluating a required operand propagates from the logical expression."
        ]
    },

    "||": {
        token: "||",
        name: "Or",
        category: "Logical Operator",
        signature: "left || right",
        description:
            "Evaluates expressions using Druim's canonical truth conversion and returns whether at least one resolves to true.",
        returns: {
            type: "flag",
            description:
                "true when at least one required operand canonically converts to true; otherwise false."
        },
        details: [
            "Or uses Druim's canonical truth conversion rather than requiring flag operands.",
            "The left operand is evaluated first.",
            "If the left operand canonically converts to true, the result is true immediately.",
            "When the left operand converts to true, the right operand is not evaluated.",
            "If the left operand converts to false, the right operand is evaluated.",
            "The right operand is then converted using the same canonical truth conversion.",
            "The result of || is always a flag.",
            "This short-circuit behavior can prevent diagnostics or other evaluation on the right side when the left side already resolves to true.",
            "Function and Core function values have no canonical truth conversion."
        ],
        example:
            'result = "" || "Druim";',
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A value with no canonical truth conversion produces a diagnostic when its truth value must be evaluated.",
            "A diagnostic produced while evaluating a required operand propagates from the logical expression."
        ]
    },

    "!": {
        token: "!",
        name: "Not",
        category: "Logical Operator",
        signature: "!expression",
        description:
            "Applies Druim's canonical truth conversion to one expression and returns the opposite flag value.",
        parameters: [
            {
                name: "expression",
                description:
                    "The value whose canonical truth result will be inverted."
            }
        ],
        returns: {
            type: "flag",
            description:
                "false when the expression canonically converts to true; true when it converts to false."
        },
        details: [
            "Not is a prefix logical operator.",
            "The operand is evaluated before logical negation occurs.",
            "The resulting value is interpreted using Druim's canonical truth conversion.",
            "The operand does not need to already be a flag.",
            "The result of ! is always a flag.",
            "!0 produces true.",
            "!1 produces false.",
            "!\"\" produces true.",
            "!void produces true.",
            "Function and Core function values have no canonical truth conversion."
        ],
        example:
            'result = !"";',
        exampleResult:
            "result contains true.",
        diagnostics: [
            "A value with no canonical truth conversion produces a diagnostic.",
            "Any diagnostic produced while evaluating the operand propagates from the expression."
        ]
    },

    ":": {
        token: ":",
        name: "Guard Separator",
        category: "Guard Structure",
        signature: "target ?= branch : branch : branch;",
        description:
            "Separates candidate expressions within a Druim Guard statement.",
        details: [
            "Standalone : acts as a branch separator only within Guard syntax.",
            "Guard evaluates branch expressions from left to right.",
            "Each evaluated branch value is interpreted using Druim's canonical truth conversion.",
            "Evaluation stops at the first branch whose value converts to true.",
            "The selected branch's original value becomes the Guard target's value.",
            "The selected value is not replaced with a flag.",
            "The final written branch remains conditional; it is not an unconditional else branch.",
            "If every branch converts to false, the Guard target is defined as void.",
            "The separator does not establish a scope or block.",
            "A Guard may contain more than two branches."
        ],
        example:
            "result ?= primary : secondary : fallback;",
        exampleResult:
            "result receives the first branch value that canonically converts to true, or void if every branch converts to false.",
        diagnostics: [
            "An empty branch between separators produces a diagnostic.",
            "A separator without a valid following branch expression produces a diagnostic.",
            "A value with no canonical truth conversion produces a diagnostic when that branch is evaluated.",
            "Using standalone : outside a syntactically valid Guard context produces a diagnostic."
        ]
    },
};

const keywords = {
    fn: {
        token: "fn",
        name: "Function",
        category: "Declaration",
        signature: "fn name :(parameters)(body):",
        description:
            "Declares a Druim function with a named parameter section and function body.",
        details: [
            "A function declaration begins with `fn` followed by the function name.",
            "`:(` begins the parameter section.",
            "`)(` separates the parameter section from the function body.",
            "`):` closes the function declaration.",
            "Parameters are valid Druim parameter forms rather than being restricted to bare identifiers.",
            "Parameters may include default values.",
            "Parameters may be plain identifiers or may define defaults using `= expression`.",
            "Required and defaulted parameters may appear in any order.",
            "Parameter names must be unique within the function.",
            "A parameter name may not collide with an already-visible binding when the function is invoked.",
            "Calling the function creates a function-local scope in which parameters and body-local bindings exist for that invocation.",
            "Function calls use ordinary parentheses, such as `calculate(value)`."
        ],
        example:
            "fn double :(value)(\n\tret value * 2;\n):",
        exampleResult:
            "Declares a function named double with one parameter named value.",
        diagnostics: [
            "A missing or invalid function name produces a diagnostic.",
            "Malformed function delimiters produce a diagnostic.",
            "Invalid parameter forms produce a diagnostic.",
            "Malformed function body structure produces a diagnostic."
        ]
    },

    ret: {
        token: "ret",
        name: "Return",
        category: "Control Statement",
        signature: "ret [expression];",
        description:
            "Ends execution of the current function and returns the evaluated expression value, or void when no expression is supplied.",
        details: [
            "Return is valid within a function body.",
            "ret expression; evaluates the expression and returns the resulting value.",
            "ret; returns void.",
            "A function that reaches the end of its body without executing ret also evaluates to void.",
            "An executed ret immediately stops the remaining function body.",
            "A ret executed inside a loop propagates through the loop and exits the enclosing function."
        ],
        example:
            "ret value * 2;",
        exampleResult:
            "Returns the evaluated result of value * 2 from the current function.",
        diagnostics: [
            "Using ret outside a function produces a diagnostic.",
            "A malformed return expression produces a diagnostic."
        ]
    },

    loc: {
        token: "loc",
        name: "Local",
        category: "Scope Modifier",
        signature: "loc statement",
        description:
            "Commits a new target or ordinary visible identity to the applicable local lifetime.",
        details: [
            "loc does not create a new lexical scope by itself.",
            "A binding identity may be ordinary, explicitly local, or explicitly global.",
            "For target-establishing forms, loc creates the new target with the applicable local lifetime.",
            "A new loc target may not reuse a name that is already visible.",
            "Inside a block chain, a loc target belongs only to the current block segment.",
            "At a `}{` continuation, block-segment local targets are discarded before the next segment begins.",
            "Inside a loop, a loc target belongs to the loop's persistent local scope and survives across iterations.",
            "loc supports Define `=`, DefineEmpty `=;`, Copy `:=`, Bind `:>`, Guard `?=`, Mutate `<<`, and function declaration.",
            "With Mutate `<<`, loc localizes an ordinary visible identity for the applicable local lifetime before applying the mutation.",
            "The right-hand side of loc Mutate is evaluated against the currently visible identity before localization.",
            "If multiple names share the identity through Bind `:>`, those names observe the localized identity during the local lifetime.",
            "When the local lifetime ends, the outer identity is restored with its original value and mutability state.",
            "An identity explicitly committed with glo cannot later be changed to local scope with loc.",
            "loc and glo are mutually exclusive on the same statement.",
            "When combined with stone, canonical modifier order is `stone loc`."
        ],
        example:
            "value = 10;\n:{\n    loc value << value + 5;\n}:",
        exampleResult:
            "value is 15 during the local lifetime. After that lifetime ends, the outer value remains 10.",
        diagnostics: [
            "Using loc together with glo on the same statement produces a diagnostic.",
            "Attempting to localize an identity explicitly committed with glo produces a diagnostic.",
            "Defining a new loc target with a name that is already visible produces a diagnostic.",
            "Placing loc in an invalid modifier position produces a diagnostic.",
            "Using loc with an unsupported statement form produces a diagnostic."
        ]
    },

    glo: {
        token: "glo",
        name: "Global",
        category: "Scope Modifier",
        signature: "glo statement",
        description:
            "Commits a new target or ordinary visible identity to global lifetime.",
        details: [
            "glo does not create a new lexical scope by itself.",
            "A binding identity may be ordinary, explicitly local, or explicitly global.",
            "For target-establishing forms, glo creates the new target directly in the program's global scope.",
            "A glo target remains globally visible after the function, loop, block, or block segment in which it was established ends.",
            "Right-hand expressions and source identifiers still resolve from the scope where the glo statement executes.",
            "glo supports Define `=`, DefineEmpty `=;`, Copy `:=`, Bind `:>`, Guard `?=`, Mutate `<<`, and function declaration.",
            "With Mutate `<<`, glo may commit an ordinary visible identity to global scope and apply the mutation to that same identity.",
            "The right-hand side of glo Mutate is evaluated in the current execution scope before the identity is committed globally.",
            "glo Mutate preserves the target identity rather than creating an independent copy.",
            "After successful glo Mutate, the target remains globally visible with its mutated value.",
            "An identity explicitly committed with loc cannot later be changed to global scope with glo.",
            "A glo Copy creates a fresh independent global identity and may copy from a shorter-lived source.",
            "A glo Bind creates a global alias and therefore requires a source identity that can survive for global lifetime.",
            "loc and glo are mutually exclusive on the same statement.",
            "When combined with stone, canonical modifier order is `stone glo`."
        ],
        example:
            ":{\n    val = 2;\n    glo val << val + 2;\n}:",
        exampleResult:
            "val is mutated to 4, committed to global scope, and remains globally visible after the block ends.",
        diagnostics: [
            "Using glo together with loc on the same statement produces a diagnostic.",
            "Attempting to globalize an identity explicitly committed with loc produces a diagnostic.",
            "A glo Bind whose source identity cannot survive for global lifetime produces a diagnostic.",
            "Placing glo in an invalid modifier position produces a diagnostic.",
            "Using glo with an unsupported statement form produces a diagnostic."
        ]
    },

    stone: {
        token: "stone",
        name: "Stone",
        category: "Identity Modifier",
        signature: "stone [loc | glo] statement",
        description:
            "Makes the binding identity affected by the following supported statement immutable.",
        details: [
            "Stone applies to binding identity rather than merely to the value currently stored in that binding.",
            "A stone identity cannot later be changed through Mutate `<<`.",
            "Stone supports Define `=`, DefineEmpty `=;`, Copy `:=`, Bind `:>`, Guard `?=`, and Mutate `<<`.",
            "Function declarations cannot be modified with stone.",
            "With Bind `:>`, stone applies to the shared underlying identity.",
            "Every name referring to a stone bound identity observes the same immutable identity.",
            "With Copy `:=`, stone applies only to the fresh copied identity and does not change the source identity.",
            "With Mutate `<<`, the mutation is applied first and the resulting target identity then becomes stone.",
            "If stone Mutate targets a bound alias, the shared underlying identity becomes stone.",
            "stone loc Mutate localizes and mutates the identity, then stones only that localized identity for the applicable local lifetime.",
            "When a stone loc Mutate lifetime ends, the outer identity is restored with its original value and mutability state.",
            "stone glo Mutate performs the global scope operation and mutation first, then stones the resulting global identity.",
            "stone does not bypass loc or glo scope-commitment restrictions.",
            "stone may be combined with loc or glo.",
            "When combined with a scope modifier, canonical modifier order places stone first."
        ],
        example:
            "count = 10;\nstone count << count + 5;",
        exampleResult:
            "count becomes 15 and its identity is then stone.",
        diagnostics: [
            "Attempting to mutate an identity that is already stone produces a diagnostic.",
            "After a successful stone Mutate, later mutation through any name sharing that identity produces a diagnostic.",
            "Using stone with a function declaration produces a diagnostic.",
            "Placing stone after loc or glo violates canonical modifier order.",
            "Using stone with an unsupported statement form produces a diagnostic."
        ]
    }
};

const types = {
    num: {
        token: "num",
        name: "Num",
        category: "Type",
        signature: "num",
        description:
            "Druim's whole-number numeric type.",
        details: [
            "num stores integer values.",
            "num values participate in Druim's arithmetic and numeric comparison operators.",
            "num 0 converts to false through canonical truth conversion.",
            "Any nonzero num converts to true through canonical truth conversion.",
            "The canonical text representation of a num is its decimal integer representation.",
            "num values may be explicitly converted with num(), dec(), text(), or flag().",
            "num() is a type-conversion expression; the bare token num is the type concept.",
            "Druim does not perform implicit coercion between num and other types."
        ],
        example:
            "count = 42;",
        exampleResult:
            "count contains the num value 42.",
        diagnostics: [
            "Arithmetic requiring num values produces a diagnostic when supplied unsupported value types.",
            "An out-of-range num conversion produces a diagnostic.",
            "Implicit conversion from text, dec, flag, or other values does not occur."
        ]
    },

    dec: {
        token: "dec",
        name: "Dec",
        category: "Type",
        signature: "dec",
        description:
            "Druim's decimal numeric type.",
        details: [
            "dec represents numeric values containing a decimal component.",
            "Druim preserves the stored decimal representation rather than replacing it with generic display formatting.",
            "dec 0.0 converts to false through canonical truth conversion.",
            "Any nonzero dec converts to true through canonical truth conversion.",
            "The canonical text representation of a dec is its stored decimal representation.",
            "dec values may be explicitly converted with num(), dec(), text(), or flag().",
            "num(dec) rounds to the nearest whole number, with exact .5 ties rounded away from zero.",
            "dec() is a type-conversion expression; the bare token dec is the type concept.",
            "Druim does not perform implicit coercion between dec and other types."
        ],
        example:
            "temperature = 12.50;",
        exampleResult:
            "temperature contains the dec value 12.50.",
        diagnostics: [
            "Unsupported numeric operations on dec values produce a diagnostic.",
            "An unrepresentable dec conversion produces a diagnostic.",
            "Implicit conversion from text, num, flag, or other values does not occur."
        ]
    },

    flag: {
        token: "flag",
        name: "Flag",
        category: "Type",
        signature: "flag",
        description:
            "Druim's explicit truth-value type.",
        details: [
            "A flag value is either true or false.",
            "Flags are the result type of comparison operators and Has `:?`.",
            "Druim uses explicit canonical truth conversion rather than generic truthiness.",
            "flag(true) is true and flag(false) is false.",
            "num 0 converts to false; nonzero num converts to true.",
            "dec 0.0 converts to false; nonzero dec converts to true.",
            "Empty or whitespace-only text converts to false; other text converts to true.",
            "void converts to false.",
            "An empty Box or Bag converts to false; a nonempty Box or Bag converts to true.",
            "Function and Core function values do not have a canonical flag conversion.",
            "flag() is a type-conversion expression; the bare token flag is the type concept."
        ],
        example:
            "active = true;",
        exampleResult:
            "active contains the flag value true.",
        diagnostics: [
            "Function and Core function values do not have a canonical flag conversion.",
            "Druim does not implicitly parse text such as \"false\" or \"0\" into flag values."
        ]
    },

    text: {
        token: "text",
        name: "Text",
        category: "Type",
        signature: "text",
        description:
            "Druim's character-sequence type.",
        details: [
            "text stores Unicode text values.",
            "Text traversal uses indexed selectors such as `value::[0]`.",
            "Text indexes are based on Unicode characters rather than UTF-8 bytes.",
            "An empty or whitespace-only text value converts to false through canonical truth conversion.",
            "Any other text value converts to true.",
            "The canonical text representation of a text value is the text itself.",
            "text() converts supported values using the same canonical textual representation used by Print and interpolation.",
            "text() does not serialize Boxes, Bags, functions, or Core function values.",
            "text() is a type-conversion expression; the bare token text is the type concept.",
            "Text concatenation uses the Core function fuse(); `+` remains arithmetic-only."
        ],
        example:
            'name = "Druim";',
        exampleResult:
            'name contains the text value "Druim".',
        diagnostics: [
            "Using an invalid traversal selector against text produces a diagnostic.",
            "Using unsupported non-text values with text-only Core functions produces a diagnostic.",
            "Implicit conversion to text does not occur."
        ]
    },

    void: {
        token: "void",
        name: "Void",
        category: "Type",
        signature: "void",
        description:
            "Represents Druim's explicit absence value.",
        details: [
            "void is a real Druim value; it is not undefined, null, or a missing binding.",
            "A binding may explicitly contain void.",
            "DefineEmpty `=;` creates a binding whose value is void.",
            "A valid Get `::` that does not resolve to an existing member or index returns void.",
            "A function with no executed ret implicitly returns void.",
            "`ret;` explicitly returns void.",
            "void converts to false through canonical truth conversion.",
            "The canonical text representation of void is \"void\".",
            "text(void) produces \"void\".",
            "flag(void) produces false.",
            "void cannot be converted to num or dec.",
            "void is not callable and does not support conversion syntax such as `void(expression)`."
        ],
        example:
            "result = void;",
        exampleResult:
            "result explicitly contains void.",
        diagnostics: [
            "Calling void as though it were a function or conversion expression produces a diagnostic.",
            "num(void) produces a diagnostic.",
            "dec(void) produces a diagnostic."
        ]
    }
};

const conversionExpressions = {
    num: {
        token: "num",
        name: "Num Conversion",
        category: "Type Conversion Expression",
        signature: "num(expression)",
        parameters: [
            {
                name: "expression",
                description:
                    "The value to explicitly convert to num."
            }
        ],
        returns: {
            type: "num",
            description:
                "The converted integer value."
        },
        description:
            "Explicitly converts a supported Druim value to num.",
        details: [
            "num is a dedicated Druim type-conversion expression, not a Core function.",
            "Exactly one complete expression is required.",
            "A num value is returned unchanged.",
            "A dec value is rounded to the nearest whole number.",
            "An exact .5 tie rounds away from zero.",
            "Decimal places are not rounded before the whole-number conversion.",
            "Numeric text may contain an optional leading + or -.",
            "Numeric text may represent either a whole number or decimal value.",
            "If a decimal point is present in numeric text, digits are required on both sides.",
            "Numeric text must be consumed completely; surrounding whitespace and trailing characters are invalid.",
            "Fractional numeric text uses the same rounding rule as dec-to-num conversion.",
            "true converts to 1 and false converts to 0.",
            "No implicit coercion is introduced by num()."
        ],
        example:
            'num("12.5")',
        exampleResult:
            "13",
        diagnostics: [
            "void cannot be converted to num.",
            "Box values cannot be converted to num.",
            "Bag values cannot be converted to num.",
            "Function and Core function values cannot be converted to num.",
            "Malformed numeric text produces a diagnostic.",
            "An out-of-range numeric conversion produces a diagnostic."
        ]
    },

    dec: {
        token: "dec",
        name: "Dec Conversion",
        category: "Type Conversion Expression",
        signature: "dec(expression)",
        parameters: [
            {
                name: "expression",
                description:
                    "The value to explicitly convert to dec."
            }
        ],
        returns: {
            type: "dec",
            description:
                "The converted decimal value."
        },
        description:
            "Explicitly converts a supported Druim value to dec.",
        details: [
            "dec is a dedicated Druim type-conversion expression, not a Core function.",
            "Exactly one complete expression is required.",
            "A dec value is returned unchanged.",
            "A num value converts to its corresponding decimal value.",
            "true converts to 1.0 and false converts to 0.0.",
            "Numeric text may contain an optional leading + or -.",
            "Numeric text may represent either a whole number or decimal value.",
            "If a decimal point is present in numeric text, digits are required on both sides.",
            "Numeric text must be consumed completely; surrounding whitespace and trailing characters are invalid.",
            "No implicit coercion is introduced by dec()."
        ],
        example:
            'dec("12.5")',
        exampleResult:
            "12.5",
        diagnostics: [
            "void cannot be converted to dec.",
            "Box values cannot be converted to dec.",
            "Bag values cannot be converted to dec.",
            "Function and Core function values cannot be converted to dec.",
            "Malformed numeric text produces a diagnostic.",
            "An unrepresentable numeric conversion produces a diagnostic."
        ]
    },

    text: {
        token: "text",
        name: "Text Conversion",
        category: "Type Conversion Expression",
        signature: "text(expression)",
        parameters: [
            {
                name: "expression",
                description:
                    "The value to explicitly convert to text."
            }
        ],
        returns: {
            type: "text",
            description:
                "The canonical textual representation of the value."
        },
        description:
            "Explicitly converts a supported Druim value to its canonical text representation.",
        details: [
            "text is a dedicated Druim type-conversion expression, not a Core function.",
            "Exactly one complete expression is required.",
            "A text value is returned unchanged.",
            "num uses its canonical integer representation.",
            "dec uses its stored decimal representation.",
            "true becomes \"true\" and false becomes \"false\".",
            "void becomes \"void\".",
            "text() uses the same canonical textual representation used by Print and interpolation.",
            "text() does not serialize collections or functions.",
            "No implicit coercion is introduced by text()."
        ],
        example:
            "text(42)",
        exampleResult:
            '"42"',
        diagnostics: [
            "Box values cannot be converted to text.",
            "Bag values cannot be converted to text.",
            "Function and Core function values cannot be converted to text.",
        ]
    },

    flag: {
        token: "flag",
        name: "Flag Conversion",
        category: "Type Conversion Expression",
        signature: "flag(expression)",
        parameters: [
            {
                name: "expression",
                description:
                    "The value to explicitly convert using Druim's canonical truth conversion."
            }
        ],
        returns: {
            type: "flag",
            description:
                "true or false according to Druim's canonical truth conversion."
        },
        description:
            "Explicitly converts a supported Druim value using canonical truth conversion.",
        details: [
            "flag is a dedicated Druim type-conversion expression, not a Core function.",
            "Exactly one complete expression is required.",
            "A flag value is returned unchanged.",
            "num 0 converts to false; every nonzero num converts to true.",
            "dec 0.0 converts to false; every nonzero dec converts to true.",
            "Empty or whitespace-only text converts to false; all other text converts to true.",
            "Text contents are not recursively parsed: \"false\" and \"0\" both convert to true.",
            "void converts to false.",
            "An empty Box converts to false; a nonempty Box converts to true.",
            "An empty Bag converts to false; a nonempty Bag converts to true.",
            "flag() uses the same canonical truth conversion used by Druim control flow."
        ],
        example:
            'flag("false")',
        exampleResult:
            "true",
        diagnostics: [
            "Function and Core function values cannot be converted to flag."
        ]
    }
};

const literals = {
    true: {
        token: "true",
        name: "True",
        category: "Flag Literal",
        signature: "true",
        description:
            "The literal true flag value.",
        details: [
            "true is a built-in flag literal.",
            "Its type is flag.",
            "Its canonical truth conversion is true.",
            "Its canonical text representation is \"true\".",
            "num(true) produces 1.",
            "dec(true) produces 1.0.",
            "text(true) produces \"true\".",
            "flag(true) remains true."
        ],
        example:
            "enabled = true;",
        exampleResult:
            "enabled contains the flag value true."
    },

    false: {
        token: "false",
        name: "False",
        category: "Flag Literal",
        signature: "false",
        description:
            "The literal false flag value.",
        details: [
            "false is a built-in flag literal.",
            "Its type is flag.",
            "Its canonical truth conversion is false.",
            "Its canonical text representation is \"false\".",
            "num(false) produces 0.",
            "dec(false) produces 0.0.",
            "text(false) produces \"false\".",
            "flag(false) remains false."
        ],
        example:
            "enabled = false;",
        exampleResult:
            "enabled contains the flag value false."
    },

    void: {
        token: "void",
        name: "Void",
        category: "Void Literal",
        signature: "void",
        description:
            "The literal form of Druim's explicit absence value.",
        details: [
            "void is an actual value in Druim.",
            "It represents explicit absence rather than an undefined or missing value.",
            "Its type is void.",
            "Its canonical truth conversion is false.",
            "Its canonical text representation is \"void\".",
            "It may be stored in a binding.",
            "It may be returned from a function.",
            "It may appear inside a Box or Bag.",
            "It may be passed as a function argument.",
            "text(void) produces \"void\".",
            "flag(void) produces false.",
            "num(void) and dec(void) produce diagnostics.",
            "void does not take parentheses and is not a type-conversion expression."
        ],
        example:
            "result = void;",
        exampleResult:
            "result contains the explicit void value.",
        diagnostics: [
            "Using void as though it were callable produces a diagnostic.",
            "num(void) produces a diagnostic.",
            "dec(void) produces a diagnostic."
        ]
    }
};

const coreFunctions = {
    rise: {
        token: "rise",
        name: "Rise",
        category: "Core Function",
        signature: "rise(text)",
        parameters: [
            {
                name: "text",
                description:
                    "The text value to convert to uppercase."
            }
        ],
        returns: {
            type: "text",
            description:
                "A new text value with all characters converted to uppercase."
        },
        description:
            "Converts all characters in a text value to uppercase.",
        details: [
            "The original text value is not modified.",
            "Unicode case conversion is supported."
        ],
        example:
            'rise("Druim");',
        exampleResult:
            '"DRUIM"',
        diagnostics: [
            "Exactly one argument is required.",
            "The argument must be text."
        ]
    },

    fall: {
        token: "fall",
        name: "Fall",
        category: "Core Function",
        signature: "fall(text)",
        parameters: [
            {
                name: "text",
                description:
                    "The text value to convert to lowercase."
            }
        ],
        returns: {
            type: "text",
            description:
                "A new text value with all characters converted to lowercase."
        },
        description:
            "Converts all characters in a text value to lowercase.",
        details: [
            "The original text value is not modified.",
            "Unicode case conversion is supported."
        ],
        example:
            'fall("DRUIM");',
        exampleResult:
            '"druim"',
        diagnostics: [
            "Exactly one argument is required.",
            "The argument must be text."
        ]
    },

    cap: {
        token: "cap",
        name: "Cap",
        category: "Core Function",
        signature: "cap(text)",
        parameters: [
            {
                name: "text",
                description:
                    "The text value whose first character will be converted to uppercase."
            }
        ],
        returns: {
            type: "text",
            description:
                "A new text value with only the first character converted to uppercase."
        },
        description:
            "Converts the first character of a text value to uppercase while leaving the remaining characters unchanged.",
        details: [
            "Only the first character is changed.",
            "The remaining characters are preserved exactly.",
            "An empty text value returns an empty text value.",
            "Unicode case conversion is supported."
        ],
        example:
            'cap("dRUIM");',
        exampleResult:
            '"DRUIM"',
        diagnostics: [
            "Exactly one argument is required.",
            "The argument must be text."
        ]
    },

    cut: {
        token: "cut",
        name: "Cut",
        category: "Core Function",
        signature: "cut(text, start, [end])",
        parameters: [
            {
                name: "text",
                description:
                    "The text value to cut."
            },
            {
                name: "start",
                description:
                    "The zero-based character index where the result begins. This index is inclusive."
            },
            {
                name: "end",
                optional: true,
                description:
                    "The zero-based character index where the result ends. This index is exclusive. If omitted, the result continues to the end of the text."
            }
        ],
        returns: {
            type: "text",
            description:
                "A new text value containing the selected character range."
        },
        description:
            "Returns a section of a text value using character indexes.",
        details: [
            "Indexes are zero-based.",
            "The start index is inclusive.",
            "The end index is exclusive.",
            "If end is omitted, the range continues through the end of the text.",
            "Text is indexed by Unicode characters rather than UTF-8 bytes.",
            "If start is beyond the end of the text, an empty text value is returned."
        ],
        example:
            'cut("Druim", 1, 4);',
        exampleResult:
            '"rui"',
        diagnostics: [
            "Two or three arguments are required.",
            "The first argument must be text.",
            "The start index must be a num.",
            "The optional end index must be a num.",
            "Indexes cannot be negative.",
            "The end index cannot be less than the start index."
        ]
    },

    size: {
        token: "size",
        name: "Size",
        category: "Core Function",
        signature: "size(text)",
        parameters: [
            {
                name: "text",
                description:
                    "The text value whose characters will be counted."
            }
        ],
        returns: {
            type: "num",
            description:
                "The number of characters contained in the text value."
        },
        description:
            "Returns the character count of a text value.",
        details: [
            "Characters are counted as Unicode characters rather than UTF-8 bytes.",
            "An empty text value has a size of 0."
        ],
        example:
            'size("Druim");',
        exampleResult:
            "5",
        diagnostics: [
            "Exactly one argument is required.",
            "The argument must be text."
        ]
    },

    fuse: {
        token: "fuse",
        name: "Fuse",
        category: "Core Function",
        signature: "fuse(text, text, ...)",
        parameters: [
            {
                name: "text",
                description:
                    "The first text value to include in the result."
            },
            {
                name: "text",
                description:
                    "The second text value to include in the result."
            },
            {
                name: "...",
                variadic: true,
                description:
                    "Any additional text values to append in argument order."
            }
        ],
        returns: {
            type: "text",
            description:
                "A new text value containing all supplied text arguments in order."
        },
        description:
            "Combines two or more text values into one text value.",
        details: [
            "At least two arguments are required.",
            "Arguments are combined in source order.",
            "Every argument must be text.",
            "The + operator remains arithmetic-only; text concatenation uses fuse."
        ],
        example:
            'fuse("Dru", "im");',
        exampleResult:
            '"Druim"',
        diagnostics: [
            "At least two arguments are required.",
            "Every argument must be text."
        ]
    }
};

const statementModifiers = {
    order: ["stone", "scope"],
    scope: ["loc", "glo"],
    mutuallyExclusive: [
        ["loc", "glo"]
    ]
};

const structureList = Object.values(structures);
const operatorList = Object.values(operators);
const keywordList = Object.values(keywords);
const typeList = Object.values(types);
const conversionExpressionList = Object.values(conversionExpressions);
const literalList = Object.values(literals);
const coreFunctionList = Object.values(coreFunctions);

const structureByToken = new Map();

for (const structure of structureList) {
    structureByToken.set(structure.open, {
        structure,
        role: "open"
    });

    structureByToken.set(structure.close, {
        structure,
        role: "close"
    });

    if (structure.continuation) {
        structureByToken.set(structure.continuation, {
            structure,
            role: "continuation"
        });
    }

    if (structure.separator) {
        structureByToken.set(structure.separator, {
            structure,
            role: "separator"
        });
    }

    if (structure.separators) {
        for (const separator of new Set(structure.separators)) {
            structureByToken.set(separator, {
                structure,
                role: "separator"
            });
        }
    }
}

const structureByOpen = new Map(
    structureList.map((structure) => [
        structure.open,
        structure
    ])
);

const structureByClose = new Map(
    structureList.map((structure) => [
        structure.close,
        structure
    ])
);

const operatorByToken = new Map(
    operatorList.map((operator) => [
        operator.token,
        operator
    ])
);

const keywordByToken = new Map(
    keywordList.map((keyword) => [
        keyword.token,
        keyword
    ])
);

const typeByToken = new Map(
    typeList.map((type) => [
        type.token,
        type
    ])
);

const conversionExpressionByToken = new Map(
    conversionExpressionList.map((conversion) => [
        conversion.token,
        conversion
    ])
);

const literalByToken = new Map(
    literalList.map((literal) => [
        literal.token,
        literal
    ])
);

const coreFunctionByToken = new Map(
    coreFunctionList.map((entry) => [
        entry.token,
        entry
    ])
);

const completionKeywords = [
    ...keywordList,
    ...typeList.filter((type) => type.token === "void"),
    ...conversionExpressionList
];

const completionOperators = operatorList;

const hoverDocs = new Map();

for (const structure of structureList) {
    hoverDocs.set(
        structure.open,
        structure
    );

    hoverDocs.set(
        structure.close,
        structure
    );

    if (structure.continuation) {
        hoverDocs.set(
            structure.continuation,
            structure
        );
    }

    if (structure.separator) {
        hoverDocs.set(
            structure.separator,
            structure
        );
    }

    if (structure.separators) {
        for (const separator of new Set(structure.separators)) {
            hoverDocs.set(
                separator,
                structure
            );
        }
    }
}

for (const operator of operatorList) {
    hoverDocs.set(
        operator.token,
        operator
    );
}

for (const keyword of keywordList) {
    hoverDocs.set(
        keyword.token,
        keyword
    );
}

for (const type of typeList) {
    hoverDocs.set(
        type.token,
        type
    );
}

for (const conversion of conversionExpressionList) {
    hoverDocs.set(
        conversion.token,
        conversion
    );
}

for (const literal of literalList) {
    if (!hoverDocs.has(literal.token)) {
        hoverDocs.set(
            literal.token,
            literal
        );
    }
}

for (const coreFunction of coreFunctionList) {
    hoverDocs.set(
        coreFunction.token,
        coreFunction
    );
}

function getStructureByOpen(token) {
    return structureByOpen.get(token);
}

function getStructureByClose(token) {
    return structureByClose.get(token);
}

function getOperator(token) {
    return operatorByToken.get(token);
}

function getKeyword(token) {
    return keywordByToken.get(token);
}

function getType(token) {
    return typeByToken.get(token);
}

function getConversionExpression(token) {
    return conversionExpressionByToken.get(token);
}

function getLiteral(token) {
    return literalByToken.get(token);
}

function getCoreFunction(token) {
    return coreFunctionByToken.get(token);
}

function getHoverDocumentation(token) {
    return hoverDocs.get(token);
}

function getStructureByToken(token) {
    return structureByToken.get(token);
}

function isStructureOpen(token) {
    return structureByOpen.has(token);
}

function isStructureClose(token) {
    return structureByClose.has(token);
}

module.exports = {
    structures,
    operators,
    keywords,
    types,
    conversionExpressions,
    literals,
    coreFunctions,
    statementModifiers,

    structureList,
    operatorList,
    keywordList,
    typeList,
    conversionExpressionList,
    literalList,
    coreFunctionList,

    structureByToken,
    structureByOpen,
    structureByClose,
    operatorByToken,
    keywordByToken,
    conversionExpressionByToken,
    typeByToken,
    literalByToken,
    coreFunctionByToken,

    completionKeywords,
    completionOperators,
    hoverDocs,

    getStructureByToken,
    getStructureByOpen,
    getStructureByClose,
    getOperator,
    getKeyword,
    getType,
    getConversionExpression,
    getLiteral,
    getCoreFunction,
    getHoverDocumentation,
    isStructureOpen,
    isStructureClose
};