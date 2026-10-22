//! The ACDSDATA section lists its ACDSRECORDs in handle order, as AutoCAD writes them, not in
//! the order the solids happened to be written (block definitions before model space), which
//! reordered a drawing's data store on every round trip.

use acadrust::entities::acis::primitives::build_box;
use acadrust::entities::{EntityType, Solid3D};
use acadrust::tables::BlockRecord;
use acadrust::types::DxfVersion;
use acadrust::{CadDocument, DxfWriter};

/// A model-space solid, then a block holding another solid: the block's solid has the higher
/// handle but is written first.
fn drawing() -> CadDocument {
    let mut doc = CadDocument::with_version(DxfVersion::AC1032);
    let sat = build_box([0.0, 0.0, 0.0], 2.0, 3.0, 4.0).to_sat_string();
    doc.add_entity(EntityType::Solid3D(Solid3D::from_sat(&sat)))
        .unwrap();

    let mut record = BlockRecord::new("SOLIDS");
    record.handle = doc.allocate_handle();
    record.block_entity_handle = doc.allocate_handle();
    record.block_end_handle = doc.allocate_handle();
    let owner = record.handle;
    doc.block_records.add(record).unwrap();
    let sat = build_box([10.0, 0.0, 0.0], 3.0, 4.0, 5.0).to_sat_string();
    let mut solid = Solid3D::from_sat(&sat);
    solid.common.owner_handle = owner;
    doc.add_entity(EntityType::Solid3D(solid)).unwrap();
    doc
}

/// The entity handle (group 320) of each ACDSRECORD, in file order.
fn record_handles(dxf: &str) -> Vec<u64> {
    let lines: Vec<&str> = dxf.lines().map(str::trim).collect();
    let mut handles = Vec::new();
    let mut in_record = false;
    for pair in lines.chunks(2) {
        let [code, value] = pair else { break };
        match *code {
            "0" => in_record = *value == "ACDSRECORD",
            "320" if in_record => {
                handles.push(u64::from_str_radix(value, 16).unwrap());
                in_record = false;
            }
            _ => {}
        }
    }
    handles
}

#[test]
fn acds_records_are_written_in_handle_order() {
    let doc = drawing();
    let dxf = String::from_utf8(DxfWriter::new(&doc).write_to_vec().unwrap()).unwrap();
    let handles = record_handles(&dxf);
    assert_eq!(handles.len(), 2, "one record per solid");
    let mut sorted = handles.clone();
    sorted.sort_unstable();
    assert_eq!(handles, sorted);
}
