#![allow(unused_variables)]

use pgrx::pg_sys::{
    BlockNumber, BufferAccessStrategy, BulkInsertStateData, CommandId, Datum, FunctionCallInfo,
    IndexFetchTableData, ItemPointer, LockTupleMode, LockWaitPolicy, MultiXactId, NodeTag,
    ParallelTableScanDesc, ParallelTableScanDescData, RelFileNode, Relation, ScanDirection,
    ScanKey, Snapshot, TM_FailureData, TM_IndexDeleteOp, TM_Result, TableAmRoutine, TableScanDesc,
    TableScanDescData, TransactionId, TupleTableSlot, TupleTableSlotOps, VacuumParams,
};
use pgrx::{prelude::*, Internal};

static TABLE_AM_ROUTINE: TableAmRoutine = TableAmRoutine {
    type_: NodeTag::T_TableAmRoutine,
    slot_callbacks: Some(slots_callback),
    scan_begin: Some(scan_begin),
    scan_end: Some(scan_end),
    scan_rescan: Some(scan_rescan),
    scan_getnextslot: Some(heap_getnextslot),
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
    scan_analyze_next_tuple: None,
    index_build_range_scan: None,
    index_validate_scan: None,
    relation_size: None,
    relation_needs_toast_table: None,
    relation_toast_am: None,
    relation_fetch_toast_slice: None,
    relation_estimate_size: None,
    scan_bitmap_next_block: None,
    scan_bitmap_next_tuple: None,
    scan_sample_next_block: None,
    scan_sample_next_tuple: None,
    #[cfg(feature = "pg13")]
    compute_xid_horizon_for_tuples: None,
};

unsafe extern "C" fn slots_callback(_relation: Relation) -> *const TupleTableSlotOps {
    unimplemented!()
}

unsafe extern "C" fn scan_begin(
    relation: Relation,
    snapshot: Snapshot,
    nkeys: i32,
    key: ScanKey,
    parallel_scan: ParallelTableScanDesc,
    flags: u32,
) -> *mut TableScanDescData {
    unimplemented!()
}

unsafe extern "C" fn scan_end(scan: *mut TableScanDescData) {
    unimplemented!()
}

unsafe extern "C" fn scan_rescan(
    scan: *mut TableScanDescData,
    key: ScanKey,
    set_params: bool,
    allow_strat: bool,
    allow_sync: bool,
    allow_pagemode: bool,
) {
    unimplemented!()
}

unsafe extern "C" fn heap_getnextslot(
    sscan: TableScanDesc,
    direction: ScanDirection,
    slot: *mut TupleTableSlot,
) -> bool {
    todo!()
}

unsafe extern "C" fn scan_set_tidrange(
    scan: TableScanDesc,
    mintid: ItemPointer,
    maxtid: ItemPointer,
) {
    todo!()
}

unsafe extern "C" fn scan_getnextslot_tidrange(
    scan: TableScanDesc,
    direction: ScanDirection,
    slot: *mut TupleTableSlot,
) -> bool {
    todo!()
}

unsafe extern "C" fn parallelscan_estimate(relation: Relation) -> usize {
    unimplemented!()
}

unsafe extern "C" fn parallelscan_initialize(
    relation: Relation,
    scan: *mut ParallelTableScanDescData,
) -> usize {
    unimplemented!()
}

unsafe extern "C" fn parallelscan_reinitialize(
    relation: Relation,
    scan: *mut ParallelTableScanDescData,
) {
    unimplemented!()
}

unsafe extern "C" fn index_delete_tuples(
    rel: Relation,
    delstate: *mut TM_IndexDeleteOp,
) -> TransactionId {
    todo!()
}

unsafe extern "C" fn index_fetch_begin(relation: Relation) -> *mut IndexFetchTableData {
    todo!()
}

unsafe extern "C" fn index_fetch_reset(data: *mut IndexFetchTableData) {
    todo!()
}

unsafe extern "C" fn index_fetch_end(data: *mut IndexFetchTableData) {
    todo!()
}

unsafe extern "C" fn index_fetch_tuple(
    scan: *mut IndexFetchTableData,
    tid: ItemPointer,
    snapshot: Snapshot,
    slot: *mut TupleTableSlot,
    call_again: *mut bool,
    all_dead: *mut bool,
) -> bool {
    todo!()
}

unsafe extern "C" fn tuple_fetch_row_version(
    relation: Relation,
    tid: ItemPointer,
    snapshot: Snapshot,
    slot: *mut TupleTableSlot,
) -> bool {
    todo!()
}

unsafe extern "C" fn tuple_tid_valid(scan: TableScanDesc, tid: ItemPointer) -> bool {
    todo!()
}

unsafe extern "C" fn tuple_get_latest_tid(scan: TableScanDesc, tid: ItemPointer) {
    todo!()
}

unsafe extern "C" fn tuple_satisfies_snapshot(
    relation: Relation,
    slot: *mut TupleTableSlot,
    snapshot: Snapshot,
) -> bool {
    todo!()
}

unsafe extern "C" fn tuple_insert(
    rel: Relation,
    slot: *mut TupleTableSlot,
    cid: CommandId,
    options: i32,
    bistate: *mut BulkInsertStateData,
) {
    todo!()
}

unsafe extern "C" fn tuple_insert_speculative(
    rel: Relation,
    slot: *mut TupleTableSlot,
    cid: CommandId,
    options: i32,
    bistate: *mut BulkInsertStateData,
    spec_token: u32,
) {
    todo!()
}

unsafe extern "C" fn tuple_complete_speculative(
    rel: Relation,
    slot: *mut TupleTableSlot,
    spec_token: u32,
    succeeded: bool,
) {
    todo!()
}

unsafe extern "C" fn multi_insert(
    rel: Relation,
    slots: *mut *mut TupleTableSlot,
    nslots: i32,
    cid: CommandId,
    options: i32,
    bistate: *mut BulkInsertStateData,
) {
    todo!()
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
    todo!()
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
    todo!()
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
    todo!()
}

unsafe extern "C" fn finish_bulk_insert(rel: Relation, options: i32) {
    todo!()
}

unsafe extern "C" fn relation_set_new_filenode(
    rel: Relation,
    newrnode: *const RelFileNode,
    persistence: i8,
    freeze_xid: *mut TransactionId,
    minmulti: *mut MultiXactId,
) {
    todo!()
}

unsafe extern "C" fn relation_nontransactional_truncate(rel: Relation) {
    todo!()
}

unsafe extern "C" fn relation_copy_data(rel: Relation, newrnode: *const RelFileNode) {
    todo!()
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
    todo!()
}

unsafe extern "C" fn relation_vacuum(
    rel: Relation,
    params: *mut VacuumParams,
    bstrategy: BufferAccessStrategy,
) {
}

unsafe extern "C" fn scan_analyze_next_block(
    scan: TableScanDesc,
    blockno: BlockNumber,
    bstrategy: BufferAccessStrategy,
) -> bool {
    true
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
