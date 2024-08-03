use super::{
    Assembly, BasicBlock, CILNode, CILRoot, ClassDef, ClassRef, ClassRefIdx, FieldDesc, FnSig,
    MethodDef, MethodRef, StaticFieldDesc, Type,
};
impl Assembly {
    pub fn translate_type(&mut self, source: &Self, tpe: Type) -> Type {
        match tpe {
            Type::Ptr(inner) => {
                let inner = self.translate_type(source, *source.get_type(inner));
                self.nptr(inner)
            }
            Type::Ref(inner) => {
                let inner = self.translate_type(source, *source.get_type(inner));
                self.nptr(inner)
            }
            Type::Int(_)
            | Type::Float(_)
            | Type::PlatformString
            | Type::PlatformChar
            | Type::Bool
            | Type::Void
            | Type::PlatformObject
            | Type::PlatformGeneric(_, _) => tpe,
            Type::ClassRef(class_ref) => {
                Type::ClassRef(self.translate_class_ref(source, class_ref))
            }
            Type::PlatformArray { elem, dims } => {
                let elem = self.translate_type(source, *source.get_type(elem));
                let elem = self.alloc_type(elem);
                Type::PlatformArray { elem, dims }
            }
            Type::FnPtr(sig) => {
                let sig = self.translate_sig(source, source.get_sig(sig).clone());
                Type::FnPtr(self.alloc_sig(sig))
            }
        }
    }
    pub fn translate_class_ref(
        &mut self,
        source: &Assembly,
        class_ref: ClassRefIdx,
    ) -> ClassRefIdx {
        let cref = source.class_ref(class_ref);
        let name = self.alloc_string(source.get_string(cref.name()).as_ref());
        let asm = cref
            .asm()
            .map(|asm_name| self.alloc_string(source.get_string(asm_name).as_ref()));
        let generics = cref
            .generics()
            .iter()
            .map(|tpe| self.translate_type(source, *tpe))
            .collect();
        self.alloc_class_ref(ClassRef::new(name, asm, cref.is_valuetype(), generics))
    }
    pub fn translate_sig(&mut self, source: &Assembly, sig: FnSig) -> FnSig {
        FnSig::new(
            sig.inputs()
                .iter()
                .map(|tpe| self.translate_type(source, *tpe))
                .collect(),
            self.translate_type(source, *sig.output()),
        )
    }
    pub fn translate_field(&mut self, source: &Assembly, field: FieldDesc) -> FieldDesc {
        let name = self.alloc_string(source.get_string(field.name()).as_ref());
        let owner = self.alloc_class_ref(source.class_ref(field.owner()).clone());
        let tpe = self.translate_type(source, field.tpe());
        FieldDesc::new(owner, name, tpe)
    }
    pub fn translate_static_field(
        &mut self,
        source: &Assembly,
        field: StaticFieldDesc,
    ) -> StaticFieldDesc {
        let name = self.alloc_string(source.get_string(field.name()).as_ref());
        let owner = self.alloc_class_ref(source.class_ref(field.owner()).clone());
        let tpe = self.translate_type(source, field.tpe());
        StaticFieldDesc::new(owner, name, tpe)
    }
    pub fn translate_method_ref(&mut self, source: &Assembly, method_ref: MethodRef) -> MethodRef {
        let class = self.alloc_class_ref(source.class_ref(method_ref.class()).clone());
        let name = self.alloc_string(source.get_string(method_ref.name()).as_ref());
        let sig = self.translate_sig(source, source.get_sig(method_ref.sig()).clone());
        let sig = self.alloc_sig(sig);
        let generics = method_ref
            .generics()
            .iter()
            .map(|tpe| self.translate_type(source, *tpe))
            .collect();
        MethodRef::new(class, name, sig, method_ref.kind(), generics)
    }
    pub fn translate_node(&mut self, source: &Assembly, node: CILNode) -> CILNode {
        match node {
            CILNode::LdLoc(_)
            | CILNode::LdLocA(_)
            | CILNode::LdArg(_)
            | CILNode::LdArgA(_)
            | CILNode::Const(_) => node,
            CILNode::BinOp(a, b, op) => {
                let a = self.translate_node(source, source.get_node(a).clone());
                let b = self.translate_node(source, source.get_node(b).clone());
                CILNode::BinOp(self.alloc_node(a), self.alloc_node(b), op)
            }
            CILNode::UnOp(a, op) => {
                let a = self.translate_node(source, source.get_node(a).clone());
                CILNode::UnOp(self.alloc_node(a), op)
            }
            CILNode::Call(call_arg) => {
                let (mref, args) = call_arg.as_ref();
                let method_ref = self.translate_method_ref(source, source.get_mref(*mref).clone());
                let mref = self.alloc_methodref(method_ref);
                let args = args
                    .iter()
                    .map(|arg| {
                        let arg = self.translate_node(source, source.get_node(*arg).clone());
                        self.alloc_node(arg)
                    })
                    .collect();
                CILNode::Call(Box::new((mref, args)))
            }
            CILNode::IntCast {
                input,
                target,
                extend,
            } => {
                let input = self.translate_node(source, source.get_node(input).clone());
                let input = self.alloc_node(input);
                CILNode::IntCast {
                    input,
                    target,
                    extend,
                }
            }
            CILNode::FloatCast {
                input,
                target,
                is_signed,
            } => {
                let input = self.translate_node(source, source.get_node(input).clone());
                let input = self.alloc_node(input);
                CILNode::FloatCast {
                    input,
                    target,
                    is_signed,
                }
            }
            CILNode::RefToPtr(input) => {
                let input = self.translate_node(source, source.get_node(input).clone());
                let input = self.alloc_node(input);
                CILNode::RefToPtr(input)
            }
            CILNode::PtrCast(input, cast_res) => {
                let input = self.translate_node(source, source.get_node(input).clone());
                let input = self.alloc_node(input);
                let cast_res = match *cast_res {
                    crate::v2::cilnode::PtrCastRes::Ptr(inner) => {
                        let inner = self.translate_type(source, *source.get_type(inner));
                        crate::v2::cilnode::PtrCastRes::Ptr(self.alloc_type(inner))
                    }
                    crate::v2::cilnode::PtrCastRes::Ref(inner) => {
                        let inner = self.translate_type(source, *source.get_type(inner));
                        crate::v2::cilnode::PtrCastRes::Ref(self.alloc_type(inner))
                    }
                    crate::v2::cilnode::PtrCastRes::FnPtr(sig) => {
                        let sig = self.translate_sig(source, source.get_sig(sig).clone());
                        crate::v2::cilnode::PtrCastRes::FnPtr(self.alloc_sig(sig))
                    }
                    crate::v2::cilnode::PtrCastRes::USize
                    | crate::v2::cilnode::PtrCastRes::ISize => *cast_res,
                };
                CILNode::PtrCast(input, Box::new(cast_res))
            }
            CILNode::LdFieldAdress { addr, field } => {
                let field = self.translate_field(source, *source.get_field(field));
                let field = self.alloc_field(field);
                let addr = self.translate_node(source, source.get_node(addr).clone());
                let addr = self.alloc_node(addr);
                CILNode::LdFieldAdress { addr, field }
            }
            CILNode::LdField { addr, field } => {
                let field = self.translate_field(source, *source.get_field(field));
                let field = self.alloc_field(field);
                let addr = self.translate_node(source, source.get_node(addr).clone());
                let addr = self.alloc_node(addr);
                CILNode::LdField { addr, field }
            }
            CILNode::LdInd {
                addr,
                tpe,
                volitale,
            } => {
                let addr = self.translate_node(source, source.get_node(addr).clone());
                let addr = self.alloc_node(addr);
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::LdInd {
                    addr,
                    tpe,
                    volitale,
                }
            }
            CILNode::SizeOf(tpe) => {
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::SizeOf(tpe)
            }
            CILNode::GetException => CILNode::GetException,
            CILNode::IsInst(object, tpe) => {
                let object = self.translate_node(source, source.get_node(object).clone());
                let object = self.alloc_node(object);
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::IsInst(object, tpe)
            }
            CILNode::CheckedCast(object, tpe) => {
                let object = self.translate_node(source, source.get_node(object).clone());
                let object = self.alloc_node(object);
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::CheckedCast(object, tpe)
            }
            CILNode::CallI(args) => {
                let (fnptr, sig, args) = args.as_ref();
                let fnptr = self.translate_node(source, source.get_node(*fnptr).clone());
                let fnptr = self.alloc_node(fnptr);
                let sig = self.translate_sig(source, source.get_sig(*sig).clone());
                let sig = self.alloc_sig(sig);
                let args = args
                    .iter()
                    .map(|arg| {
                        let arg = self.translate_node(source, source.get_node(*arg).clone());
                        self.alloc_node(arg)
                    })
                    .collect();
                CILNode::CallI(Box::new((fnptr, sig, args)))
            }
            CILNode::LocAlloc { size } => {
                let size = self.translate_node(source, source.get_node(size).clone());
                let size = self.alloc_node(size);
                CILNode::LocAlloc { size }
            }
            CILNode::LdStaticField(sfld) => {
                let sfld = self.translate_static_field(source, *source.get_static_field(sfld));
                let sfld = self.alloc_sfld(sfld);
                CILNode::LdStaticField(sfld)
            }
            CILNode::LdFtn(mref) => {
                let method_ref = self.translate_method_ref(source, source.get_mref(mref).clone());
                let mref = self.alloc_methodref(method_ref);
                CILNode::LdFtn(mref)
            }
            CILNode::LdTypeToken(tpe) => {
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::LdTypeToken(tpe)
            }
            CILNode::LdLen(len) => {
                let len = self.translate_node(source, source.get_node(len).clone());
                let len = self.alloc_node(len);
                CILNode::LdLen(len)
            }
            CILNode::LocAllocAlgined { tpe, align } => {
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::LocAllocAlgined { tpe, align }
            }
            CILNode::LdElelemRef { array, index } => {
                let array = self.translate_node(source, source.get_node(array).clone());
                let array = self.alloc_node(array);
                let index = self.translate_node(source, source.get_node(index).clone());
                let index = self.alloc_node(index);
                CILNode::LdElelemRef { array, index }
            }
            CILNode::UnboxAny { object, tpe } => {
                let object = self.translate_node(source, source.get_node(object).clone());
                let object = self.alloc_node(object);
                let tpe = self.translate_type(source, *source.get_type(tpe));
                let tpe = self.alloc_type(tpe);
                CILNode::UnboxAny { object, tpe }
            }
        }
    }
    pub fn translate_root(&mut self, source: &Assembly, root: CILRoot) -> CILRoot {
        match root {
            CILRoot::StLoc(loc, node) => {
                let node = self.translate_node(source, source.get_node(node).clone());
                let node = self.alloc_node(node);
                CILRoot::StLoc(loc, node)
            }
            CILRoot::StArg(loc, node) => {
                let node = self.translate_node(source, source.get_node(node).clone());
                let node = self.alloc_node(node);
                CILRoot::StArg(loc, node)
            }
            CILRoot::Ret(node) => {
                let node = self.translate_node(source, source.get_node(node).clone());
                let node = self.alloc_node(node);
                CILRoot::Ret(node)
            }
            CILRoot::Pop(node) => {
                let node = self.translate_node(source, source.get_node(node).clone());
                let node = self.alloc_node(node);
                CILRoot::Pop(node)
            }
            CILRoot::Throw(node) => {
                let node = self.translate_node(source, source.get_node(node).clone());
                let node = self.alloc_node(node);
                CILRoot::Throw(node)
            }
            CILRoot::Branch(branch) => {
                let (target, sub_target, cond) = branch.as_ref();
                let cond = cond.as_ref().map(|cond| match cond {
                    super::cilroot::BranchCond::True(cond) => {
                        let cond = self.translate_node(source, source.get_node(*cond).clone());
                        let cond = self.alloc_node(cond);
                        super::cilroot::BranchCond::True(cond)
                    }
                    super::cilroot::BranchCond::False(cond) => {
                        let cond = self.translate_node(source, source.get_node(*cond).clone());
                        let cond = self.alloc_node(cond);
                        super::cilroot::BranchCond::False(cond)
                    }
                    super::cilroot::BranchCond::Eq(a, b) => {
                        let a = self.translate_node(source, source.get_node(*a).clone());
                        let a = self.alloc_node(a);
                        let b = self.translate_node(source, source.get_node(*b).clone());
                        let b = self.alloc_node(b);
                        super::cilroot::BranchCond::Eq(a, b)
                    }
                    super::cilroot::BranchCond::Ne(a, b) => {
                        let a = self.translate_node(source, source.get_node(*a).clone());
                        let a = self.alloc_node(a);
                        let b = self.translate_node(source, source.get_node(*b).clone());
                        let b = self.alloc_node(b);
                        super::cilroot::BranchCond::Ne(a, b)
                    }
                    super::cilroot::BranchCond::Lt(a, b, cmp_kind) => {
                        let a = self.translate_node(source, source.get_node(*a).clone());
                        let a = self.alloc_node(a);
                        let b = self.translate_node(source, source.get_node(*b).clone());
                        let b = self.alloc_node(b);
                        super::cilroot::BranchCond::Lt(a, b, cmp_kind.clone())
                    }
                    super::cilroot::BranchCond::Gt(a, b, cmp_kind) => {
                        let a = self.translate_node(source, source.get_node(*a).clone());
                        let a = self.alloc_node(a);
                        let b = self.translate_node(source, source.get_node(*b).clone());
                        let b = self.alloc_node(b);
                        super::cilroot::BranchCond::Gt(a, b, cmp_kind.clone())
                    }
                });
                CILRoot::Branch(Box::new((*target, *sub_target, cond)))
            }
            CILRoot::VoidRet | CILRoot::Break | CILRoot::Nop | CILRoot::ReThrow => root,
            CILRoot::SourceFileInfo {
                line_start,
                line_len,
                col_start,
                col_len,
                file,
            } => {
                let file = self.alloc_string(source.get_string(file).as_ref());
                CILRoot::SourceFileInfo {
                    line_start,
                    line_len,
                    col_start,
                    col_len,
                    file,
                }
            }
            CILRoot::SetField(info) => {
                let (field, addr, val) = info.as_ref();
                let field = self.translate_field(source, *source.get_field(*field));
                let field = self.alloc_field(field);
                let addr = self.translate_node(source, source.get_node(*addr).clone());
                let addr = self.alloc_node(addr);
                let val = self.translate_node(source, source.get_node(*val).clone());
                let val = self.alloc_node(val);
                CILRoot::SetField(Box::new((field, addr, val)))
            }
            CILRoot::Call(call_arg) => {
                let (mref, args) = call_arg.as_ref();
                let method_ref = self.translate_method_ref(source, source.get_mref(*mref).clone());
                let mref = self.alloc_methodref(method_ref);
                let args = args
                    .iter()
                    .map(|arg| {
                        let arg = self.translate_node(source, source.get_node(*arg).clone());
                        self.alloc_node(arg)
                    })
                    .collect();
                CILRoot::Call(Box::new((mref, args)))
            }
            CILRoot::StInd(info) => {
                let (addr, val, tpe, vol) = info.as_ref();
                let addr = self.translate_node(source, source.get_node(*addr).clone());
                let addr = self.alloc_node(addr);
                let val = self.translate_node(source, source.get_node(*val).clone());
                let val = self.alloc_node(val);
                let tpe = self.translate_type(source, *tpe);
                CILRoot::StInd(Box::new((addr, val, tpe, *vol)))
            }
            CILRoot::InitBlk(info) => {
                let (dst, val, count) = info.as_ref();
                let dst = self.translate_node(source, source.get_node(*dst).clone());
                let dst = self.alloc_node(dst);
                let val = self.translate_node(source, source.get_node(*val).clone());
                let val = self.alloc_node(val);
                let count = self.translate_node(source, source.get_node(*count).clone());
                let count = self.alloc_node(count);
                CILRoot::InitBlk(Box::new((dst, val, count)))
            }
            CILRoot::CpBlk(info) => {
                let (dst, src, len) = info.as_ref();
                let dst = self.translate_node(source, source.get_node(*dst).clone());
                let dst = self.alloc_node(dst);
                let src = self.translate_node(source, source.get_node(*src).clone());
                let src = self.alloc_node(src);
                let len = self.translate_node(source, source.get_node(*len).clone());
                let len = self.alloc_node(len);
                CILRoot::CpBlk(Box::new((dst, src, len)))
            }
            CILRoot::CallI(args) => {
                let (fnptr, sig, args) = args.as_ref();
                let fnptr = self.translate_node(source, source.get_node(*fnptr).clone());
                let fnptr = self.alloc_node(fnptr);
                let sig = self.translate_sig(source, source.get_sig(*sig).clone());
                let sig = self.alloc_sig(sig);
                let args = args
                    .iter()
                    .map(|arg| {
                        let arg = self.translate_node(source, source.get_node(*arg).clone());
                        self.alloc_node(arg)
                    })
                    .collect();
                CILRoot::CallI(Box::new((fnptr, sig, args)))
            }
            CILRoot::ExitSpecialRegion { target, source } => {
                CILRoot::ExitSpecialRegion { target, source }
            }
            CILRoot::SetStaticField { field, val } => {
                let val = self.translate_node(source, source.get_node(val).clone());
                let val = self.alloc_node(val);
                let field = self.translate_static_field(source, *source.get_static_field(field));
                let field = self.alloc_sfld(field);
                CILRoot::SetStaticField { field, val }
            }
        }
    }
    pub fn translate_block(&mut self, source: &Assembly, block: &BasicBlock) -> BasicBlock {
        let roots = block
            .roots()
            .iter()
            .map(|root| {
                let root = self.translate_root(source, source.get_root(*root).clone());
                self.alloc_root(root)
            })
            .collect();
        let handler = block.handler().map(|blocks| {
            blocks
                .iter()
                .map(|block| self.translate_block(source, block))
                .collect()
        });
        BasicBlock::new(roots, block.block_id(), handler)
    }
    pub fn translate_method_def(&mut self, source: &Assembly, def: &MethodDef) -> MethodDef {
        let class = self.translate_class_ref(source, *def.class());
        let class = self.class_ref_to_def(class).unwrap();
        let name = self.alloc_string(source.get_string(def.name()).as_ref());
        let sig = self.translate_sig(source, source.get_sig(def.sig()).clone());
        let sig = self.alloc_sig(sig);
        let method_impl = match def.implementation() {
            super::MethodImpl::MethodBody { blocks, locals } => {
                let blocks = blocks
                    .iter()
                    .map(|block| self.translate_block(source, block))
                    .collect();
                let locals = locals
                    .iter()
                    .map(|(name, tpe)| {
                        let tpe = self.translate_type(source, *source.get_type(*tpe));
                        (
                            name.map(|name| self.alloc_string(source.get_string(name).as_ref())),
                            self.alloc_type(tpe),
                        )
                    })
                    .collect();
                super::MethodImpl::MethodBody { blocks, locals }
            }
            super::MethodImpl::Extern {
                lib,
                preserve_errno,
            } => {
                let lib = self.alloc_string(source.get_string(*lib).as_ref());
                super::MethodImpl::Extern {
                    lib,
                    preserve_errno: *preserve_errno,
                }
            }
            super::MethodImpl::AliasFor(mref) => {
                let method_ref = self.translate_method_ref(source, source.get_mref(*mref).clone());
                let mref = self.alloc_methodref(method_ref);
                super::MethodImpl::AliasFor(mref)
            }
            super::MethodImpl::Missing => super::MethodImpl::Missing,
        };
        let arg_names = def
            .arg_names()
            .iter()
            .map(|arg| arg.map(|arg| self.alloc_string(source.get_string(arg).as_ref())))
            .collect();
        MethodDef::new(
            *def.access(),
            class,
            name,
            sig,
            def.kind(),
            method_impl,
            arg_names,
        )
    }
    pub fn translate_class_def(&mut self, source: &Assembly, def: &ClassDef) -> ClassDef {
        let name = self.alloc_string(source.get_string(def.name()).as_ref());
        let extends = def
            .extends()
            .map(|cref| self.translate_class_ref(source, cref));
        let fields = def
            .fields()
            .iter()
            .map(|(tpe, name, offset)| {
                let tpe = self.translate_type(source, *tpe);
                let name = self.alloc_string(source.get_string(def.name()).as_ref());
                (tpe, name, *offset)
            })
            .collect();
        let static_fields = def
            .static_fields()
            .iter()
            .map(|(tpe, name, thread_local)| {
                let tpe = self.translate_type(source, *tpe);
                let name = self.alloc_string(source.get_string(def.name()).as_ref());
                (tpe, name, *thread_local)
            })
            .collect();
        let methods = def
            .methods()
            .iter()
            .map(|mdef| {
                let method_def = self.translate_method_def(source, source.method_def(*mdef));
                self.new_method(method_def)
            })
            .collect();
        ClassDef::new(
            name,
            def.is_valuetype(),
            def.generics(),
            extends,
            fields,
            static_fields,
            methods,
            *def.access(),
            def.explict_size(),
        )
    }
}
