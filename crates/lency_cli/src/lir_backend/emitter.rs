use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueType {
    I64,
    I1,
    Ptr,
    Void,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ExternSig {
    pub(super) arg_tys: Vec<ValueType>,
    pub(super) ret_ty: ValueType,
}

pub(super) struct Emitter {
    pub(super) lines: Vec<String>,
    temp_counter: usize,
    vars: HashSet<String>,
    temps: HashMap<String, ValueType>,
    pub(super) string_globals: HashMap<String, (String, usize, String)>,
    pub(super) extern_funcs: HashMap<String, ExternSig>,
    pub(super) terminated: bool,
    string_global_offset: usize,
}

impl Emitter {
    pub(super) fn new(vars: HashSet<String>) -> Self {
        Self {
            lines: Vec::new(),
            temp_counter: 0,
            vars,
            temps: HashMap::new(),
            string_globals: HashMap::new(),
            extern_funcs: HashMap::new(),
            terminated: false,
            string_global_offset: 0,
        }
    }

    pub(super) fn set_string_global_offset(&mut self, offset: usize) {
        self.string_global_offset = offset;
    }

    pub(super) fn push(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    pub(super) fn next_tmp(&mut self, prefix: &str) -> String {
        let name = format!("%{}_{}", prefix, self.temp_counter);
        self.temp_counter += 1;
        name
    }

    pub(super) fn parse_i64_literal(op: &str) -> Option<i64> {
        if op == "true" {
            return Some(1);
        }
        if op == "false" || op == "null" {
            return Some(0);
        }
        if let Some(v) = Self::parse_char_literal(op) {
            return Some(v);
        }
        op.parse::<i64>().ok()
    }

    fn parse_string_literal(op: &str) -> Option<Vec<u8>> {
        if op.len() < 2 || !op.starts_with('"') || !op.ends_with('"') {
            return None;
        }
        let inner = &op[1..op.len() - 1];
        let mut out = Vec::new();
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.extend_from_slice(ch.to_string().as_bytes());
                continue;
            }
            let escaped = chars.next()?;
            match escaped {
                'n' => out.push(b'\n'),
                'r' => out.push(b'\r'),
                't' => out.push(b'\t'),
                '0' => out.push(0),
                '"' => out.push(b'"'),
                '\\' => out.push(b'\\'),
                _ => return None,
            }
        }
        Some(out)
    }

    fn llvm_c_string(bytes: &[u8]) -> String {
        let mut out = String::new();
        for byte in bytes {
            if (32..=126).contains(byte) && *byte != b'\\' && *byte != b'"' {
                out.push(*byte as char);
            } else {
                out.push_str(&format!("\\{:02X}", byte));
            }
        }
        out.push_str("\\00");
        out
    }

    fn ensure_string_global(&mut self, literal: &str) -> Option<(String, usize)> {
        if let Some((name, len, _)) = self.string_globals.get(literal) {
            return Some((name.clone(), *len));
        }
        let bytes = Self::parse_string_literal(literal)?;
        let global_name = format!(
            "@.str.{}",
            self.string_global_offset + self.string_globals.len()
        );
        let global_len = bytes.len() + 1;
        let decl = format!(
            "{} = private unnamed_addr constant [{} x i8] c\"{}\"",
            global_name,
            global_len,
            Self::llvm_c_string(&bytes)
        );
        self.string_globals
            .insert(literal.to_string(), (global_name.clone(), global_len, decl));
        Some((global_name, global_len))
    }

    fn parse_char_literal(op: &str) -> Option<i64> {
        if op.len() < 3 || !op.starts_with('\'') || !op.ends_with('\'') {
            return None;
        }
        let inner = &op[1..op.len() - 1];
        let value = if let Some(escaped) = inner.strip_prefix('\\') {
            match escaped {
                "n" => '\n',
                "r" => '\r',
                "t" => '\t',
                "0" => '\0',
                "'" => '\'',
                "\\" => '\\',
                _ => return None,
            }
        } else {
            let mut chars = inner.chars();
            let ch = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            ch
        };
        Some(value as i64)
    }

    pub(super) fn ensure_i64(&mut self, repr: String, ty: ValueType) -> (String, ValueType) {
        match ty {
            ValueType::I64 => (repr, ValueType::I64),
            ValueType::I1 => {
                let widened = self.next_tmp("zext");
                self.push(format!("  {} = zext i1 {} to i64", widened, repr));
                (widened, ValueType::I64)
            }
            ValueType::Ptr => {
                let casted = self.next_tmp("ptrtoint");
                self.push(format!("  {} = ptrtoint i8* {} to i64", casted, repr));
                (casted, ValueType::I64)
            }
            ValueType::Void => (String::from("0"), ValueType::I64),
        }
    }

    pub(super) fn ensure_i1(&mut self, repr: String, ty: ValueType) -> (String, ValueType) {
        match ty {
            ValueType::I1 => (repr, ValueType::I1),
            ValueType::I64 => {
                let narrowed = self.next_tmp("to_bool");
                self.push(format!("  {} = icmp ne i64 {}, 0", narrowed, repr));
                (narrowed, ValueType::I1)
            }
            ValueType::Ptr => {
                let narrowed = self.next_tmp("ptr_to_bool");
                self.push(format!("  {} = icmp ne i8* {}, null", narrowed, repr));
                (narrowed, ValueType::I1)
            }
            ValueType::Void => (String::from("false"), ValueType::I1),
        }
    }

    pub(super) fn ensure_ptr(&mut self, repr: String, ty: ValueType) -> (String, ValueType) {
        match ty {
            ValueType::Ptr => (repr, ValueType::Ptr),
            ValueType::I64 => {
                let casted = self.next_tmp("inttoptr");
                self.push(format!("  {} = inttoptr i64 {} to i8*", casted, repr));
                (casted, ValueType::Ptr)
            }
            ValueType::I1 => {
                let (as_i64, _) = self.ensure_i64(repr, ValueType::I1);
                let casted = self.next_tmp("inttoptr");
                self.push(format!("  {} = inttoptr i64 {} to i8*", casted, as_i64));
                (casted, ValueType::Ptr)
            }
            ValueType::Void => {
                let casted = self.next_tmp("inttoptr");
                self.push(format!("  {} = inttoptr i64 0 to i8*", casted));
                (casted, ValueType::Ptr)
            }
        }
    }

    pub(super) fn cast_to_type(
        &mut self,
        repr: String,
        from_ty: ValueType,
        target_ty: ValueType,
    ) -> (String, ValueType) {
        match target_ty {
            ValueType::I64 => self.ensure_i64(repr, from_ty),
            ValueType::I1 => self.ensure_i1(repr, from_ty),
            ValueType::Ptr => self.ensure_ptr(repr, from_ty),
            ValueType::Void => (String::new(), ValueType::Void),
        }
    }

    pub(super) fn emit_operand(&mut self, op: &str) -> Result<(String, ValueType)> {
        if let Some(v) = Self::parse_i64_literal(op) {
            return Ok((v.to_string(), ValueType::I64));
        }
        if let Some((global_name, global_len)) = self.ensure_string_global(op) {
            let gep = self.next_tmp("str");
            self.push(format!(
                "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                gep, global_len, global_len, global_name
            ));
            self.mark_temp(&gep, ValueType::Ptr);
            return Ok((gep, ValueType::Ptr));
        }
        if !op.starts_with('%') {
            bail!("unsupported operand: {}", op);
        }

        if self.vars.contains(op) {
            let loaded = self.next_tmp("load");
            self.push(format!("  {} = load i64, i64* {}.addr", loaded, op));
            return Ok((loaded, ValueType::I64));
        }

        if let Some(ty) = self.temps.get(op).copied() {
            return Ok((op.to_string(), ty));
        }

        bail!("unknown SSA value: {}", op);
    }

    pub(super) fn emit_store_var(&mut self, var: &str, repr: String, ty: ValueType) -> Result<()> {
        if !self.vars.contains(var) {
            bail!("unknown variable: {}", var);
        }
        let (repr, _) = self.ensure_i64(repr, ty);
        self.push(format!("  store i64 {}, i64* {}.addr", repr, var));
        Ok(())
    }

    pub(super) fn mark_temp(&mut self, name: &str, ty: ValueType) {
        self.temps.insert(name.to_string(), ty);
    }

    pub(super) fn note_extern_func(
        &mut self,
        name: &str,
        arg_tys: Vec<ValueType>,
        ret_ty: ValueType,
    ) -> Result<()> {
        if let Some(prev) = self.extern_funcs.get(name) {
            if prev.arg_tys != arg_tys || prev.ret_ty != ret_ty {
                bail!(
                    "extern function '{}' called with inconsistent signature",
                    name
                );
            }
            return Ok(());
        }
        self.extern_funcs
            .insert(name.to_string(), ExternSig { arg_tys, ret_ty });
        Ok(())
    }

    pub(super) fn emit_binary(&mut self, dst: &str, op: &str, lhs: &str, rhs: &str) -> Result<()> {
        let (lhs_repr, lhs_ty) = self.emit_operand(lhs)?;
        let (rhs_repr, rhs_ty) = self.emit_operand(rhs)?;

        match op {
            "concat" => {
                let (lhs_ptr, _) = self.ensure_ptr(lhs_repr, lhs_ty);
                let (rhs_ptr, _) = self.ensure_ptr(rhs_repr, rhs_ty);
                self.note_extern_func(
                    "lency_string_concat",
                    vec![ValueType::Ptr, ValueType::Ptr],
                    ValueType::Ptr,
                )?;
                self.push(format!(
                    "  {} = call i8* @lency_string_concat(i8* {}, i8* {})",
                    dst, lhs_ptr, rhs_ptr
                ));
                self.mark_temp(dst, ValueType::Ptr);
            }
            "add" | "sub" | "mul" | "div" => {
                let (lhs_i64, _) = self.ensure_i64(lhs_repr, lhs_ty);
                let (rhs_i64, _) = self.ensure_i64(rhs_repr, rhs_ty);
                let llvm_op = match op {
                    "add" => "add",
                    "sub" => "sub",
                    "mul" => "mul",
                    "div" => "sdiv",
                    _ => unreachable!(),
                };
                self.push(format!(
                    "  {} = {} i64 {}, {}",
                    dst, llvm_op, lhs_i64, rhs_i64
                ));
                self.mark_temp(dst, ValueType::I64);
            }
            "cmp_eq" | "cmp_ne" | "cmp_lt" | "cmp_le" | "cmp_gt" | "cmp_ge" => {
                let (lhs_i64, _) = self.ensure_i64(lhs_repr, lhs_ty);
                let (rhs_i64, _) = self.ensure_i64(rhs_repr, rhs_ty);
                let pred = match op {
                    "cmp_eq" => "eq",
                    "cmp_ne" => "ne",
                    "cmp_lt" => "slt",
                    "cmp_le" => "sle",
                    "cmp_gt" => "sgt",
                    "cmp_ge" => "sge",
                    _ => unreachable!(),
                };
                self.push(format!(
                    "  {} = icmp {} i64 {}, {}",
                    dst, pred, lhs_i64, rhs_i64
                ));
                self.mark_temp(dst, ValueType::I1);
            }
            "cmp_str_eq" | "cmp_str_ne" => {
                let (lhs_ptr, _) = self.ensure_ptr(lhs_repr, lhs_ty);
                let (rhs_ptr, _) = self.ensure_ptr(rhs_repr, rhs_ty);
                self.note_extern_func(
                    "lency_string_eq",
                    vec![ValueType::Ptr, ValueType::Ptr],
                    ValueType::I64,
                )?;
                let call_tmp = self.next_tmp("str_eq");
                self.push(format!(
                    "  {} = call i64 @lency_string_eq(i8* {}, i8* {})",
                    call_tmp, lhs_ptr, rhs_ptr
                ));
                let pred = if op == "cmp_str_eq" { "ne" } else { "eq" };
                self.push(format!("  {} = icmp {} i64 {}, 0", dst, pred, call_tmp));
                self.mark_temp(dst, ValueType::I1);
            }
            "and" | "or" => {
                let (lhs_i1, _) = self.ensure_i1(lhs_repr, lhs_ty);
                let (rhs_i1, _) = self.ensure_i1(rhs_repr, rhs_ty);
                let llvm_op = if op == "and" { "and" } else { "or" };
                self.push(format!("  {} = {} i1 {}, {}", dst, llvm_op, lhs_i1, rhs_i1));
                self.mark_temp(dst, ValueType::I1);
            }
            _ => bail!("unsupported binary op: {}", op),
        }
        Ok(())
    }

    pub(super) fn emit_unary(&mut self, dst: &str, op: &str, rhs: &str) -> Result<()> {
        let (rhs_repr, rhs_ty) = self.emit_operand(rhs)?;
        match op {
            "neg" => {
                let (rhs_i64, _) = self.ensure_i64(rhs_repr, rhs_ty);
                self.push(format!("  {} = sub i64 0, {}", dst, rhs_i64));
                self.mark_temp(dst, ValueType::I64);
            }
            "not" => {
                let (rhs_i1, _) = self.ensure_i1(rhs_repr, rhs_ty);
                self.push(format!("  {} = xor i1 {}, true", dst, rhs_i1));
                self.mark_temp(dst, ValueType::I1);
            }
            _ => bail!("unsupported unary op: {}", op),
        }
        Ok(())
    }
}

pub(super) fn llvm_type_str(ty: ValueType) -> &'static str {
    match ty {
        ValueType::I64 => "i64",
        ValueType::I1 => "i1",
        ValueType::Ptr => "i8*",
        ValueType::Void => "void",
    }
}
