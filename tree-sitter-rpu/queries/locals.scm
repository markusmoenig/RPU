(function_definition
  name: (identifier) @local.definition.function)

(parameter_list
  (identifier) @local.definition.parameter)

(let_statement
  name: (identifier) @local.definition.var)

(state_declaration
  name: (identifier) @local.definition.var)

(identifier) @local.reference
