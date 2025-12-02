#![cfg(target_arch = "wasm32")]

use crate::ast::{AstNode, Span};
use crate::error::{CoreError, LexError, ParseError, RuntimeError};
use crate::serializer::{to_json_string, to_json_string_pretty, to_msgpack_bytes, to_yaml_string};
use crate::value::OrbitValue;
use crate::{evaluate, evaluate_ast, parse, parse_with_recovery};
use serde::Serialize;
use std::mem;
use std::ptr;
use std::slice;
use std::str;

const STATUS_OK: i32 = 0;
const STATUS_ERR: i32 = 1;

#[repr(C)]
pub(crate) struct OrbitSlice {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

#[derive(Serialize)]
struct JsError {
    kind: &'static str,
    message: String,
    span: Span,
}

impl JsError {
    fn new(kind: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    fn from_core(error: CoreError) -> Self {
        match error {
            CoreError::Lex(err) => err.into(),
            CoreError::Parse(err) => err.into(),
            CoreError::Runtime(err) => err.into(),
        }
    }

    fn serde(message: impl Into<String>) -> Self {
        JsError::new("Serde", message, Span::default())
    }

    fn utf8(message: impl Into<String>) -> Self {
        JsError::new("Utf8", message, Span::default())
    }

    fn serializer(kind: &'static str, message: impl Into<String>) -> Self {
        JsError::new(kind, message, Span::default())
    }
}

impl From<LexError> for JsError {
    fn from(error: LexError) -> Self {
        JsError::new("Lex", error.message, error.span)
    }
}

impl From<ParseError> for JsError {
    fn from(error: ParseError) -> Self {
        JsError::new("Parse", error.message, error.span)
    }
}

impl From<RuntimeError> for JsError {
    fn from(error: RuntimeError) -> Self {
        JsError::new("Runtime", error.message, error.span)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_alloc(size: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }
    let mut buffer = Vec::<u8>::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    mem::forget(buffer);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_dealloc(ptr: *mut u8, len: usize, cap: usize) {
    if ptr.is_null() || cap == 0 {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(ptr, len.min(cap), cap);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_version(result_ptr: *mut OrbitSlice) -> i32 {
    write_bytes(result_ptr, env!("CARGO_PKG_VERSION").as_bytes().to_vec());
    STATUS_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_parse(
    source_ptr: *const u8,
    source_len: usize,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(source_ptr, source_len) {
        Ok(source) => match parse(source) {
            Ok(ast) => write_json(result_ptr, &ast),
            Err(err) => write_error(result_ptr, JsError::from_core(err)),
        },
        Err(err) => write_error(result_ptr, err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_parse_with_recovery(
    source_ptr: *const u8,
    source_len: usize,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(source_ptr, source_len) {
        Ok(source) => match parse_with_recovery(source) {
            Ok(report) => write_json(result_ptr, &report),
            Err(err) => write_error(result_ptr, JsError::from_core(err)),
        },
        Err(err) => write_error(result_ptr, err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_evaluate(
    source_ptr: *const u8,
    source_len: usize,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(source_ptr, source_len) {
        Ok(source) => match evaluate(source) {
            Ok(value) => write_json(result_ptr, &value),
            Err(err) => write_error(result_ptr, JsError::from_core(err)),
        },
        Err(err) => write_error(result_ptr, err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_evaluate_ast(
    ast_ptr: *const u8,
    ast_len: usize,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(ast_ptr, ast_len).and_then(|json| deserialize_ast(json)) {
        Ok(ast) => match evaluate_ast(&ast) {
            Ok(value) => write_json(result_ptr, &value),
            Err(err) => write_error(result_ptr, JsError::from(err)),
        },
        Err(err) => write_error(result_ptr, err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_value_to_json(
    value_ptr: *const u8,
    value_len: usize,
    pretty: u32,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(value_ptr, value_len).and_then(deserialize_value) {
        Ok(value) => {
            let rendered = if pretty != 0 {
                to_json_string_pretty(&value)
            } else {
                to_json_string(&value)
            };
            match rendered {
                Ok(json) => {
                    write_bytes(result_ptr, json.into_bytes());
                    STATUS_OK
                }
                Err(err) => write_error(
                    result_ptr,
                    JsError::serializer("Serializer", err.to_string()),
                ),
            }
        }
        Err(err) => write_error(result_ptr, err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_value_to_yaml(
    value_ptr: *const u8,
    value_len: usize,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(value_ptr, value_len).and_then(deserialize_value) {
        Ok(value) => match to_yaml_string(&value) {
            Ok(yaml) => {
                write_bytes(result_ptr, yaml.into_bytes());
                STATUS_OK
            }
            Err(err) => write_error(
                result_ptr,
                JsError::serializer("Serializer", err.to_string()),
            ),
        },
        Err(err) => write_error(result_ptr, err),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn orbit_value_to_msgpack(
    value_ptr: *const u8,
    value_len: usize,
    result_ptr: *mut OrbitSlice,
) -> i32 {
    match read_source(value_ptr, value_len).and_then(deserialize_value) {
        Ok(value) => match to_msgpack_bytes(&value) {
            Ok(bytes) => {
                write_bytes(result_ptr, bytes);
                STATUS_OK
            }
            Err(err) => write_error(
                result_ptr,
                JsError::serializer("Serializer", err.to_string()),
            ),
        },
        Err(err) => write_error(result_ptr, err),
    }
}

fn read_source<'a>(ptr: *const u8, len: usize) -> Result<&'a str, JsError> {
    if len == 0 {
        return Ok("");
    }
    if ptr.is_null() {
        return Err(JsError::new("Input", "null pointer", Span::default()));
    }
    unsafe {
        let bytes = slice::from_raw_parts(ptr, len);
        str::from_utf8(bytes).map_err(|err| JsError::utf8(err.to_string()))
    }
}

fn deserialize_ast(json: &str) -> Result<AstNode, JsError> {
    serde_json::from_str(json).map_err(|err| JsError::serde(err.to_string()))
}

fn deserialize_value(json: &str) -> Result<OrbitValue, JsError> {
    serde_json::from_str(json).map_err(|err| JsError::serde(err.to_string()))
}

fn write_json<T: Serialize>(result_ptr: *mut OrbitSlice, value: &T) -> i32 {
    match serde_json::to_vec(value) {
        Ok(bytes) => {
            write_bytes(result_ptr, bytes);
            STATUS_OK
        }
        Err(err) => write_error(
            result_ptr,
            JsError::serializer("Serializer", err.to_string()),
        ),
    }
}

fn write_error(result_ptr: *mut OrbitSlice, error: JsError) -> i32 {
    let payload = serde_json::to_vec(&error).unwrap_or_else(|_| Vec::new());
    write_bytes(result_ptr, payload);
    STATUS_ERR
}

fn write_bytes(result_ptr: *mut OrbitSlice, bytes: Vec<u8>) {
    unsafe {
        ptr::write(result_ptr, OrbitSlice::from_vec(bytes));
    }
}

impl OrbitSlice {
    fn from_vec(mut data: Vec<u8>) -> Self {
        let cap = data.capacity();
        let ptr = if cap == 0 {
            ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        let len = data.len();
        mem::forget(data);
        Self { ptr, len, cap }
    }
}
