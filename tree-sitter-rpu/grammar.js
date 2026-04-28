const PREC = {
  or: 1,
  and: 2,
  compare: 3,
  add: 4,
  multiply: 5,
  unary: 6,
  call: 7,
  property: 8,
};

module.exports = grammar({
  name: "rpu",

  extras: ($) => [/\s/, $.comment],

  word: ($) => $.identifier,

  conflicts: ($) => [
    [$.parenthesized_expression, $.tuple_expression],
  ],

  rules: {
    source_file: ($) => repeat($._top_level_item),

    _top_level_item: ($) =>
      choice(
        $.scene_definition,
        $.function_definition,
        $.event_handler,
        $._statement,
      ),

    scene_definition: ($) =>
      seq("scene", field("name", $.identifier), $.scene_block),

    scene_block: ($) => seq("{", repeat($._scene_item), "}"),

    _scene_item: ($) =>
      choice(
        $.meta_block,
        $.camera_definition,
        $.rect_definition,
        $.sprite_definition,
        $.text_definition,
        $.stack_definition,
        $.highscore_definition,
        $.map_definition,
        $.shape_map_definition,
      ),

    meta_block: ($) => seq("meta", $.property_block),

    camera_definition: ($) =>
      seq("camera", field("name", $.identifier), $.property_block),

    rect_definition: ($) =>
      seq("rect", field("name", $.identifier), $.visual_block),

    sprite_definition: ($) =>
      seq("sprite", field("name", $.identifier), $.visual_block),

    text_definition: ($) =>
      seq("text", field("name", $.identifier), $.visual_block),

    stack_definition: ($) =>
      seq("stack", field("name", $.identifier), $.visual_block),

    highscore_definition: ($) =>
      seq("highscore", field("name", $.identifier), $.visual_block),

    visual_block: ($) =>
      seq(
        "{",
        repeat(choice($.property_assignment, $.animation_definition, $.state_declaration, $.function_definition, $.event_handler)),
        "}",
      ),

    animation_definition: ($) =>
      seq("animation", field("name", $.identifier), $.property_block),

    property_block: ($) =>
      seq("{", repeat($.property_assignment), "}"),

    map_definition: ($) => seq("map", field("name", $.identifier), $.map_block),

    map_block: ($) =>
      seq(
        "{",
        repeat(choice($.property_assignment, $.legend_block, $.ascii_block)),
        "}",
      ),

    shape_map_definition: ($) => seq("shape_map", field("name", $.identifier), $.shape_map_block),

    shape_map_block: ($) =>
      seq(
        "{",
        repeat(choice($.property_assignment, $.legend_block, $.ascii_block, $.wall_definition, $.pipe_definition, $.sdf_wall_definition, $.polyline_definition, $.bumper_definition, $.flipper_definition, $.spring_definition)),
        "}",
      ),

    wall_definition: ($) =>
      seq("wall", field("name", $.identifier), $.property_block),

    pipe_definition: ($) =>
      seq("pipe", field("name", $.identifier), $.property_block),

    sdf_wall_definition: ($) =>
      seq("sdf_wall", field("name", $.identifier), $.property_block),

    polyline_definition: ($) =>
      seq("polyline", field("name", $.identifier), $.property_block),

    bumper_definition: ($) =>
      seq("bumper", field("name", $.identifier), $.property_block),

    flipper_definition: ($) =>
      seq("flipper", field("name", $.identifier), $.property_block),

    spring_definition: ($) =>
      seq("spring", field("name", $.identifier), $.property_block),

    legend_block: ($) => seq("legend", "{", repeat($.legend_entry), "}"),

    legend_entry: ($) =>
      seq(
        field("symbol", $.legend_symbol),
        "=",
        field("value", $.legend_value),
      ),

    legend_symbol: ($) => token(/[^=\s{}]+/),
    legend_value: ($) => token(/[^\s{}\n][^{}\n]*/),

    ascii_block: ($) => seq("ascii", "{", repeat($.ascii_row), "}"),

    ascii_row: ($) => token(prec(-1, /[^{}\n][^\n]*/)),

    property_assignment: ($) =>
      seq(field("name", $.identifier), "=", field("value", $._value)),

    _value: ($) => $.expression,

    state_declaration: ($) =>
      seq("state", field("name", $.identifier), "=", field("value", $.expression)),

    function_definition: ($) =>
      seq("fn", field("name", $.identifier), $.parameter_list, $.statement_block),

    event_handler: ($) =>
      seq("on", field("name", $.identifier), $.parameter_list, $.statement_block),

    parameter_list: ($) => seq("(", optional(commaSep1($.identifier)), ")"),

    statement_block: ($) => seq("{", repeat($._statement), "}"),

    _statement: ($) =>
      choice(
        $.state_declaration,
        $.if_statement,
        $.return_statement,
        $.call_statement,
        $.let_statement,
        $.assignment_statement,
        $.expression_statement,
      ),

    if_statement: ($) =>
      prec.right(
        seq(
          "if",
          field("condition", $.expression),
          field("consequence", $.statement_block),
          optional(
            seq(
              "else",
              field("alternative", choice($.if_statement, $.statement_block)),
            ),
          ),
        ),
      ),

    return_statement: ($) => seq("return", field("value", $.expression)),

    call_statement: ($) => seq("call", $.call_expression),

    let_statement: ($) =>
      seq(
        "let",
        field("name", choice($.identifier, "_")),
        "=",
        field("value", $.expression),
      ),

    assignment_statement: ($) =>
      seq(
        field("left", choice($.property_access, $.identifier)),
        "=",
        field("right", $.expression),
      ),

    expression_statement: ($) => $.expression,

    expression: ($) =>
      choice(
        $.call_expression,
        $.property_access,
        $.binary_expression,
        $.unary_expression,
        $.tuple_expression,
        $.array,
        $.self,
        $.identifier,
        $.string,
        $.number,
        $.color_literal,
        $.boolean,
        $.parenthesized_expression,
      ),

    parenthesized_expression: ($) => seq("(", $.expression, ")"),

    tuple_expression: ($) => seq("(", commaSep1($.expression), ")"),

    array: ($) => seq("[", optional(commaSep1($.expression)), "]"),

    call_expression: ($) =>
      prec(
        PREC.call,
        seq(field("function", $.identifier), field("arguments", $.argument_list)),
      ),

    argument_list: ($) => seq("(", optional(commaSep1($.expression)), ")"),

    property_access: ($) =>
      prec.left(
        PREC.property,
        seq(field("object", choice($.identifier, $.self)), ".", field("property", $.identifier)),
      ),

    unary_expression: ($) =>
      prec(
        PREC.unary,
        seq(field("operator", choice("-", "!")), field("argument", $.expression)),
      ),

    binary_expression: ($) =>
      choice(
        ...[
          ["||", PREC.or],
          ["&&", PREC.and],
          ["==", PREC.compare],
          ["!=", PREC.compare],
          ["<=", PREC.compare],
          [">=", PREC.compare],
          ["<", PREC.compare],
          [">", PREC.compare],
          ["+", PREC.add],
          ["-", PREC.add],
          ["*", PREC.multiply],
          ["/", PREC.multiply],
        ].map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("left", $.expression),
              field("operator", operator),
              field("right", $.expression),
            ),
          ),
        ),
      ),

    boolean: ($) => choice("true", "false"),
    self: ($) => "self",
    identifier: ($) => /[A-Za-z_][A-Za-z0-9_]*/,
    number: ($) => /\d+(\.\d+)?/,
    string: ($) => token(seq('"', repeat(choice(/[^"\\]/, /\\./)), '"')),
    color_literal: ($) => /#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?/,
    comment: ($) => token(seq("//", /.*/)),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)), optional(","));
}
