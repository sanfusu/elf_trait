use std::{convert::Into, ops::Range};
pub trait Field {
    type FieldType;
    fn from_le_bytes(val: &[u8]) -> Self::FieldType;
    fn from_be_bytes(val: &[u8]) -> Self::FieldType;
    fn range() -> Range<usize>;
}

/// Layout trait 用于定义修改二进制格式相关的函数
///（[`Layout`] 为只读布局，关于可变布局参见 [`LayoutMut`]）。
/// 通常除了需要实现者使用枚举类型定义各字段类型外，还需要定义各字段的顺序或偏移量，以方便实现 [`Self::with`]  方法。
///
/// 另外，在实现 [`Self::with`] 时需要注意字节序。
/// 这一点可以配合 [`Accessor::get`] 方法使用。
///
/// **Elf 中大部分结构体都需要实现该 trait**
pub trait Layout {
    /// with 函数返回某个字段的值
    /// # Example
    /// ```not_run
    /// let value = self.with::<Field1>();
    /// ```
    fn with<T>(&self) -> T::FieldType
    where
        T: Field;
}

/// 和 [`Layout`] 类似，但是 `LayoutMut` 可以配合 [`Accessor::set`] 使用
pub trait LayoutMut {
    /// with 函数一般用于修改二进制格式中的某个字段，
    /// 返回 `&mut Self` 类型，方便链式调用
    /// # Example
    /// ```not_run
    /// self.with::<Field1>(value1).with::<Field2>(value2);
    /// ```
    fn with<T: Field>(&mut self, value: T::FieldType) -> &mut Self;
}

pub trait Getter {
    type Encode;
    /// Accessor 类型中存储 Encode 字段，一般是对 Self 类型的封装。
    type Accessor: Layout;
    fn getter(&self, encode: Self::Encode) -> Self::Accessor;
}

pub trait Setter {
    type Setter: LayoutMut;
    /// 用于描述字节序的类型
    /// 一般来讲只有两种：大端和小段。
    /// 但是为了防止不同的二进制格式还有其他的字节序或编码方式，这里保留给实现者定义。
    type Encode;
    fn setter(&mut self, encode: Self::Encode) -> &mut Self::Setter;
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

pub trait Ident: Setter {
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
pub trait Ehdr: AsBytes + Setter {
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
pub trait Shdr: AsBytes + Setter {
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

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    #![allow(unused_variables)]

    use std::{convert::TryInto, ops::Range};

    use crate::{Field, Getter, Layout};

    enum TestField {
        Field1,
        Field2,
    }
    struct Field1(u8);
    impl Field for Field1 {
        fn range() -> Range<usize> {
            0..1
        }

        fn from_le_bytes(val: &[u8]) -> u8 {
            u8::from_le(val[0])
        }
        fn from_be_bytes(val: &[u8]) -> u8 {
            u8::from_be(val[0])
        }

        type FieldType = u8;
    }
    struct Field2(u32);
    impl Field for Field2 {
        fn range() -> Range<usize> {
            1..5
        }

        fn from_le_bytes(val: &[u8]) -> Self::FieldType {
            u32::from_le_bytes(val.try_into().unwrap())
        }
        fn from_be_bytes(val: &[u8]) -> Self::FieldType {
            u32::from_be_bytes(val.try_into().unwrap())
        }

        type FieldType = u32;
    }
    enum TestFieldMut {
        Field1(u8),
        Field2(u8),
    }

    impl super::Layout for Test<'_> {
        fn with<T>(&self) -> T::FieldType
        where
            T: Field,
        {
            match self.encode {
                Encode::Le => T::from_le_bytes(&self.data[T::range()]),
                Encode::Be => T::from_be_bytes(&self.data[T::range()]),
            }
        }
    }
    enum Encode {
        Le,
        Be,
    }
    struct Test<'a> {
        data: &'a [u8],
        encode: Encode,
    }
    impl<'a> Test<'a> {
        fn new(data: &'a [u8]) -> Test {
            Test {
                data,
                encode: Encode::Le,
            }
        }
    }
    impl<'a> super::Getter for Test<'a> {
        type Encode = Encode;

        fn getter(&self, encode: Self::Encode) -> Self::Accessor {
            Self::Accessor {
                data: self.data,
                encode,
            }
        }
        type Accessor = Test<'a>;
    }
    #[test]
    fn test() {
        let s1 = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let a = Test::new(&s1[0..5]);
        let getter = a.getter(Encode::Le);
        println!("{:#x?}", getter.with::<Field2>());
        println!("{:#x?}", getter.with::<Field1>());
    }
}
