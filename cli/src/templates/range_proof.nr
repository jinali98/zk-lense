/*
File: range_proof.nr
Purpose: Prove that a value is within a range.
Use Cases:
- Collateral checks
- Age ranges
- Bid validation
*/

fn main(value: u64, max_value: pub u64, min_value: pub u64) {
    assert(value <= max_value, "Value is greater than max value");
    assert(value >= min_value, "Value is less than min value");
}

#[test]
fn test_valid_range_proof() {
    let value = 50000;
    let min = 1000;
    let max = 100000;

    main(value, max, min);
}

#[test(should_fail)]
fn test_value_below_minimum() {
    let value = 999;
    let min = 1000;
    let max = 100000;
    main(value, min, max);
}
