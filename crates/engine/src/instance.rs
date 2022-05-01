// Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::rc::Rc;

use anvm_parser::{types::Value, ast::{GlobalType, MemoryType, TableType, FunctionType}};

/// 模块的接口
///
/// 在运行 WebAssembly 应用程序时，一个 Wasm 模块对应一个 Module 实例，
/// 模块之间的链接和调用的实现基于 Module/Function/Table/Memory/Global 等接口，
/// 注意本地函数（native function）也将会存储在一个拥有 Module 接口的模块当中。
///
/// 模块的实现思路受启发于张秀宏先生所著的《WebAssembly 原理与核心技术》，详细的原理
/// 讲解可以参阅该本书。
pub trait Module {
    fn get_export(&self, name: &str) -> Option<Export>;
    fn eval_func(&self, args: &[Value]) -> Result<Vec<Value>, EngineError>;
    fn get_global_value(&self, name: &str) -> Option<Value>;
    fn set_global_value(&mut self, name: &str, value: Value) -> Result<(), EngineError>;
}

pub trait Function {
    fn eval(&self, args: &[Value]) -> Result<Vec<Value>, EngineError>;
    fn get_function_type(&self) -> FunctionType;
}

pub trait Table {
    fn get_size(&self) -> u32;
    fn increase_size(&mut self, increase_number: u32) -> Result<u32, EngineError>;

    fn get_element(&self, index: usize) -> Result<Option<Rc<dyn Function>>, EngineError>;
    fn set_element(&mut self, index: usize, func: Rc<dyn Function>) -> Result<(), EngineError>;
    fn get_table_type(&self) -> TableType;
}

pub trait Memory {
    fn get_page_count(&self) -> u32;
    fn incrase_page(&mut self, increase_number: u32) -> Result<u32, EngineError>;

    fn read_bytes(&self, address: usize, length: usize) -> &[u8]; // Vec<u8>;
    fn write_bytes(&mut self, address: usize, data: &[u8], length: usize);
    fn get_memory_type(&self) -> MemoryType;
}

pub trait GlobalVariable {
    fn get_value(&self) -> Value;
    fn set_value(&mut self, value: Value) -> Result<(), EngineError>;
    fn get_global_type(&self) -> GlobalType;
}

pub enum Export {
    Function(Rc<dyn Function>),
    Table(Rc<dyn Table>),
    Memory(Rc<dyn Memory>),
    GlobalVariable(Rc<dyn GlobalVariable>),
}

#[derive(Debug)]
pub enum EngineError {
    OutOfRange(String),
    Overflow(String),
    ObjectNotFound(String),
    InvalidOperation(String)
}
