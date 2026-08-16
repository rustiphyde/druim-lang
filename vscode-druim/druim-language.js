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
            "Delimits a Druim function-call argument list or groups the expression supplied to Print `|>`."
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
        category: "statement",
        description: "Defines a new binding."
    },

    "=;": {
        token: "=;",
        name: "DefineEmpty",
        category: "statement",
        description: "Defines a new binding whose value is void."
    },

    "<<": {
        token: "<<",
        name: "Mutate",
        category: "statement",
        description: "Changes the value stored in an existing binding."
    },

    ":=": {
        token: ":=",
        name: "Copy",
        category: "statement",
        description: "Creates a new binding using a copied value."
    },

    ":>": {
        token: ":>",
        name: "Bind",
        category: "statement",
        description: "Creates another name for the same binding identity."
    },

    "?=": {
        token: "?=",
        name: "Guard",
        category: "statement",
        description: "Defines a binding conditionally according to Druim's guard rules."
    },

    "|>": {
        token: "|>",
        name: "Print",
        category: "statement",
        description: "Emits one expression as line-oriented output."
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
        category: "arithmetic",
        description: "Adds values."
    },

    "-": {
        token: "-",
        name: "Subtract / Negate",
        category: "arithmetic",
        description: "Subtracts values or negates a value when used as a prefix."
    },

    "*": {
        token: "*",
        name: "Multiply",
        category: "arithmetic",
        description: "Multiplies values."
    },

    "/": {
        token: "/",
        name: "Divide",
        category: "arithmetic",
        description: "Divides values."
    },

    "%": {
        token: "%",
        name: "Modulo",
        category: "arithmetic",
        description: "Returns the remainder of a division."
    },

    "==": {
        token: "==",
        name: "Equal",
        category: "comparison",
        description: "Tests whether two values are equal."
    },

    "!=": {
        token: "!=",
        name: "Not Equal",
        category: "comparison",
        description: "Tests whether two values are not equal."
    },

    "<": {
        token: "<",
        name: "Less Than",
        category: "comparison",
        description: "Tests whether the left value is less than the right value."
    },

    "<=": {
        token: "<=",
        name: "Less Than or Equal",
        category: "comparison",
        description: "Tests whether the left value is less than or equal to the right value."
    },

    ">": {
        token: ">",
        name: "Greater Than",
        category: "comparison",
        description: "Tests whether the left value is greater than the right value."
    },

    ">=": {
        token: ">=",
        name: "Greater Than or Equal",
        category: "comparison",
        description: "Tests whether the left value is greater than or equal to the right value."
    },

    "&&": {
        token: "&&",
        name: "And",
        category: "logical",
        description: "Logical AND."
    },

    "||": {
        token: "||",
        name: "Or",
        category: "logical",
        description: "Logical OR."
    },

    "!": {
        token: "!",
        name: "Not",
        category: "logical",
        description: "Logical negation."
    },
    ":": {
        token: ":",
        name: "Guard Separator",
        category: "statement",
        description:
            "Separates candidate expressions in a Druim Guard statement. Guard selects the first truthy value, or void if none are truthy."
    },
};

const keywords = {
    fn: {
        token: "fn",
        name: "Function",
        category: "declaration",
        description: "Declares a Druim function."
    },

    ret: {
        token: "ret",
        name: "Return",
        category: "control",
        description: "Returns a value from the current function."
    },

    loc: {
        token: "loc",
        name: "Local",
        category: "modifier",
        description: "Applies local scope to the following declaration or statement form."
    },

    glo: {
        token: "glo",
        name: "Global",
        category: "modifier",
        description: "Applies global scope to the following declaration or statement form."
    },

    stone: {
        token: "stone",
        name: "Stone",
        category: "modifier",
        description: "Creates an immutable binding identity."
    }
};

const types = {
    num: {
        token: "num",
        name: "Number",
        description: "Druim integer numeric type."
    },

    dec: {
        token: "dec",
        name: "Decimal",
        description: "Druim decimal numeric type."
    },

    flag: {
        token: "flag",
        name: "Flag",
        description: "Druim boolean/truth-value type."
    },

    text: {
        token: "text",
        name: "Text",
        description: "Druim text type."
    },

    void: {
        token: "void",
        name: "Void",
        description: "Represents explicit absence of a value."
    }
};

const literals = {
    true: {
        token: "true",
        name: "True",
        description: "Druim true flag literal."
    },

    false: {
        token: "false",
        name: "False",
        description: "Druim false flag literal."
    },

    void: {
        token: "void",
        name: "Void",
        description: "Druim void literal."
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
    ...typeList
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
    literals,
    coreFunctions,
    statementModifiers,

    structureList,
    operatorList,
    keywordList,
    typeList,
    literalList,
    coreFunctionList,

    structureByToken,
    structureByOpen,
    structureByClose,
    operatorByToken,
    keywordByToken,
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
    getLiteral,
    getCoreFunction,
    getHoverDocumentation,
    isStructureOpen,
    isStructureClose
};