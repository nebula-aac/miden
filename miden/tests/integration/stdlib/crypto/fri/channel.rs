use miden::MerkleSet;
use vm_core::{Felt, FieldElement};
use winter_fri::{FriProof, VerifierError};

use winterfell::{
    crypto::{BatchMerkleProof, ElementHasher, Hasher as HasherTrait},
    DeserializationError,
};

pub trait UnBatch<E: FieldElement, H: ElementHasher> {
    fn unbatch<const N: usize, const W: usize>(
        &mut self,
        positions: &[usize],
        domain_size: usize,
        layer_commitments: Vec<<H as HasherTrait>::Digest>,
    ) -> (Vec<MerkleSet>, Vec<([u8; 32], Vec<Felt>)>);
}

pub struct MidenFriVerifierChannel<E: FieldElement, H: ElementHasher<BaseField = E::BaseField>> {
    layer_commitments: Vec<H::Digest>,
    layer_proofs: Vec<BatchMerkleProof<H>>,
    layer_queries: Vec<Vec<E>>,
    remainder: Vec<E>,
}

impl<E, H> MidenFriVerifierChannel<E, H>
where
    E: FieldElement,
    H: ElementHasher<BaseField = E::BaseField>,
{
    /// Builds a new verifier channel from the specified [FriProof].
    ///
    /// # Errors
    /// Returns an error if the specified `proof` could not be parsed correctly.
    pub fn new(
        proof: FriProof,
        layer_commitments: Vec<H::Digest>,
        domain_size: usize,
        folding_factor: usize,
    ) -> Result<Self, DeserializationError> {
        let remainder = proof.parse_remainder()?;
        let (layer_queries, layer_proofs) =
            proof.parse_layers::<H, E>(domain_size, folding_factor)?;

        Ok(MidenFriVerifierChannel {
            layer_commitments,
            layer_proofs,
            layer_queries,
            remainder,
        })
    }

    pub fn take_fri_remainder(&mut self) -> Vec<E> {
        self.remainder.clone()
    }

    pub fn layer_proofs(&mut self) -> Vec<BatchMerkleProof<H>> {
        self.layer_proofs.drain(..).collect()
    }

    pub fn layer_queries(&mut self) -> Vec<Vec<E>> {
        self.layer_queries.clone()
    }

    pub fn read_fri_layer_commitments(&mut self) -> Vec<H::Digest> {
        self.layer_commitments.drain(..).collect()
    }

    pub fn read_remainder<const N: usize>(
        &mut self,
        _commitment: &<H as HasherTrait>::Digest,
    ) -> Result<Vec<E>, VerifierError> {
        let remainder = self.take_fri_remainder();

        // TODO: Verifiy commitment in the case where the remainder codeword is sent
        Ok(remainder)
    }
}