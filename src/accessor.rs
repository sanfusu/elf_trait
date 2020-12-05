// TODO 作为 accessor 可以单独一个仓
use std::ops::Range;

pub enum Encode {
    Le,
    Be,
}

pub trait Field {
    type FieldType;
    /// TODO: BytesType 可以直接替换为 [u8; std::mem::size_of::<Self::FieldType>()]，但目前 rust 不支持这种写法
    type BytesType: AsRef<[u8]>;

    fn from_le_bytes(val: &[u8]) -> Self::FieldType;
    fn from_be_bytes(val: &[u8]) -> Self::FieldType;
    fn to_be_bytes(self) -> Self::BytesType;
    fn to_le_bytes(self) -> Self::BytesType;
    /// range 函数一般会被 Layout 中的 with 函数调用，获取 slice 后，在调用 from_le(ge)_bytes 从而获取字段的值。
    fn range() -> Range<usize>;
}

pub trait Getter {
    /// get 函数返回某个字段的值
    /// # Example
    /// ```not_run
    /// let value = self.get::<Field1>();
    /// ```
    fn get<T>(&self) -> T::FieldType
    where
        T: Field;
    fn getter(&self, encode: Encode) -> Self;

    /// out 将字段值赋值给 dest，并返回 Getter 自身的引用，方便链式调用一条语句输出多个值。
    fn out<T: Field>(&self, dest: &mut T::FieldType) -> &Self {
        *dest = self.get::<T>();
        self
    }
}

pub trait Setter {
    /// with 函数一般用于修改二进制格式中的某个字段，
    /// 返回 `&mut Self` 类型，方便链式调用
    /// # Example
    /// ```not_run
    /// self.with::<Field1>(value1).with::<Field2>(value2);
    /// ```
    fn with<T: Field + Mutable>(&self, value: T) -> &Self;
    fn setter(&self, encode: Encode) -> Self;
}

/// 空接口，用于限制 Field
pub trait Mutable {}
