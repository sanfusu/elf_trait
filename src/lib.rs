// Copyright (C) 2020 sanfusu@foxmail.com
// 
// This file is part of accessor.
// 
// accessor is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// accessor is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with accessor.  If not, see <http://www.gnu.org/licenses/>.

use accessor::*;
use std::ops::Range;
// 这里定义的所有 trait 都应该是为了解析而必备的。

pub trait Ident: Setter + Getter {
    /// Encode 字段必须确保大小端两种读法一致，所以 BytesType 字节长度必须为 1.
    /// 该字段的值通常会被缓存在结构体中。
    type Encode: Field<FieldType = Encode, BytesType = [u8; 1]>;
}
pub trait Ehdr: Setter + Getter {
    /// PhtRange 字段的值用于表示 Program Header Table 的范围。
    /// 方便 Elf 解析函数解析之处能够确定 pht 的原始值。
    type PhtRange: Field<FieldType = Range<usize>>;
    /// ShtRange 字段的值用于表示 Section Header Table 的范围。
    type ShtRange: Field<FieldType = Range<usize>>;
    /// Shstrndx 字段用于存储 section 字符串表头在 shdt 中的位置（索引）,
    /// section 字符串表头用于描述 section 的名称。
    type Shstrndx: Field<FieldType = usize>;
}

/// Strtab 需要实现的 trait，需实现索引操作
/// # 用法
/// ```
/// let strTable = StrTableImpl{};
/// let name = strTable[Shdr.sh_name]
/// ```
pub trait Strtab: std::ops::Index<usize, Output = String> {}

/// Section Header 需要实现的 trait
pub trait Shdr: Setter + Getter {
    /// Section 在文件中的范围
    type SecRange: Field<FieldType = Range<usize>>;
}

pub trait ShdrTab<T>: std::ops::Index<usize, Output = T>
where
    T: Shdr,
{
}

/// Section Header 需要实现的 trait
pub trait Phdr: Setter + Getter {
    /// Section 在文件中的范围
    type SegRange: Field<FieldType = Range<usize>>;
}
pub trait PhdrTab<T>: std::ops::Index<usize, Output = T>
where
    T: Phdr,
{
}

pub trait Segmemt<T>: std::ops::Index<usize, Output = T>
where
    T: Shdr,
{
    type Header: Phdr + Field;
}

pub trait Section: Getter {
    type Header: Shdr + Field;
}
