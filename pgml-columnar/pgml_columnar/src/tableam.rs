#![allow(unused_variables)]

use pgrx::pg_sys::{
    BlockNumber, BufferAccessStrategy, BulkInsertStateData, CommandId, Datum, FunctionCallInfo,
    IndexFetchTableData, ItemPointer, LockTupleMode, LockWaitPolicy, MultiXactId, NodeTag,
    ParallelTableScanDesc, ParallelTableScanDescData, RelFileNode, Relation, ScanDirection,
    ScanKey, Snapshot, TM_FailureData, TM_IndexDeleteOp, TM_Result, TableAmRoutine, TableScanDesc,
    TableScanDescData, TransactionId, TupleTableSlot, TupleTableSlotOps, VacuumParams, IndexBuildCallback, IndexInfo,
    ValidateIndexState, ForkNumber, SampleScanState, InvalidTransactionId, HeapTupleData, MinimalTupleData, HeapScanDesc, palloc, pfree, TTS_FLAG_EMPTY,
};
use pgrx::{prelude::*, Internal};
use std::alloc;

static TABLE_AM_ROUTINE: TableAmRoutine = TableAmRoutine {
    type_: NodeTag::T_TableAmRoutine,
    slot_callbacks: Some(slots_callback),
    scan_begin: Some(scan_begin),
    scan_end: Some(scan_end),
    scan_rescan: Some(scan_rescan),
    scan_getnextslot: Some(scan_getnextslot),
    #[cfg(feature = "pg14")]
    scan_set_tidrange: Some(scan_set_tidrange),
    #[cfg(feature = "pg14")]
    scan_getnextslot_tidrange: Some(scan_getnextslot_tidrange),
    parallelscan_estimate: Some(parallelscan_estimate),
    parallelscan_initialize: Some(parallelscan_initialize),
    parallelscan_reinitialize: Some(parallelscan_reinitialize),
    index_fetch_begin: Some(index_fetch_begin),
    index_fetch_reset: Some(index_fetch_reset),
    index_fetch_end: Some(index_fetch_end),
    index_fetch_tuple: Some(index_fetch_tuple),
    tuple_fetch_row_version: Some(tuple_fetch_row_version),
    tuple_tid_valid: Some(tuple_tid_valid),
    tuple_get_latest_tid: Some(tuple_get_latest_tid),
    tuple_satisfies_snapshot: Some(tuple_satisfies_snapshot),
    #[cfg(feature = "pg14")]
    index_delete_tuples: Some(index_delete_tuples),
    tuple_insert: Some(tuple_insert),
    tuple_insert_speculative: Some(tuple_insert_speculative),
    tuple_complete_speculative: Some(tuple_complete_speculative),
    multi_insert: Some(multi_insert),
    tuple_delete: Some(tuple_delete),
    tuple_update: Some(tuple_update),
    tuple_lock: Some(tuple_lock),
    finish_bulk_insert: Some(finish_bulk_insert),
    relation_set_new_filenode: Some(relation_set_new_filenode),
    relation_nontransactional_truncate: Some(relation_nontransactional_truncate),
    relation_copy_data: Some(relation_copy_data),
    relation_copy_for_cluster: Some(relation_copy_for_cluster),
    relation_vacuum: Some(relation_vacuum),
    scan_analyze_next_block: Some(scan_analyze_next_block),
    scan_analyze_next_tuple: Some(scan_analyze_next_tuple),
    index_build_range_scan: Some(index_build_range_scan),
    index_validate_scan: Some(index_validate_scan),
    relation_size: Some(relation_size),
    relation_needs_toast_table: Some(relation_needs_toast_table),
    relation_toast_am: None,
    relation_fetch_toast_slice: None,
    relation_estimate_size: Some(relation_estimate_size),
    scan_bitmap_next_block: None,
    scan_bitmap_next_tuple: None,
    scan_sample_next_block: Some(scan_sample_next_block),
    scan_sample_next_tuple: Some(scan_sample_next_tuple),
    #[cfg(feature = "pg13")]
    compute_xid_horizon_for_tuples: None,
};

static TUPLE_TABLE_SLOT_OPS: TupleTableSlotOps = TupleTableSlotOps {
    base_slot_size: 8,

    // pub init: Option<unsafe extern "C" fn(_: *mut TupleTableSlot)>,
    // pub release: Option<unsafe extern "C" fn(_: *mut TupleTableSlot)>,
    // pub clear: Option<unsafe extern "C" fn(_: *mut TupleTableSlot)>,
    // pub getsomeattrs: Option<unsafe extern "C" fn(_: *mut TupleTableSlot, _: i32)>,
    // pub getsysattr: Option<unsafe extern "C" fn(_: *mut TupleTableSlot, _: i32, _: *mut bool) -> Datum>,
    // pub materialize: Option<unsafe extern "C" fn(_: *mut TupleTableSlot)>,
    // pub copyslot: Option<unsafe extern "C" fn(_: *mut TupleTableSlot, _: *mut TupleTableSlot)>,
    // pub get_heap_tuple: Option<unsafe extern "C" fn(_: *mut TupleTableSlot) -> *mut HeapTupleData>,
    // pub get_minimal_tuple: Option<unsafe extern "C" fn(_: *mut TupleTableSlot) -> *mut MinimalTupleData>,
    // pub copy_heap_tuple: Option<unsafe extern "C" fn(_: *mut TupleTableSlot) -> *mut HeapTupleData>,
    // pub copy_minimal_tuple: Option<unsafe extern "C" fn(_: *mut TupleTableSlot) -> *mut MinimalTupleData>,


    init:  Some(table_ops_init),
    release:  Some(table_ops_release),
    clear:  Some(table_ops_clear),
    getsomeattrs: Some(table_ops_getsomeattrs),
    getsysattr: Some(table_ops_getsysattr),
    materialize:  Some(table_ops_materialize),
    copyslot: Some(table_ops_copyslot),
    get_heap_tuple:  Some(table_ops_get_heap_tuple),
    get_minimal_tuple:  Some(table_ops_get_minimal_tuple),
    copy_heap_tuple:  Some(table_ops_copy_heap_tuple),
    copy_minimal_tuple: Some(table_ops_copy_minimal_tuple),
};

macro_rules! debug_method {
    ($name:expr) => ({
        println!("{}", $name); 
        todo!()
    })
}


unsafe extern "C" fn table_ops_init(slot: *mut TupleTableSlot) {
    println!("table_ops_init");

}

unsafe extern "C" fn table_ops_release(slot: *mut TupleTableSlot) {
    println!("table_ops_release");

}

unsafe extern "C" fn table_ops_clear(slot: *mut TupleTableSlot) {
    println!("table_ops_clear");

}

unsafe extern "C" fn table_ops_getsomeattrs(slot: *mut TupleTableSlot, natts: i32) {
    println!("table_ops_getsomeattrs");
    println!("num attrs: {}", natts);
    println!("size of datum: {}", std::mem::size_of((*slot).tts_values));
    let tuple_desc = (*slot).tts_tupleDescriptor;
    println!("tuple desc: {:?}", tuple_desc);
    (*slot).tts_values =  0 as *mut Datum;
    (*slot).tts_isnull = 1 as *mut bool;
    println!("Done with table_ops_getsomeattrs");

}

unsafe extern "C" fn table_ops_getsysattr(slot: *mut TupleTableSlot, attnum: i32, isnull: *mut bool) -> Datum {
    debug_method!("table_ops_getsysattr")
}

unsafe extern "C" fn table_ops_materialize(slot: *mut TupleTableSlot) {
    println!("table_ops_materialize");

}

unsafe extern "C" fn table_ops_copyslot(dest: *mut TupleTableSlot, src: *mut TupleTableSlot) {
    println!("table_ops_copyslot");

}

unsafe extern "C" fn table_ops_get_heap_tuple(slot: *mut TupleTableSlot) -> *mut HeapTupleData {
    debug_method!("table_ops_get_heap_tuple")
}

unsafe extern "C" fn table_ops_get_minimal_tuple(slot: *mut TupleTableSlot) -> *mut MinimalTupleData {
    debug_method!("table_ops_get_minimal_tuple")
}

unsafe extern "C" fn table_ops_copy_heap_tuple(slot: *mut TupleTableSlot) -> *mut HeapTupleData {
    debug_method!("table_ops_copy_heap_tuple")
}

unsafe extern "C" fn table_ops_copy_minimal_tuple(slot: *mut TupleTableSlot) -> *mut MinimalTupleData {
    debug_method!("table_ops_copy_minimal_tuple")
}

unsafe extern "C" fn slots_callback(
    relation: Relation
) -> *const TupleTableSlotOps {
    println!("slots_callback");

    &TUPLE_TABLE_SLOT_OPS
}

#[repr(C)]
pub struct PgmlColumnarScan {
    rs_base: TableScanDescData,
    rows: usize,
}

unsafe extern "C" fn scan_begin(
    relation: Relation,
    snapshot: Snapshot,
    nkeys: i32,
    key: ScanKey,
    parallel_scan: ParallelTableScanDesc,
    flags: u32,
) -> *mut TableScanDescData {
    println!("scan_begin, nkeys: {}, flags: {}, key: {:?}, relation: {:?}\n, snapshot: {:?}", nkeys, flags, key, *relation, *snapshot);
    let scan = unsafe {
        let ptr = palloc(std::mem::size_of::<PgmlColumnarScan>()) as *mut PgmlColumnarScan;
        ptr
    };

    (*scan).rs_base.rs_rd = relation;
    (*scan).rs_base.rs_snapshot = snapshot;
    (*scan).rs_base.rs_nkeys = nkeys;
    (*scan).rs_base.rs_key = key;
    (*scan).rs_base.rs_flags = flags;
    (*scan).rs_base.rs_parallel = parallel_scan;
    (*scan).rows = 25;

    scan as *mut TableScanDescData
}

unsafe extern "C" fn scan_end(scan: *mut TableScanDescData) {
    let scan = scan as *mut PgmlColumnarScan;
    println!("rows: {}", (*scan).rows);

    unsafe {
        pfree(scan as *mut std::ffi::c_void);
    }
    println!("scan_end");
}

unsafe extern "C" fn scan_rescan(
    scan: *mut TableScanDescData,
    key: ScanKey,
    set_params: bool,
    allow_strat: bool,
    allow_sync: bool,
    allow_pagemode: bool,
) {
    debug_method!("scan_rescan");
}

unsafe extern "C" fn scan_getnextslot(
    sscan: TableScanDesc,
    direction: ScanDirection,
    slot: *mut TupleTableSlot,
) -> bool {
    let scan = sscan as *mut PgmlColumnarScan;
    let rows = (*scan).rows;

    let datum = Datum::from(12345);

    // std::mem::replace(&mut *(*slot).tts_values, datum);

    println!("scan_getnextslot, rows: {}, slot: {:?}", (*scan).rows, *slot);
    // let datum = (*(*slot).tts_values);
    (*slot).tts_nvalid = 0;
    (*slot).tts_flags &= !TTS_FLAG_EMPTY as u16;

    // println!("value: {}", datum.value());

    println!("scan_getnextslot, rows: {}, slot: {:?}", (*scan).rows, *slot);

    if rows < 1 {
        println!("Done");
        false
    } else {
        println!("Not done");
        (*scan).rows -= 1;
        true
    }
}

unsafe extern "C" fn scan_set_tidrange(
    scan: TableScanDesc,
    mintid: ItemPointer,
    maxtid: ItemPointer,
) {
    debug_method!("scan_set_tidrange");
}

unsafe extern "C" fn scan_getnextslot_tidrange(
    scan: TableScanDesc,
    direction: ScanDirection,
    slot: *mut TupleTableSlot,
) -> bool {
    debug_method!("scan_getnextslot_tidrange");
}

unsafe extern "C" fn parallelscan_estimate(
    relation: Relation
) -> usize {
    debug_method!("parallelscan_estimate");
}

unsafe extern "C" fn parallelscan_initialize(
    relation: Relation,
    scan: *mut ParallelTableScanDescData,
) -> usize {
    debug_method!("parallelscan_initialize");
}

unsafe extern "C" fn parallelscan_reinitialize(
    relation: Relation,
    scan: *mut ParallelTableScanDescData,
) {
    debug_method!("parallelscan_reinitialize");
}

unsafe extern "C" fn index_delete_tuples(
    rel: Relation,
    delstate: *mut TM_IndexDeleteOp,
) -> TransactionId {
    debug_method!("index_delete_tuples");
}

unsafe extern "C" fn index_fetch_begin(
    relation: Relation
) -> *mut IndexFetchTableData {
    debug_method!("index_fetch_begin");
}

unsafe extern "C" fn index_fetch_reset(
    data: *mut IndexFetchTableData
) {
    debug_method!("index_fetch_reset");
}

unsafe extern "C" fn index_fetch_end(
    data: *mut IndexFetchTableData
) {
    debug_method!("index_fetch_end");
}

unsafe extern "C" fn index_fetch_tuple(
    scan: *mut IndexFetchTableData,
    tid: ItemPointer,
    snapshot: Snapshot,
    slot: *mut TupleTableSlot,
    call_again: *mut bool,
    all_dead: *mut bool,
) -> bool {
    debug_method!("index_fetch_tuple");
}

unsafe extern "C" fn tuple_fetch_row_version(
    relation: Relation,
    tid: ItemPointer,
    snapshot: Snapshot,
    slot: *mut TupleTableSlot,
) -> bool {
    debug_method!("tuple_fetch_row_version");
}

unsafe extern "C" fn tuple_tid_valid(
    scan: TableScanDesc,
    tid: ItemPointer
) -> bool {
    debug_method!("tuple_tid_valid");
}

unsafe extern "C" fn tuple_get_latest_tid(
    scan: TableScanDesc,
    tid: ItemPointer
) {
    debug_method!("tuple_get_latest_tid");
}

unsafe extern "C" fn tuple_satisfies_snapshot(
    relation: Relation,
    slot: *mut TupleTableSlot,
    snapshot: Snapshot,
) -> bool {
    debug_method!("tuple_satisfies_snapshot");
}

unsafe extern "C" fn tuple_insert(
    rel: Relation,
    slot: *mut TupleTableSlot,
    cid: CommandId,
    options: i32,
    bistate: *mut BulkInsertStateData,
) {
    debug_method!("tuple_insert");
}

unsafe extern "C" fn tuple_insert_speculative(
    rel: Relation,
    slot: *mut TupleTableSlot,
    cid: CommandId,
    options: i32,
    bistate: *mut BulkInsertStateData,
    spec_token: u32,
) {
    debug_method!("tuple_insert_speculative");
}

unsafe extern "C" fn tuple_complete_speculative(
    rel: Relation,
    slot: *mut TupleTableSlot,
    spec_token: u32,
    succeeded: bool,
) {
    debug_method!("tuple_complete_speculative");
}

unsafe extern "C" fn multi_insert(
    rel: Relation,
    slots: *mut *mut TupleTableSlot,
    nslots: i32,
    cid: CommandId,
    options: i32,
    bistate: *mut BulkInsertStateData,
) {
    debug_method!("multi_insert");
}

unsafe extern "C" fn tuple_delete(
    rel: Relation,
    tid: ItemPointer,
    cid: CommandId,
    snapshot: Snapshot,
    crosscheck: Snapshot,
    wait: bool,
    tmfd: *mut TM_FailureData,
    changing_part: bool,
) -> TM_Result {
    debug_method!("tuple_delete");
}

unsafe extern "C" fn tuple_update(
    rel: Relation,
    otid: ItemPointer,
    slot: *mut TupleTableSlot,
    cid: CommandId,
    snapshot: Snapshot,
    crosscheck: Snapshot,
    wait: bool,
    tmfd: *mut TM_FailureData,
    lockmode: *mut LockTupleMode,
    update_indexes: *mut bool,
) -> TM_Result {
    debug_method!("tuple_update");
}

unsafe extern "C" fn tuple_lock(
    rel: Relation,
    tid: ItemPointer,
    snapshot: Snapshot,
    slot: *mut TupleTableSlot,
    cid: CommandId,
    mode: LockTupleMode,
    wait_policy: LockWaitPolicy,
    flags: u8,
    tmfd: *mut TM_FailureData,
) -> TM_Result {
    debug_method!("tuple_lock");
}

unsafe extern "C" fn finish_bulk_insert(
    rel: Relation,
    options: i32
) {
    debug_method!("finish_bulk_insert");
}

unsafe extern "C" fn relation_set_new_filenode(
    rel: Relation,
    newrnode: *const RelFileNode,
    persistence: i8,
    freeze_xid: *mut TransactionId,
    minmulti: *mut MultiXactId,
) {
    *freeze_xid = InvalidTransactionId;
    *minmulti = 0;
    println!("relation_set_new_filenode");
}

unsafe extern "C" fn relation_nontransactional_truncate(
    rel: Relation
) {
    debug_method!("relation_nontransactional_truncate");
}

unsafe extern "C" fn relation_copy_data(
    rel: Relation,
    newrnode: *const RelFileNode
) {
    debug_method!("relation_copy_data");
}

unsafe extern "C" fn relation_copy_for_cluster(
    NewTable: Relation,
    OldTable: Relation,
    OldIndex: Relation,
    use_sort: bool,
    OldestXmin: TransactionId,
    xid_cutoff: *mut TransactionId,
    multi_cutoff: *mut MultiXactId,
    num_tuples: *mut f64,
    tups_vacuumed: *mut f64,
    tups_recently_dead: *mut f64,
) {
    debug_method!("relation_copy_for_cluster");
}

unsafe extern "C" fn relation_vacuum(
    rel: Relation,
    params: *mut VacuumParams,
    bstrategy: BufferAccessStrategy,
) {
    debug_method!("relation_vacuum");
}

unsafe extern "C" fn scan_analyze_next_block(
    scan: TableScanDesc,
    blockno: BlockNumber,
    bstrategy: BufferAccessStrategy,
) -> bool {
    debug_method!("scan_analyze_next_block");
}

unsafe extern "C" fn scan_analyze_next_tuple(
    scan: TableScanDesc,
    OldestXmin: TransactionId,
    liverows: *mut f64,
    deadrows: *mut f64,
    slot: *mut TupleTableSlot,
) -> bool {
    debug_method!("scan_analyze_next_tuple");
}

unsafe extern "C" fn index_build_range_scan(
    table_rel: Relation,
    index_rel: Relation,
    index_info: *mut IndexInfo,
    allow_sync: bool,
    anyvisible: bool,
    progress: bool,
    start_blockno: BlockNumber,
    numblocks: BlockNumber,
    callback: IndexBuildCallback,
    callback_state: *mut std::ffi::c_void,
    scan: TableScanDesc,
) -> f64 {
    debug_method!("index_build_range_scan");
}

unsafe extern "C" fn index_validate_scan(
    table_rel: Relation,
    index_rel: Relation,
    index_info: *mut IndexInfo,
    snapshot: Snapshot,
    state: *mut ValidateIndexState,
) {
    debug_method!("index_validate_scan");
}

unsafe extern "C" fn relation_size(
    rel: Relation,
    fork_number: ForkNumber,
) -> u64 {
    debug_method!("relation_size");
}

unsafe extern "C" fn relation_needs_toast_table(
    rel: Relation,
) -> bool {
    println!("relation_needs_toast_table");
    false
}

unsafe extern "C" fn relation_estimate_size(
    rel: Relation,
    attr_widths: *mut i32,
    pages: *mut BlockNumber,
    tuples: *mut f64,
    allvisfrac: *mut f64,
) {
    println!("relation_estimate_size");    
}

unsafe extern "C" fn scan_sample_next_block(
    scan: TableScanDesc,
    scanstate: *mut SampleScanState,
) -> bool {
    debug_method!("scan_sample_next_block");
}

unsafe extern "C" fn scan_sample_next_tuple(
    scan: TableScanDesc,
    scanstate: *mut SampleScanState,
    slot: *mut TupleTableSlot,
) -> bool {
    debug_method!("scan_sample_next_tuple");
}

#[no_mangle]
pub unsafe extern "C" fn pgml_columnar_table_am(fcinfo: FunctionCallInfo) -> Datum {
    Datum::from(&TABLE_AM_ROUTINE as *const TableAmRoutine)
}

#[no_mangle]
#[doc(hidden)]
pub extern "C" fn pg_finfo_pgml_columnar_table_am() -> &'static ::pgrx::pg_sys::Pg_finfo_record {
    const V1_API: ::pgrx::pg_sys::Pg_finfo_record =
        ::pgrx::pg_sys::Pg_finfo_record { api_version: 1 };
    &V1_API
}
