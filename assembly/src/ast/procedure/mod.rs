mod alias;
mod id;
mod name;
#[allow(clippy::module_inception)]
mod procedure;
mod resolver;

use alloc::string::String;

pub use self::{
    alias::{AliasTarget, ProcedureAlias},
    id::ProcedureIndex,
    name::{ProcedureName, QualifiedProcedureName},
    procedure::{Procedure, Visibility},
    resolver::{LocalNameResolver, ResolvedProcedure},
};
use crate::{
    ast::{AstSerdeOptions, Invoke},
    ByteReader, ByteWriter, DeserializationError, SourceSpan, Span, Spanned,
};

// EXPORT
// ================================================================================================

/// Represents an exportable entity from a [super::Module].
///
/// Currently only procedures (either locally-defined or re-exported) are exportable, but in the
/// future this may be expanded.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Export {
    /// A locally-defined procedure.
    Procedure(Procedure) = 0,
    /// An alias for an externally-defined procedure, i.e. a re-exported import.
    Alias(ProcedureAlias) = 1,
}

impl Export {
    /// Adds documentation to this export.
    pub fn with_docs(self, docs: Option<Span<String>>) -> Self {
        match self {
            Self::Procedure(proc) => Self::Procedure(proc.with_docs(docs)),
            Self::Alias(alias) => Self::Alias(alias.with_docs(docs)),
        }
    }

    /// Returns the name of the exported procedure.
    pub fn name(&self) -> &ProcedureName {
        match self {
            Self::Procedure(ref proc) => proc.name(),
            Self::Alias(ref alias) => alias.name(),
        }
    }

    /// Returns the documentation for this procedure.
    pub fn docs(&self) -> Option<&str> {
        match self {
            Self::Procedure(ref proc) => proc.docs().map(|spanned| spanned.as_deref().into_inner()),
            Self::Alias(ref alias) => alias.docs().map(|spanned| spanned.as_deref().into_inner()),
        }
    }

    /// Returns the visibility of this procedure (e.g. public or private).
    ///
    /// See [Visibility] for more details on what visibilities are supported.
    pub fn visibility(&self) -> Visibility {
        match self {
            Self::Procedure(ref proc) => proc.visibility(),
            Self::Alias(_) => Visibility::Public,
        }
    }

    /// Returns the number of automatically-allocated words of memory this function requires
    /// for the storage of temporaries/local variables.
    pub fn num_locals(&self) -> usize {
        match self {
            Self::Procedure(ref proc) => proc.num_locals() as usize,
            Self::Alias(_) => 0,
        }
    }

    /// Returns true if this procedure is the program entrypoint.
    pub fn is_main(&self) -> bool {
        self.name().is_main()
    }

    /// Unwraps this [Export] as a [Procedure], or panic.
    #[track_caller]
    pub fn unwrap_procedure(&self) -> &Procedure {
        match self {
            Self::Procedure(ref proc) => proc,
            Self::Alias(_) => panic!("attempted to unwrap alias export as procedure definition"),
        }
    }

    /// Get an iterator over the set of other procedures invoked from this procedure.
    ///
    /// NOTE: This only applies to [Procedure]s, other types currently return an empty
    /// iterator whenever called.
    pub(crate) fn invoked<'a, 'b: 'a>(&'b self) -> impl Iterator<Item = &'a Invoke> + 'a {
        match self {
            Self::Procedure(ref proc) if proc.invoked.is_empty() => procedure::InvokedIter::Empty,
            Self::Procedure(ref proc) => procedure::InvokedIter::NonEmpty(proc.invoked.iter()),
            Self::Alias(_) => procedure::InvokedIter::Empty,
        }
    }
}

/// Serialization
impl Export {
    pub fn write_into_with_options<W: ByteWriter>(&self, target: &mut W, options: AstSerdeOptions) {
        target.write_u8(self.tag());
        match self {
            Self::Procedure(ref proc) => proc.write_into_with_options(target, options),
            Self::Alias(ref proc) => proc.write_into_with_options(target, options),
        }
    }

    pub fn read_from_with_options<R: ByteReader>(
        source: &mut R,
        options: AstSerdeOptions,
    ) -> Result<Self, DeserializationError> {
        match source.read_u8()? {
            0 => Procedure::read_from_with_options(source, options).map(Self::Procedure),
            1 => ProcedureAlias::read_from_with_options(source, options).map(Self::Alias),
            n => {
                Err(DeserializationError::InvalidValue(format!("invalid procedure kind tag: {n}")))
            },
        }
    }

    fn tag(&self) -> u8 {
        // SAFETY: This is safe because we have given this enum a primitive representation with
        // #[repr(u8)], with the first field of the underlying union-of-structs the discriminant.
        //
        // See the section on "accessing the numeric value of the discriminant"
        // here: https://doc.rust-lang.org/std/mem/fn.discriminant.html
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl crate::prettier::PrettyPrint for Export {
    fn render(&self) -> crate::prettier::Document {
        match self {
            Self::Procedure(ref proc) => proc.render(),
            Self::Alias(ref proc) => proc.render(),
        }
    }
}

impl Spanned for Export {
    fn span(&self) -> SourceSpan {
        match self {
            Self::Procedure(ref spanned) => spanned.span(),
            Self::Alias(ref spanned) => spanned.span(),
        }
    }
}
