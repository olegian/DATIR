use std::f32::INFINITY;

use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;
use rustc_middle::mir::visit::MutVisitor;
use rustc_middle::mir::{Local, Location};
use rustc_middle::mir::visit::PlaceContext;
use rustc_hir::{Node, Item, ItemKind};

struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>
}

impl<'tcx> MutVisitor<'tcx> for MirVisitor<'tcx> {
    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> { self.tcx }

    fn visit_local(
        &mut self,
        local: &mut Local,
        context: PlaceContext,
        location: Location
    ) {

        println!("{:?}", local);
        println!("{:?}", context);
        println!("{:?}", location);

    }
}

pub fn after_analysis_helper<'tcx>(
    compiler: &interface::Compiler,
    tcx: TyCtxt<'tcx>,
) {

    // for owner_id in tcx.hir_body_owners() {
    //     // println!("{:?}", owner_id);
    //     let hir_node = tcx.hir_node_by_def_id(owner_id);
    //     println!("{:#?}", hir_node);
    // }

    for local_def_id in tcx.hir_body_owners() {
            let body = tcx.hir_body_owned_by(local_def_id);
            // let body = tcx.hir_body(body_id);
            
            let def_id = local_def_id.to_def_id();
            let def_path = tcx.def_path_str(def_id);
            
            println!("Found body for: {}", def_path);
            println!("  Body: {:#?}", body);
            println!();
        }

    // recompiles crate?
    // let krate = tcx.hir_crate(());
    // println!("{:#?}", krate)


    // MARK: get hir specific ndoe
    // for ldid in tcx.iter_local_def_id() {
    //     let did = tcx.local_def_id_to_hir_id(ldid);
    //     let node = tcx.hir_node(did);
    //     match node {
    //         Node::Item(Item {
    //             kind: ItemKind::Fn {
    //                 ident,
    //                 body,
    //                 ..
    //             },
    //             ..
    //         }) => {
    //             println!("{:#?}", ident);
    //             println!("{:#?}", body);
    //         },
    //         _ => { }
    //     }
    //     // println!("{:#?}", node);
    // }

    // MARK: MUT VISITOR
    // for did in tcx.iter_local_def_id() {
    //     let mir = tcx.mir_built(did);
    //     let mut visitor = MirVisitor { tcx };
    //     visitor.visit_body(&mut mir.steal());
    // }
}

// MARK: Register passes?
// pub fn mir_drops_elaborated_and_const_checked(
//     self,
//     key: impl IntoQueryParam<LocalDefId>,
// ) -> &'tcx Steal<Body<'tcx>> {