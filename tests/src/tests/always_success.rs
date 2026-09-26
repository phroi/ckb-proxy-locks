use super::*;

#[test]
fn unlocks() {
    let mut env = Env::new();
    let lock = env.contract("always-success", Bytes::new());
    let filler = env.always_success(&[]);
    let cell = env.create((lock, None, Bytes::new()));
    assert_eq!(
        env.verify(&[cell], vec![(filler, None, Bytes::new())]),
        Ok(())
    );
}
