use super::*;
use ckb_testtool::ckb_hash::blake2b_256;

const INSUFFICIENT_ARGS: i8 = 5;
const DATA_HASH_NOT_MATCH: i8 = 6;

const DATA: &[u8] = b"discoverable";

/// Creates outputs typed with easy-to-discover-type, holding `data`.
fn create(args: Bytes, data: &[&[u8]]) -> Result<(), i8> {
    let mut env = Env::new();
    let type_ = Some(env.contract("easy-to-discover-type", args));
    let filler = env.always_success(&[]);
    let input = env.create((filler.clone(), None, Bytes::new()));
    let outputs = data
        .iter()
        .map(|d| (filler.clone(), type_.clone(), Bytes::copy_from_slice(d)))
        .collect();
    env.verify(&[input], outputs)
}

fn data_hash_args() -> Bytes {
    Bytes::copy_from_slice(&blake2b_256(DATA))
}

#[test]
fn creates_matching_data() {
    assert_eq!(create(data_hash_args(), &[DATA]), Ok(()));
}

#[test]
fn creates_several_matching_cells() {
    assert_eq!(create(data_hash_args(), &[DATA, DATA]), Ok(()));
}

#[test]
fn fails_on_other_data() {
    assert_eq!(
        create(data_hash_args(), &[b"other"]),
        Err(DATA_HASH_NOT_MATCH)
    );
}

#[test]
fn fails_when_any_output_differs() {
    assert_eq!(
        create(data_hash_args(), &[DATA, b"other"]),
        Err(DATA_HASH_NOT_MATCH)
    );
}

#[test]
fn fails_on_short_args() {
    assert_eq!(
        create(data_hash_args().slice(0..31), &[DATA]),
        Err(INSUFFICIENT_ARGS)
    );
}

#[test]
fn consumes_without_output() {
    let mut env = Env::new();
    let type_ = Some(env.contract("easy-to-discover-type", data_hash_args()));
    let filler = env.always_success(&[]);
    let cell = env.create((filler.clone(), type_, Bytes::copy_from_slice(DATA)));
    assert_eq!(
        env.verify(&[cell], vec![(filler, None, Bytes::new())]),
        Ok(())
    );
}

// CKB defines the data hash of empty data as 32 zero bytes, not blake2b of the empty string.
#[test]
fn empty_data_hashes_to_zero() {
    assert_eq!(create(Bytes::from(vec![0u8; 32]), &[b""]), Ok(()));
    let empty_blake2b = Bytes::copy_from_slice(&blake2b_256(b""));
    assert_eq!(create(empty_blake2b, &[b""]), Err(DATA_HASH_NOT_MATCH));
}

#[test]
fn fails_on_transfer_with_other_data() {
    let mut env = Env::new();
    let type_ = Some(env.contract("easy-to-discover-type", data_hash_args()));
    let filler = env.always_success(&[]);
    let cell = env.create((filler.clone(), type_.clone(), Bytes::copy_from_slice(DATA)));
    let outputs = vec![(filler, type_, Bytes::from_static(b"other"))];
    assert_eq!(env.verify(&[cell], outputs), Err(DATA_HASH_NOT_MATCH));
}
