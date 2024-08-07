use core::fmt;

use crate::{
    ast::{AstSerdeOptions, Ident, ProcedureName},
    ByteReader, ByteWriter, Deserializable, DeserializationError, LibraryPath, RpoDigest,
    Serializable, SourceSpan, Span, Spanned,
};

// INVOKE
// ================================================================================================

/// Represents the kind of an invocation
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InvokeKind {
    Exec = 0,
    Call,
    SysCall,
    ProcRef,
}

/// Represents a specific invocation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Invoke {
    pub kind: InvokeKind,
    pub target: InvocationTarget,
}

impl Spanned for Invoke {
    fn span(&self) -> SourceSpan {
        self.target.span()
    }
}

impl Invoke {
    pub fn new(kind: InvokeKind, target: InvocationTarget) -> Self {
        Self { kind, target }
    }
}

// INVOCATION TARGET
// ================================================================================================

/// Describes targets of `exec`, `call`, and `syscall` instructions.
///
/// A label of an invoked procedure must comply with the following rules:
/// - It can be a hexadecimal string representing a MAST root digest ([RpoDigest]). In this case,
///   the label must start with "0x" and must be followed by a valid hexadecimal string
///   representation of an [RpoDigest].
/// - It can contain a single procedure name. In this case, the label must comply with procedure
///   name rules.
/// - It can contain module name followed by procedure name (e.g., "module::procedure"). In this
///   case both module and procedure name must comply with relevant name rules.
///
/// All other combinations will result in an error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InvocationTarget {
    /// An absolute procedure reference, but opaque in that we do not know where the callee is
    /// defined. However, it does not actually matter, we consider such references to be _a priori_
    /// valid.
    MastRoot(Span<RpoDigest>) = 0,
    /// A locally-defined procedure.
    ProcedureName(ProcedureName) = 1,
    /// A context-sensitive procedure path, which references the name of an import in the
    /// containing module.
    ProcedurePath { name: ProcedureName, module: Ident } = 2,
    /// A fully-resolved procedure path, which refers to a specific externally-defined procedure
    /// with its full path.
    AbsoluteProcedurePath { name: ProcedureName, path: LibraryPath } = 3,
}

impl Spanned for InvocationTarget {
    fn span(&self) -> SourceSpan {
        match self {
            Self::MastRoot(ref spanned) => spanned.span(),
            Self::ProcedureName(ref spanned) => spanned.span(),
            Self::ProcedurePath { ref name, .. } | Self::AbsoluteProcedurePath { ref name, .. } => {
                name.span()
            },
        }
    }
}

impl crate::prettier::PrettyPrint for InvocationTarget {
    fn render(&self) -> crate::prettier::Document {
        use vm_core::utils::DisplayHex;

        use crate::prettier::*;

        match self {
            Self::MastRoot(digest) => display(DisplayHex(digest.as_bytes().as_slice())),
            Self::ProcedureName(name) => display(name),
            Self::ProcedurePath { name, module } => display(format_args!("{}::{}", module, name)),
            Self::AbsoluteProcedurePath { name, path } => {
                display(format_args!("::{}::{}", path, name))
            },
        }
    }
}
impl fmt::Display for InvocationTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::prettier::PrettyPrint;

        self.pretty_print(f)
    }
}

impl InvocationTarget {
    fn tag(&self) -> u8 {
        // SAFETY: This is safe because we have given this enum a primitive representation with
        // #[repr(u8)], with the first field of the underlying union-of-structs the discriminant.
        //
        // See the section on "accessing the numeric value of the discriminant"
        // here: https://doc.rust-lang.org/std/mem/fn.discriminant.html
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    /// Serialize to `target` using `options`
    pub fn write_into_with_options<W: ByteWriter>(&self, target: &mut W, options: AstSerdeOptions) {
        target.write_u8(self.tag());
        match self {
            Self::MastRoot(spanned) => spanned.write_into_with_options(target, options.debug_info),
            Self::ProcedureName(name) => name.write_into_with_options(target, options),
            Self::ProcedurePath { name, module } => {
                name.write_into_with_options(target, options);
                module.write_into(target);
            },
            Self::AbsoluteProcedurePath { name, path } => {
                name.write_into_with_options(target, options);
                path.write_into(target);
            },
        }
    }

    /// Deserialize from `source` using `options`
    pub fn read_from_with_options<R: ByteReader>(
        source: &mut R,
        options: AstSerdeOptions,
    ) -> Result<Self, DeserializationError> {
        match source.read_u8()? {
            0 => {
                let root = Span::<RpoDigest>::read_from_with_options(source, options.debug_info)?;
                Ok(Self::MastRoot(root))
            },
            1 => {
                let name = ProcedureName::read_from_with_options(source, options)?;
                Ok(Self::ProcedureName(name))
            },
            2 => {
                let name = ProcedureName::read_from_with_options(source, options)?;
                let module = Ident::read_from(source)?;
                Ok(Self::ProcedurePath { name, module })
            },
            3 => {
                let name = ProcedureName::read_from_with_options(source, options)?;
                let path = LibraryPath::read_from(source)?;
                Ok(Self::AbsoluteProcedurePath { name, path })
            },
            n => Err(DeserializationError::InvalidValue(format!(
                "{} is not a valid invocation target type",
                n
            ))),
        }
    }
}

impl Serializable for InvocationTarget {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.write_into_with_options(target, AstSerdeOptions::new(false, true));
    }
}

impl Deserializable for InvocationTarget {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        Self::read_from_with_options(source, AstSerdeOptions::new(false, true))
    }
}
