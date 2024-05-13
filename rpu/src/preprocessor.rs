use crate::prelude::*;

pub struct Preprocessor {
    defines: FxHashMap<String, String>,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor {
    fn new() -> Self {
        Preprocessor {
            defines: FxHashMap::default(),
        }
    }

    /// Process a line of code and store any #define statements. This is very limited right now, only simple single line defines are supported.
    fn process_line(&mut self, line: &str) -> String {
        if line.trim_start().starts_with("#define") {
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() > 2 {
                self.defines
                    .insert(parts[1].to_string(), parts[2..].join(" "));
            }
            String::new() // Return an empty string for define lines
        } else {
            self.expand_macros(line)
        }
    }

    /// Expand a line of code by replacing any defined macros
    fn expand_macros(&self, line: &str) -> String {
        let mut expanded_line = line.to_string();
        for (key, value) in &self.defines {
            expanded_line = expanded_line.replace(key, value);
        }
        expanded_line
    }

    /// Process a module of code, line by line
    pub fn process_module(&mut self, module: &str) -> String {
        let mut processed_file = String::new();
        for line in module.lines() {
            processed_file.push_str(&self.process_line(line));
            processed_file.push('\n');
        }
        processed_file
    }
}
