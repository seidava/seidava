// formula-parser/src/lib.rs

use magnus::{Ruby, prelude::*};
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
    let ruby = match Ruby::get() {
        Ok(ruby) => ruby,
        Err(e) => return Err(magnus::Error::new(magnus::exception::runtime_error(), e.to_string())),
    };

    // 3. Load the formula file's content and extract metadata lines
    let file_content = fs::read_to_string(path).expect("Could not read formula file.");
    
    // Extract only the metadata lines we care about (desc, homepage, url, sha256)
    let mut metadata_lines = Vec::new();
    for line in file_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("desc ") || 
           trimmed.starts_with("homepage ") || 
           trimmed.starts_with("url ") {
            metadata_lines.push(line);
        } else if trimmed.starts_with("sha256 \"") {
            // Only include the main sha256 line (not the bottle ones)
            metadata_lines.push(line);
        }
    }
    let extracted_metadata = metadata_lines.join("\n");

    // 4. Create a Ruby "bridge" class to help us inspect the formula
    // This Ruby code defines a Formula base class and then loads the actual formula
    let inspector_code = format!(
        r#"
        require 'ostruct'
        
        # Define a base Formula class that tracks the values set by DSL methods
        class Formula
          @@formulas = {{}}
          
          def self.desc(value = nil)
            if value
              @@formulas[self.name] ||= {{}}
              @@formulas[self.name][:desc] = value
            else
              @@formulas[self.name]&.dig(:desc)
            end
          end
          
          def self.homepage(value = nil)
            if value
              @@formulas[self.name] ||= {{}}
              @@formulas[self.name][:homepage] = value
            else
              @@formulas[self.name]&.dig(:homepage)
            end
          end
          
          def self.url(value = nil)
            if value
              @@formulas[self.name] ||= {{}}
              @@formulas[self.name][:url] = value
            else
              @@formulas[self.name]&.dig(:url)
            end
          end
          
          def self.sha256(value = nil)
            if value
              @@formulas[self.name] ||= {{}}
              @@formulas[self.name][:sha256] = value
            else
              @@formulas[self.name]&.dig(:sha256)
            end
          end
          
          def self.depends_on(*args)
            # Ignore for now
          end
          
          def self.bottle(&block)
            # Ignore for now
          end
          
          def self.license(value)
            # Ignore for now
          end
          
          def self.mirror(value)
            # Ignore for now
          end
          
          def self.uses_from_macos(*args)
            # Ignore for now
          end
          
          def install
            # Ignore for now
          end
          
          def test(&block)
            # Ignore for now
          end
          
          # Additional methods that might be called in install/test blocks
          def system(*args)
            # Ignore system calls
          end
          
          def inreplace(*args)
            # Ignore inreplace calls
          end
          
          # Path/directory methods
          def etc
            "/usr/local/etc"
          end
          
          def elisp
            "/usr/local/share/emacs/site-lisp"
          end
          
          def tap
            OpenStruct.new(user: "homebrew", issues_url: "https://github.com/Homebrew/homebrew-core/issues")
          end
          
          def pkg_version
            "1.0.0"
          end
          
          def std_configure_args
            []
          end
          
          def prefix
            "/usr/local"
          end
          
          def opt_prefix
            "/usr/local/opt"
          end
          
          def testpath
            "/tmp"
          end
          
          def bin
            OpenStruct.new
          end
          
          def assert(*args)
            # Ignore assertions
          end
          
          # Make File available to the test block
          File = Object.const_get(:File) if Object.const_defined?(:File)
        end

        # Load only the metadata from the actual formula code
        class {} < Formula
          {}
        end
    "#,
        class_name, extracted_metadata
    );

    ruby.eval::<magnus::Value>(&inspector_code)?;

    // 5. Get the formula class and extract data from it
    let formula_class: magnus::Value = ruby.eval(&format!("Object.const_get('{}')", class_name))?;

    // Use `funcall` to safely call methods on the Ruby object to get the stored values
    let description: Option<String> = formula_class.funcall("desc", ()).ok();
    let homepage: Option<String> = formula_class.funcall("homepage", ()).ok();
    let url: Option<String> = formula_class.funcall("url", ()).ok();
    let sha256: Option<String> = formula_class.funcall("sha256", ()).ok();

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
    fn it_parses_a2ps() {
        let _cleanup = unsafe { magnus::embed::init() };
        
        let path_to_formula = Path::new("tests/fixtures/a2ps.rb");
        let formula_result = parse_formula(path_to_formula);
        assert!(formula_result.is_ok(), "Failed to parse formula: {:?}", formula_result.err());
        let formula = formula_result.unwrap();

        assert_eq!(formula.name, "a2ps");
        assert!(formula.description.is_some());
        assert!(formula.homepage.is_some());
        assert!(formula.url.is_some());
        assert!(formula.sha256.is_some());
    }
}