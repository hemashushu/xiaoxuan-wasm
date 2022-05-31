// Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # 指令转换
//!
//! - block -> Block
//! - loop -> Block
//! - if -> BlockJumpEqZero
//! - br/else/return -> Jump
//! - br -> Recur
//! - br_if -> JumpNotEqZero
//! - br_if -> RecurNotEqZero
//! - br_table -> Branch ([BranchTarget::Jump(relative_depth, addr)], BranchTarget::Recur(relative_depth, addr))
//! - call -> CallInternal/CallExternal/CallNative
//! - call_indirect -> DynamicCall
//! - end -> Return

use anvm_ast::{
    instruction::BlockType,
    types::{check_value_types, ValueType, ValueTypeCheckError},
};

use crate::{error::EngineError, vm::VM, vm_stack::INFO_SEGMENT_ITEM_COUNT};

pub enum ControlResult {
    Sequence,

    /// 进入一个函数或者一个结构块
    ///
    /// 参数用于更新虚拟机的 pc 值
    FunctionIn {
        is_function_call: bool,
        vm_module_index: usize,
        function_index: usize,
        frame_type: BlockType,
        address: usize,
    },

    /// 从一个函数或者一个结构块跳出
    ///
    /// 参数用于更新虚拟机的 pc 值
    FunctionOut {
        is_function_call: bool,
        vm_module_index: usize,
        function_index: usize,
        frame_type: BlockType,
        address: usize,
    },

    ProgramEnd,
}

pub fn do_return(vm: &mut VM) -> Result<ControlResult, EngineError> {
    let frame_type = &vm.status.frame_type;
    let vm_module_index = vm.status.vm_module_index;
    let function_index = vm.status.function_index;

    // 获取当前帧的返回值类型
    let result_types = {
        match frame_type {
            BlockType::ResultEmpty => vec![],
            BlockType::ResultI32 => vec![ValueType::I32],
            BlockType::ResultI64 => vec![ValueType::I64],
            BlockType::ResultF32 => vec![ValueType::F32],
            BlockType::ResultF64 => vec![ValueType::F64],
            BlockType::TypeIndex(type_index) => {
                let vm_module = &vm.resource.vm_modules[vm_module_index];
                let function_type = &vm_module.function_types[*type_index as usize];
                function_type.results.clone()
            }
        }
    };

    // 判断操作数是否足够当前函数或结构块用于返回
    let result_count = result_types.len();
    let stack_size = vm.stack.get_size();
    let operands_count = stack_size - vm.status.base_pointer - INFO_SEGMENT_ITEM_COUNT;
    if operands_count < result_count {
        return Err(EngineError::InvalidOperation(format!(
            "failed to return result from function {} (module {}), not enough operands, expected: {}, actual: {}",
            function_index, vm_module_index, result_count, operands_count)));
    }

    // 判断返回值的数据类型
    let results = vm.stack.peek_values(stack_size - result_count, stack_size);
    match check_value_types(results, &result_types) {
        Err(ValueTypeCheckError::LengthMismatch) => unreachable!(),
        Err(ValueTypeCheckError::DataTypeMismatch(index)) => {
            return Err(EngineError::InvalidOperation(format!(
                "failed to return result from function {} (module {}), The data type of result {} does not match, expected: {}, actual: {}",
                function_index,
                vm_module_index,
                index +1,
                result_types[index],
                results[index].get_type())));
        }
        _ => {
            // pass
        }
    }

    let (is_call_frame, is_program_end, vm_module_index, function_index, frame_type, address) =
        vm.pop_frame(result_count);

    // let is_program_end = vm.pop_frame(result_count);
    if is_program_end {
        Ok(ControlResult::ProgramEnd)
    } else {
        Ok(ControlResult::FunctionOut {
            is_function_call: is_call_frame,
            vm_module_index,
            function_index,
            frame_type,
            address,
        })
    }
}
