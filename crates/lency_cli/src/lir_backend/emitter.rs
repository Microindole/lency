use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueType {
    I64,
    I1,
    Ptr,
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
    pub(super) extern_funcs: HashMap<String, ExternSig>,
    pub(super) terminated: bool,
}

impl Emitter {
    pub(super) fn new(vars: HashSet<String>) -> Self {
        Self {
            lines: Vec::new(),
            temp_counter: 0,
            vars,
            temps: HashMap::new(),
            extern_funcs: HashMap::new(),
            terminated: false,
        }
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
                self.push(format!("  {} = ptrtoint ptr {} to i64", casted, repr));
                (casted, ValueType::I64)
            }
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
                self.push(format!("  {} = icmp ne ptr {}, null", narrowed, repr));
                (narrowed, ValueType::I1)
            }
        }
    }

    pub(super) fn ensure_ptr(&mut self, repr: String, ty: ValueType) -> (String, ValueType) {
        match ty {
            ValueType::Ptr => (repr, ValueType::Ptr),
            ValueType::I64 => {
                let casted = self.next_tmp("inttoptr");
                self.push(format!("  {} = inttoptr i64 {} to ptr", casted, repr));
                (casted, ValueType::Ptr)
            }
            ValueType::I1 => {
                let (as_i64, _) = self.ensure_i64(repr, ValueType::I1);
                let casted = self.next_tmp("inttoptr");
                self.push(format!("  {} = inttoptr i64 {} to ptr", casted, as_i64));
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
        }
    }

    pub(super) fn emit_operand(&mut self, op: &str) -> Result<(String, ValueType)> {
        if let Some(v) = Self::parse_i64_literal(op) {
            return Ok((v.to_string(), ValueType::I64));
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
        ValueType::Ptr => "ptr",
    }
}
