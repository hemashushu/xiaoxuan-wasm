// Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anvm_parser::types::Value;

pub trait Module {
    fn get_export(&self, name: &str) -> Option<Export>;
    fn eval_func(&self, args: &[Value]) -> Result<Vec<Value>, EngineError>;
    fn get_global_value(&self, name: &str) -> Option<Value>;
    fn set_global_value(&self, name: &str, value: Value) -> Result<(), EngineError>;
}

pub trait Function {
    fn eval(&self, args: &[Value]) -> Result<Vec<Value>, EngineError>;
}

pub trait Table {
    fn get_size(&self) -> u32;
    fn grow_size(&self, increase_number: u32) -> Result<u32, EngineError>;
    fn get_element(&self, index: usize) -> Result<Box<dyn Function>, EngineError>;
    fn set_element(&self, index: usize, func: Box<dyn Function>) -> Result<(), EngineError>;
}

pub trait Memory {
    fn get_page_size(&self) -> u32;
    fn grow_page_size(&self, increase_number: u32) -> Result<u32, EngineError>;
    fn read(&self, address: usize) -> Result<Vec<u8>, EngineError>;
    fn write(&self, address: usize, data: &[u8]) -> Result<(), EngineError>;
}

pub trait Global {
    fn get(&self) -> Value;
    fn set(&self, value: Value) -> Result<(), EngineError>;
}

pub enum Export {
    Function(Box<dyn Function>),
    Table(Box<dyn Table>),
    Memory(Box<dyn Memory>),
    Global(Box<dyn Global>),
}

pub enum EngineError {
    OutOfIndex,
    ObjectNotFound,
}
