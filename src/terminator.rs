use crate::utilis::garg_to_string;
use crate::{
    cil_op::{CILOp, CallSite},
    function_sig::FnSig,
    operand::handle_operand,
    r#type::DotnetTypeRef,
    utilis::monomorphize,
    utilis::CTOR_FN_NAME,
    utilis::MANAGED_CALL_FN_NAME,
    utilis::MANAGED_CALL_VIRT_FN_NAME,
};
use rustc_middle::ty::InstanceDef;
use rustc_middle::{
    mir::{Body, Const, ConstValue, Operand, Place, SwitchTargets, Terminator, TerminatorKind},
    ty::{GenericArg, Instance, ParamEnv, Ty, TyCtxt, TyKind},
};
use rustc_span::def_id::DefId;
/// Calls a non-virtual managed function(used for interop)
fn call_managed<'ctx>(
    tyctx: TyCtxt<'ctx>,
    _def_id: DefId,
    subst_ref: &[GenericArg<'ctx>],
    function_name: &str,
    args: &[Operand<'ctx>],
    destination: &Place<'ctx>,
    method: &'ctx Body<'ctx>,
    method_instance: Instance<'ctx>,
    fn_type: &Ty<'ctx>,
) -> Vec<CILOp> {
    let argc_start =
        function_name.find(MANAGED_CALL_FN_NAME).unwrap() + (MANAGED_CALL_FN_NAME.len());
    let argc_end = argc_start + function_name[argc_start..].find('_').unwrap();
    let argc = &function_name[argc_start..argc_end];
    let argc = argc.parse::<u32>().unwrap();
    assert!(subst_ref.len() as u32 == argc + 3 || subst_ref.len() as u32 == argc + 4 || true);
    assert!(args.len() as u32 == argc);
    let asm = garg_to_string(&subst_ref[0], tyctx);
    let asm = Some(asm).filter(|asm| !asm.is_empty());
    let class_name = garg_to_string(&subst_ref[1], tyctx);
    let is_valuetype = crate::utilis::garag_to_bool(&subst_ref[2], tyctx);
    let managed_fn_name = garg_to_string(&subst_ref[3], tyctx);
    let mut tpe = DotnetTypeRef::new(asm.as_ref().map(|x| x.as_str()), &class_name);
    tpe.set_valuetype(is_valuetype);
    let signature = FnSig::from_poly_sig(&fn_type.fn_sig(tyctx), tyctx, &method_instance)
        .expect("Can't get the function signature");
    if argc == 0 {
        let ret = crate::r#type::Type::Void;
        let call = vec![CILOp::Call(CallSite::boxed(
            Some(tpe.clone()),
            managed_fn_name.into(),
            FnSig::new(&[], &ret),
            true,
        ))];
        if *signature.output() == crate::r#type::Type::Void {
            call
        } else {
            crate::place::place_set(destination, tyctx, call, method, method_instance)
        }
    } else {
        let is_static = crate::utilis::garag_to_bool(&subst_ref[4], tyctx);

        let mut call = Vec::new();
        for arg in args {
            call.extend(crate::operand::handle_operand(
                arg,
                tyctx,
                method,
                method_instance,
            ));
        }
        call.push(CILOp::Call(CallSite::boxed(
            Some(tpe.clone()),
            managed_fn_name.into(),
            signature.clone(),
            is_static,
        )));
        if *signature.output() == crate::r#type::Type::Void {
            call
        } else {
            crate::place::place_set(destination, tyctx, call, method, method_instance)
        }
    }
}
/// Calls a virtual managed function(used for interop)
fn callvirt_managed<'ctx>(
    tyctx: TyCtxt<'ctx>,
    _def_id: DefId,
    subst_ref: &[GenericArg<'ctx>],
    function_name: &str,
    args: &[Operand<'ctx>],
    destination: &Place<'ctx>,
    method: &'ctx Body<'ctx>,
    method_instance: Instance<'ctx>,
    fn_type: &Ty<'ctx>,
) -> Vec<CILOp> {
    let argc_start =
        function_name.find(MANAGED_CALL_VIRT_FN_NAME).unwrap() + (MANAGED_CALL_VIRT_FN_NAME.len());
    let argc_end = argc_start + function_name[argc_start..].find('_').unwrap();
    let argc = &function_name[argc_start..argc_end];
    let argc = argc.parse::<u32>().unwrap();
    assert!(subst_ref.len() as u32 == argc + 3 || subst_ref.len() as u32 == argc + 4 || true);
    assert!(args.len() as u32 == argc);
    let asm = garg_to_string(&subst_ref[0], tyctx);
    let asm = Some(asm).filter(|asm| !asm.is_empty());
    let class_name = garg_to_string(&subst_ref[1], tyctx);
    let is_valuetype = crate::utilis::garag_to_bool(&subst_ref[2], tyctx);

    let managed_fn_garg = &subst_ref[3];
    let managed_fn_garg = crate::utilis::monomorphize(&method_instance, *managed_fn_garg, tyctx);
    let managed_fn_name = garg_to_string(&managed_fn_garg, tyctx);

    let mut tpe = DotnetTypeRef::new(asm.as_ref().map(|x| x.as_str()), &class_name);
    tpe.set_valuetype(is_valuetype);
    let signature = FnSig::from_poly_sig(&fn_type.fn_sig(tyctx), tyctx, &method_instance)
        .expect("Can't get the function signature");
    if argc == 0 {
        let ret = crate::r#type::Type::Void;
        let call = vec![CILOp::Call(CallSite::boxed(
            Some(tpe.clone()),
            managed_fn_name.into(),
            FnSig::new(&[], &ret),
            true,
        ))];
        if *signature.output() == crate::r#type::Type::Void {
            call
        } else {
            crate::place::place_set(destination, tyctx, call, method, method_instance)
        }
    } else {
        let is_static = crate::utilis::garag_to_bool(&subst_ref[4], tyctx);

        let mut call = Vec::new();
        for arg in args {
            call.extend(crate::operand::handle_operand(
                arg,
                tyctx,
                method,
                method_instance,
            ));
        }
        call.push(CILOp::CallVirt(CallSite::boxed(
            Some(tpe.clone()),
            managed_fn_name.into(),
            signature.clone(),
            is_static,
        )));
        if *signature.output() == crate::r#type::Type::Void {
            call
        } else {
            crate::place::place_set(destination, tyctx, call, method, method_instance)
        }
    }
}
/// Creates a new managed object, and places a reference to it in destination
fn call_ctor<'ctx>(
    tyctx: TyCtxt<'ctx>,
    _def_id: DefId,
    subst_ref: &[GenericArg<'ctx>],
    function_name: &str,
    args: &[Operand<'ctx>],
    destination: &Place<'ctx>,
    method: &'ctx Body<'ctx>,
    method_instance: Instance<'ctx>,
) -> Vec<CILOp> {
    let argc_start = function_name.find(CTOR_FN_NAME).unwrap() + (CTOR_FN_NAME.len());
    let argc_end = argc_start + function_name[argc_start..].find('_').unwrap();
    let argc = &function_name[argc_start..argc_end];
    let argc = argc.parse::<u32>().unwrap();
    // Check that there are enough function path and argument specifers
    assert!(subst_ref.len() as u32 == argc + 3);
    // Check that a proper number of arguments is used
    assert!(args.len() as u32 == argc);
    // Get the name of the assembly the constructed object resides in
    let asm = garg_to_string(&subst_ref[0], tyctx);
    // If empty, make it none(for consitent encoing of No-assembly)
    let asm = Some(asm).filter(|asm| !asm.is_empty());
    // Get the name of the constructed object
    let class_name = garg_to_string(&subst_ref[1], tyctx);
    // Check if the costructed object is valuetype. TODO: this may be unnecesary. Are valuetpes constructed using newobj?
    let is_valuetype = crate::utilis::garag_to_bool(&subst_ref[2], tyctx);
    let mut tpe = DotnetTypeRef::new(asm.as_ref().map(|x| x.as_str()), &class_name);
    tpe.set_valuetype(is_valuetype);
    // If no arguments, inputs don't have to be handled, so a simpler call handling is used.
    if argc == 0 {
        crate::place::place_set(
            destination,
            tyctx,
            vec![CILOp::NewObj(CallSite::boxed(
                Some(tpe.clone()),
                ".ctor".into(),
                FnSig::new(&[tpe.into()], &crate::r#type::Type::Void),
                false,
            ))],
            method,
            method_instance,
        )
    } else {
        let mut inputs: Vec<_> = subst_ref[3..]
            .iter()
            .map(|ty| crate::r#type::Type::from_ty(ty.as_type().unwrap(), tyctx, &method_instance))
            .collect();
        inputs.insert(0, tpe.clone().into());
        let sig = FnSig::new(&inputs, &crate::r#type::Type::Void);
        let mut call = Vec::new();
        for arg in args {
            call.extend(crate::operand::handle_operand(
                arg,
                tyctx,
                method,
                method_instance,
            ));
        }
        call.push(CILOp::NewObj(CallSite::boxed(
            Some(tpe.clone()),
            ".ctor".into(),
            sig,
            false,
        )));
        crate::place::place_set(destination, tyctx, call, method, method_instance)
    }
}
/// Calls `fn_type` with `args`, placing the return value in destination.
fn call<'ctx>(
    fn_type: &Ty<'ctx>,
    body: &'ctx Body<'ctx>,
    tyctx: TyCtxt<'ctx>,
    args: &[Operand<'ctx>],
    destination: &Place<'ctx>,
    method_instance: Instance<'ctx>,
) -> Vec<CILOp> {
    let (instance, def_id, subst_ref) = if let TyKind::FnDef(def_id, subst_ref) = fn_type.kind() {
        let env = ParamEnv::reveal_all();
        let instance = Instance::resolve(tyctx, env, *def_id, subst_ref)
            .expect("Invalid function def")
            .expect("No function?");
        (instance, def_id, subst_ref)
    } else {
        todo!("Trying to call a type which is not a function definition!");
    };
    let signature = FnSig::from_poly_sig_mono(&fn_type.fn_sig(tyctx), tyctx, &method_instance)
        .expect("Can't get the function signature");
    let function_name = crate::utilis::function_name(tyctx.symbol_name(instance));
    // Checks if function is "magic"
    if function_name.contains(CTOR_FN_NAME) {
        // Constructor
        return call_ctor(
            tyctx,
            *def_id,
            subst_ref,
            &function_name,
            args,
            destination,
            body,
            method_instance,
        );
    } else if function_name.contains(MANAGED_CALL_VIRT_FN_NAME) {
        // Virtual (for interop)
        return callvirt_managed(
            tyctx,
            *def_id,
            subst_ref,
            &function_name,
            args,
            destination,
            body,
            method_instance,
            fn_type,
        );
    } else if function_name.contains(MANAGED_CALL_FN_NAME) {
        // Not-Virtual (for interop)
        return call_managed(
            tyctx,
            *def_id,
            subst_ref,
            &function_name,
            args,
            destination,
            body,
            method_instance,
            fn_type,
        );
    }
    let mut call = Vec::new();
    for arg in args {
        call.extend(crate::operand::handle_operand(
            arg,
            tyctx,
            body,
            method_instance,
        ));
    }
    let is_void = matches!(signature.output(), crate::r#type::Type::Void);
    call.push(CILOp::Call(CallSite::boxed(
        None,
        function_name,
        signature,
        true,
    )));
    // Hande
    if is_void {
        call
    } else {
        crate::place::place_set(destination, tyctx, call, body, method_instance)
    }
}
pub fn handle_terminator<'ctx>(
    terminator: &Terminator<'ctx>,
    body: &'ctx Body<'ctx>,
    tyctx: TyCtxt<'ctx>,
    method: &rustc_middle::mir::Body<'ctx>,
    method_instance: Instance<'ctx>,
) -> Vec<CILOp> {
    match &terminator.kind {
        TerminatorKind::Call {
            func,
            args,
            destination,
            target,
            unwind: _,
            call_source: _,
            fn_span: _,
        } => {
            let mut ops = Vec::new();
            match func {
                Operand::Constant(fn_const) => {
                    let fn_ty = fn_const.ty();
                    assert!(
                        fn_ty.is_fn(),
                        "fn_ty{fn_ty:?} in call is not a function type!"
                    );
                    let fn_ty = monomorphize(&method_instance, fn_ty, tyctx);
                    let call_ops = call(&fn_ty, body, tyctx, args, destination, method_instance);
                    ops.extend(call_ops);
                }
                _ => panic!("called func must be const!"),
            }
            if let Some(target) = target {
                ops.push(CILOp::GoTo(target.as_u32()));
            }
            ops
        }
        TerminatorKind::Return => {
            if crate::r#type::Type::from_ty(method.return_ty(), tyctx, &method_instance)
                != crate::r#type::Type::Void
            {
                vec![CILOp::LDLoc(0), CILOp::Ret]
            } else {
                vec![CILOp::Ret]
            }
        }
        TerminatorKind::SwitchInt { discr, targets } => {
            let ty = crate::utilis::monomorphize(&method_instance, discr.ty(method, tyctx), tyctx);
            let discr = crate::operand::handle_operand(discr, tyctx, method, method_instance);
            handle_switch(ty, discr, targets)
        }
        TerminatorKind::Assert {
            cond,
            expected,
            msg,
            target,
            unwind: _,
        } => {
            let mut ops = handle_operand(cond, tyctx, method, method_instance);
            ops.push(CILOp::LdcI32(i32::from(*expected)));
            ops.push(CILOp::BEq(target.as_u32()));
            ops.extend(throw_assert_msg(msg, tyctx, method, method_instance));
            ops
        }
        TerminatorKind::Goto { target } => vec![CILOp::GoTo((*target).into())],
        TerminatorKind::UnwindResume => {
            eprintln!("WARNING: stack unwiniding is not supported yet in rustc_codegen_clr!");
            vec![CILOp::Comment(
                "WARNING: stack unwiniding is not supported yet in rustc_codegen_clr!".into(),
            )]
        }
        TerminatorKind::Drop {
            place,
            target,
            unwind: _,
            replace: _,
        } => {
            let ty = monomorphize(&method_instance, place.ty(method, tyctx).ty, tyctx);
            let drop_instance = Instance::resolve_drop_in_place(tyctx, ty).polymorphize(tyctx);
            if let InstanceDef::DropGlue(_, None) = drop_instance.def {
                //Empty drop, nothing needs to happen.
                vec![]
            } else {
                eprintln!("WARNING: drop is not supported yet in rustc_codegen_clr! drop_instance:{drop_instance:?}");
                vec![
                    CILOp::Comment(
                        "WARNING: drop is not supported yet in rustc_codegen_clr!".into(),
                    ),
                    CILOp::GoTo(target.as_u32()),
                ]
            }
        }
        TerminatorKind::Unreachable => {
            /*
            let string_type = crate::r#type::Type::DotnetType(Box::new(DotnetTypeRef::new(
                Some("System.Runtime"),
                "System.String",
            )));
            let exception = DotnetTypeRef::new(Some("System.Runtime"), "System.Exception");
            let sig = FnSig::new(&[string_type], &crate::r#type::Type::Void);
            vec![
                CILOp::LdStr("Undefined behaviour! Unreachable terminator reached!".into()),
                CILOp::NewObj(CallSite::boxed(Some(exception), ".ctor".into(), sig, false)),
                CILOp::Throw,
            ]*/
            vec![]
        }
        _ => todo!("Unhandled terminator kind {kind:?}", kind = terminator.kind),
    }
}
fn throw_assert_msg<'ctx>(
    msg: &rustc_middle::mir::AssertMessage<'ctx>,
    tyctx: TyCtxt<'ctx>,
    method: &rustc_middle::mir::Body<'ctx>,
    method_instance: Instance<'ctx>,
) -> Vec<CILOp> {
    use rustc_middle::mir::AssertKind;
    // Assertion messages cause miscomplations.
    if true {
        return vec![CILOp::LdNull, CILOp::Throw];
    };
    match msg {
        AssertKind::BoundsCheck { len, index } => {
            let mut ops = Vec::with_capacity(8);
            ops.push(CILOp::LdStr("index out of bounds: the len is ".into()));
            ops.extend(handle_operand(len, tyctx, method, method_instance));
            let usize_class = crate::utilis::usize_class();
            let string_class = crate::utilis::string_class();
            let string_type = crate::r#type::Type::DotnetType(Box::new(string_class.clone()));
            let sig = FnSig::new(&[], &string_type);
            let usize_to_string = CallSite::boxed(Some(usize_class), "ToString".into(), sig, false);
            ops.push(CILOp::Call(usize_to_string.clone()));
            ops.push(CILOp::LdStr(" but the index is".into()));
            ops.extend(handle_operand(index, tyctx, method, method_instance));
            ops.push(CILOp::Call(usize_to_string.clone()));

            let sig = FnSig::new(
                &[
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                ],
                &string_type.clone(),
            );
            let out_of_range_exception =
                DotnetTypeRef::new(Some("System.Runtime"), "System.IndexOutOfRangeException");
            ops.push(CILOp::Call(CallSite::boxed(
                Some(string_class),
                "Concat".into(),
                sig,
                true,
            )));
            let sig = FnSig::new(&[string_type], &crate::r#type::Type::Void);
            ops.push(CILOp::NewObj(CallSite::boxed(
                Some(out_of_range_exception),
                ".ctor".into(),
                sig,
                false,
            )));
            ops.push(CILOp::Throw);
            ops
        }
        AssertKind::DivisionByZero(_operand) => {
            let mut ops = Vec::with_capacity(8);

            let sig = FnSig::new(&[], &crate::r#type::Type::Void);
            let div_by_zero_exception =
                DotnetTypeRef::new(Some("System.Runtime"), "System.DivideByZeroException");
            ops.push(CILOp::NewObj(CallSite::boxed(
                Some(div_by_zero_exception),
                ".ctor".into(),
                sig,
                false,
            )));
            ops.push(CILOp::Throw);
            ops
        }
        AssertKind::RemainderByZero(_operand) => {
            let mut ops = Vec::with_capacity(8);

            let sig = FnSig::new(&[], &crate::r#type::Type::Void);
            let div_by_zero_exception =
                DotnetTypeRef::new(Some("System.Runtime"), "System.DivideByZeroException");
            ops.push(CILOp::NewObj(CallSite::boxed(
                Some(div_by_zero_exception),
                ".ctor".into(),
                sig,
                false,
            )));
            ops.push(CILOp::Throw);
            ops
        }
        AssertKind::Overflow(binop, a, b) => {
            let mut ops = Vec::with_capacity(8);
            let string_class = crate::utilis::string_class();
            ops.push(CILOp::LdStr(
                format!("attempt to {binop:?} with overflow lhs:").into(),
            ));
            ops.extend(handle_operand(a, tyctx, method, method_instance));
            let usize_class = crate::utilis::usize_class();
            let string_type = crate::r#type::Type::DotnetType(Box::new(string_class.clone()));
            let sig = FnSig::new(&[], &string_type);
            let usize_to_string = CallSite::boxed(Some(usize_class), "ToString".into(), sig, false);
            ops.push(CILOp::Call(usize_to_string.clone()));
            ops.push(CILOp::LdStr("rhs:".into()));
            ops.extend(handle_operand(b, tyctx, method, method_instance));
            ops.push(CILOp::Call(usize_to_string.clone()));

            let sig = FnSig::new(
                &[
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                ],
                &string_type.clone(),
            );
            ops.push(CILOp::Call(CallSite::boxed(
                Some(string_class),
                "Concat".into(),
                sig,
                true,
            )));
            let sig = FnSig::new(&[string_type], &crate::r#type::Type::Void);
            let ovefow_exception =
                DotnetTypeRef::new(Some("System.Runtime"), "System.ArithmeticException");
            ops.push(CILOp::NewObj(CallSite::boxed(
                Some(ovefow_exception),
                ".ctor".into(),
                sig,
                false,
            )));
            ops.push(CILOp::Throw);
            ops
        }
        AssertKind::MisalignedPointerDereference { required, found } => {
            let mut ops = Vec::with_capacity(8);
            let string_class = crate::utilis::string_class();
            ops.push(CILOp::LdStr(
                format!("Missaligned pointer dereference. required: ").into(),
            ));
            ops.extend(handle_operand(required, tyctx, method, method_instance));
            let usize_class = crate::utilis::usize_class();
            let string_type = crate::r#type::Type::DotnetType(string_class.clone().into());
            let sig = FnSig::new(&[], &string_type);
            let usize_to_string = CallSite::boxed(Some(usize_class), "ToString".into(), sig, false);
            ops.push(CILOp::Call(usize_to_string.clone()));
            ops.push(CILOp::LdStr(" found: ".into()));
            ops.extend(handle_operand(found, tyctx, method, method_instance));
            ops.push(CILOp::Call(usize_to_string.clone()));

            let sig = FnSig::new(
                &[
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                ],
                &string_type.clone(),
            );
            ops.push(CILOp::Call(CallSite::boxed(
                Some(string_class),
                "Concat".into(),
                sig,
                true,
            )));
            let sig = FnSig::new(&[string_type], &crate::r#type::Type::Void);
            let ovefow_exception = DotnetTypeRef::new(Some("System.Runtime"), "System.Exception");
            ops.push(CILOp::NewObj(CallSite::boxed(
                Some(ovefow_exception),
                ".ctor".into(),
                sig,
                false,
            )));
            ops.push(CILOp::Throw);
            ops
        }
        AssertKind::OverflowNeg(value) => {
            let mut ops = Vec::with_capacity(8);
            let string_class = crate::utilis::string_class();
            ops.push(CILOp::LdStr(
                format!("attempt to neg with overflow value:").into(),
            ));
            ops.extend(handle_operand(value, tyctx, method, method_instance));
            let usize_class = crate::utilis::usize_class();
            let string_type = crate::r#type::Type::DotnetType(Box::new(string_class.clone()));
            let sig = FnSig::new(&[], &string_type);
            let usize_to_string = CallSite::boxed(Some(usize_class), "ToString".into(), sig, false);
            ops.push(CILOp::Call(usize_to_string.clone()));
            ops.push(CILOp::LdStr("rhs:".into()));

            let sig = FnSig::new(
                &[
                    string_type.clone(),
                    string_type.clone(),
                    string_type.clone(),
                ],
                &string_type.clone(),
            );
            ops.push(CILOp::Call(CallSite::boxed(
                Some(string_class),
                "Concat".into(),
                sig,
                true,
            )));
            let sig = FnSig::new(&[string_type], &crate::r#type::Type::Void);
            let ovefow_exception =
                DotnetTypeRef::new(Some("System.Runtime"), "System.ArithmeticException");
            ops.push(CILOp::NewObj(CallSite::boxed(
                Some(ovefow_exception),
                ".ctor".into(),
                sig,
                false,
            )));
            ops.push(CILOp::Throw);
            ops
        }
        _ => todo!("unsuported assertion message:{msg:?}"),
    }
}
fn handle_switch(ty: Ty, discr: Vec<CILOp>, switch: &SwitchTargets) -> Vec<CILOp> {
    let mut ops = Vec::new();
    for (value, target) in switch.iter() {
        ops.extend(discr.iter().cloned());
        ops.extend(match ty.kind() {
            TyKind::Int(int) => crate::constant::load_const_int(value, int),
            TyKind::Uint(uint) => crate::constant::load_const_uint(value, uint),
            TyKind::Bool => vec![CILOp::LdcI32(value as u8 as i32)],
            _ => todo!("Unsuported switch discriminant type {ty:?}"),
        });
        //ops.push(CILOp::LdcI64(value as i64));
        ops.push(CILOp::BEq(target.into()));
    }
    ops.push(CILOp::GoTo(switch.otherwise().into()));
    ops
}
