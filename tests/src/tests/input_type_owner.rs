use super::*;

const ENCODING: i8 = 4;
const INVALID_UNLOCK: i8 = 6;

/// Spends a input-type-proxy-lock cell next to an extra input typed `input_type`,
/// and creates an output typed `output_type`.
fn run(args: impl Fn(&Script) -> Bytes, input_type: &[u8], output_type: &[u8]) -> Result<(), i8> {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let lock = env.contract("input-type-proxy-lock", args(&owner));
    let filler = env.always_success(&[]);
    let locked = env.create((lock, None, Bytes::new()));
    let input_type = Some(env.always_success(input_type));
    let extra = env.create((filler.clone(), input_type, Bytes::new()));
    let output_type = Some(env.always_success(output_type));
    env.verify(&[locked, extra], vec![(filler, output_type, Bytes::new())])
}

/// Spends a input-type-proxy-lock cell while a cell typed with the owner type is only a cell dep.
fn run_with_typed_dep() -> Result<(), i8> {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let lock = env.contract("input-type-proxy-lock", hash_args(&owner));
    let filler = env.always_success(&[]);
    let locked = env.create((lock, None, Bytes::new()));
    let typed = env.create((filler.clone(), Some(owner), Bytes::new()));
    env.add_dep(typed);
    env.verify(&[locked], vec![(filler, None, Bytes::new())])
}

#[test]
fn unlocks_with_owner_type_input() {
    assert_eq!(run(hash_args, b"owner", b"other"), Ok(()));
}

#[test]
fn fails_without_owner_type() {
    assert_eq!(run(hash_args, b"other", b"other"), Err(INVALID_UNLOCK));
}

#[test]
fn fails_with_owner_type_only_in_output() {
    assert_eq!(run(hash_args, b"other", b"owner"), Err(INVALID_UNLOCK));
}

#[test]
fn fails_with_owner_type_only_in_dep() {
    assert_eq!(run_with_typed_dep(), Err(INVALID_UNLOCK));
}

#[test]
fn fails_when_owner_script_is_only_a_lock() {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let proxy = env.contract("input-type-proxy-lock", hash_args(&owner));
    let locked = env.create((proxy, None, Bytes::new()));
    let extra = env.create((owner.clone(), None, Bytes::new()));
    assert_eq!(
        env.verify(&[locked, extra], vec![(owner, None, Bytes::new())]),
        Err(INVALID_UNLOCK)
    );
}

#[test]
fn fails_on_short_args() {
    assert_eq!(
        run(|s| hash_args(s).slice(0..31), b"owner", b"other"),
        Err(ENCODING)
    );
}

// Source::Input includes the locked cell itself, so it can carry its own owner type.
#[test]
fn unlocks_when_locked_cell_carries_owner_type() {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let proxy = env.contract("input-type-proxy-lock", hash_args(&owner));
    let filler = env.always_success(&[]);
    let locked = env.create((proxy, Some(owner), Bytes::new()));
    assert_eq!(
        env.verify(&[locked], vec![(filler, None, Bytes::new())]),
        Ok(())
    );
}
