use std::{convert::Into, ops::Range};
/// Layout trait 用于定义修改二进制格式相关的函数
///（[`Layout`] 为只读布局，关于可变布局参见 [`LayoutMut`]）。
/// 通常除了需要实现者使用枚举类型定义各字段类型外，还需要定义各字段的顺序或偏移量，以方便实现 [`Self::with`]  方法。
///
/// 另外，在实现 [`Self::with`] 时需要注意字节序。
/// 这一点可以配合 [`Accessor::get`] 方法使用。
///
/// **Elf 中大部分结构体都需要实现该 trait**
pub trait Layout {
    /// [`Self::Field`] 定义二进制格式中的各字段类型，一般用枚举类型定义
    /// # Example
    /// ```
    /// enum Ehdr32Layout{
    ///     e_type(Otype),
    ///     e_phoff(u32),
    /// }
    /// ```
    type Field;
    /// with 函数一般用于修改二进制格式中的某个字段，
    /// 返回 `&mut Self` 类型，方便链式调用
    /// # Example
    /// ```not_run
    /// self.with(Ehdr32Layout::e_type(1)).with(Ehdr32Layout::e_phoff(2))
    /// ```
    fn with(&self, layout: Self::Field) -> Self::Field;
}

/// 和 [`Layout`] 类似，但是 `LayoutMut` 可以配合 [`Accessor::set`] 使用
pub trait LayoutMut {
    type Field;
    fn with(&mut self, layout: Self::Field) -> &mut Self;
}

pub trait Accessor {
    type Setter: LayoutMut;
    type Getter: Layout;
    /// 用于描述字节序的类型
    /// 一般来讲只有两种：大端和小段。
    /// 但是为了防止不同的二进制格式还有其他的字节序或编码方式，这里保留给实现者定义。
    type Encode;
    fn set(&mut self, encode: Self::Encode) -> &mut Self::Setter;
    fn get(&self, encode: Self::Encode) -> &Self::Getter;
}

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

pub trait Ident: Accessor {
    fn magic(&self) -> u32;
    fn encode(&self) -> Self::Encode;
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
pub trait Ehdr: AsBytes + Accessor {
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
pub trait Shdr: AsBytes + Accessor {
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
