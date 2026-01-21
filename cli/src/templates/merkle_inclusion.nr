/*
File: merkle_inclusion.nr
Purpose: Prove membership in a Merkle tree using a value and a Merkle path.
*/

global DEPTH: u32 = 3;

fn main(
    value: Field,
    path_elements: pub [Field; DEPTH],
    path_indices: pub [Field; DEPTH],
    root: pub Field,
) {
    let hashed_value = calculate_hash(value);

    let merkle_tree_root = construct_merkle_tree(hashed_value, path_indices, path_elements);

    assert(merkle_tree_root == root, "Merkle tree root does not match");
}

fn calculate_hash(value: Field) -> Field {
    let hashed_value = std::hash::pedersen_hash([value]);
    hashed_value
}

fn construct_merkle_tree(
    leaf: Field,
    path_indices: [Field; DEPTH],
    path_elements: [Field; DEPTH],
) -> Field {
    let mut current_leaf = leaf;

    for i in 0..DEPTH {
        let sibling_index = path_indices[i];
        let sibling_element = path_elements[i];

        if sibling_index == 0 {
            current_leaf = std::hash::pedersen_hash([current_leaf, sibling_element]);
        } else {
            current_leaf = std::hash::pedersen_hash([sibling_element, current_leaf]);
        }
    }

    current_leaf
}
