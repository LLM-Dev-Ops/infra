//! Schema builder for programmatic schema creation.

use crate::types::{Format, SchemaType};
use serde_json::{json, Value};

/// Builder for creating JSON schemas
pub struct SchemaBuilder {
    schema: Value,
}

impl SchemaBuilder {
    /// Create a new schema builder
    pub fn new() -> Self {
        Self {
            schema: json!({
                "$schema": "http://json-schema.org/draft-07/schema#"
            }),
        }
    }

    /// Set the schema type
    pub fn schema_type(mut self, schema_type: SchemaType) -> Self {
        self.schema["type"] = json!(schema_type.as_str());
        self
    }

    /// Set the title
    pub fn title(mut self, title: &str) -> Self {
        self.schema["title"] = json!(title);
        self
    }

    /// Set the description
    pub fn description(mut self, description: &str) -> Self {
        self.schema["description"] = json!(description);
        self
    }

    /// Add a required field
    pub fn required(mut self, fields: &[&str]) -> Self {
        self.schema["required"] = json!(fields);
        self
    }

    /// Add properties
    pub fn properties(mut self, properties: Value) -> Self {
        self.schema["properties"] = properties;
        self
    }

    /// Add a property
    pub fn property(mut self, name: &str, property: Value) -> Self {
        if self.schema.get("properties").is_none() {
            self.schema["properties"] = json!({});
        }
        self.schema["properties"][name] = property;
        self
    }

    /// Set minimum value (for numbers)
    pub fn minimum(mut self, min: f64) -> Self {
        self.schema["minimum"] = json!(min);
        self
    }

    /// Set maximum value (for numbers)
    pub fn maximum(mut self, max: f64) -> Self {
        self.schema["maximum"] = json!(max);
        self
    }

    /// Set minimum length (for strings/arrays)
    pub fn min_length(mut self, min: usize) -> Self {
        self.schema["minLength"] = json!(min);
        self
    }

    /// Set maximum length (for strings/arrays)
    pub fn max_length(mut self, max: usize) -> Self {
        self.schema["maxLength"] = json!(max);
        self
    }

    /// Set pattern (for strings)
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.schema["pattern"] = json!(pattern);
        self
    }

    /// Set format (for strings)
    pub fn format(mut self, format: Format) -> Self {
        self.schema["format"] = json!(format.as_str());
        self
    }

    /// Set enum values
    pub fn enum_values(mut self, values: &[&str]) -> Self {
        self.schema["enum"] = json!(values);
        self
    }

    /// Set array items schema
    pub fn items(mut self, items: Value) -> Self {
        self.schema["items"] = items;
        self
    }

    /// Set minimum items (for arrays)
    pub fn min_items(mut self, min: usize) -> Self {
        self.schema["minItems"] = json!(min);
        self
    }

    /// Set maximum items (for arrays)
    pub fn max_items(mut self, max: usize) -> Self {
        self.schema["maxItems"] = json!(max);
        self
    }

    /// Set unique items (for arrays)
    pub fn unique_items(mut self, unique: bool) -> Self {
        self.schema["uniqueItems"] = json!(unique);
        self
    }

    /// Disallow additional properties
    pub fn additional_properties(mut self, allowed: bool) -> Self {
        self.schema["additionalProperties"] = json!(allowed);
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: Value) -> Self {
        self.schema["default"] = value;
        self
    }

    /// Build the schema
    pub fn build(self) -> Value {
        self.schema
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a string property schema
pub fn string_property() -> Value {
    json!({ "type": "string" })
}

/// Create an integer property schema
pub fn integer_property() -> Value {
    json!({ "type": "integer" })
}

/// Create a number property schema
pub fn number_property() -> Value {
    json!({ "type": "number" })
}

/// Create a boolean property schema
pub fn boolean_property() -> Value {
    json!({ "type": "boolean" })
}

/// Create an array property schema
pub fn array_property(items: Value) -> Value {
    json!({ "type": "array", "items": items })
}

/// Create an object property schema
pub fn object_property(properties: Value) -> Value {
    json!({ "type": "object", "properties": properties })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder() {
        let schema = SchemaBuilder::new()
            .schema_type(SchemaType::Object)
            .title("Person")
            .property("name", json!({ "type": "string" }))
            .property("age", json!({ "type": "integer", "minimum": 0 }))
            .required(&["name"])
            .build();

        assert_eq!(schema["type"], "object");
        assert_eq!(schema["title"], "Person");
        assert_eq!(schema["properties"]["name"]["type"], "string");
        assert_eq!(schema["required"], json!(["name"]));
    }

    #[test]
    fn test_property_helpers() {
        assert_eq!(string_property()["type"], "string");
        assert_eq!(integer_property()["type"], "integer");
        assert_eq!(number_property()["type"], "number");
        assert_eq!(boolean_property()["type"], "boolean");
    }
}
