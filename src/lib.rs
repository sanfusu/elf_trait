use std::{convert::Into, ops::Range};
pub mod accessor;
pub use accessor::*;

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

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    #![allow(unused_variables)]

    use std::{cell::RefCell, convert::TryInto, ops::Range, rc::Rc};

    use crate::{Encode, Field, Getter, Mutable, Setter};
    struct Field1(u8);
    impl Mutable for Field1 {}
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
        type BytesType = [u8; 1];
        type FieldType = u8;

        fn to_be_bytes(self) -> Self::BytesType {
            self.0.to_be_bytes()
        }

        fn to_le_bytes(self) -> Self::BytesType {
            self.0.to_le_bytes()
        }
    }
    struct Field2(u32);
    impl Mutable for Field2 {}
    impl Field for Field2 {
        type FieldType = u32;
        type BytesType = [u8; 4];

        fn range() -> Range<usize> {
            1..5
        }

        fn from_le_bytes(val: &[u8]) -> Self::FieldType {
            u32::from_le_bytes(val.try_into().unwrap())
        }
        fn from_be_bytes(val: &[u8]) -> Self::FieldType {
            u32::from_be_bytes(val.try_into().unwrap())
        }

        fn to_be_bytes(self) -> Self::BytesType {
            self.0.to_be_bytes()
        }

        fn to_le_bytes(self) -> Self::BytesType {
            self.0.to_le_bytes()
        }
    }

    struct Test {
        data: Rc<RefCell<[u8]>>,
        encode: Encode,
    }
    impl Test {
        fn new(data: Rc<RefCell<[u8]>>) -> Test {
            Test {
                data,
                encode: Encode::Le,
            }
        }
    }

    impl super::Getter for Test {
        fn getter(&self, encode: Encode) -> Self {
            Self {
                data: self.data.clone(),
                encode,
            }
        }
        fn get<T>(&self) -> T::FieldType
        where
            T: Field,
        {
            match self.encode {
                Encode::Le => T::from_le_bytes(&self.data.borrow()[T::range()]),
                Encode::Be => T::from_be_bytes(&self.data.borrow()[T::range()]),
            }
        }
    }
    impl super::Setter for Test {
        fn setter(&self, encode: Encode) -> Self {
            Self {
                data: self.data.clone(),
                encode,
            }
        }
        fn with<T: Field>(&self, value: T) -> &Self {
            match self.encode {
                Encode::Le => {
                    self.data.borrow_mut()[T::range()]
                        .copy_from_slice(value.to_le_bytes().as_ref());
                }
                Encode::Be => {
                    self.data.borrow_mut()[T::range()]
                        .copy_from_slice(value.to_be_bytes().as_ref());
                }
            };
            self
        }
    }
    #[test]
    fn test() {
        let a = Test::new(Rc::new(RefCell::new([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        ])));

        let getter = a.getter(Encode::Be);
        println!("{:#x?}", getter.get::<Field1>());
        println!("{:#x?}", getter.get::<Field2>());

        let setter = a.setter(Encode::Le);
        setter
            .with(Field1(0x12))
            .with(Field2(0x12345678))
            .setter(Encode::Le)
            .with(Field1(0x23));
        println!("{:#x?}", getter.get::<Field1>());
        println!("{:#x?}", getter.get::<Field2>());

        let mut field1: <Field1 as Field>::FieldType = 0;
        let mut field2: <Field2 as Field>::FieldType = 0;
        getter.out::<Field1>(&mut field1).out::<Field2>(&mut field2);
        println!("{:#x?}, {:#x?}", field1, field2);
    }
}
