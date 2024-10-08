use serde::{Deserialize, Serialize};

use super::bimap::BiMapIndex;
use super::field::{StaticFieldDesc, StaticFieldIdx};
use super::{bimap::IntoBiMapIndex, Assembly, Const, Int, MethodRefIdx, SigIdx, TypeIdx};
use super::{FieldDesc, FieldIdx, Float, StringIdx};
use crate::r#type::Type as V1Type;
use crate::IString;
use crate::{
    cil_node::CILNode as V1Node,
    v2::{ClassRef, FnSig, MethodRef, Type},
};
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NodeIdx(BiMapIndex);
impl IntoBiMapIndex for NodeIdx {
    fn from_index(val: BiMapIndex) -> Self {
        Self(val)
    }
    fn as_bimap_index(&self) -> BiMapIndex {
        self.0
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum CILNode {
    Const(Box<Const>),
    BinOp(NodeIdx, NodeIdx, BinOp),
    UnOp(NodeIdx, UnOp),
    LdLoc(u32),
    LdLocA(u32),
    LdArg(u32),
    LdArgA(u32),
    Call(Box<(MethodRefIdx, Box<[NodeIdx]>)>),
    IntCast {
        input: NodeIdx,
        target: Int,
        extend: ExtendKind,
    },
    FloatCast {
        input: NodeIdx,
        target: Float,
        is_signed: bool,
    },
    RefToPtr(NodeIdx),
    /// Changes the type of a pointer to `PtrCastRes`
    PtrCast(NodeIdx, Box<PtrCastRes>),
    /// Loads the address of a field at `addr`
    LdFieldAdress {
        addr: NodeIdx,
        field: FieldIdx,
    },
    /// Loads the value of a field at `addr`
    LdField {
        addr: NodeIdx,
        field: FieldIdx,
    },
    /// Loads a value of `tpe` at `addr`
    LdInd {
        addr: NodeIdx,
        tpe: TypeIdx,
        volitale: bool,
    },
    /// Calcualtes the size of a type.
    SizeOf(TypeIdx),
    /// Gets the currenrt exception, if it exisits. UB outside an exception handler.
    GetException,
    /// Checks if the object is an instace of a class.
    IsInst(NodeIdx, TypeIdx),
    /// Casts  the object to instace of a clsass.
    CheckedCast(NodeIdx, TypeIdx),
    /// Calls fn pointer with args
    CallI(Box<(NodeIdx, SigIdx, Box<[NodeIdx]>)>),
    /// Allocates memory from a local pool. It will get freed when this function return
    LocAlloc {
        size: NodeIdx,
    },
    /// Loads a static field at descr
    LdStaticField(StaticFieldIdx),
    /// Loads a pointer to a function
    LdFtn(MethodRefIdx),
    /// Loads a "type token"
    LdTypeToken(TypeIdx),
    /// Gets the length of a platform array
    LdLen(NodeIdx),
    /// Allocates a local buffer sizeof type, and aligned to algin.
    LocAllocAlgined {
        tpe: TypeIdx,
        align: u64,
    },
    /// Loads a reference to array element at index.
    LdElelemRef {
        array: NodeIdx,
        index: NodeIdx,
    },
    /// Turns a managed reference to object into type
    UnboxAny {
        object: NodeIdx,
        tpe: TypeIdx,
    },
}
#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum PtrCastRes {
    Ptr(TypeIdx),
    Ref(TypeIdx),
    FnPtr(SigIdx),
    USize,
    ISize,
}
#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]

pub enum ExtendKind {
    ZeroExtend,
    SignExtend,
}
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum MethodKind {
    Static,
    Instance,
    Virtual,
    Constructor,
}
#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum UnOp {
    Not,
    Neg,
}
#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize)]

pub enum BinOp {
    Add,
    Eq,
    Sub,
    Mul,
    LtUn,
    Lt,
    GtUn,
    Gt,
    Or,
    XOr,
    And,
    Rem,
    RemUn,
    Shl,
    Shr,
    ShrUn,
    DivUn,
    Div,
}
impl CILNode {
    pub fn get_type(
        &self,
        sig: SigIdx,
        locals: &[(Option<StringIdx>, TypeIdx)],
        asm: &Assembly,
    ) -> Result<Type, IString> {
        match self {
            CILNode::Const(cst) => Ok(cst.as_ref().get_type()),
            CILNode::BinOp(lhs, rhs, BinOp::Add | BinOp::Sub | BinOp::Mul) => {
                let lhs = asm.get_node(*lhs);
                let rhs = asm.get_node(*rhs);
                let lhs = lhs.get_type(sig, locals, asm)?;
                let rhs = rhs.get_type(sig, locals, asm)?;
                if lhs != rhs {
                    match (rhs, lhs) {
                        (Type::Int(Int::USize | Int::ISize), Type::Ptr(_)) => Ok(rhs),
                        (Type::Ptr(_), Type::Int(Int::USize | Int::ISize)) => Ok(lhs),
                        _ => Err(format!("mismatched binop args. {lhs:?} != {rhs:?}").into()),
                    }
                } else {
                    Ok(lhs)
                }
            }
            CILNode::BinOp(lhs, rhs, op) => todo!("op:{op:?}"),
            CILNode::UnOp(_, _) => todo!(),
            CILNode::LdLoc(_) => todo!(),
            CILNode::LdLocA(_) => todo!(),
            CILNode::LdArg(_) => todo!(),
            CILNode::LdArgA(_) => todo!(),
            CILNode::Call(_) => todo!(),
            CILNode::IntCast {
                input,
                target,
                extend,
            } => todo!(),
            CILNode::FloatCast {
                input,
                target,
                is_signed,
            } => todo!(),
            CILNode::RefToPtr(_) => todo!(),
            CILNode::PtrCast(_, _) => todo!(),
            CILNode::LdFieldAdress { addr, field } => todo!(),
            CILNode::LdField { addr, field } => todo!(),
            CILNode::LdInd {
                addr,
                tpe,
                volitale,
            } => todo!(),
            CILNode::SizeOf(_) => todo!(),
            CILNode::GetException => todo!(),
            CILNode::IsInst(_, _) => todo!(),
            CILNode::CheckedCast(_, _) => todo!(),
            CILNode::CallI(_) => todo!(),
            CILNode::LocAlloc { size } => todo!(),
            CILNode::LdStaticField(_) => todo!(),
            CILNode::LdFtn(_) => todo!(),
            CILNode::LdTypeToken(_) => todo!(),
            CILNode::LdLen(_) => todo!(),
            CILNode::LocAllocAlgined { tpe, align } => todo!(),
            CILNode::LdElelemRef { array, index } => todo!(),
            CILNode::UnboxAny { object, tpe } => todo!(),
        }
    }
}
impl CILNode {
    pub fn from_v1(v1: &V1Node, asm: &mut Assembly) -> Self {
        match v1 {
            // Varaible access
            V1Node::LDArg(arg) => CILNode::LdArg(*arg),
            V1Node::LDLoc(arg) => CILNode::LdLoc(*arg),
            V1Node::LDArgA(arg) => CILNode::LdArgA(*arg),
            V1Node::LDLocA(arg) => CILNode::LdLocA(*arg),
            // Ptr deref
            V1Node::LDIndBool { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Bool),
                    volitale: false,
                }
            }
            V1Node::LDIndU8 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::U8)),
                    volitale: false,
                }
            }
            V1Node::LDIndU16 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::U16)),
                    volitale: false,
                }
            }
            V1Node::LDIndU32 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::U32)),
                    volitale: false,
                }
            }
            V1Node::LDIndU64 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::U64)),
                    volitale: false,
                }
            }
            V1Node::LDIndUSize { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::USize)),
                    volitale: false,
                }
            }
            V1Node::LDIndI8 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::I8)),
                    volitale: false,
                }
            }
            V1Node::LDIndI16 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::I16)),
                    volitale: false,
                }
            }
            V1Node::LDIndI32 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::I32)),
                    volitale: false,
                }
            }
            V1Node::LDIndI64 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::I64)),
                    volitale: false,
                }
            }
            V1Node::LDIndISize { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Int(Int::ISize)),
                    volitale: false,
                }
            }
            V1Node::LDIndF32 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Float(Float::F32)),
                    volitale: false,
                }
            }
            V1Node::LDIndF64 { ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(Type::Float(Float::F64)),
                    volitale: false,
                }
            }
            V1Node::LdObj { ptr, obj } => {
                let obj = Type::from_v1(obj, asm);
                let ptr = Self::from_v1(ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(obj),
                    volitale: false,
                }
            }
            V1Node::LDIndPtr { ptr, loaded_ptr } => {
                let ptr = Self::from_v1(ptr, asm);
                let loaded_ptr = Type::from_v1(loaded_ptr, asm);
                Self::LdInd {
                    addr: asm.alloc_node(ptr),
                    tpe: asm.alloc_type(loaded_ptr),
                    volitale: false,
                }
            }
            // Casts
            V1Node::ZeroExtendToU64(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::U64,
                    extend: ExtendKind::ZeroExtend,
                }
            }
            V1Node::SignExtendToU64(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::U64,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ZeroExtendToUSize(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::USize,
                    extend: ExtendKind::ZeroExtend,
                }
            }
            V1Node::SignExtendToUSize(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::USize,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ConvU8(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::U8,
                    extend: ExtendKind::ZeroExtend,
                }
            }
            V1Node::ConvU16(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::U16,
                    extend: ExtendKind::ZeroExtend,
                }
            }
            V1Node::ConvU32(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::U32,
                    extend: ExtendKind::ZeroExtend,
                }
            }
            V1Node::SignExtendToI64(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::I64,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ZeroExtendToISize(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::ISize,
                    extend: ExtendKind::ZeroExtend,
                }
            }
            V1Node::SignExtendToISize(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::ISize,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ConvI8(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::I8,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ConvI16(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::I16,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ConvI32(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::IntCast {
                    input: asm.alloc_node(node),
                    target: Int::I32,
                    extend: ExtendKind::SignExtend,
                }
            }
            V1Node::ConvF32(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::FloatCast {
                    input: asm.alloc_node(node),
                    target: Float::F32,
                    is_signed: true,
                }
            }
            V1Node::ConvF64(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::FloatCast {
                    input: asm.alloc_node(node),
                    target: Float::F64,
                    is_signed: true,
                }
            }
            V1Node::ConvF64Un(inner) => {
                let node = Self::from_v1(inner, asm);
                CILNode::FloatCast {
                    input: asm.alloc_node(node),
                    target: Float::F64,
                    is_signed: false,
                }
            }
            V1Node::MRefToRawPtr(inner) => {
                let raw = Self::from_v1(inner, asm);
                CILNode::RefToPtr(asm.alloc_node(raw))
            }
            V1Node::CastPtr { val, new_ptr } => {
                let val = Self::from_v1(val, asm);

                let ptr = match &**new_ptr {
                    V1Type::USize => PtrCastRes::USize,
                    V1Type::ISize => PtrCastRes::ISize,
                    V1Type::Ptr(inner) => {
                        let inner = Type::from_v1(inner, asm);
                        PtrCastRes::Ptr(asm.alloc_type(inner))
                    }
                    V1Type::ManagedReference(inner) => {
                        let inner = Type::from_v1(inner, asm);
                        PtrCastRes::Ref(asm.alloc_type(inner))
                    }
                    V1Type::DelegatePtr(sig) => {
                        let sig = FnSig::from_v1(sig, asm);
                        let sig = asm.alloc_sig(sig);
                        PtrCastRes::FnPtr(sig)
                    }
                    _ => panic!("Type {new_ptr:?} is not a pointer."),
                };
                CILNode::PtrCast(asm.alloc_node(val), Box::new(ptr))
            }
            // Binops
            V1Node::Add(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Add)
            }
            V1Node::Sub(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Sub)
            }
            V1Node::Mul(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Mul)
            }
            V1Node::Eq(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Eq)
            }
            V1Node::Or(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Or)
            }
            V1Node::XOr(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::XOr)
            }
            V1Node::And(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::And)
            }
            V1Node::LtUn(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::LtUn)
            }
            V1Node::Lt(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Lt)
            }
            V1Node::GtUn(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::GtUn)
            }
            V1Node::Gt(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Gt)
            }
            V1Node::Rem(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Rem)
            }
            V1Node::RemUn(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::RemUn)
            }
            V1Node::Shl(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Shl)
            }
            V1Node::Shr(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Shr)
            }
            V1Node::ShrUn(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::ShrUn)
            }
            V1Node::Div(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::Div)
            }
            V1Node::DivUn(lhs, rhs) => {
                let lhs = Self::from_v1(lhs, asm);
                let rhs = Self::from_v1(rhs, asm);
                Self::BinOp(asm.alloc_node(lhs), asm.alloc_node(rhs), BinOp::DivUn)
            }
            // Unops
            V1Node::Not(val) => {
                let val = Self::from_v1(val, asm);
                Self::UnOp(asm.alloc_node(val), UnOp::Not)
            }
            V1Node::Neg(val) => {
                let val = Self::from_v1(val, asm);
                Self::UnOp(asm.alloc_node(val), UnOp::Neg)
            }
            // Field access
            V1Node::LDField { addr, field } => {
                let field = FieldDesc::from_v1(field, asm);
                let field = asm.alloc_field(field);
                let addr = Self::from_v1(addr, asm);
                Self::LdField {
                    addr: asm.alloc_node(addr),
                    field,
                }
            }
            V1Node::LDFieldAdress { addr, field } => {
                let field = FieldDesc::from_v1(field, asm);
                let field = asm.alloc_field(field);
                let addr = Self::from_v1(addr, asm);
                Self::LdFieldAdress {
                    addr: asm.alloc_node(addr),
                    field,
                }
            }
            // Calls
            V1Node::Call(callargs) => {
                let args: Box<[_]> = callargs
                    .args
                    .iter()
                    .map(|arg| {
                        let node = Self::from_v1(arg, asm);
                        asm.alloc_node(node)
                    })
                    .collect();
                let sig = FnSig::from_v1(callargs.site.signature(), asm);
                let sig = asm.alloc_sig(sig);
                let generics: Box<[_]> = callargs
                    .site
                    .generics()
                    .iter()
                    .map(|gen| Type::from_v1(gen, asm))
                    .collect();
                let class = callargs
                    .site
                    .class()
                    .map(|dt| {
                        let cref = ClassRef::from_v1(dt, asm);
                        asm.alloc_class_ref(cref)
                    })
                    .unwrap_or_else(|| *asm.main_module());
                let name = asm.alloc_string(callargs.site.name());
                let method_ref = if callargs.site.is_static() {
                    MethodRef::new(class, name, sig, MethodKind::Static, generics)
                } else {
                    MethodRef::new(class, name, sig, MethodKind::Instance, generics)
                };
                let method_ref = asm.alloc_methodref(method_ref);
                Self::Call(Box::new((method_ref, args)))
            }
            V1Node::CallVirt(callargs) => {
                let args: Box<[_]> = callargs
                    .args
                    .iter()
                    .map(|arg| {
                        let node = Self::from_v1(arg, asm);
                        asm.alloc_node(node)
                    })
                    .collect();
                let sig = FnSig::from_v1(callargs.site.signature(), asm);
                let sig = asm.alloc_sig(sig);
                let generics: Box<[_]> = callargs
                    .site
                    .generics()
                    .iter()
                    .map(|gen| Type::from_v1(gen, asm))
                    .collect();
                let class = callargs
                    .site
                    .class()
                    .map(|dt| {
                        let cref = ClassRef::from_v1(dt, asm);
                        asm.alloc_class_ref(cref)
                    })
                    .unwrap_or_else(|| *asm.main_module());
                let name = asm.alloc_string(callargs.site.name());
                assert!(!callargs.site.is_static());
                let method_ref = MethodRef::new(class, name, sig, MethodKind::Virtual, generics);
                let method_ref = asm.alloc_methodref(method_ref);
                Self::Call(Box::new((method_ref, args)))
            }
            V1Node::NewObj(callargs) => {
                let args: Box<[_]> = callargs
                    .args
                    .iter()
                    .map(|arg| {
                        let node = Self::from_v1(arg, asm);
                        asm.alloc_node(node)
                    })
                    .collect();
                let sig = FnSig::from_v1(callargs.site.signature(), asm);
                let sig = asm.alloc_sig(sig);
                let generics: Box<[_]> = callargs
                    .site
                    .generics()
                    .iter()
                    .map(|gen| Type::from_v1(gen, asm))
                    .collect();
                let class = callargs
                    .site
                    .class()
                    .map(|dt| {
                        let cref = ClassRef::from_v1(dt, asm);
                        asm.alloc_class_ref(cref)
                    })
                    .unwrap_or_else(|| *asm.main_module());
                let name = asm.alloc_string(callargs.site.name());
                assert!(
                    !callargs.site.is_static(),
                    "Newobj site invalid(is static):{:?}",
                    callargs.site
                );
                let method_ref =
                    MethodRef::new(class, name, sig, MethodKind::Constructor, generics);
                let method_ref = asm.alloc_methodref(method_ref);
                Self::Call(Box::new((method_ref, args)))
            }
            // Special
            V1Node::GetException => Self::GetException,
            // Consts
            V1Node::LdStr(string) => {
                let string = asm.alloc_string(string.clone());
                Const::PlatformString(string).into()
            }
            V1Node::SizeOf(tpe) => {
                let tpe = Type::from_v1(tpe, asm);
                Self::SizeOf(asm.alloc_type(tpe))
            }
            V1Node::LDTypeToken(tpe) => {
                let tpe = Type::from_v1(tpe, asm);
                Self::LdTypeToken(asm.alloc_type(tpe))
            }
            V1Node::LdcU64(val) => Const::U64(*val).into(),
            V1Node::LdcU32(val) => Const::U32(*val).into(),
            V1Node::LdcU16(val) => Const::U16(*val).into(),
            V1Node::LdcU8(val) => Const::U8(*val).into(),
            V1Node::LdcI64(val) => Const::I64(*val).into(),
            V1Node::LdcI32(val) => Const::I32(*val).into(),
            V1Node::LdcI16(val) => Const::I16(*val).into(),
            V1Node::LdcI8(val) => Const::I8(*val).into(),
            V1Node::LdFalse => Const::Bool(false).into(),
            V1Node::LdTrue => Const::Bool(true).into(),
            V1Node::LdcF64(val) => Const::F64(*val).into(),
            V1Node::LdcF32(val) => Const::F32(*val).into(),
            // Special
            V1Node::IsInst(combined) => {
                let (val, tpe) = combined.as_ref();
                let tpe = ClassRef::from_v1(tpe, asm);
                let tpe = asm.alloc_class_ref(tpe);
                let tpe = asm.alloc_type(tpe);
                let val = Self::from_v1(val, asm);

                Self::IsInst(asm.alloc_node(val), tpe)
            }
            V1Node::CheckedCast(combined) => {
                let (val, tpe) = combined.as_ref();
                let tpe = ClassRef::from_v1(tpe, asm);
                let tpe = asm.alloc_class_ref(tpe);
                let tpe = asm.alloc_type(tpe);
                let val = Self::from_v1(val, asm);

                Self::CheckedCast(asm.alloc_node(val), tpe)
            }
            V1Node::CallI(sig_ptr_args) => {
                let sig = FnSig::from_v1(&sig_ptr_args.0, asm);
                let sig = asm.alloc_sig(sig);
                let ptr = Self::from_v1(&sig_ptr_args.1, asm);
                let ptr = asm.alloc_node(ptr);
                let args: Box<[_]> = sig_ptr_args
                    .2
                    .iter()
                    .map(|arg| {
                        let arg = Self::from_v1(arg, asm);
                        asm.alloc_node(arg)
                    })
                    .collect();
                Self::CallI(Box::new((ptr, sig, args)))
            }
            V1Node::LocAlloc { size } => {
                let size = Self::from_v1(size, asm);
                let size = asm.alloc_node(size);
                CILNode::LocAlloc { size }
            }
            V1Node::LocAllocAligned { tpe, align } => {
                let tpe = Type::from_v1(tpe, asm);
                let tpe = asm.alloc_type(tpe);
                CILNode::LocAllocAlgined { tpe, align: *align }
            }
            V1Node::LDStaticField(sfld) => {
                let sfld = StaticFieldDesc::from_v1(sfld, asm);
                Self::LdStaticField(asm.alloc_sfld(sfld))
            }
            V1Node::LDFtn(site) => {
                let sig = FnSig::from_v1(site.signature(), asm);
                let sig = asm.alloc_sig(sig);
                let generics: Box<[_]> = site
                    .generics()
                    .iter()
                    .map(|gen| Type::from_v1(gen, asm))
                    .collect();
                let class = site
                    .class()
                    .map(|dt| {
                        let cref = ClassRef::from_v1(dt, asm);
                        asm.alloc_class_ref(cref)
                    })
                    .unwrap_or_else(|| *asm.main_module());
                let name = asm.alloc_string(site.name());

                let method_ref = if site.is_static() {
                    MethodRef::new(class, name, sig, MethodKind::Static, generics)
                } else {
                    MethodRef::new(class, name, sig, MethodKind::Instance, generics)
                };
                let method_ref = asm.alloc_methodref(method_ref);
                Self::LdFtn(method_ref)
            }
            V1Node::Volatile(inner) => {
                let mut tmp = Self::from_v1(inner, asm);
                match &mut tmp {
                    Self::LdInd { volitale, .. } => *volitale = true,
                    _ => panic!(),
                }
                tmp
            }
            V1Node::LDLen { arr } => {
                let arr = Self::from_v1(arr, asm);
                let arr = asm.alloc_node(arr);
                Self::LdLen(arr)
            }
            V1Node::LDElelemRef { arr, idx } => {
                let arr = Self::from_v1(arr, asm);
                let array = asm.alloc_node(arr);
                let idx = Self::from_v1(idx, asm);
                let index = asm.alloc_node(idx);
                Self::LdElelemRef { array, index }
            }
            V1Node::UnboxAny(object, tpe) => {
                let object = Self::from_v1(object, asm);
                let object = asm.alloc_node(object);
                let tpe = Type::from_v1(tpe, asm);
                let tpe = asm.alloc_type(tpe);
                Self::UnboxAny { object, tpe }
            }
            _ => todo!("v1:{v1:?}"),
        }
    }
}
