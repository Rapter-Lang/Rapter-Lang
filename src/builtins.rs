/// Built-in generic types (Option, Result) that are recognized by the compiler
/// These types don't need to be defined in source code - the compiler knows about them

use crate::ast::Type;
use std::collections::HashMap;

/// Information about a built-in generic type
#[derive(Debug, Clone)]
pub struct BuiltinGenericType {
    pub name: String,
    pub type_params: Vec<String>,  // e.g., ["T"] for Option<T>, ["T", "E"] for Result<T,E>
    pub variants: Vec<BuiltinVariant>,
}

/// Variant of a built-in generic enum
#[derive(Debug, Clone)]
pub struct BuiltinVariant {
    pub name: String,
    pub has_value: bool,  // true for Some(T), Ok(T); false for None, Err(E)
    pub value_type_param: Option<String>,  // Which type param? "T" for Some, "E" for Err
}

impl BuiltinGenericType {
    /// Create the Option<T> built-in type
    pub fn option() -> Self {
        BuiltinGenericType {
            name: "Option".to_string(),
            type_params: vec!["T".to_string()],
            variants: vec![
                BuiltinVariant {
                    name: "Some".to_string(),
                    has_value: true,
                    value_type_param: Some("T".to_string()),
                },
                BuiltinVariant {
                    name: "None".to_string(),
                    has_value: false,
                    value_type_param: None,
                },
            ],
        }
    }

    /// Create the Result<T, E> built-in type
    pub fn result() -> Self {
        BuiltinGenericType {
            name: "Result".to_string(),
            type_params: vec!["T".to_string(), "E".to_string()],
            variants: vec![
                BuiltinVariant {
                    name: "Ok".to_string(),
                    has_value: true,
                    value_type_param: Some("T".to_string()),
                },
                BuiltinVariant {
                    name: "Err".to_string(),
                    has_value: true,
                    value_type_param: Some("E".to_string()),
                },
            ],
        }
    }

    /// Get all built-in generic types
    pub fn all_builtins() -> HashMap<String, BuiltinGenericType> {
        let mut builtins = HashMap::new();
        builtins.insert("Option".to_string(), Self::option());
        builtins.insert("Result".to_string(), Self::result());
        builtins
    }

    /// Check if a type name is a built-in generic type
    pub fn is_builtin(name: &str) -> bool {
        matches!(name, "Option" | "Result")
    }

    /// Substitute type parameters to create a concrete type
    /// Example: Option<T> with T=int becomes Option<int>
    pub fn substitute(&self, type_args: &[Type]) -> Result<Type, String> {
        if type_args.len() != self.type_params.len() {
            return Err(format!(
                "Type {} expects {} type parameters, got {}",
                self.name,
                self.type_params.len(),
                type_args.len()
            ));
        }

        Ok(Type::Generic {
            name: self.name.clone(),
            type_params: type_args.to_vec(),
        })
    }

    /// Get the variant info for a specific variant name
    pub fn get_variant(&self, variant_name: &str) -> Option<&BuiltinVariant> {
        self.variants.iter().find(|v| v.name == variant_name)
    }

    /// Get the type of a variant's value
    /// Example: Option<int>::Some has type int
    pub fn variant_value_type(&self, variant_name: &str, type_args: &[Type]) -> Option<Type> {
        let variant = self.get_variant(variant_name)?;
        
        if !variant.has_value {
            return None;
        }

        // Find which type parameter this variant uses
        let param_name = variant.value_type_param.as_ref()?;
        let param_index = self.type_params.iter().position(|p| p == param_name)?;
        
        // Return the corresponding type argument
        type_args.get(param_index).cloned()
    }
}

/// Registry of all built-in types
pub struct BuiltinRegistry {
    generics: HashMap<String, BuiltinGenericType>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        BuiltinRegistry {
            generics: BuiltinGenericType::all_builtins(),
        }
    }

    pub fn is_generic_builtin(&self, name: &str) -> bool {
        self.generics.contains_key(name)
    }

    pub fn get_generic(&self, name: &str) -> Option<&BuiltinGenericType> {
        self.generics.get(name)
    }

    /// Validate that a generic type instantiation is correct
    /// Example: Option<int> is valid, Option<int, string> is not
    pub fn validate_instantiation(&self, name: &str, type_args: &[Type]) -> Result<(), String> {
        let builtin = self.get_generic(name)
            .ok_or_else(|| format!("Type {} is not a built-in generic type", name))?;

        if type_args.len() != builtin.type_params.len() {
            return Err(format!(
                "Type {} expects {} type parameters, got {}",
                name,
                builtin.type_params.len(),
                type_args.len()
            ));
        }

        Ok(())
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_builtin() {
        let opt = BuiltinGenericType::option();
        assert_eq!(opt.name, "Option");
        assert_eq!(opt.type_params.len(), 1);
        assert_eq!(opt.variants.len(), 2);
        
        let some_variant = opt.get_variant("Some").unwrap();
        assert!(some_variant.has_value);
        
        let none_variant = opt.get_variant("None").unwrap();
        assert!(!none_variant.has_value);
    }

    #[test]
    fn test_result_builtin() {
        let res = BuiltinGenericType::result();
        assert_eq!(res.name, "Result");
        assert_eq!(res.type_params.len(), 2);
        assert_eq!(res.variants.len(), 2);
    }

    #[test]
    fn test_variant_value_type() {
        let opt = BuiltinGenericType::option();
        let type_args = vec![Type::Int];
        
        // Some(T) with T=int should return int
        let some_type = opt.variant_value_type("Some", &type_args);
        assert_eq!(some_type, Some(Type::Int));
        
        // None has no value
        let none_type = opt.variant_value_type("None", &type_args);
        assert_eq!(none_type, None);
    }

    #[test]
    fn test_registry() {
        let registry = BuiltinRegistry::new();
        assert!(registry.is_generic_builtin("Option"));
        assert!(registry.is_generic_builtin("Result"));
        assert!(!registry.is_generic_builtin("Vec"));
    }
}
