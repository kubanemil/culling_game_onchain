//! Allowlist proofs. Leaves and inner nodes use different prefixes so a node can never be
//! passed off as a leaf, and pairs are hashed in sorted order so proofs need no left/right bits.

use anchor_lang::prelude::Pubkey;
use solana_sha256_hasher::hashv;

const LEAF_PREFIX: &[u8] = &[0];
const NODE_PREFIX: &[u8] = &[1];

pub fn leaf(wallet: &Pubkey, max_amount: u64) -> [u8; 32] {
    hashv(&[LEAF_PREFIX, wallet.as_ref(), &max_amount.to_le_bytes()]).to_bytes()
}

pub fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    hashv(&[NODE_PREFIX, lo, hi]).to_bytes()
}

pub fn verify(proof: &[[u8; 32]], root: &[u8; 32], leaf: [u8; 32]) -> bool {
    proof
        .iter()
        .fold(leaf, |node, sibling| hash_pair(&node, sibling))
        == *root
}

/// Off-chain helper: the root and one proof per leaf. An odd node is carried up unpaired.
#[cfg(not(target_os = "solana"))]
pub fn build(leaves: &[[u8; 32]]) -> ([u8; 32], Vec<Vec<[u8; 32]>>) {
    assert!(!leaves.is_empty(), "allowlist is empty");
    let mut proofs = vec![Vec::new(); leaves.len()];
    let mut positions: Vec<usize> = (0..leaves.len()).collect();
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        for (proof, position) in proofs.iter_mut().zip(positions.iter_mut()) {
            let sibling = *position ^ 1;
            if sibling < level.len() {
                proof.push(level[sibling]);
            }
            *position /= 2;
        }
        level = level
            .chunks(2)
            .map(|pair| match pair {
                [a, b] => hash_pair(a, b),
                [a] => *a,
                _ => unreachable!(),
            })
            .collect();
    }
    (level[0], proofs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wallets(n: usize) -> Vec<(Pubkey, u64)> {
        (0..n)
            .map(|i| (Pubkey::new_unique(), 1_000 * (i as u64 + 1)))
            .collect()
    }

    fn leaves(list: &[(Pubkey, u64)]) -> Vec<[u8; 32]> {
        list.iter().map(|(w, a)| leaf(w, *a)).collect()
    }

    #[test]
    fn every_member_verifies_for_any_tree_size() {
        for n in 1..=17 {
            let list = wallets(n);
            let (root, proofs) = build(&leaves(&list));
            for ((wallet, amount), proof) in list.iter().zip(&proofs) {
                assert!(verify(proof, &root, leaf(wallet, *amount)), "n={n}");
            }
        }
    }

    #[test]
    fn single_leaf_tree_has_an_empty_proof() {
        let list = wallets(1);
        let (root, proofs) = build(&leaves(&list));
        assert!(proofs[0].is_empty());
        assert_eq!(root, leaf(&list[0].0, list[0].1));
    }

    #[test]
    fn a_higher_allocation_does_not_verify() {
        let list = wallets(5);
        let (root, proofs) = build(&leaves(&list));
        assert!(!verify(&proofs[2], &root, leaf(&list[2].0, list[2].1 + 1)));
    }

    #[test]
    fn a_stranger_cannot_reuse_a_members_proof() {
        let list = wallets(5);
        let (root, proofs) = build(&leaves(&list));
        assert!(!verify(
            &proofs[0],
            &root,
            leaf(&Pubkey::new_unique(), list[0].1)
        ));
    }

    #[test]
    fn a_tampered_proof_does_not_verify() {
        let list = wallets(8);
        let (root, mut proofs) = build(&leaves(&list));
        proofs[3][1][0] ^= 1;
        assert!(!verify(&proofs[3], &root, leaf(&list[3].0, list[3].1)));
    }
}
