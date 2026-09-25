use crate::Loader;
use ckb_std::since::Since;
use ckb_testtool::{
    builtin::ALWAYS_SUCCESS,
    ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*},
    context::Context,
};

const MAX_CYCLES: u64 = 10_000_000;

// Include your tests here
// See https://github.com/xxuejie/ckb-native-build-sample/blob/main/tests/src/tests.rs for more examples

#[test]
fn test_time_lock() {
    let mut context = Context::default();

    // * bin => out_point => Script, cell_dep
    let required_lock_script_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    let required_lock_script = context
        .build_script(&required_lock_script_out_point.clone(), Default::default())
        .expect("script");
    let required_lock_script_cell_dep = CellDep::new_builder()
        .out_point(required_lock_script_out_point.clone())
        .build();

    let required_script_hash_bytes32 = required_lock_script.calc_script_hash();
    let locked_until_abs_blk_num_4 = Since::from_block_number(4, true).expect("error");
    // convert locked_until_blk_num to bytes
    let locked_until_bytes: [u8; 8] = locked_until_abs_blk_num_4.as_u64().to_le_bytes();

    // concat lock_until and required_lock_script_hash
    let mut time_lock_script_args_vec: Vec<u8> = Vec::new();
    time_lock_script_args_vec.extend_from_slice(&required_script_hash_bytes32.as_bytes());
    time_lock_script_args_vec.extend_from_slice(&locked_until_bytes);
    let time_lock_script_args: Bytes = Bytes::from(time_lock_script_args_vec);
    let time_lock_bin: Bytes = Loader::default().load_binary("time-lock");
    let time_lock_bin_out_point = context.deploy_cell(time_lock_bin);
    let time_lock_script = context
        .build_script(&time_lock_bin_out_point.clone(), time_lock_script_args)
        .expect("script");
    let time_lock_cell_dep = CellDep::new_builder()
        .out_point(time_lock_bin_out_point.clone())
        .build();

    let cell_deps: Vec<CellDep> = vec![required_lock_script_cell_dep, time_lock_cell_dep];

    // prepare cells
    let cannot_unlock_abs_blk_num_2 = Since::from_block_number(2, true).expect("error");
    let can_unlock_abs_blk_num_233 = Since::from_block_number(233, true).expect("error");
    let out_point_0 = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(time_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let cell_input_0 = CellInput::new_builder()
        .previous_output(out_point_0.clone())
        .since(can_unlock_abs_blk_num_233.as_u64().pack())
        .build();
    let cell_input_1 = CellInput::new_builder()
        .previous_output(out_point_0.clone())
        .since(cannot_unlock_abs_blk_num_2.as_u64().pack())
        .build();
    let out_point_1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(required_lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let cell_input_2 = CellInput::new_builder()
        .previous_output(out_point_1.clone())
        .since(cannot_unlock_abs_blk_num_2.as_u64().pack())
        .build();

    let outputs = vec![
        CellOutput::new_builder().capacity(500u64.pack()).build(),
        CellOutput::new_builder().capacity(500u64.pack()).build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    let cell_inputs_missing_required_lock: Vec<CellInput> = vec![cell_input_0.clone()];
    let cell_inputs_cannot_unlock_yet: Vec<CellInput> =
        vec![cell_input_1.clone(), cell_input_2.clone()];
    let cell_inputs_ok: Vec<CellInput> = vec![cell_input_0.clone(), cell_input_2.clone()];

    // build transaction
    let tx_missing_required_lock = TransactionBuilder::default()
        .cell_deps(cell_deps.clone())
        .inputs(cell_inputs_missing_required_lock)
        .outputs(outputs.clone())
        .outputs_data(outputs_data.clone().pack())
        .build();
    let tx_missing_required_lock = tx_missing_required_lock.as_advanced_builder().build();
    context
        .verify_tx(&tx_missing_required_lock, MAX_CYCLES)
        .expect_err("cannot unlock without required lock script");

    let tx_cannot_unlock_yet = TransactionBuilder::default()
        .cell_deps(cell_deps.clone())
        .inputs(cell_inputs_cannot_unlock_yet)
        .outputs(outputs.clone())
        .outputs_data(outputs_data.clone().pack())
        .build();
    let tx_cannot_unlock_yet = tx_cannot_unlock_yet.as_advanced_builder().build();
    context
        .verify_tx(&tx_cannot_unlock_yet, MAX_CYCLES)
        .expect_err("cannot unlock yet");

    let tx_ok = TransactionBuilder::default()
        .cell_deps(cell_deps)
        .inputs(cell_inputs_ok)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();

    let tx_ok = tx_ok.as_advanced_builder().build();
    let cycles = context
        .verify_tx(&tx_ok, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_time_lock_rejects_incomparable_since() {
    let mut context = Context::default();

    let required_lock_script_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let required_lock_script = context
        .build_script(&required_lock_script_out_point, Default::default())
        .expect("script");
    let time_lock_out_point = context.deploy_cell(Loader::default().load_binary("time-lock"));
    let cell_deps = vec![
        CellDep::new_builder()
            .out_point(required_lock_script_out_point)
            .build(),
        CellDep::new_builder()
            .out_point(time_lock_out_point.clone())
            .build(),
    ];

    // Locked until an absolute timestamp, as in the README
    let locked_until = Since::from_timestamp(1_000, true).expect("since");
    let mut args = required_lock_script.calc_script_hash().as_bytes().to_vec();
    args.extend_from_slice(&locked_until.as_u64().to_le_bytes());
    let time_lock_script = context
        .build_script(&time_lock_out_point, Bytes::from(args))
        .expect("script");

    let verify = |context: &mut Context, since: u64| {
        let time_locked = context.create_cell(
            CellOutput::new_builder()
                .capacity(500u64.pack())
                .lock(time_lock_script.clone())
                .build(),
            Bytes::new(),
        );
        let required = context.create_cell(
            CellOutput::new_builder()
                .capacity(500u64.pack())
                .lock(required_lock_script.clone())
                .build(),
            Bytes::new(),
        );
        let tx = TransactionBuilder::default()
            .cell_deps(cell_deps.clone())
            .input(
                CellInput::new_builder()
                    .previous_output(time_locked)
                    .since(since.pack())
                    .build(),
            )
            .input(CellInput::new_builder().previous_output(required).build())
            .output(CellOutput::new_builder().capacity(1000u64.pack()).build())
            .output_data(Bytes::new().pack())
            .build();
        context.verify_tx(&tx, MAX_CYCLES)
    };

    // A default since of zero reads as absolute block number 0, a different metric
    verify(&mut context, 0).expect_err("since zero must not unlock");
    let block_number = Since::from_block_number(233, true).expect("since");
    verify(&mut context, block_number.as_u64()).expect_err("block number must not unlock");
    let relative = Since::from_timestamp(2_000, false).expect("since");
    verify(&mut context, relative.as_u64()).expect_err("relative timestamp must not unlock");
    let later = Since::from_timestamp(2_000, true).expect("since");
    verify(&mut context, later.as_u64()).expect("later absolute timestamp unlocks");
}
