// Copyright (c) 2022 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anvm_ast::ast;
use anvm_ast::instruction::{self, BlockType};

#[derive(Debug, PartialEq, Clone)]
pub struct NamedAstModule {
    pub name: String,
    pub module: ast::Module,
}

impl NamedAstModule {
    pub fn new(name: &str, module: ast::Module) -> Self {
        Self {
            name: name.to_string(),
            module,
        }
    }
}

/// 一个模块里的所有类型函数的信息
///
/// 函数包括导入的函数（分为本地函数和外部模块普通函数）以及
/// 模块内部定义的普通函数。
/// 经过链接和解析之后，最终函数被分为：本地函数和普通函数两种，
/// 即在模块之外的用户函数以及在模块之内的用户函数合并，它们仅
/// 函数的模块索引值不同。
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionItem {
    Native {
        native_module_index: usize, //
        type_index: usize,          // 目标函数在目标模块当中的类型索引
        function_index: usize,      //
    },
    Normal {
        vm_module_index: usize,         // 目标模块的索引
        type_index: usize,              // 目标函数在目标模块当中的类型索引
        function_index: usize,          // 目标函数在目标模块当中的索引（索引从导入函数开始计数）
        internal_function_index: usize, // 目标函数在目标模块当中的内部函数列表里的索引
        start_address: usize,           // 目标函数的起始位置
        end_address: usize,             // 函数 `end 指令` 所在的位置
        block_items: Vec<BlockItem>,    // 函数内部结构块的位置信息
    },
}

/// 函数当中的流程控制结构块的信息
#[derive(Debug, PartialEq, Clone)]
pub enum BlockItem {
    Block {
        block_type: BlockType,
        start_address: usize,
        end_address: usize,
    },
    Loop {
        block_type: BlockType,
        start_address: usize,
        end_address: usize,
    },
    If {
        block_type: BlockType,
        start_address: usize,
        end_address: usize,
        alternate_address: Option<usize>, // 有些 if 结构块缺少 else 结构
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum BranchTarget {
    Jump(/* relative_depth */ usize, /* address */ usize),
    Recur(/* relative_depth */ usize, /* address */ usize),
}

/// 控制指令
#[derive(Debug, PartialEq, Clone)]
pub enum Control {
    /// 进入一个新的栈帧
    /// 对应 block/loop 指令
    Block {
        block_type: BlockType,
        block_index: usize,
        end_address: usize,
    },

    /// 进入一个新的栈帧，并当原栈顶的数值等于 0 时，跳转到指定的地址 alternate_address,
    /// 对应 if 指令
    /// 有时 if 指令结构缺少 else 结构，这时 alternate_address 的值跟 end_address 的值相同。
    BlockJumpEqZero {
        block_type: BlockType,
        block_index: usize,
        alternate_address: usize,
        end_address: usize,
    },

    /// 跳转到指定的地址
    /// 其中 relative_depth 为当前栈帧距离目标栈帧的层次数量，当数量为 0 时，表示
    /// 跳转到当前栈帧层的其他地址，当数量 >0 时，表示需要弹出相应的栈帧数量。
    /// 对应 else/br/return 指令
    /// 显然对于 else 指令，relative_depth 的值为 0.
    Jump(/* relative_depth */ usize, /* address */ usize),

    /// 跳转到指定的地址
    /// 跟 Jump 指令类似，但仅当原栈顶的数值不等于 0 时才跳转，否则什么事都不做
    /// 对应 br_if 指令
    JumpNotEqZero(/* relative_depth */ usize, /* address */ usize),

    /// 重复执行当前结构块
    /// 对应 br 跳转到 loop 结构块的情况
    /// - 如果 relative_depth 为 0，只需简单地跳到 loop 指令所在地位置即可，
    ///   不需要弹出/压入参数，也不需要弹出/压入栈帧
    /// - 如果 relative_depth 大于 0，则需要弹出目标 loop 结构块所需要的参数，
    ///   然后弹出跟 relative_depth 的值一样数量的栈帧，再压入实参，然而还是不需要创建新的栈帧
    Recur(/* relative_depth */ usize, /* address */ usize),

    RecurNotEqZero(/* relative_depth */ usize, /* address */ usize),

    /// 原 `br_table 指令`
    Branch(Vec<BranchTarget>, BranchTarget),

    /// 调用（普通）函数
    Call {
        /// 目标模块的索引
        vm_module_index: usize,

        /// 目标函数在原模块当中的类型索引
        /// 这是一个冗余信息，用于省去函数调用时的一次查询过程
        type_index: usize,

        /// 目标函数在原模块当中的索引
        /// 此索引包括导入的外部函数，也包括模块内部函数，此索引值为 call 指令参数所指定的值
        function_index: usize,

        /// 目标函数在原模块当中内部函数索引
        /// 这是一个冗余信息，用于快速获取内部函数的局部变量信息
        internal_function_index: usize,

        /// 这是一个冗余信息，用于省去函数调用时的一次查询过程
        address: usize,
    },

    /// 调用本地函数（native function）模块的本地函数
    CallNative {
        native_module_index: usize,

        /// 目标函数在原模块当中的类型索引
        /// 冗余信息，用于省去函数调用时的一次查询过程
        type_index: usize,

        /// 目标函数在原模块当中的函数索引
        /// 本地模块没有导入函数，所以该索引值等于本地函数列表的索引
        function_index: usize,
    },

    /// 原 `call_indirect 指令`
    CallIndirect {
        type_index: usize,
        table_index: usize,
    },

    /// 原 `end 指令`，表示函数或者结构块结束
    ///
    /// 参数是流程控制结构块的索引，对于函数的结束指令（即函数最后一条指令，`end 指令`）
    /// 它的参数值是 None。结构块索引主要用于调式程序时方便定位出错的结构位置信息。
    Return(Option<usize>),
}

/// 编译后的指令
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    /// 按顺序执行的指令
    /// 即执行完一条指令后，不改变程序的执行顺序，继续执行下一条指令
    Sequence(instruction::Instruction),

    /// 控制指令
    /// 会控制或者会改变程序执行顺序的指令
    Control(Control),
}
