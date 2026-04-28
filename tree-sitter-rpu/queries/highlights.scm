; Keywords
[
  "scene"
  "meta"
  "camera"
  "rect"
  "sprite"
  "animation"
  "text"
  "map"
  "shape_map"
  "legend"
  "ascii"
  "wall"
  "pipe"
  "sdf_wall"
  "polyline"
  "bumper"
  "flipper"
  "spring"
  "on"
  "fn"
  "if"
  "else"
  "return"
  "let"
  "state"
  "call"
] @keyword

[
  "true"
  "false"
] @constant.builtin.boolean

; Definitions
(scene_definition name: (identifier) @type)
(camera_definition name: (identifier) @type)
(rect_definition name: (identifier) @type)
(sprite_definition name: (identifier) @type)
(animation_definition name: (identifier) @function)
(text_definition name: (identifier) @type)
(stack_definition name: (identifier) @type)
(highscore_definition name: (identifier) @type)
(map_definition name: (identifier) @type)
(shape_map_definition name: (identifier) @type)
(wall_definition name: (identifier) @type)
(pipe_definition name: (identifier) @type)
(sdf_wall_definition name: (identifier) @type)
(polyline_definition name: (identifier) @type)
(bumper_definition name: (identifier) @type)
(flipper_definition name: (identifier) @type)
(spring_definition name: (identifier) @type)

(function_definition name: (identifier) @function)
(event_handler name: (identifier) @function.method)
(parameter_list (identifier) @variable.parameter)
(state_declaration name: (identifier) @variable.member)

; Properties and fields
(property_assignment name: (identifier) @property)
(property_access
  object: (identifier) @variable
  property: (identifier) @property)

((identifier) @variable.builtin
  (#eq? @variable.builtin "self"))

; Calls
(call_expression function: (identifier) @function.call)
(call_statement (call_expression function: (identifier) @function.call))

; Literals
(string) @string
(number) @number
(color_literal) @string.special
(legend_value) @string.special
(comment) @comment
(legend_symbol) @character

; Operators / punctuation
[
  "="
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "+"
  "-"
  "*"
  "/"
  "&&"
  "||"
  "!"
] @operator

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
  ","
  "."
] @punctuation.delimiter
