// Include your tests here
// See https://github.com/xxuejie/ckb-native-build-sample/blob/main/tests/src/tests.rs for examples

mod always_success;
mod easy_to_discover_type;
mod input_type_owner;
mod lock_owner;
mod output_type_owner;
mod single_use_lock;
mod time_lock;
mod type_burn_lock;

use crate::Loader;
use ckb_testtool::{
    builtin::ALWAYS_SUCCESS,
    ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*},
    context::Context,
};

const MAX_CYCLES: u64 = 10_000_000;

/// A cell to create: lock, optional type, and data.
pub type Cell = (Script, Option<Script>, Bytes);

/// Test context with an always-success script for filler locks and owner types.
pub struct Env {
    context: Context,
    always_success: OutPoint,
    deps: Vec<CellDep>,
}

impl Env {
    pub fn new() -> Self {
        let mut context = Context::default();
        let always_success = context.deploy_cell(ALWAYS_SUCCESS.clone());
        let deps = vec![CellDep::new_builder()
            .out_point(always_success.clone())
            .build()];
        Env {
            context,
            always_success,
            deps,
        }
    }

    /// Always-success script; different args give different script hashes.
    pub fn always_success(&mut self, args: &[u8]) -> Script {
        let args = Bytes::copy_from_slice(args);
        self.context
            .build_script(&self.always_success, args)
            .unwrap()
    }

    /// Deploys a contract binary from this repo and builds its script.
    pub fn contract(&mut self, name: &str, args: Bytes) -> Script {
        let out_point = self
            .context
            .deploy_cell(Loader::default().load_binary(name));
        self.deps
            .push(CellDep::new_builder().out_point(out_point.clone()).build());
        self.context.build_script(&out_point, args).unwrap()
    }

    pub fn create(&mut self, (lock, type_, data): Cell) -> OutPoint {
        let output = CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock)
            .type_(type_.pack())
            .build();
        self.context.create_cell(output, data)
    }

    pub fn add_dep(&mut self, out_point: OutPoint) {
        self.deps
            .push(CellDep::new_builder().out_point(out_point).build());
    }

    /// Verifies a transaction spending `inputs` and creating `outputs`.
    /// On failure returns the script's exit code; panics on any other kind of failure.
    pub fn verify(&mut self, inputs: &[OutPoint], outputs: Vec<Cell>) -> Result<(), i8> {
        let inputs = inputs
            .iter()
            .map(|o| CellInput::new_builder().previous_output(o.clone()).build());
        let (outputs, data): (Vec<_>, Vec<_>) = outputs
            .into_iter()
            .map(|(lock, type_, data)| {
                let output = CellOutput::new_builder()
                    .capacity(500u64.pack())
                    .lock(lock)
                    .type_(type_.pack())
                    .build();
                (output, data.pack())
            })
            .unzip();
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(data)
            .cell_deps(self.deps.clone())
            .build();
        match self.context.verify_tx(&tx, MAX_CYCLES) {
            Ok(_) => Ok(()),
            Err(err) => {
                // ckb-script exposes the exit code only in its message: "see the error code {code} in ..."
                let msg = err.to_string();
                let code = msg
                    .split("error code ")
                    .nth(1)
                    .and_then(|rest| rest.split(' ').next())
                    .and_then(|code| code.parse().ok());
                Err(code.unwrap_or_else(|| panic!("not a script exit code: {msg}")))
            }
        }
    }
}

/// Script args holding a 32-byte hash.
pub fn hash_args(script: &Script) -> Bytes {
    script.calc_script_hash().as_bytes()
}
