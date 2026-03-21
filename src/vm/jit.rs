use std::collections::HashMap;
use crate::vm::chunk::{Chunk, Const};
use crate::vm::opcode::Op;

pub struct JitCache {
    pub compiled: HashMap<String, CompiledFn>,
    pub call_counts: HashMap<String, u32>,
    pub threshold: u32,
}

pub struct CompiledFn {
    pub ptr: *const u8,
    pub size: usize,
    pub param_count: usize,
}

unsafe impl Send for CompiledFn {}
unsafe impl Sync for CompiledFn {}

impl Drop for CompiledFn {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { platform::free_exec(self.ptr as *mut u8, self.size); }
        }
    }
}

impl JitCache {
    pub fn new() -> Self {
        JitCache { compiled: HashMap::new(), call_counts: HashMap::new(), threshold: 50 }
    }
    pub fn record_call(&mut self, name: &str) {
        *self.call_counts.entry(name.to_string()).or_insert(0) += 1;
    }
    pub fn should_compile(&self, name: &str) -> bool {
        self.call_counts.get(name).copied().unwrap_or(0) >= self.threshold
            && !self.compiled.contains_key(name)
    }
}

mod platform {
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn alloc_exec(size: usize) -> *mut u8 {
        extern "C" { fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut u8; }
        let ptr = mmap(std::ptr::null_mut(), size, 3, 0x22, -1, 0);
        if ptr as isize == -1 { std::ptr::null_mut() } else { ptr }
    }
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn make_exec(ptr: *mut u8, size: usize) {
        extern "C" { fn mprotect(addr: *mut u8, len: usize, prot: i32) -> i32; }
        mprotect(ptr, size, 5);
    }
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn free_exec(ptr: *mut u8, size: usize) {
        extern "C" { fn munmap(addr: *mut u8, len: usize) -> i32; }
        munmap(ptr, size);
    }
    #[cfg(not(target_arch = "aarch64"))]
    pub unsafe fn alloc_exec(_: usize) -> *mut u8 { std::ptr::null_mut() }
    #[cfg(not(target_arch = "aarch64"))]
    pub unsafe fn make_exec(_: *mut u8, _: usize) {}
    #[cfg(not(target_arch = "aarch64"))]
    pub unsafe fn free_exec(_: *mut u8, _: usize) {}
}

struct Asm { code: Vec<u8> }

impl Asm {
    fn new() -> Self { Asm { code: Vec::new() } }
    fn emit(&mut self, v: u32) { self.code.extend_from_slice(&v.to_le_bytes()); }
    fn len(&self) -> usize { self.code.len() }

    fn prologue(&mut self, frame_bytes: u32) {
        let aligned = (frame_bytes + 15) & !15;
        self.emit(0xA9BF7BFD); // stp x29, x30, [sp, #-16]!
        self.emit(0x910003FD); // mov x29, sp
        if aligned > 0 {
            self.emit(0xD10003FF | ((aligned & 0xFFF) << 10)); // sub sp, sp, #n
        }
        // x19 = our eval stack pointer, starts at sp
        self.emit(0x910003F3); // mov x19, sp
    }

    fn epilogue(&mut self) {
        self.emit(0x910003BF); // mov sp, x29
        self.emit(0xA8C17BFD); // ldp x29, x30, [sp], #16
        self.emit(0xD65F03C0); // ret
    }

    fn push_x0(&mut self) { self.emit(0xF81F8260); } // str x0, [x19, #-8]!
    fn pop_x0(&mut self)  { self.emit(0xF8408260); } // ldr x0, [x19], #8
    fn pop_x1(&mut self)  { self.emit(0xF8408261); } // ldr x1, [x19], #8

    // locals stored at [x29, -8*(slot+1)]
    fn load_local(&mut self, slot: u32) {
        let off = (slot + 1) * 8;
        // ldr x0, [x29, #-off] — use unscaled offset (LDUR)
        let simm9 = (-(off as i64) as u64) & 0x1FF;
        self.emit(0xF8400000 | (simm9 << 12) as u32 | (29 << 5) | 0);
    }
    fn store_local_xN(&mut self, slot: u32, reg: u32) {
        let off = (slot + 1) * 8;
        let simm9 = (-(off as i64) as u64) & 0x1FF;
        self.emit(0xF8000000 | (simm9 << 12) as u32 | (29 << 5) | reg);
    }
    fn store_local(&mut self, slot: u32) { self.store_local_xN(slot, 0); }

    fn mov_x0_imm(&mut self, n: i64) {
        let u = n as u64;
        let w0 = (u) & 0xFFFF;
        let w1 = (u >> 16) & 0xFFFF;
        let w2 = (u >> 32) & 0xFFFF;
        let w3 = (u >> 48) & 0xFFFF;
        if n >= 0 {
            self.emit(0xD2800000 | ((w0 as u32) << 5)); // movz x0, w0
            if w1 != 0 { self.emit(0xF2A00000 | ((w1 as u32) << 5)); } // movk x0, w1, lsl 16
            if w2 != 0 { self.emit(0xF2C00000 | ((w2 as u32) << 5)); }
            if w3 != 0 { self.emit(0xF2E00000 | ((w3 as u32) << 5)); }
        } else {
            // use movn for negatives
            let inv = (!n) as u64;
            let iw0 = inv & 0xFFFF;
            self.emit(0x92800000 | ((iw0 as u32) << 5)); // movn x0, ~n
            let iw1 = (inv >> 16) & 0xFFFF;
            if iw1 != 0xFFFF { self.emit(0xF2A00000 | (((n as u64 >> 16) & 0xFFFF) as u32) << 5); }
        }
    }

    fn mov_xN_imm(&mut self, reg: u32, n: i64) {
        let u = n as u64;
        let w0 = u & 0xFFFF;
        self.emit(0xD2800000 | ((w0 as u32) << 5) | reg);
        let w1 = (u >> 16) & 0xFFFF;
        if w1 != 0 { self.emit(0xF2A00000 | ((w1 as u32) << 5) | reg); }
        let w2 = (u >> 32) & 0xFFFF;
        if w2 != 0 { self.emit(0xF2C00000 | ((w2 as u32) << 5) | reg); }
        let w3 = (u >> 48) & 0xFFFF;
        if w3 != 0 { self.emit(0xF2E00000 | ((w3 as u32) << 5) | reg); }
    }

    // arithmetic: pop two, operate, push result
    fn arith_add(&mut self) { self.pop_x1(); self.pop_x0(); self.emit(0x8B010000); self.push_x0(); } // add x0,x0,x1
    fn arith_sub(&mut self) { self.pop_x0(); self.pop_x1(); self.emit(0xCB000020); self.push_x0(); } // sub x0,x1,x0
    fn arith_mul(&mut self) { self.pop_x1(); self.pop_x0(); self.emit(0x9B017C00); self.push_x0(); } // mul x0,x0,x1
    fn arith_div(&mut self) { self.pop_x0(); self.pop_x1(); self.emit(0x9AC00C20); self.push_x0(); } // sdiv x0,x1,x0
    fn arith_mod(&mut self) {
        self.pop_x0(); self.pop_x1();
        self.emit(0x9AC00C22); // sdiv x2, x1, x0
        self.emit(0x9B008440); // msub x0, x2, x0, x1  => x0 = x1 - x2*x0
        self.push_x0();
    }
    fn arith_neg(&mut self) { self.pop_x0(); self.emit(0xCB0003E0); self.push_x0(); } // neg x0,x0

    fn cmp_op(&mut self, cond: u32) {
        self.pop_x0(); self.pop_x1();
        self.emit(0xEB00003F); // cmp x1, x0
        self.emit(0x9A9F07E0 | (cond << 12)); // cset x0, cond
        self.push_x0();
    }
    fn cmp_lt(&mut self)  { self.cmp_op(0xB); }
    fn cmp_gt(&mut self)  { self.cmp_op(0xC); }
    fn cmp_le(&mut self)  { self.cmp_op(0xD); }
    fn cmp_ge(&mut self)  { self.cmp_op(0xA); }
    fn cmp_eq(&mut self)  { self.cmp_op(0x0); }
    fn cmp_ne(&mut self)  { self.cmp_op(0x1); }

    fn cbz_x0(&mut self) -> usize { let p = self.len(); self.emit(0xB4000000); p }
    fn b_fwd(&mut self) -> usize  { let p = self.len(); self.emit(0x14000000); p }

    fn patch_cbz(&mut self, at: usize, to: usize) {
        let off = ((to as i64 - at as i64) / 4) as i32;
        let v = 0xB4000000u32 | (((off as u32) & 0x7FFFF) << 5);
        self.code[at..at+4].copy_from_slice(&v.to_le_bytes());
    }
    fn patch_b(&mut self, at: usize, to: usize) {
        let off = ((to as i64 - at as i64) / 4) as i32;
        let v = 0x14000000u32 | ((off as u32) & 0x03FFFFFF);
        self.code[at..at+4].copy_from_slice(&v.to_le_bytes());
    }
}

fn is_jit_eligible(chunk: &Chunk) -> bool {
    let code = &chunk.code;
    let mut ip = 0;
    while ip < code.len() {
        let b = code[ip]; ip += 1;
        match Op::from_u8(b) {
            Some(Op::PushInt) | Some(Op::PushIntInline) => { ip += 2; }
            Some(Op::LoadLocal) | Some(Op::StoreLocal) => { ip += 2; }
            Some(Op::Add) | Some(Op::Sub) | Some(Op::Mul) | Some(Op::Div) | Some(Op::Mod) | Some(Op::Neg) => {}
            Some(Op::Lt) | Some(Op::Gt) | Some(Op::LtEq) | Some(Op::GtEq) | Some(Op::Eq) | Some(Op::Neq) => {}
            Some(Op::JumpIfNot) | Some(Op::Jump) => { ip += 2; }
            Some(Op::Return) | Some(Op::Pop) | Some(Op::PushNull) => {}
            Some(Op::Call) => { ip += 3; }
            _ => return false,
        }
    }
    true
}

fn count_locals(chunk: &Chunk, param_count: usize) -> usize {
    let mut max = param_count;
    let code = &chunk.code;
    let mut ip = 0;
    while ip < code.len() {
        let b = code[ip]; ip += 1;
        match Op::from_u8(b) {
            Some(Op::LoadLocal) | Some(Op::StoreLocal) => {
                let s = u16::from_le_bytes([code[ip], code[ip+1]]) as usize; ip += 2;
                if s + 1 > max { max = s + 1; }
            }
            Some(Op::PushInt) | Some(Op::PushIntInline) |
            Some(Op::LoadGlobal) | Some(Op::StoreGlobal) |
            Some(Op::Jump) | Some(Op::JumpIf) | Some(Op::JumpIfNot) => { ip += 2; }
            Some(Op::Call) => { ip += 3; }
            _ => {}
        }
    }
    max
}

pub fn try_jit_compile(
    chunk: &Chunk,
    param_count: usize,
    get_ptr: &dyn Fn(&str) -> Option<*const u8>,
) -> Option<(*const u8, usize)> {
    if !is_jit_eligible(chunk) { return None; }

    let n_locals = count_locals(chunk, param_count);
    // frame: n_locals*8 for locals + 64 for eval stack scratch (max 8 deep)
    let frame_bytes = ((n_locals as u32 + 8) * 8 + 15) & !15;

    let mut asm = Asm::new();
    let mut ip_map: HashMap<usize, usize> = HashMap::new();
    let mut patches: Vec<(usize, usize, bool)> = Vec::new(); // (asm_pos, bc_target, is_cbz)

    asm.prologue(frame_bytes);

    // store incoming args (x0..x{n-1}) into locals
    for i in 0..param_count.min(8) {
        asm.store_local_xN(i as u32, i as u32);
    }

    let code = &chunk.code;
    let mut ip = 0;
    while ip < code.len() {
        ip_map.insert(ip, asm.len());
        let b = code[ip]; ip += 1;
        match Op::from_u8(b)? {
            Op::PushInt => {
                let idx = u16::from_le_bytes([code[ip], code[ip+1]]) as usize; ip += 2;
                if let Const::Int(n) = &chunk.consts[idx] { asm.mov_x0_imm(*n); asm.push_x0(); }
                else { return None; }
            }
            Op::PushIntInline => {
                let v = u16::from_le_bytes([code[ip], code[ip+1]]) as i64 - 32768; ip += 2;
                asm.mov_x0_imm(v); asm.push_x0();
            }
            Op::PushNull => { asm.mov_x0_imm(0); asm.push_x0(); }
            Op::Pop => { asm.pop_x0(); }
            Op::LoadLocal => {
                let s = u16::from_le_bytes([code[ip], code[ip+1]]) as u32; ip += 2;
                asm.load_local(s); asm.push_x0();
            }
            Op::StoreLocal => {
                let s = u16::from_le_bytes([code[ip], code[ip+1]]) as u32; ip += 2;
                asm.pop_x0(); asm.store_local(s);
            }
            Op::Add => asm.arith_add(),
            Op::Sub => asm.arith_sub(),
            Op::Mul => asm.arith_mul(),
            Op::Div => asm.arith_div(),
            Op::Mod => asm.arith_mod(),
            Op::Neg => asm.arith_neg(),
            Op::Lt  => asm.cmp_lt(),
            Op::Gt  => asm.cmp_gt(),
            Op::LtEq => asm.cmp_le(),
            Op::GtEq => asm.cmp_ge(),
            Op::Eq  => asm.cmp_eq(),
            Op::Neq => asm.cmp_ne(),
            Op::JumpIfNot => {
                let tgt = u16::from_le_bytes([code[ip], code[ip+1]]) as usize; ip += 2;
                asm.pop_x0();
                let p = asm.cbz_x0();
                patches.push((p, tgt, true));
            }
            Op::Jump => {
                let tgt = u16::from_le_bytes([code[ip], code[ip+1]]) as usize; ip += 2;
                let p = asm.b_fwd();
                patches.push((p, tgt, false));
            }
            Op::Call => {
                let ni = u16::from_le_bytes([code[ip], code[ip+1]]) as usize; ip += 2;
                let argc = code[ip] as usize; ip += 1;
                let name = if let Const::Str(s) = &chunk.consts[ni] { s.clone() } else { return None; };
                let fptr = get_ptr(&name)?;
                // save x19 (callee-saved, fine), save eval stack state
                // pop args into temp locals, then load into x0..xN
                let tmp_base = n_locals as u32;
                // pop argc args from eval stack into tmp locals (they're in reverse)
                for i in (0..argc.min(8)).rev() {
                    asm.pop_x0();
                    asm.store_local(tmp_base + i as u32);
                }
                // load into argument registers
                for i in 0..argc.min(8) {
                    asm.load_local(tmp_base + i as u32);
                    if i > 0 {
                        // mov x{i}, x0
                        asm.emit(0xAA0003E0 | (i as u32));
                    }
                }
                // load function pointer into x9 (temp), call via blr
                let ptr_val = fptr as u64;
                asm.emit(0xD2800009 | (((ptr_val & 0xFFFF) as u32) << 5));
                if (ptr_val >> 16) & 0xFFFF != 0 { asm.emit(0xF2A00009 | ((((ptr_val>>16) & 0xFFFF) as u32) << 5)); }
                if (ptr_val >> 32) & 0xFFFF != 0 { asm.emit(0xF2C00009 | ((((ptr_val>>32) & 0xFFFF) as u32) << 5)); }
                if (ptr_val >> 48) & 0xFFFF != 0 { asm.emit(0xF2E00009 | ((((ptr_val>>48) & 0xFFFF) as u32) << 5)); }
                asm.emit(0xD63F0120); // blr x9
                asm.push_x0();
            }
            Op::Return => {
                asm.pop_x0();
                asm.epilogue();
            }
            _ => return None,
        }
    }

    // patch jumps
    for (pos, tgt_bc, is_cbz) in patches {
        let tgt_asm = *ip_map.get(&tgt_bc)?;
        if is_cbz { asm.patch_cbz(pos, tgt_asm); }
        else { asm.patch_b(pos, tgt_asm); }
    }

    // alloc executable memory
    let size = (asm.code.len() + 4095) & !4095;
    unsafe {
        let ptr = platform::alloc_exec(size);
        if ptr.is_null() { return None; }
        std::ptr::copy_nonoverlapping(asm.code.as_ptr(), ptr, asm.code.len());
        platform::make_exec(ptr, size);
        Some((ptr as *const u8, size))
    }
}

pub unsafe fn jit_call(ptr: *const u8, args: &[i64]) -> i64 {
    match args.len() {
        0 => { let f: extern "C" fn() -> i64 = std::mem::transmute(ptr); f() }
        1 => { let f: extern "C" fn(i64) -> i64 = std::mem::transmute(ptr); f(args[0]) }
        2 => { let f: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(ptr); f(args[0], args[1]) }
        3 => { let f: extern "C" fn(i64, i64, i64) -> i64 = std::mem::transmute(ptr); f(args[0], args[1], args[2]) }
        4 => { let f: extern "C" fn(i64, i64, i64, i64) -> i64 = std::mem::transmute(ptr); f(args[0], args[1], args[2], args[3]) }
        _ => 0
    }
}
