/// An item the FFI cares about.
#[derive(Debug)]
pub enum Item {
    /// A method or function.
    Function(Function),

    /// A built-in type.
    BaseType(BaseType),

    /// A structure.
    Structure(Structure),
}

/// A method or function.
#[derive(Debug)]
pub struct Function {
    /// The fully qualified name of the function.
    pub full_name: String,

    /// The name of the function, as it appears in the `.so`.
    pub linkage_name: String,

    /// The name of the function, as written in an `fn` item.
    pub name: Option<String>,

    /// The module in which the function appeared.
    pub module: Vec<String>,

    /// The index of the return type. If `None`, the function doesn't return a value.
    pub ret_type_index: Option<usize>,

    /// The arguments to the function, as pairs of `(name, type index)`.
    pub arguments: Vec<(String, usize)>,
}

/// A built-in type.
#[derive(Debug)]
pub struct BaseType {
    /// The name of the type.
    pub name: String,

    /// The module in which the type appeared.
    pub module: Vec<String>,

    /// The size of the type, in bytes.
    pub size: u64,

    /// The kind of type this is.
    pub kind: BaseTypeKind,
}

/// The kind of type a `BaseType` is.
#[derive(Debug)]
pub enum BaseTypeKind {
    UnsignedInt,
    SignedInt,
    Float,

    Char,

    Never,
    Unit,
}

/// A structure.
#[derive(Debug)]
pub struct Structure {
    /// The name of the type.
    pub name: String,

    /// The module in which the type appeared.
    pub module: Vec<String>,

    /// The size of the type, in bytes.
    pub size: u64,

    /// The alignment of the type, in bytes.
    pub alignment: u64,

    /// The members of the struct.
    pub members: Vec<StructureMember>,
}

/// A structure member.
#[derive(Debug)]
pub struct StructureMember {
    /// The name of the member.
    pub name: String,

    /// The index of the type.
    pub type_index: usize,

    /// The offset of the member within the struct, in bytes.
    pub offset: u64,

    /// The alignment of the member, in bytes.
    pub alignment: u64,
}
