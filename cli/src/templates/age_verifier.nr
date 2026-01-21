/*
File: age_verifier.nr
Purpose: Verify if a person is within the age threshold based on their year of birth and the current year.
*/

fn main(year_of_birth: u64, current_year: pub u64, age_threshold: pub u64) -> bool {
    let age = current_year - year_of_birth;
    assert(age >= age_threshold);
}

#[test]
fn test_age_verifier() {
    let year_of_birth = 2000;
    let current_year = 2024;
    let age_threshold = 18;
    let is_eligible = main(year_of_birth, current_year, age_threshold);
    assert(is_eligible);
}
