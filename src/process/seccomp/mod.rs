//! High-level APIs for libseccomp.
mod libseccomp;
use libseccomp::*;

use std::ffi::CString;
use std::ops::Deref;
use std::os::unix::process::CommandExt as _;
use std::process::Command;

use nix;

/// Syscall wrapper.
pub struct Syscall(u32);

impl Syscall {
    /// Resolve the name of a syscall.
    ///
    /// Panic if the argument is not a available syscall name.
    pub fn from_name(name: &str) -> Syscall {
        let name = CString::new(name).unwrap();
        let syscall = unsafe { seccomp_syscall_resolve_name(name.as_ptr()) };
        assert!(syscall >= 0, "No such syscall");
        Syscall(syscall as u32)
    }
}

impl Deref for Syscall {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

/// Seccomp context.
pub struct Context {
    ctx: scmp_filter_ctx,
}

/// Default seccomp context with the `kill` action.
impl Default for Context {
    fn default() -> Context {
        Context {
            ctx: unsafe { seccomp_init(Act::Kill as u32) },
        }
    }
}

impl Context {
    /// Create a new seccomp context with the given action.
    pub fn new(act: Act) -> Context {
        Context {
            ctx: unsafe { seccomp_init(act as u32) },
        }
    }

    /// Add a new rule to the context.
    pub fn add_rule(&mut self, rule: Rule) -> nix::Result<()> {
        let rc = unsafe {
            seccomp_rule_add_array(
                self.ctx,
                rule.act as u32,
                *rule.syscall as i32,
                rule.args().len() as u32,
                rule.to_arg_cmp().as_ptr(),
            )
        };

        if rc < 0 {
            return Err(nix::Error::from(nix::errno::from_i32(-rc)));
        }

        Ok(())
    }

    /// Reset the context with a new default action.
    pub fn reset(&mut self, default_act: Act) -> nix::Result<()> {
        let rc = unsafe { seccomp_reset(self.ctx, default_act as u32) };

        if rc < 0 {
            return Err(nix::Error::from_errno(nix::errno::from_i32(-rc)));
        }

        Ok(())
    }

    /// Load the current seccomp filter into the kernel.
    pub fn load(&self) -> nix::Result<()> {
        let rc = unsafe { seccomp_load(self.ctx) };

        if rc < 0 {
            return Err(nix::Error::from_errno(nix::errno::from_i32(-rc)));
        }

        Ok(())
    }
}

/// Release the context.
impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            seccomp_release(self.ctx);
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

/// Filter rule for seccomp context.
pub struct Rule {
    act: Act,
    syscall: Syscall,
    pattern: Vec<(CmpOp, i64)>,
}

impl Rule {
    /// Create a new rule for `syscall` with the `act` action.
    pub fn new(act: Act, syscall: Syscall) -> Rule {
        Rule {
            act,
            syscall,
            pattern: Vec::new(),
        }
    }

    /// Create a whitelist rule for `syscall`.
    pub fn whitelist(syscall: Syscall) -> Rule {
        Rule::new(Act::Allow, syscall)
    }

    /// Create a blacklist rule for `syscall`.
    pub fn blacklist(syscall: Syscall) -> Rule {
        Rule::new(Act::Kill, syscall)
    }

    /// Add the new argument to the rule filter.
    pub fn match_arg(&mut self, op: CmpOp, arg: i64) {
        self.pattern.push((op, arg));
    }

    /// Get the arguments of the rule.
    pub fn args(&self) -> &Vec<(CmpOp, i64)> {
        &self.pattern
    }

    fn to_arg_cmp(&self) -> Vec<scmp_arg_cmp> {
        self.pattern
            .iter()
            .enumerate()
            .map(|(index, (op, value))| scmp_arg_cmp {
                arg: index as u32,
                op: *op as u32,
                datum_a: *value as u64,
                datum_b: 0,
            })
            .collect()
    }
}

/// Support actions in seccomp.
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Act {
    Allow = SCMP_ACT_ALLOW,
    Kill = SCMP_ACT_KILL,
    // KillProcess = SCMP_ACT_KILL_PROCESS,
    // Trap = SCMP_ACT_TRAP,
    // Errno = SCMP_ACT_ERRNO,
    // Trace = SCMP_ACT_TRACE,
    // Log = SCMP_ACT_LOG,
}

/// Comparing operations the filter uses.
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CmpOp {
    // Min = scmp_compare__SCMP_CMP_MIN,
    NE = scmp_compare_SCMP_CMP_NE,
    LT = scmp_compare_SCMP_CMP_LT,
    LE = scmp_compare_SCMP_CMP_LE,
    EQ = scmp_compare_SCMP_CMP_EQ,
    GE = scmp_compare_SCMP_CMP_GE,
    GT = scmp_compare_SCMP_CMP_GT,
    // MaskedEq = scmp_compare_SCMP_CMP_MASKED_EQ,
    // Max = scmp_compare__SCMP_CMP_MAX,
}

/// Extra features make Command run in a new container.
pub trait CommandExt {
    /// Load the seccomp config in child process.
    fn seccomp(&mut self, ctx: Context) -> &mut Command;
}

impl CommandExt for Command {
    fn seccomp(&mut self, ctx: Context) -> &mut Command {
        unsafe {
            self.pre_exec(move || {
                ctx.load().expect("Failed to load seccomp context");
                Ok(())
            });
        }
        self
    }
}
