#![allow(unused_variables)]

use pgrx::pg_sys::{
    Datum, FunctionCallInfo, IndexFetchTableData, ItemPointer, NodeTag, ParallelTableScanDesc,
    ParallelTableScanDescData, Relation, ScanDirection, ScanKey, Snapshot, TableAmRoutine,
    TableScanDesc, TableScanDescData, TupleTableSlot, TupleTableSlotOps,
};
use pgrx::{prelude::*, Internal};

static TABLE_AM_ROUTINE: TableAmRoutine = TableAmRoutine {
    type_: NodeTag::T_TableAmRoutine,
    slot_callbacks: Some(slots_callback),
    scan_begin: Some(scan_begin),
    scan_end: Some(scan_end),
    scan_rescan: Some(scan_rescan),
    scan_getnextslot: Some(heap_getnextslot),
    // scan_set_tidrange: None,
    // scan_getnextslot_tidrange: None,
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
    // index_delete_tuples: None,
    tuple_insert: None,
    tuple_insert_speculative: None,
    tuple_complete_speculative: None,
    multi_insert: None,
    tuple_delete: None,
    tuple_update: None,
    tuple_lock: None,
    finish_bulk_insert: None,
    relation_set_new_filenode: None,
    relation_nontransactional_truncate: None,
    relation_copy_data: None,
    relation_copy_for_cluster: None,
    relation_vacuum: None,
    scan_analyze_next_block: None,
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
