use accessor::*;
use std::{convert::Into, ops::Range};

pub trait AsBytes {
    /// 将 Ehdr 结构体序列化
    fn as_bytes<'a>(&'a self) -> &'a [u8]
    where
        Self: Sized,
    {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    /// 除了是可修改的，其余和 as_bytes 类似
    fn as_bytes_mut<'a>(&'a mut self) -> &'a mut [u8]
    where
        Self: Sized,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self as *mut Self as *mut u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

pub trait Ident: Setter + Getter {
    fn magic(&self) -> u32;
    fn encode(&self) -> Encode;
}

/// 实现 Ehdr trait 的结构体可以有很多种定义方法。
/// # Example
/// ```
/// enum Ehdr32Layout {
///     phoff(u32),
/// }
/// const EHDR32_PHOFF_OFFSET: usize;
/// struct Ehdr32(&[u8])
/// ```
pub trait Ehdr: AsBytes + Setter + Getter {
    type Machine: Into<usize>;
    type Version: Into<usize>;
    type Otype: Into<usize>;
    type Ident: Ident;

    fn ident(&self) -> Self::Ident;
    /// 返回 program header table 的起始偏移地址（相对于文件开始）
    fn phoff(&self) -> usize;
    /// 返回 program header table 中的条目数量
    fn phnum(&self) -> usize;
    /// 返回 program header table 中条目的大小
    fn phentsize(&self) -> usize;
    /// 返回 program header table 在文件中的范围
    fn phrange(&self) -> Range<usize> {
        self.phoff()..(self.phoff() + self.phentsize() * self.phnum())
    }

    /// 返回 section header table 的起始偏移地址（相对于文件开始）
    fn shoff(&self) -> usize;
    /// 返回 section header table 中的条目数量
    fn shnum(&self) -> usize;
    /// 返回 section header table 中条目的大小
    fn shentsize(&self) -> usize;
    /// 返回 section header table 在文件中的范围
    fn shrange(&self) -> Range<usize> {
        self.shoff()..(self.shoff() + self.shentsize() * self.shnum())
    }
    /// 返回对象文件类型
    fn otype(&self) -> Self::Otype;
    /// 返回机器类型，[`Self::Machine`] 一般定义为枚举类型
    fn machine(&self) -> Self::Machine;
    /// 返回 Elf 对象文件的版本
    fn version(&self) -> Self::Version;
    /// 返回进程的入口虚拟地址
    fn entry(&self) -> usize;
    /// 返回 section 名称字符串表在 section header table 中的入口索引
    fn shstrndx(&self) -> usize;
}

/// Strtab 需要实现的 trait，需实现索引操作
/// # 用法
/// ```
/// let strTable = StrTableImpl{};
/// let name = strTable[Shdr.sh_name]
/// ```
pub trait Strtab: std::ops::Index<usize, Output = String> {}

/// Section Header 需要实现的 trait
pub trait Shdr: AsBytes + Setter + Getter {
    /// 返回 shdr 中的 sh_name 字段
    fn name_idx(&self) -> usize;
    /// 返回 section 相对于文件起始的偏移量
    fn offset(&self) -> usize;
    /// 返回 section 大小
    fn size(&self) -> usize;
    /// 返回 section 在文件中的范围
    fn sec_range(&self) -> Range<usize> {
        self.offset()..(self.offset() + self.size())
    }
}