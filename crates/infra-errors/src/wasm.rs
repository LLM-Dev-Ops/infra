//! WASM bindings for InfraError.

use crate::InfraError;
use wasm_bindgen::prelude::*;

/// JavaScript-compatible error representation
#[wasm_bindgen]
pub struct JsInfraError {
    error_type: String,
    message: String,
    details: JsValue,
}

#[wasm_bindgen]
impl JsInfraError {
    /// Get the error type
    #[wasm_bindgen(getter)]
    pub fn error_type(&self) -> String {
        self.error_type.clone()
    }

    /// Get the error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get additional error details as JSON
    #[wasm_bindgen(getter)]
    pub fn details(&self) -> JsValue {
        self.details.clone()
    }

    /// Check if the error is retryable
    #[wasm_bindgen]
    pub fn is_retryable(&self) -> bool {
        // This is a simplified check - in practice, we'd need the original error
        matches!(
            self.error_type.as_str(),
            "http" | "external" | "message_queue" | "timeout"
        )
    }

    /// Convert to a JavaScript Error object
    #[wasm_bindgen(js_name = toError)]
    pub fn to_js_error(&self) -> js_sys::Error {
        js_sys::Error::new(&self.message)
    }
}

impl From<InfraError> for JsInfraError {
    fn from(err: InfraError) -> Self {
        let error_type = err.error_type().to_string();
        let message = err.to_string();
        let details = serialize_error_details(&err);

        Self {
            error_type,
            message,
            details,
        }
    }
}

impl From<InfraError> for JsValue {
    fn from(err: InfraError) -> Self {
        let js_err = JsInfraError::from(err);
        JsValue::from(js_err)
    }
}

fn serialize_error_details(err: &InfraError) -> JsValue {
    match serde_wasm_bindgen::to_value(err) {
        Ok(val) => val,
        Err(_) => JsValue::NULL,
    }
}

/// Create a config error from JavaScript
#[wasm_bindgen(js_name = createConfigError)]
pub fn create_config_error(message: &str, key: Option<String>) -> JsInfraError {
    let err = if let Some(k) = key {
        InfraError::config_with_key(message, k)
    } else {
        InfraError::config(message)
    };
    JsInfraError::from(err)
}

/// Create a validation error from JavaScript
#[wasm_bindgen(js_name = createValidationError)]
pub fn create_validation_error(message: &str, field: Option<String>) -> JsInfraError {
    let err = if let Some(f) = field {
        InfraError::validation_field(f, message, None, None)
    } else {
        InfraError::validation(message)
    };
    JsInfraError::from(err)
}
