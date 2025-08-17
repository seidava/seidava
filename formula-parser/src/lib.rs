// formula-parser/src/lib.rs

use magnus::{Ruby, eval, prelude::*};
use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Formula {
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub url: Option<String>,
    pub sha256: Option<String>,
    pub dependencies: Vec<String>,
}

pub fn parse_formula(path: &Path) -> Result<Formula, magnus::Error> {
    // 1. Get the class name from the file name (e.g., libpng.rb -> Libpng)
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let class_name = file_stem
        .split('-')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<String>();

    if class_name.is_empty() {
        // Handle error for invalid file name
        panic!("Could not determine class name from file path.");
    }

    // 2. Initialize the Ruby VM
    let ruby = Ruby::new()?;

    // 3. Load the formula file's content
    let file_content = fs::read_to_string(path).expect("Could not read formula file.");

    // 4. Create a Ruby "bridge" class to help us inspect the formula
    // This Ruby code defines a simple class `FormulaInspector` that loads
    // and creates an instance of our formula class.
    let inspector_code = format!(
        r#"
        # Define a dummy Formula class that Homebrew formulae inherit from.
        # It just needs to exist.
        class Formula; end

        # Load the actual formula code
        {}

        # Create an inspector that gives us access to the formula instance
        class SeidavaInspector
          def self.get_formula
            {}
          end
        end
    "#,
        file_content, class_name
    );

    ruby.eval::<magnus::Value>(&inspector_code)?;

    // 5. Get the formula class and extract data from it
    let formula_class: magnus::Value = ruby.eval(format!("Object.const_get('{}')", class_name))?;

    // Use `funcall` to safely call methods on the Ruby object.
    // The `?` operator allows us to handle cases where a field might not exist.
    let description: Option<String> = formula_class.funcall("desc", ())?;
    let homepage: Option<String> = formula_class.funcall("homepage", ())?;
    let url: Option<String> = formula_class.funcall("url", ())?;
    let sha256: Option<String> = formula_class.funcall("sha256", ())?;

    // More complex fields like dependencies might require more detailed parsing,
    // but for now, we'll keep it simple.

    Ok(Formula {
        name: file_stem.to_string(),
        description,
        homepage,
        url,
        sha256,
        dependencies: Vec::new(), // Placeholder for now
    })
}

// At the end of formula-parser/src/lib.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn it_parses_libpng() {
        // IMPORTANT: Adjust this path to where you cloned homebrew-core
        let path_to_formula = Path::new("../../docs/homebrew-core/Formula/a/a2ps.rb");

        let formula_result = parse_formula(path_to_formula);

        assert!(formula_result.is_ok());
        let formula = formula_result.unwrap();

        assert_eq!(formula.name, "libpng");
        assert_eq!(
            formula.description,
            Some("Library for manipulating PNG images".to_string())
        );
        assert_eq!(
            formula.homepage,
            Some("http://www.libpng.org/pub/png/libpng.html".to_string())
        );
        assert!(
            formula
                .url
                .as_ref()
                .unwrap()
                .contains("libpng-1.6.43.tar.xz")
        );
        assert_eq!(
            formula.sha256,
            Some("ca74f02a8a81f341b5501865b4f8b4d8d1e779cb74a0092687c4f10705edd5b1".to_string())
        );
    }
}
