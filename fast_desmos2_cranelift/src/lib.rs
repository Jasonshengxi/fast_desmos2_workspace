use codegen::ir::UserFuncName;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use fast_desmos2_eval::EvalNode;
pub use translate::Translate;

mod translate;

pub fn compile(expr: &EvalNode) -> fn() -> f64 {
    let jit_builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
    let mut module = JITModule::new(jit_builder);

    let mut sig = module.make_signature();
    sig.returns.push(AbiParam::new(types::F64));

    let func_id = module
        .declare_function("main", Linkage::Export, &sig)
        .unwrap();
    let mut context = module.make_context();
    context.func.signature = sig;
    context.func.name = UserFuncName::user(0, 0);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut func_ctx);

    let block = builder.create_block();
    builder.switch_to_block(block);

    let outcome = expr.translate(&mut builder);
    builder.ins().return_(&[outcome]);

    builder.seal_all_blocks();
    builder.finalize();

    let flags = settings::Flags::new(settings::builder());
    codegen::verify_function(&context.func, &flags).unwrap();

    module.define_function(func_id, &mut context).unwrap();
    module.finalize_definitions().unwrap();

    unsafe { std::mem::transmute::<*const u8, fn() -> f64>(module.get_finalized_function(func_id)) }
}
