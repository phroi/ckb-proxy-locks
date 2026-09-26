use super::*;

const ENCODING: i8 = 4;
const INVALID_UNLOCK: i8 = 6;

/// Spends a lock-proxy-lock cell next to an extra input with `extra_lock`,
/// and creates an output with `output_lock`.
fn run(args: impl Fn(&Script) -> Bytes, extra_lock: &[u8], output_lock: &[u8]) -> Result<(), i8> {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let proxy = env.contract("lock-proxy-lock", args(&owner));
    let locked = env.create((proxy, None, Bytes::new()));
    let extra_lock = env.always_success(extra_lock);
    let extra = env.create((extra_lock, None, Bytes::new()));
    let output_lock = env.always_success(output_lock);
    env.verify(&[locked, extra], vec![(output_lock, None, Bytes::new())])
}

#[test]
fn unlocks_with_owner_lock_input() {
    assert_eq!(run(hash_args, b"owner", b"other"), Ok(()));
}

#[test]
fn fails_without_owner_lock_input() {
    assert_eq!(run(hash_args, b"other", b"other"), Err(INVALID_UNLOCK));
}

#[test]
fn fails_with_owner_lock_only_in_output() {
    assert_eq!(run(hash_args, b"other", b"owner"), Err(INVALID_UNLOCK));
}

#[test]
fn fails_with_owner_lock_only_in_dep() {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let proxy = env.contract("lock-proxy-lock", hash_args(&owner));
    let locked = env.create((proxy, None, Bytes::new()));
    let owner_cell = env.create((owner, None, Bytes::new()));
    env.add_dep(owner_cell);
    let filler = env.always_success(&[]);
    assert_eq!(
        env.verify(&[locked], vec![(filler, None, Bytes::new())]),
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

#[test]
fn fails_with_owner_script_only_as_input_type() {
    let mut env = Env::new();
    let owner = env.always_success(b"owner");
    let proxy = env.contract("lock-proxy-lock", hash_args(&owner));
    let filler = env.always_success(&[]);
    let locked = env.create((proxy, None, Bytes::new()));
    let extra = env.create((filler.clone(), Some(owner), Bytes::new()));
    assert_eq!(
        env.verify(&[locked, extra], vec![(filler, None, Bytes::new())]),
        Err(INVALID_UNLOCK)
    );
}
