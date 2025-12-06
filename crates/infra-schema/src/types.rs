//! Schema types.

/// JSON Schema types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

impl SchemaType {
    /// Get the type as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Array => "array",
            Self::Object => "object",
            Self::Null => "null",
        }
    }
}

impl std::fmt::Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// String format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    DateTime,
    Date,
    Time,
    Email,
    Uri,
    Uuid,
    Hostname,
    Ipv4,
    Ipv6,
    Regex,
}

impl Format {
    /// Get the format as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DateTime => "date-time",
            Self::Date => "date",
            Self::Time => "time",
            Self::Email => "email",
            Self::Uri => "uri",
            Self::Uuid => "uuid",
            Self::Hostname => "hostname",
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
            Self::Regex => "regex",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type() {
        assert_eq!(SchemaType::String.as_str(), "string");
        assert_eq!(SchemaType::Object.as_str(), "object");
    }

    #[test]
    fn test_format() {
        assert_eq!(Format::Email.as_str(), "email");
        assert_eq!(Format::Uuid.as_str(), "uuid");
    }
}
