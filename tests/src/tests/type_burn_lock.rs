use super::*;

const ENCODING: i8 = 4;
const NOT_BURNT: i8 = 6;

/// Spends a type-burn-lock cell next to an extra input typed `input_type`,
/// and creates an output typed `output_type`.
fn run(args: impl Fn(&Script) -> Bytes, input_type: &[u8], output_type: &[u8]) -> Result<(), i8> {
    let mut env = Env::new();
    let burnt = env.always_success(b"burnt");
    let lock = env.contract("type-burn-lock", args(&burnt));
    let filler = env.always_success(&[]);
    let locked = env.create((lock, None, Bytes::new()));
    let input_type = Some(env.always_success(input_type));
    let extra = env.create((filler.clone(), input_type, Bytes::new()));
    let output_type = Some(env.always_success(output_type));
    env.verify(&[locked, extra], vec![(filler, output_type, Bytes::new())])
}

/// Spends a type-burn-lock cell while a cell typed with the burnt type is only a cell dep.
fn run_with_typed_dep() -> Result<(), i8> {
    let mut env = Env::new();
    let burnt = env.always_success(b"burnt");
    let lock = env.contract("type-burn-lock", hash_args(&burnt));
    let filler = env.always_success(&[]);
    let locked = env.create((lock, None, Bytes::new()));
    let typed = env.create((filler.clone(), Some(burnt), Bytes::new()));
    env.add_dep(typed);
    env.verify(&[locked], vec![(filler, None, Bytes::new())])
}

#[test]
fn unlocks_when_type_is_burnt() {
    assert_eq!(run(hash_args, b"burnt", b"other"), Ok(()));
}

#[test]
fn fails_when_type_is_recreated() {
    assert_eq!(run(hash_args, b"burnt", b"burnt"), Err(NOT_BURNT));
}

#[test]
fn fails_when_type_is_absent() {
    assert_eq!(run(hash_args, b"other", b"other"), Err(NOT_BURNT));
}

#[test]
fn fails_when_type_is_only_created() {
    assert_eq!(run(hash_args, b"other", b"burnt"), Err(NOT_BURNT));
}

#[test]
fn fails_when_type_is_only_in_dep() {
    assert_eq!(run_with_typed_dep(), Err(NOT_BURNT));
}

#[test]
fn fails_on_short_args() {
    assert_eq!(
        run(|s| hash_args(s).slice(0..31), b"burnt", b"other"),
        Err(ENCODING)
    );
}

#[test]
fn fails_with_burnt_script_only_as_input_lock() {
    let mut env = Env::new();
    let burnt = env.always_success(b"burnt");
    let lock = env.contract("type-burn-lock", hash_args(&burnt));
    let filler = env.always_success(&[]);
    let locked = env.create((lock, None, Bytes::new()));
    let extra = env.create((burnt, None, Bytes::new()));
    assert_eq!(
        env.verify(&[locked, extra], vec![(filler, None, Bytes::new())]),
        Err(NOT_BURNT)
    );
}

/// Spends a type-burn-lock cell that itself carries the burnt type, keeping the type on the output when `keep_type`.
fn run_self_typed(keep_type: bool) -> Result<(), i8> {
    let mut env = Env::new();
    let burnt = env.always_success(b"burnt");
    let lock = env.contract("type-burn-lock", hash_args(&burnt));
    let filler = env.always_success(&[]);
    let locked = env.create((lock, Some(burnt.clone()), Bytes::new()));
    let output_type = if keep_type { Some(burnt) } else { None };
    env.verify(&[locked], vec![(filler, output_type, Bytes::new())])
}

#[test]
fn unlocks_when_locked_cell_type_is_burnt() {
    assert_eq!(run_self_typed(false), Ok(()));
}

#[test]
fn fails_when_locked_cell_type_is_kept() {
    assert_eq!(run_self_typed(true), Err(NOT_BURNT));
}
