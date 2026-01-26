/*
File: hash_preimage.nr
Purpose: Prove that a value is the preimage of a hash.
Use Cases:
- Sealed bids
- Lotteries
- Predictions
*/

use dep::std;

fn main(pre_image: Field, salt: Field, commitment: pub Field) {
    let computed_hash = std::hash::pedersen_hash([pre_image, salt]);
    assert(computed_hash == commitment, "Computed hash does not match the provided commitment");
}

#[test]

fn test_valid_pre_image() {
    let pre_image = 42000;
    let salt = 0x1a2b3c4d5e6f;
    let commitment = std::hash::pedersen_hash([pre_image, salt]);

    main(pre_image, salt, commitment);
}

#[test(should_fail)]
fn test_wrong_pre_image() {
    let pre_image = 42000;
    let wrong_pre_image = 42001;
    let salt = 0x1a2b3c4d5e6f;
    let commitment = std::hash::pedersen_hash([pre_image, salt]);

    main(wrong_pre_image, salt, commitment);
}

#[test(should_fail)]
fn test_wrong_salt() {
    let pre_image = 42000;
    let salt = 0x1a2b3c4d5e6f1;
    let wrong_salt = 0x1a2b3c4d5e6f2;
    let commitment = std::hash::pedersen_hash([pre_image, salt]);

    main(pre_image, wrong_salt, commitment);
}
