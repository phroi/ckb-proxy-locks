use super::*;

const ENCODING: i8 = 4;
const OUTPOINT_NOT_FOUND: i8 = 7;

/// Spends a single-use-lock cell whose args come from `args` of the seal OutPoint.
/// The seal is spent when `spend_seal`, otherwise another cell is; `seal_as_dep` adds the seal as a cell dep.
fn run(args: impl Fn(&OutPoint) -> Bytes, spend_seal: bool, seal_as_dep: bool) -> Result<(), i8> {
    let mut env = Env::new();
    let filler = env.always_success(&[]);
    let seal = env.create((filler.clone(), None, Bytes::new()));
    let other = env.create((filler.clone(), None, Bytes::new()));
    let lock = env.contract("single-use-lock", args(&seal));
    let locked = env.create((lock, None, Bytes::new()));
    if seal_as_dep {
        env.deps
            .push(CellDep::new_builder().out_point(seal.clone()).build());
    }
    let spent = if spend_seal { seal } else { other };
    env.verify(&[locked, spent], vec![(filler, None, Bytes::new())])
}

fn exact(seal: &OutPoint) -> Bytes {
    seal.as_bytes()
}

#[test]
fn unlocks_when_seal_is_spent() {
    assert_eq!(run(exact, true, false), Ok(()));
}

#[test]
fn fails_without_seal() {
    assert_eq!(run(exact, false, false), Err(OUTPOINT_NOT_FOUND));
}

#[test]
fn fails_with_seal_only_as_dep() {
    assert_eq!(run(exact, false, true), Err(OUTPOINT_NOT_FOUND));
}

#[test]
fn fails_with_other_index() {
    let other_index = |seal: &OutPoint| {
        seal.clone()
            .as_builder()
            .index(9u32.pack())
            .build()
            .as_bytes()
    };
    assert_eq!(run(other_index, true, false), Err(OUTPOINT_NOT_FOUND));
}

#[test]
fn fails_on_short_args() {
    assert_eq!(
        run(|seal| exact(seal).slice(0..35), true, false),
        Err(ENCODING)
    );
}

// Only the first 36 bytes are read; trailing bytes still change the script hash.
#[test]
fn ignores_trailing_args() {
    let trailing = |seal: &OutPoint| [exact(seal).as_ref(), &[7]].concat().into();
    assert_eq!(run(trailing, true, false), Ok(()));
}

// "Single use" means one transaction: every cell sealed by the same OutPoint unlocks with it.
#[test]
fn unlocks_every_cell_sealed_by_the_spent_seal() {
    let mut env = Env::new();
    let filler = env.always_success(&[]);
    let seal = env.create((filler.clone(), None, Bytes::new()));
    let lock = env.contract("single-use-lock", exact(&seal));
    let first = env.create((lock.clone(), None, Bytes::new()));
    let second = env.create((lock, None, Bytes::new()));
    assert_eq!(
        env.verify(&[first, second, seal], vec![(filler, None, Bytes::new())]),
        Ok(())
    );
}
